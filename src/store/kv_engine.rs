use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct KvEngine {
    data: Arc<HashMap<String, String>>,
    // In a real implementation, this would use a persistent storage engine like Sled
    // storage_path: PathBuf,
}

impl KvEngine {
    pub fn new(_storage_path: &str) -> Result<Self> {
        // In a real implementation, we would initialize the storage engine here
        // and load existing data if available
        Ok(Self {
            data: Arc::new(HashMap::new()),
        })
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    pub fn set(&mut self, key: String, value: String) {
        // Create a new HashMap with the updated value
        let mut new_data = (*self.data).clone();
        new_data.insert(key, value);
        self.data = Arc::new(new_data);
    }

    pub fn delete(&mut self, key: &str) {
        // Create a new HashMap without the deleted key
        let mut new_data = (*self.data).clone();
        new_data.remove(key);
        self.data = Arc::new(new_data);
    }

    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_kv_operations() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap();
        
        let mut engine = KvEngine::new(storage_path).unwrap();
        
        // Test set and get
        engine.set("key1".to_string(), "value1".to_string());
        assert_eq!(engine.get("key1"), Some("value1".to_string()));
        
        // Test overwrite
        engine.set("key1".to_string(), "new_value".to_string());
        assert_eq!(engine.get("key1"), Some("new_value".to_string()));
        
        // Test delete
        engine.delete("key1");
        assert_eq!(engine.get("key1"), None);
        
        // Test keys
        engine.set("key2".to_string(), "value2".to_string());
        engine.set("key3".to_string(), "value3".to_string());
        
        let keys = engine.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }
}
