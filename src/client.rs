use std::net::SocketAddr;

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::model::server::MinecraftServer;

pub struct MinecraftClient {
    pub stream: Option<TcpStream>,
}

impl MinecraftClient {
    pub async fn connect(mut self, address: SocketAddr) -> std::io::Result<()> {
        if let Some(mut old_stream) = self.stream {
            old_stream.shutdown().await?;
        }

        // Create stream
        let stream = TcpStream::connect(address).await?;

        // Initiate connection

        Ok(())
    }
}

pub async fn connect(server: &MinecraftServer) {}
