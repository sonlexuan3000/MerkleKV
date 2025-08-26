//! # Storage Engine Module
//!
//! This module contains the storage components for MerkleKV:
//! 
//! - **`kv_engine`**: Core key-value storage engine (currently in-memory)
//! - **`merkle`**: Merkle tree implementation for efficient synchronization
//! 
//! ## Design Philosophy
//! 
//! The storage layer is designed to be modular and replaceable. The current
//! in-memory implementation serves as a prototype, but the interfaces are
//! designed to support persistent storage engines in the future.
//! 
//! ## Future Enhancements
//! 
//! - Replace in-memory storage with persistent engine (RocksDB, Sled, etc.)
//! - Add Write-Ahead Logging (WAL) for durability
//! - Implement compression and efficient serialization
//! - Add support for range queries and iteration
//! - Optimize Merkle tree for incremental updates

pub mod kv_engine;
pub mod merkle;
