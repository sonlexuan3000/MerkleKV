use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MerkleNode {
    pub hash: Vec<u8>,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: Option<MerkleNode>,
    leaf_map: HashMap<String, Vec<u8>>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            root: None,
            leaf_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", key, value));
        let hash = hasher.finalize().to_vec();
        
        self.leaf_map.insert(key.to_string(), hash);
        self.rebuild();
    }

    pub fn remove(&mut self, key: &str) {
        self.leaf_map.remove(key);
        self.rebuild();
    }

    pub fn get_root_hash(&self) -> Option<&Vec<u8>> {
        self.root.as_ref().map(|node| &node.hash)
    }

    fn rebuild(&mut self) {
        if self.leaf_map.is_empty() {
            self.root = None;
            return;
        }
        
        let mut nodes: Vec<MerkleNode> = self
            .leaf_map
            .values()
            .map(|hash| MerkleNode {
                hash: hash.clone(),
                left: None,
                right: None,
            })
            .collect();
        
        while nodes.len() > 1 {
            let mut new_level = Vec::new();
            
            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
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
                    new_level.push(chunk[0].clone());
                }
            }
            
            nodes = new_level;
        }
        
        self.root = nodes.into_iter().next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let mut tree = MerkleTree::new();
        
        // Test empty tree
        assert_eq!(tree.root, None);
        
        // Test single item
        tree.insert("key1", "value1");
        assert!(tree.root.is_some());
        
        // Test multiple items
        tree.insert("key2", "value2");
        let root_hash_before = tree.get_root_hash().unwrap().clone();
        
        // Verify changing a value updates the root hash
        tree.insert("key2", "new_value");
        let root_hash_after = tree.get_root_hash().unwrap().clone();
        assert_ne!(root_hash_before, root_hash_after);
        
        // Test remove
        tree.remove("key1");
        assert!(tree.root.is_some());
        
        tree.remove("key2");
        assert_eq!(tree.root, None);
    }
}
