//! # Key-Value Storage Engine
//!
//! This module provides the core storage functionality for MerkleKV.
//! Currently implements an in-memory storage engine using HashMap.
//!
//! ## Current Implementation
//!
//! The current implementation is a simple in-memory store that:
//! - Uses `Arc<HashMap<String, String>>` for thread-safe access
//! - Creates new HashMap instances on every write (copy-on-write pattern)
//! - Provides basic get/set/delete operations
//! - Returns all keys for iteration
//!
//! ## Future Implementation Plans
//!
//! This is a placeholder implementation. A production version should:
//! - Use a persistent storage engine (e.g., RocksDB, Sled)
//! - Support transactions and atomic operations
//! - Implement Write-Ahead Logging (WAL)
//! - Add compression and efficient serialization
//! - Support range queries and iteration
//! - Implement proper error handling for I/O operations

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::kv_trait::KVEngineStoreTrait;

/// In-memory key-value storage engine.
///
/// This is a simplified storage implementation that keeps all data in memory.
/// The `Arc<HashMap>` allows for efficient cloning of the engine while sharing
/// the underlying data until a write operation occurs.
///
/// **Note**: This implementation is not persistent! All data is lost when
/// the process terminates.
#[derive(Clone)]
pub struct KvEngine {
    /// Shared reference to the key-value data
    /// Using Arc allows multiple readers while writes create new instances
    data: Arc<HashMap<String, String>>,
    // TODO: Add persistent storage implementation
    // In a real implementation, this would use a persistent storage engine like Sled:
    // storage_path: PathBuf,
    // sled_db: sled::Db,
}

impl KvEngine {
    /// Create a new storage engine instance.
    ///
    /// # Arguments
    /// * `_storage_path` - Path where data should be stored (currently unused)
    ///
    /// # Returns
    /// * `Result<KvEngine>` - New storage engine instance or error
    ///
    /// # Current Behavior
    /// Creates an empty in-memory HashMap. The storage_path is ignored.
    ///
    /// # Future Implementation
    /// Should initialize or open a persistent storage engine at the given path.
    pub fn new(_storage_path: &str) -> Result<Self> {
        // TODO: Initialize persistent storage engine here
        // For example, with Sled:
        // let db = sled::open(storage_path)?;
        // Ok(Self { storage_path: storage_path.into(), sled_db: db })

        Ok(Self {
            data: Arc::new(HashMap::new()),
        })
    }

    /// Retrieve a value by its key.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// * `Option<String>` - The value if found, None otherwise
    ///
    /// # Example
    /// ```rust
    /// let engine = KvEngine::new("./data")?;
    /// if let Some(value) = engine.get("user:123") {
    ///     println!("Found user: {}", value);
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    /// Store a key-value pair.
    ///
    /// This operation creates a new HashMap with the updated data due to the
    /// immutable nature of the `Arc<HashMap>` design. This is inefficient but
    /// simple for the current prototype.
    ///
    /// # Arguments
    /// * `key` - The key to store
    /// * `value` - The value to associate with the key
    ///
    /// # Example
    /// ```rust
    /// let mut engine = KvEngine::new("./data")?;
    /// engine.set("user:123".to_string(), "john_doe".to_string());
    /// ```
    ///
    /// # Performance Note
    /// This implementation is O(n) in the number of keys due to HashMap cloning.
    /// A persistent storage engine would be much more efficient.
    pub fn set(&mut self, key: String, value: String) {
        // Create a new HashMap with the updated value
        // TODO: Replace with efficient persistent storage writes
        let mut new_data = (*self.data).clone();
        new_data.insert(key, value);
        self.data = Arc::new(new_data);
    }

    /// Delete a key-value pair.
    ///
    /// Like `set`, this creates a new HashMap without the deleted key.
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Example
    /// ```rust
    /// let mut engine = KvEngine::new("./data")?;
    /// engine.delete("user:123");
    /// ```
    pub fn delete(&mut self, key: &str) {
        // Create a new HashMap without the deleted key
        // TODO: Replace with efficient persistent storage deletes
        let mut new_data = (*self.data).clone();
        new_data.remove(key);
        self.data = Arc::new(new_data);
    }

