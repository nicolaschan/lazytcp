use anyhow::Result;
use std::{collections::HashMap, future::Future};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};
use tracing::{debug, warn};

use crate::registry::{minecraft::varint::read_varint, DownstreamRegistry};

pub struct CachingRegistry<R: DownstreamRegistry> {
    cache: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    delegate: R,
}

impl<R: DownstreamRegistry> CachingRegistry<R> {
    pub fn new(delegate: R) -> Self {
        CachingRegistry {
            cache: Mutex::new(HashMap::new()),
            delegate,
        }
    }
}

impl<R: DownstreamRegistry + Sync> DownstreamRegistry for CachingRegistry<R> {
    async fn connect(&self, upstream: &mut TcpStream) -> Result<TcpStream> {
        let mut initial_buffer: Vec<u8> = Vec::new();
        let mut temp_buffer = [0u8; 8192];

        loop {
            match upstream.read(&mut temp_buffer).await {
                Ok(bytes_read) => {
                    initial_buffer.extend_from_slice(&temp_buffer[..bytes_read]);
                }
                Err(e) => {
                    eprintln!("Error reading from upstream: {}", e);
                    return Err(e.into());
                }
            };
            match read_varint(&initial_buffer) {
                Ok((value, offset)) => {
                    if initial_buffer.len() >= offset + value as usize {
                        debug!("Got varint size: value={:?} offset={:?}", value, offset);
                        let next_state = read_varint(&initial_buffer[offset + value as usize..]);
                        debug!("Next state: {:?}", next_state);
                        break;
                    }
                }
                Err(e) => {
                    warn!("Error reading varint: {:?}", e);
                    break;
                }
            }
        }

        let hex_string = hex::encode(&initial_buffer);
        let separated = hex_string
            .chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ");
        debug!("Initial buffer: {}", separated);

        {
            let cache_guard = self.cache.lock().await;
            if let Some(cached_response) = cache_guard.get(&initial_buffer) {
                debug!("Cache hit: {:?}", initial_buffer);
                upstream.write_all(cached_response).await?;
                upstream.flush().await?;
                return Err(anyhow::Error::msg("Using cached response"));
            }
        }

        let mut downstream = self.delegate.connect(upstream).await?;
        downstream.write_all(&initial_buffer).await?;
        Ok(downstream)
    }

    fn disconnect(&self) -> impl Future<Output = ()> + Send {
        self.delegate.disconnect()
    }

    async fn active_connections(&self) -> usize {
        self.delegate.active_connections().await
    }
}
