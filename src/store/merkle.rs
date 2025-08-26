//! # Merkle Tree Implementation
//!
//! This module implements a Merkle tree data structure for efficient synchronization
//! between distributed nodes. Merkle trees allow nodes to quickly compare their
//! datasets and identify differences without transferring all data.
//!
//! ## How Merkle Trees Work
//! 
//! A Merkle tree is a binary tree where:
//! 1. **Leaf nodes** contain hashes of individual key-value pairs
//! 2. **Internal nodes** contain hashes of their children's hashes
//! 3. **Root node** represents the hash of the entire dataset
//! 
//! ## Use in Distributed Systems
//! 
//! Two nodes can compare their root hashes:
//! - If root hashes match → datasets are identical
//! - If root hashes differ → datasets differ, can recursively compare subtrees
//! 
//! This enables efficient "anti-entropy" protocols where nodes only sync
//! the differences rather than the entire dataset.
//!
//! ## Current Implementation
//! 
//! This is a basic implementation that rebuilds the entire tree on each change.
//! A production implementation would use incremental updates for efficiency.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A node in the Merkle tree.
/// 
/// Each node contains a cryptographic hash and optional pointers to child nodes.
/// Leaf nodes have no children, while internal nodes have exactly two children.
#[derive(Debug, Clone, PartialEq)]
pub struct MerkleNode {
    /// SHA-256 hash of this node's content
    /// - For leaf nodes: hash of "key:value" string
    /// - For internal nodes: hash of concatenated child hashes
    pub hash: Vec<u8>,
    
    /// Left child node (None for leaf nodes)
    pub left: Option<Box<MerkleNode>>,
    
    /// Right child node (None for leaf nodes)
    pub right: Option<Box<MerkleNode>>,
}

/// Merkle tree data structure for efficient dataset comparison.
/// 
/// The tree maintains a mapping of keys to their hashes and builds a binary
/// tree structure where the root hash represents the entire dataset.
/// 
/// # Example Usage
/// ```rust
/// let mut tree = MerkleTree::new();
/// tree.insert("user:123", "john_doe");
/// tree.insert("user:456", "jane_doe");
/// 
/// let root_hash = tree.get_root_hash();
/// // Compare with another node's root hash for synchronization
/// ```
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// The root node of the tree (None if empty)
    pub root: Option<MerkleNode>,
    
    /// Map of keys to their hashes for efficient lookups and updates
    /// This allows O(1) key updates without scanning the entire tree
    leaf_map: HashMap<String, Vec<u8>>,
}

impl MerkleTree {
    /// Create a new empty Merkle tree.
    /// 
    /// # Returns
    /// * `MerkleTree` - A new empty tree
    pub fn new() -> Self {
        Self {
            root: None,
            leaf_map: HashMap::new(),
        }
    }

    /// Insert or update a key-value pair in the tree.
    /// 
    /// This operation:
    /// 1. Computes SHA-256 hash of "key:value"
    /// 2. Updates the leaf_map with the new hash
    /// 3. Rebuilds the entire tree structure
    /// 
    /// # Arguments
    /// * `key` - The key to insert/update
    /// * `value` - The value to associate with the key
    /// 
    /// # Performance Note
    /// Currently rebuilds the entire tree, which is O(n log n).
    /// A production implementation should use incremental updates.
    pub fn insert(&mut self, key: &str, value: &str) {
        // Create a deterministic hash of the key-value pair
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", key, value));
        let hash = hasher.finalize().to_vec();
        
        // Update the leaf mapping and rebuild the tree
        self.leaf_map.insert(key.to_string(), hash);
        self.rebuild();
    }

    /// Remove a key-value pair from the tree.
    /// 
    /// # Arguments
    /// * `key` - The key to remove
    /// 
    /// # Behavior
    /// If the key doesn't exist, this is a no-op.
    /// After removal, the tree is rebuilt to reflect the change.
    pub fn remove(&mut self, key: &str) {
        self.leaf_map.remove(key);
        self.rebuild();
    }

    /// Get the root hash of the tree.
    /// 
    /// The root hash represents the entire dataset and can be compared
    /// with other nodes for synchronization purposes.
    /// 
    /// # Returns
    /// * `Option<&Vec<u8>>` - Root hash if tree is not empty, None otherwise
    /// 
    /// # Example
    /// ```rust
    /// let tree = MerkleTree::new();
    /// if let Some(root_hash) = tree.get_root_hash() {
    ///     println!("Dataset hash: {:x}", root_hash.iter().format(""));
    /// }
    /// ```
    pub fn get_root_hash(&self) -> Option<&Vec<u8>> {
        self.root.as_ref().map(|node| &node.hash)
    }

    /// Rebuild the entire tree structure from the current leaf mappings.
    /// 
    /// This is the core algorithm that constructs the Merkle tree:
    /// 1. Create leaf nodes from all key-value hashes
    /// 2. Repeatedly pair up nodes and create parents until only one remains
    /// 3. The final node becomes the root
    /// 
    /// # Algorithm Details
    /// - Uses a bottom-up approach, starting with leaves
    /// - Each level pairs adjacent nodes to create the next level
    /// - If odd number of nodes, the last one is carried up unchanged
    /// - Parent hash = SHA256(left_child_hash || right_child_hash)
    /// 
    /// # Time Complexity
    /// O(n log n) where n is the number of key-value pairs
    fn rebuild(&mut self) {
        // Handle empty tree case
        if self.leaf_map.is_empty() {
            self.root = None;
            return;
        }
        
        // Create leaf nodes from all current mappings
        let mut nodes: Vec<MerkleNode> = self
            .leaf_map
            .values()
            .map(|hash| MerkleNode {
                hash: hash.clone(),
                left: None,
                right: None,
            })
            .collect();
        
        // Build the tree bottom-up by repeatedly pairing nodes
        while nodes.len() > 1 {
            let mut new_level = Vec::new();
            
            // Process nodes in pairs to create the next level
            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
                    // Create parent node by hashing concatenated child hashes
                    let mut hasher = Sha256::new();
                    hasher.update(&chunk[0].hash);
                    hasher.update(&chunk[1].hash);
                    let hash = hasher.finalize().to_vec();
                    
                    new_level.push(MerkleNode {
                        hash,
                        left: Some(Box::new(chunk[0].clone())),
                        right: Some(Box::new(chunk[1].clone())),
                    });
                } else {
                    // Odd number of nodes - carry the last one up unchanged
                    new_level.push(chunk[0].clone());
                }
            }
            
            // Move to the next level up
            nodes = new_level;
        }
        
        // The remaining node becomes the root
        self.root = nodes.into_iter().next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let mut tree = MerkleTree::new();
        
        // Test empty tree has no root
        assert_eq!(tree.root, None);
        
        // Test single item creates a root
        tree.insert("key1", "value1");
        assert!(tree.root.is_some());
        
        // Test multiple items and hash stability
        tree.insert("key2", "value2");
        let root_hash_before = tree.get_root_hash().unwrap().clone();
        
        // Verify changing a value updates the root hash
        // This is crucial for detecting changes in distributed systems
        tree.insert("key2", "new_value");
        let root_hash_after = tree.get_root_hash().unwrap().clone();
        assert_ne!(root_hash_before, root_hash_after);
        
        // Test removal operations
        tree.remove("key1");
        assert!(tree.root.is_some()); // Should still have key2
        
        // Remove the last key should result in empty tree
        tree.remove("key2");
        assert_eq!(tree.root, None);
    }
}
