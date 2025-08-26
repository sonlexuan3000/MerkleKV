use anyhow::Result;
use log::{error, info};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::store::kv_engine::KvEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplicationMessage {
    pub operation: String,
    pub key: String,
    pub value: Option<String>,
    pub source_node: String,
}

pub struct Replicator {
    client: AsyncClient,
    topic_prefix: String,
    node_id: String,
}

impl Replicator {
    pub async fn new(config: &Config) -> Result<Self> {
        let mut mqtt_options = MqttOptions::new(
            &config.replication.client_id,
            &config.replication.mqtt_broker,
            config.replication.mqtt_port,
        );
        mqtt_options.set_keep_alive(Duration::from_secs(30));
        
        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
        
        // Subscribe to the replication topic
        let topic = format!("{}/events/#", config.replication.topic_prefix);
        client.subscribe(&topic, QoS::AtLeastOnce).await?;
        
        // Handle received messages in a background task
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(notification) => {
                        info!("Received MQTT notification: {:?}", notification);
                    }
                    Err(e) => {
                        error!("MQTT eventloop error: {}", e);
                        // Could implement reconnection logic here
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });
        
        Ok(Self {
            client,
            topic_prefix: config.replication.topic_prefix.clone(),
            node_id: config.replication.client_id.clone(),
        })
    }
    
    pub async fn publish_set(&self, key: &str, value: &str) -> Result<()> {
        let message = ReplicationMessage {
            operation: "SET".to_string(),
            key: key.to_string(),
            value: Some(value.to_string()),
            source_node: self.node_id.clone(),
        };
        
        let topic = format!("{}/events", self.topic_prefix);
        let payload = serde_json::to_string(&message)?;
        
        self.client.publish(&topic, QoS::AtLeastOnce, false, payload).await?;
        Ok(())
    }
    
    pub async fn publish_delete(&self, key: &str) -> Result<()> {
        let message = ReplicationMessage {
            operation: "DELETE".to_string(),
            key: key.to_string(),
            value: None,
            source_node: self.node_id.clone(),
        };
        
        let topic = format!("{}/events", self.topic_prefix);
        let payload = serde_json::to_string(&message)?;
        
        self.client.publish(&topic, QoS::AtLeastOnce, false, payload).await?;
        Ok(())
    }
    
    pub async fn start_replication_handler(
        &self,
        mut rx: mpsc::Receiver<ReplicationMessage>,
        _store: KvEngine,
    ) {
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message.operation.as_str() {
                    "SET" => {
                        if let Some(value) = &message.value {
                            // Avoid loops by checking if we're the source
                            // In a real implementation, we would update the store directly
                            info!(
                                "Replication: SET {} = {} from {}",
                                message.key, value, message.source_node
                            );
                        }
                    }
                    "DELETE" => {
                        info!("Replication: DELETE {} from {}", message.key, message.source_node);
                    }
                    _ => {
                        error!("Unknown replication operation: {}", message.operation);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    // In a real implementation, we would mock the MQTT client
    // and test the replication logic
}
