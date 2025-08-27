//! # KV Storage Engine Trait
//!
//! This module defines the common interface for all key-value storage engines
//! in MerkleKV. This allows for easy swapping between different storage implementations
//! without changing the rest of the codebase.
//!
//! ## Implementations
//!
//! - `RwLockEngine`: Thread-safe in-memory storage using RwLock<HashMap>
//! - `KvEngine`: Non-thread-safe in-memory storage using Arc<HashMap>
//! - Future: Persistent storage engines (RocksDB, Sled, etc.)

use anyhow::Result;

/// Common interface for all key-value storage engines.
///
/// This trait defines the core operations that any storage engine must implement.
/// All engines should be safe to share across multiple threads (Send + Sync).
pub trait KVEngineStoreTrait: Send + Sync {
    /// Retrieve a value by its key.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// * `Option<String>` - The value if found, None otherwise
    fn get(&self, key: &str) -> Option<String>;

    /// Store a key-value pair.
    ///
    /// # Arguments
    /// * `key` - The key to store
    /// * `value` - The value to associate with the key
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn set(&self, key: String, value: String) -> Result<()>;

    /// Delete a key-value pair.
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Returns
    /// * `bool` - True if the key existed and was deleted, false otherwise
    fn delete(&self, key: &str) -> bool;

    /// Get all keys currently stored in the engine.
    ///
    /// # Returns
    /// * `Vec<String>` - Vector of all keys in the store
    fn keys(&self) -> Vec<String>;

    /// Get the number of key-value pairs in the store.
    ///
    /// # Returns
    /// * `usize` - Number of key-value pairs
    fn len(&self) -> usize;

    /// Check if the store is empty.
    ///
    /// # Returns
    /// * `bool` - True if the store is empty, false otherwise
    fn is_empty(&self) -> bool;
}