    /// Get all keys currently stored in the engine.
    ///
    /// This is primarily used by the Merkle tree to rebuild its state
    /// and for debugging purposes.
    ///
    /// # Returns
    /// * `Vec<String>` - Vector of all keys in the store
    ///
    /// # Performance Note
    /// This operation is O(n) and creates a new vector. In a production
    /// system, this should be replaced with an iterator-based approach.
    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

impl KVEngineStoreTrait for KvEngine {
    /// Retrieve a value by its key.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// * `Option<String>` - The value if found, None otherwise
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    /// Store a key-value pair.
    ///
    /// ⚠️ **WARNING**: This method is NOT thread-safe!
    ///
    /// This operation creates a new HashMap with the updated data due to the
    /// immutable nature of the `Arc<HashMap>` design. This is inefficient but
    /// simple for the current prototype.
    ///
    /// # Arguments
    /// * `key` - The key to store
    /// * `value` - The value to associate with the key
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Thread Safety
    /// ⚠️ This method is NOT safe for concurrent access!
    /// Concurrent writes can lead to data corruption or lost updates.
    fn set(&self, key: String, value: String) -> Result<()> {
        // This is unsafe for concurrent access!
        // We need to clone the HashMap, modify it, and create a new Arc
        let mut new_data = HashMap::clone(&self.data);
        new_data.insert(key, value);
        // This is a race condition if multiple threads do this simultaneously
        unsafe {
            let arc_ptr = Arc::into_raw(self.data.clone());
            let mutex_ptr = arc_ptr as *mut HashMap<String, String>;
            *mutex_ptr = new_data;
            let _ = Arc::from_raw(arc_ptr);
        }
        Ok(())
    }

    /// Delete a key-value pair.
    ///
    /// ⚠️ **WARNING**: This method is NOT thread-safe!
    ///
    /// Like `set`, this creates a new HashMap without the deleted key.
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Returns
    /// * `bool` - True if the key existed and was deleted, false otherwise
    ///
    /// # Thread Safety
    /// ⚠️ This method is NOT safe for concurrent access!
    fn delete(&self, key: &str) -> bool {
        // This is unsafe for concurrent access!
        let mut new_data = HashMap::clone(&self.data);
        let existed = new_data.remove(key).is_some();
        if existed {
            unsafe {
                let arc_ptr = Arc::into_raw(self.data.clone());
                let mutex_ptr = arc_ptr as *mut HashMap<String, String>;
                *mutex_ptr = new_data;
                let _ = Arc::from_raw(arc_ptr);
            }
        }
        existed
    }

    /// Get all keys currently stored in the engine.
    ///
    /// # Returns
    /// * `Vec<String>` - Vector of all keys in the store
    fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Get the number of key-value pairs in the store.
    ///
    /// # Returns
    /// * `usize` - Number of key-value pairs
    fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the store is empty.
    ///
    /// # Returns
    /// * `bool` - True if the store is empty, false otherwise
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_kv_operations() {
        // Use a temporary directory for testing (even though it's not used yet)
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap();

        let mut engine = KvEngine::new(storage_path).unwrap();

        // Test basic set and get operations
        engine.set("key1".to_string(), "value1".to_string());
        assert_eq!(engine.get("key1"), Some("value1".to_string()));

        // Test overwriting an existing key
        engine.set("key1".to_string(), "new_value".to_string());
        assert_eq!(engine.get("key1"), Some("new_value".to_string()));

        // Test delete operation
        engine.delete("key1");
        assert_eq!(engine.get("key1"), None);

        // Test keys() method with multiple entries
        engine.set("key2".to_string(), "value2".to_string());
        engine.set("key3".to_string(), "value3".to_string());

        let keys = engine.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }
}
