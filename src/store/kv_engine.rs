//! # Key-Value Storage Engine
//!
//! This module provides the core storage functionality for MerkleKV.
//! Implements a thread-safe in-memory storage engine using RwLock<HashMap>.
//!
//! ## Thread Safety Implementation
//!
//! The current implementation uses `RwLock<HashMap<String, String>>` for thread-safe access:
//! - **Multiple concurrent readers**: Multiple threads can read simultaneously
//! - **Single writer**: Only one thread can write at a time
//! - **No race conditions**: All operations are properly synchronized
//! - **Efficient**: Readers don't block each other, only writers block
//!
//! ## Future Implementation Plans
//!
//! This is a production-ready in-memory implementation. Future versions could:
//! - Add persistent storage (e.g., RocksDB, Sled)
//! - Support transactions and atomic operations
//! - Implement Write-Ahead Logging (WAL)
//! - Add compression and efficient serialization
//! - Support range queries and iteration

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe in-memory key-value storage engine.
///
/// This implementation uses `RwLock<HashMap>` to provide thread-safe access:
/// - Multiple threads can read simultaneously (shared read lock)
/// - Only one thread can write at a time (exclusive write lock)
/// - All operations are atomic and race-condition free
///
/// **Note**: This implementation is not persistent! All data is lost when
/// the process terminates.
#[derive(Clone)]
pub struct KvEngine {
    /// Thread-safe shared reference to the key-value data
    /// Using RwLock allows multiple readers or a single writer
    data: Arc<RwLock<HashMap<String, String>>>,
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
    /// # Thread Safety
    /// The returned engine is safe to share across multiple threads.
    pub fn new(_storage_path: &str) -> Result<Self> {
        // TODO: Initialize persistent storage engine here
        // For example, with Sled:
        // let db = sled::open(storage_path)?;
        // Ok(Self { storage_path: storage_path.into(), sled_db: db })

        Ok(Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Retrieve a value by its key.
    ///
    /// This method acquires a **shared read lock**, allowing multiple threads
    /// to read simultaneously without blocking each other.
    ///
    /// # Arguments
    /// * `key` - The key to look up
    ///
    /// # Returns
    /// * `Option<String>` - The value if found, None otherwise
    ///
    /// # Thread Safety
    /// Multiple threads can call this method concurrently without issues.
    ///
    /// # Example
    /// ```rust
    /// let engine = KvEngine::new("./data")?;
    /// if let Some(value) = engine.get("user:123") {
    ///     println!("Found user: {}", value);
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<String> {
        // Acquire shared read lock - multiple readers can proceed simultaneously
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    /// Store a key-value pair.
    ///
    /// This method acquires an **exclusive write lock**, ensuring only one
    /// thread can write at a time. This prevents race conditions and data corruption.
    ///
    /// # Arguments
    /// * `key` - The key to store
    /// * `value` - The value to associate with the key
    ///
    /// # Thread Safety
    /// Only one thread can write at a time. Other threads will wait for the
    /// write lock to be released.
    ///
    /// # Example
    /// ```rust
    /// let engine = KvEngine::new("./data")?;
    /// engine.set("user:123".to_string(), "john_doe".to_string());
    /// ```
    pub fn set(&self, key: String, value: String) -> Result<()> {
        // Acquire exclusive write lock - only one writer at a time
        let mut data = self.data.write().unwrap();
        data.insert(key, value);
        Ok(())
    }

    /// Delete a key-value pair.
    ///
    /// Like `set`, this method acquires an **exclusive write lock** to ensure
    /// thread safety during deletion operations.
    ///
    /// # Arguments
    /// * `key` - The key to delete
    ///
    /// # Returns
    /// * `bool` - True if the key existed and was deleted, false otherwise
    ///
    /// # Thread Safety
    /// Only one thread can delete at a time. Other threads will wait for the
    /// write lock to be released.
    ///
    /// # Example
    /// ```rust
    /// let engine = KvEngine::new("./data")?;
    /// if engine.delete("user:123") {
    ///     println!("User deleted successfully");
    /// }
    /// ```
    pub fn delete(&self, key: &str) -> bool {
        // Acquire exclusive write lock - only one writer at a time
        let mut data = self.data.write().unwrap();
        data.remove(key).is_some()
    }

    /// Get all keys currently stored in the engine.
    ///
    /// This method acquires a **shared read lock** to safely iterate over all keys.
    ///
    /// # Returns
    /// * `Vec<String>` - Vector of all keys in the store
    ///
    /// # Thread Safety
    /// Multiple threads can call this method concurrently without issues.
    ///
    /// # Performance Note
    /// This operation creates a new vector. In a production system, consider
    /// using an iterator-based approach for better memory efficiency.
    pub fn keys(&self) -> Vec<String> {
        // Acquire shared read lock - multiple readers can proceed simultaneously
        let data = self.data.read().unwrap();
        data.keys().cloned().collect()
    }

    /// Get the number of key-value pairs in the store.
    ///
    /// # Returns
    /// * `usize` - Number of key-value pairs
    ///
    /// # Thread Safety
    /// Multiple threads can call this method concurrently without issues.
    pub fn len(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }

    /// Check if the store is empty.
    ///
    /// # Returns
    /// * `bool` - True if the store is empty, false otherwise
    ///
    /// # Thread Safety
    /// Multiple threads can call this method concurrently without issues.
    pub fn is_empty(&self) -> bool {
        let data = self.data.read().unwrap();
        data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use tempfile::tempdir;

    #[test]
    fn test_kv_operations() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap();

        let engine = KvEngine::new(storage_path).unwrap();

        // Test basic set and get operations
        engine
            .set("key1".to_string(), "value1".to_string())
            .unwrap();
        assert_eq!(engine.get("key1"), Some("value1".to_string()));

        // Test overwriting an existing key
        engine
            .set("key1".to_string(), "new_value".to_string())
            .unwrap();
        assert_eq!(engine.get("key1"), Some("new_value".to_string()));

        // Test delete operation
        assert!(engine.delete("key1"));
        assert_eq!(engine.get("key1"), None);

        // Test keys() method with multiple entries
        engine
            .set("key2".to_string(), "value2".to_string())
            .unwrap();
        engine
            .set("key3".to_string(), "value3".to_string())
            .unwrap();

        let keys = engine.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));

        // Test len() and is_empty()
        assert_eq!(engine.len(), 2);
        assert!(!engine.is_empty());
    }

    #[test]
    fn test_concurrent_reads() {
        let engine = Arc::new(KvEngine::new("./test_data").unwrap());

        // Set up some test data
        engine
            .set("key1".to_string(), "value1".to_string())
            .unwrap();
        engine
            .set("key2".to_string(), "value2".to_string())
            .unwrap();

        // Spawn multiple reader threads
        let mut handles = vec![];
        for i in 0..10 {
            let engine_clone = engine.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    assert_eq!(engine_clone.get("key1"), Some("value1".to_string()));
                    assert_eq!(engine_clone.get("key2"), Some("value2".to_string()));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_single_writer() {
        let engine = Arc::new(KvEngine::new("./test_data").unwrap());

        // Spawn multiple writer threads - they should serialize
        let mut handles = vec![];
        for i in 0..5 {
            let engine_clone = engine.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let key = format!("key_{}_{}", i, j);
                    let value = format!("value_{}_{}", i, j);
                    engine_clone.set(key, value).unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all data was written correctly
        assert_eq!(engine.len(), 50);
        for i in 0..5 {
            for j in 0..10 {
                let key = format!("key_{}_{}", i, j);
                let expected_value = format!("value_{}_{}", i, j);
                assert_eq!(engine.get(&key), Some(expected_value));
            }
        }
    }

    #[test]
    fn test_mixed_operations() {
        let engine = Arc::new(KvEngine::new("./test_data").unwrap());

        // Spawn reader and writer threads simultaneously
        let mut handles = vec![];

        // Writer thread
        let engine_writer = engine.clone();
        let writer_handle = thread::spawn(move || {
            for i in 0..100 {
                engine_writer
                    .set(format!("key{}", i), format!("value{}", i))
                    .unwrap();
                thread::yield_now(); // Give readers a chance
            }
        });
        handles.push(writer_handle);

        // Reader threads
        for _ in 0..3 {
            let engine_reader = engine.clone();
            let reader_handle = thread::spawn(move || {
                for _ in 0..50 {
                    let keys = engine_reader.keys();
                    let len = engine_reader.len();
                    // Readers should never see inconsistent state
                    assert!(keys.len() <= len);
                    thread::yield_now();
                }
            });
            handles.push(reader_handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Final verification
        assert_eq!(engine.len(), 100);
    }
}
