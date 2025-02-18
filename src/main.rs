use anyhow::Result;

use clap::Parser;
use lazy_listener::LazyListener;
use registry::{ChildProcessRegistry, MinecraftRegistry};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    listen_addr: String,

    #[arg(short, long)]
    command: String,

    #[arg(short, long)]
    downstream_addr: String,

    #[arg(short, long)]
    stdout_ready_pattern: String,

    #[arg(short, long)]
    shutdown_stdin_command: String,

    #[arg(short, long)]
    debounce_time_millis: u64,
}

mod lazy_listener;
mod registry;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let registry = ChildProcessRegistry::new(
        args.downstream_addr,
        args.command,
        args.stdout_ready_pattern,
        args.shutdown_stdin_command + "\n",
        std::time::Duration::from_millis(args.debounce_time_millis),
    );
    let registry = MinecraftRegistry::new(registry);

    let listener = LazyListener::new(args.listen_addr.clone(), registry).await;
    listener.run().await;
    Ok(())
}
