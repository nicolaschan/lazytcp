use std::sync::Arc;

use tokio::{io, net::TcpListener};
use tracing::{debug, error};

use crate::downstream_registry::DownstreamRegistry;

pub struct LazyListener<R: DownstreamRegistry> {
    listener: TcpListener,
    registry: Arc<R>,
}

impl<R: DownstreamRegistry> LazyListener<R> {
    pub async fn new(listen_addr: String, registry: R) -> Self {
        let listener = TcpListener::bind(&listen_addr).await.unwrap();
        LazyListener {
            listener,
            registry: Arc::new(registry),
        }
    }
}

impl<R: DownstreamRegistry + Send + Sync + 'static> LazyListener<R> {
    pub async fn run(&self) {
        loop {
            let (mut upstream, _) = self.listener.accept().await.unwrap();
            let upstream_addr = upstream.peer_addr().unwrap();
            debug!("Accepted connection from {:?}", upstream_addr);

            let registry = self.registry.clone();
            tokio::spawn(async move {
                match registry.connect(&mut upstream).await {
                    Ok(mut downstream) => {
                        let registry_clone = registry.clone();
                        match io::copy_bidirectional(&mut upstream, &mut downstream).await {
                            Ok((_, _)) => debug!("Connection closed to {:?}", upstream_addr),
                            Err(e) => error!("Error {:?}: {}", upstream_addr, e),
                        }
                        registry_clone.disconnect().await;
                    }
                    Err(e) => error!("Error connecting to downstream: {}", e),
                }
            });
        }
    }
}
