use anyhow::Result;
use std::future::Future;

use tokio::net::TcpStream;

pub trait DownstreamRegistry {
    fn connect(&self, upstream: &mut TcpStream) -> impl Future<Output = Result<TcpStream>> + Send;
    fn disconnect(&self) -> impl Future<Output = ()> + Send;
    async fn active_connections(&self) -> usize;
}
