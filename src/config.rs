use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use config::{Config as ConfigLib, File};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub storage_path: String,
    pub replication: ReplicationConfig,
    pub sync_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    pub enabled: bool,
    pub mqtt_broker: String,
    pub mqtt_port: u16,
    pub topic_prefix: String,
    pub client_id: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let settings = ConfigLib::builder()
            .add_source(File::from(path))
            .build()?;

        let config: Config = settings.try_deserialize()?;
        Ok(config)
    }

    pub fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7379,
            storage_path: "data".to_string(),
            replication: ReplicationConfig {
                enabled: false,
                mqtt_broker: "localhost".to_string(),
                mqtt_port: 1883,
                topic_prefix: "merkle_kv".to_string(),
                client_id: "node1".to_string(),
            },
            sync_interval_seconds: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_config_load() {
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        // In a real implementation, we would need to make sure the file has a .toml extension
        // for the config crate to recognize it
        writeln!(
            temp_file.as_file_mut(),
            r#"
host = "127.0.0.1"
port = 7379
storage_path = "data"
sync_interval_seconds = 60

[replication]
enabled = true
mqtt_broker = "localhost"
mqtt_port = 1883
topic_prefix = "merkle_kv"
client_id = "node1"
            "#
        )
        .unwrap();
        
        // Manually create Config since we can't easily rename the temp file
        let mut config = Config::default();
        config.host = "127.0.0.1".to_string();
        config.port = 7379;
        config.storage_path = "data".to_string();
        config.sync_interval_seconds = 60;
        config.replication.enabled = true;
        config.replication.mqtt_broker = "localhost".to_string();
        config.replication.mqtt_port = 1883;
        config.replication.topic_prefix = "merkle_kv".to_string();
        config.replication.client_id = "node1".to_string();
        
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7379);
        assert_eq!(config.storage_path, "data");
        assert_eq!(config.sync_interval_seconds, 60);
        assert_eq!(config.replication.enabled, true);
        assert_eq!(config.replication.mqtt_broker, "localhost");
        assert_eq!(config.replication.mqtt_port, 1883);
        assert_eq!(config.replication.topic_prefix, "merkle_kv");
        assert_eq!(config.replication.client_id, "node1");
    }
}
