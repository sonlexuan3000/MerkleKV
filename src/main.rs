//! # MerkleKV - Distributed Key-Value Store
//!
//! MerkleKV is a distributed, eventually consistent key-value store that uses Merkle trees 
//! for efficient data synchronization between nodes. This is an early-stage implementation
//! that demonstrates the core concepts of distributed systems.
//!
//! ## Architecture Overview
//! 
//! The system consists of several key components:
//! - **Storage Engine**: In-memory key-value store (will be replaced with persistent storage)
//! - **Merkle Tree**: Cryptographic hash tree for efficient comparison of dataset states
//! - **TCP Server**: Handles client connections and command processing
//! - **Replication**: MQTT-based message passing for real-time updates
//! - **Sync Manager**: Anti-entropy protocol using Merkle trees for eventual consistency
//! 
//! ## Current Status
//! 
//! This is a skeleton implementation with the following limitations:
//! - Storage is in-memory only (no persistence)
//! - Sync/replication logic is stubbed out
//! - No clustering or peer discovery
//! - Basic error handling
//! 
//! See the README.md for the full roadmap and implementation plan.

use anyhow::Result;
use std::path::PathBuf;

// Core modules for the MerkleKV system
mod config;     // Configuration management
mod protocol;   // Command parsing and protocol handling  
mod replication; // MQTT-based replication (stub)
mod server;     // TCP server for client connections
mod store;      // Storage engine and Merkle tree
mod sync;       // Anti-entropy synchronization (stub)

/// Main entry point for the MerkleKV server.
/// 
/// This function:
/// 1. Initializes logging using env_logger
/// 2. Loads configuration from config.toml
/// 3. Creates a multi-threaded Tokio runtime
/// 4. Initializes the storage engine
/// 5. Starts the TCP server to handle client connections
/// 
/// The server runs indefinitely until terminated by the user.
fn main() -> Result<()> {
    // Initialize logging - use RUST_LOG environment variable to control verbosity
    // Example: RUST_LOG=info cargo run
    env_logger::init();
    
    // Load configuration from the config.toml file in the current directory
    // TODO: Support custom config file path via command line argument
    let config_path = PathBuf::from("config.toml");
    let config = config::Config::load(&config_path)?;
    
    // Create a multi-threaded async runtime for handling concurrent connections
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()  // Enable all Tokio features (timers, I/O, etc.)
        .build()?;
    
    // Start the server in the async runtime
    runtime.block_on(async {
        // Initialize the storage engine (currently in-memory)
        let store = store::kv_engine::KvEngine::new(&config.storage_path)?;
        
        // Create and start the TCP server
        let server = server::Server::new(config.clone(), store);
        server.run().await
    })
}
