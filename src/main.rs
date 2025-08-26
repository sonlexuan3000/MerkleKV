use anyhow::Result;
use std::path::PathBuf;

mod config;
mod protocol;
mod replication;
mod server;
mod store;
mod sync;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Load configuration
    let config_path = PathBuf::from("config.toml");
    let config = config::Config::load(&config_path)?;
    
    // Initialize services and start the server
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    
    runtime.block_on(async {
        let store = store::kv_engine::KvEngine::new(&config.storage_path)?;
        let server = server::Server::new(config.clone(), store);
        server.run().await
    })
}
