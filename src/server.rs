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

use crate::store::KVEngineStoreTrait;
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
    store: Box<dyn KVEngineStoreTrait + Send + Sync>,
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
    pub fn new(config: Config, store: Box<dyn KVEngineStoreTrait + Send + Sync>) -> Self {
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
    /// let store = Box::new(RwLockEngine::new("./data")?);
    /// let server = Server::new(config, store);
    /// server.run().await?; // Runs forever
    /// ```
    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);

        // Wrap the storage in `Arc<Mutex<>>` for safe concurrent access
        let store = Arc::new(Mutex::new(self.store));

        // TODO: Add graceful shutdown handling
        // TODO: Add connection limits and rate limiting
        // TODO: Add metrics collection (connections, commands/sec, etc.)

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("Accepted connection from {}", addr);

                    // Clone the Arc for this connection
                    let store_clone = store.clone();

                    // Spawn a new task to handle this connection
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(socket, addr, store_clone).await {
                            error!("Error handling connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Handle a single client connection.
    ///
    /// This method processes commands from a single client until the connection
    /// is closed or an error occurs.
    ///
    /// # Arguments
    /// * `socket` - The TCP socket for the client connection
    /// * `addr` - Client address for logging purposes
    /// * `store` - Shared storage engine wrapped in Arc<Mutex<>>
    ///
    /// # Returns
    /// * `Result<()>` - Success if connection handled cleanly, error otherwise
    async fn handle_connection(
        mut socket: TcpStream,
        addr: SocketAddr,
        store: Arc<Mutex<Box<dyn KVEngineStoreTrait + Send + Sync>>>,
    ) -> Result<()> {
        let mut buffer = [0; 1024];
        let mut protocol = Protocol::new();

        loop {
            // Read data from the client
            let n = match socket.read(&mut buffer).await {
                Ok(0) => {
                    // Client closed the connection
                    info!("Client {} disconnected", addr);
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from client {}: {}", addr, e);
                    break;
                }
            };

            // Process the received data
            let data = String::from_utf8_lossy(&buffer[..n]);
            let response = match protocol.parse(&data) {
                Ok(command) => {
                    // We have a complete command, execute it
                    let store_guard = store.lock().await;
                    Self::execute_command(&command, &**store_guard)
                }
                Err(e) => {
                    error!("Protocol error from client {}: {}", addr, e);
                    format!("ERROR {}\r\n", e)
                }
            };

            // Send the response back to the client
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                error!("Error writing to client {}: {}", addr, e);
                break;
            }
        }

        Ok(())
    }

    /// Execute a command against the storage engine.
    ///
    /// This method takes a parsed command and executes it against the provided
    /// storage engine, returning the appropriate response string.
    ///
    /// # Arguments
    /// * `command` - The parsed command to execute
    /// * `store` - Reference to the storage engine
    ///
    /// # Returns
    /// * `String` - The response to send back to the client
    fn execute_command(command: &Command, store: &dyn KVEngineStoreTrait) -> String {
        match command {
            Command::Get { key } => match store.get(key) {
                Some(value) => format!("VALUE {}\r\n", value),
                None => "NOT_FOUND\r\n".to_string(),
            },
            Command::Set { key, value } => match store.set(key.clone(), value.clone()) {
                Ok(_) => "OK\r\n".to_string(),
                Err(e) => format!("ERROR {}\r\n", e),
            },
            Command::Delete { key } => {
                store.delete(key); // Always return OK, regardless of whether key existed
                "OK\r\n".to_string()
            }
        }
    }
}
