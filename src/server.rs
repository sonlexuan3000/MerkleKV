use crate::store::kv_engine::KvEngine;
use anyhow::Result;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::protocol::{Command, Protocol};

pub struct Server {
    config: Config,
    store: KvEngine,
}

impl Server {
    pub fn new(config: Config, store: KvEngine) -> Self {
        Self { config, store }
    }

    pub async fn run(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);

        let store = Arc::new(Mutex::new(self.store.clone()));







        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("Accepted connection from {}", addr);
                    let store_clone = Arc::clone(&store);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(socket, addr, store_clone).await {
                            error!("Error handling connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
    store: Arc<Mutex<KvEngine>>,
) -> Result<()> {
    let mut buffer = [0; 1024];

    while let Ok(n) = socket.read(&mut buffer).await {
        if n == 0 {
            info!("Connection closed by {}", addr);
            return Ok(());
        }

        let request = std::str::from_utf8(&buffer[..n])?;
        let protocol = Protocol::new();
        
        match protocol.parse(request) {
            Ok(command) => {
                let mut store = store.lock().await;
                let response = match command {
                    Command::Get { key } => {
                        match store.get(&key) {
                            Some(value) => format!("VALUE {}\r\n", value),
                            None => "NOT_FOUND\r\n".to_string(),
                        }
                    }
                    Command::Set { key, value } => {
                        store.set(key, value);
                        "OK\r\n".to_string()
                    }
                    Command::Delete { key } => {
                        store.delete(&key);
                        "OK\r\n".to_string()
                    }
                };
                socket.write_all(response.as_bytes()).await?;
            }
            Err(e) => {
                let error_msg = format!("ERROR {}\r\n", e);
                socket.write_all(error_msg.as_bytes()).await?;
            }
        }
    }

    Ok(())
}
