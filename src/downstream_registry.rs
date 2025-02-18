use anyhow::Result;
use std::{
    future::Future,
    ops::Add,
    process::Stdio,
    time::{Duration, Instant},
};
use tracing::{debug, error, info};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub trait DownstreamRegistry {
    fn connect(&self, upstream: &mut TcpStream) -> impl Future<Output = Result<TcpStream>> + Send;
    fn disconnect(&self) -> impl Future<Output = ()> + Send;
    async fn active_connections(&self) -> usize;
}

pub struct ChildProcessRegistry {
    downstream_addr: String,
    command: String,
    stdout_ready_pattern: String,
    shutdown_stdin_command: String,
    debounce_time: Duration,
    active_connections: Mutex<usize>,
    process: Mutex<Option<tokio::process::Child>>,
    last_accessed: Mutex<Instant>,
}

impl ChildProcessRegistry {
    pub fn new(
        downstream_addr: String,
        command: String,
        stdout_ready_pattern: String,
        shutdown_stdin_command: String,
        debounce_time: Duration,
    ) -> Self {
        ChildProcessRegistry {
            downstream_addr,
            command,
            stdout_ready_pattern,
            shutdown_stdin_command,
            debounce_time,
            active_connections: Mutex::new(0),
            process: Mutex::new(None),
            last_accessed: Mutex::new(Instant::now()),
        }
    }

    async fn ensure_process_started(&self) {
        let mut guard = self.process.lock().await;
        if guard.is_none() {
            let parts = shell_words::split(&self.command).unwrap();

            let program = parts[0].clone();
            let args = &parts[1..];

            let mut command = tokio::process::Command::new(&program);
            command.args(args);
            command.stdout(Stdio::piped());
            command.stdin(Stdio::piped());
            *guard = Some(command.spawn().expect("Failed to start process"));

            // print stdout to console
            let child = guard.as_mut().unwrap();
            let stdout = child.stdout.take().expect("Failed to open stdout");

            let mut reader = tokio::io::BufReader::new(stdout);
            let mut buffer = String::new();
            loop {
                match reader.read_line(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if buffer.contains(&self.stdout_ready_pattern) {
                            info!("Downstream process is ready");
                            break;
                        }
                        print!("{}", buffer);
                        buffer = String::new();
                    }
                    Err(e) => error!("Error reading stdout: {}", e),
                }
            }

            tokio::spawn(async move {
                loop {
                    match reader.read_line(&mut buffer).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            print!("{}", buffer);
                            buffer = String::new();
                        }
                        Err(e) => error!("Error reading stdout: {}", e),
                    }
                }
            });
        }
    }

    async fn shutdown_now(&self) {
        let mut guard = self.process.lock().await;
        let mut stdin = guard.as_mut().unwrap().stdin.take().unwrap();
        info!("Shutting down downstream process");
        stdin
            .write_all(self.shutdown_stdin_command.as_bytes())
            .await
            .unwrap();
        stdin.flush().await.unwrap();
        guard.as_mut().unwrap().wait().await.unwrap();
        *guard = None;
    }
}

impl DownstreamRegistry for ChildProcessRegistry {
    async fn connect(&self, _upstream: &mut TcpStream) -> Result<TcpStream> {
        {
            let mut active_connections_guard = self.active_connections.lock().await;
            *active_connections_guard += 1;
        }

        self.ensure_process_started().await;
        let stream = TcpStream::connect(self.downstream_addr.clone()).await?;
        Ok(stream)
    }

    async fn disconnect(&self) {
        let connections_remaining = {
            let mut active_connections_guard = self.active_connections.lock().await;
            *active_connections_guard -= 1;
            *active_connections_guard
        };

        debug!(
            "Disconnecting with {} connections remaining",
            connections_remaining
        );

        {
            let mut last_accessed_guard = self.last_accessed.lock().await;
            *last_accessed_guard = Instant::now();
        }

        if connections_remaining != 0 {
            return;
        }

        // Sleep and shutdown child process if there's no disconnect in the meantime
        // Add an extra ms due to ms resolution of sleep
        let sleep_time = self.debounce_time.add(Duration::from_millis(1));
        tokio::time::sleep(sleep_time).await;

        let active_connections_guard = self.active_connections.lock().await;
        let last_accessed_guard = self.last_accessed.lock().await;
        if *active_connections_guard == 0 && last_accessed_guard.elapsed() >= self.debounce_time {
            self.shutdown_now().await;
        }
    }

    async fn active_connections(&self) -> usize {
        let guard = self.active_connections.lock().await;
        *guard
    }
}
