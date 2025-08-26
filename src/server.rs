//! # TCP Server Implementation
//!
//! This module provides the TCP server that handles client connections and processes
//! commands. It implements a simple request-response protocol over TCP sockets.
//!
//! ## Architecture
//! 
//! The server uses an asynchronous, multi-connection design:
//! - Main server loop accepts incoming connections
//! - Each connection spawns a separate async task
//! - Commands are parsed and executed against the shared storage
//! - Responses are sent back to the client
//!
//! ## Protocol
//! 
//! The server implements a Redis-like text protocol:
//! - Commands: `GET key`, `SET key value`, `DELETE key`
//! - Responses: `VALUE data`, `OK`, `NOT_FOUND`, `ERROR message`
//! - All messages are terminated with `\r\n`
//!
//! ## Concurrency
//! 
//! The storage engine is wrapped in `Arc<Mutex<>>` to allow safe concurrent access
//! from multiple client connections. Each connection gets its own task but shares
//! the same underlying storage.

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

/// TCP server for handling client connections.
/// 
/// The server binds to a specified address and port, then accepts incoming
/// connections and processes commands asynchronously.
pub struct Server {
    /// Server configuration including bind address and port
    config: Config,
    
    /// The storage engine that will be shared across all client connections
    store: KvEngine,
}

impl Server {
    /// Create a new server instance.
    /// 
    /// # Arguments
    /// * `config` - Server configuration (address, port, etc.)
    /// * `store` - Storage engine instance to use for all operations
    /// 
    /// # Returns
    /// * `Server` - New server instance ready to run
    pub fn new(config: Config, store: KvEngine) -> Self {
        Self { config, store }
    }

    /// Start the server and begin accepting connections.
    /// 
    /// This method runs indefinitely, accepting new connections and spawning
    /// tasks to handle them. Each connection gets its own async task but all
    /// share the same storage engine.
    /// 
    /// # Returns
    /// * `Result<()>` - Never returns normally, only on bind errors
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Unable to bind to the specified address/port
    /// - Network-level errors occur
    /// 
    /// # Example
    /// ```rust
    /// let config = Config::default();
    /// let store = KvEngine::new("./data")?;
    /// let server = Server::new(config, store);
    /// server.run().await?; // Runs forever
    /// ```
    pub async fn run(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);

        // Wrap the storage in `Arc<Mutex<>>` for safe concurrent access
        let store = Arc::new(Mutex::new(self.store.clone()));

        // TODO: Add graceful shutdown handling
        // TODO: Add connection limits and rate limiting
        // TODO: Add metrics collection (connections, commands/sec, etc.)

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("Accepted connection from {}", addr);
                    let store_clone = Arc::clone(&store);
                    
                    // Spawn a new task for each client connection
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(socket, addr, store_clone).await {
                            error!("Error handling connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    // Continue accepting other connections despite this error
                }
            }
        }
    }
}

/// Handle a single client connection.
/// 
/// This function processes commands from a client connection until the client
/// disconnects or an error occurs. Each command is parsed, executed against
/// the storage, and a response is sent back.
/// 
/// # Arguments
/// * `socket` - The TCP stream for this client connection
/// * `addr` - Client's address (for logging)
/// * `store` - Shared reference to the storage engine
/// 
/// # Returns
/// * `Result<()>` - Success when client disconnects normally, error on failures
/// 
/// # Protocol Handling
/// - Reads commands from the socket in 1KB chunks
/// - Parses commands using the Protocol parser
/// - Executes commands against the storage engine
/// - Sends appropriate responses back to the client
/// 
/// # Error Handling
/// - Invalid commands result in ERROR responses
/// - Network errors terminate the connection
/// - Storage errors are converted to ERROR responses
async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
    store: Arc<Mutex<KvEngine>>,
) -> Result<()> {
    let mut buffer = [0; 1024];

    while let Ok(n) = socket.read(&mut buffer).await {
        if n == 0 {
            // Client closed the connection
            info!("Connection closed by {}", addr);
            return Ok(());
        }

        // Convert received bytes to string
        let request = std::str::from_utf8(&buffer[..n])?;
        let protocol = Protocol::new();
        
        match protocol.parse(request) {
            Ok(command) => {
                // Lock the storage for the duration of this command
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
                
                // Send response back to client
                socket.write_all(response.as_bytes()).await?;
            }
            Err(e) => {
                // Send error response for invalid commands
                let error_msg = format!("ERROR {}\r\n", e);
                socket.write_all(error_msg.as_bytes()).await?;
            }
        }
    }

    Ok(())
}