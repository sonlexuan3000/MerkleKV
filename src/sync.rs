//! sync.rs — Minimal, easy-to-read Merkle-based one-way sync (local ← remote)
//
//! What this does (high-level):
//! 1) Build a Merkle snapshot from the *local* store (via scan("") + get()).
//! 2) Build a Merkle snapshot from the *remote* peer (via SCAN + GET).
//! 3) Compare the two trees with `diff_keys()` to find differing keys.
//! 4) Apply changes so that local == remote (set/update or delete).
//!
//! Notes:
//! - This is intentionally simple (no prefix fanout).
//! - Wire format matches your server.rs today:
//!     SCAN <prefix>   →  "KEYS <n>\r\n<k1>\r\n...<kn>\r\n"
//!     GET <key>       →  "VALUE <plain>\r\n"   or "NOT_FOUND\r\n"
//! - MerkleTree expects &str values, so we assume UTF-8 strings.
//!
//! How the SYNC command handler should call this:
//!     let mut mgr = sync_manager.lock().await;
//!     match mgr.sync_once(&host, port).await { Ok(_) => "OK", Err(e) => ... };

use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
    time,
};

use crate::config::Config;
use crate::store::merkle::MerkleTree;
use crate::store::KVEngineStoreTrait;

pub struct SyncManager {
    /// Shared storage used by all connections and the sync process
    store: Arc<Mutex<Box<dyn KVEngineStoreTrait + Send + Sync>>>,
    /// Optional background interval
    sync_interval: Duration,
}

impl SyncManager {
    /// Construct a SyncManager that uses the *same* shared store as the server.
    pub fn new_with_shared_store(
        cfg: &Config,
        store: Arc<Mutex<Box<dyn KVEngineStoreTrait + Send + Sync>>>,
    ) -> Self {
        Self {
            store,
            sync_interval: Duration::from_secs(cfg.sync_interval_seconds),
        }
    }

    /// One-shot sync: make local data equal to remote data.
    pub async fn sync_once(&mut self, host: &str, port: u16) -> Result<()> {
        let addr = format!("{host}:{port}");
        info!("SYNC (Merkle diff) → {}", addr);

        // 1) Local snapshot
        let (local_tree, _local_map) = self.build_local_merkle_snapshot().await;

        // 2) Remote snapshot
        let (remote_tree, remote_map) = self.build_remote_merkle_snapshot(&addr).await?;

        // 3) Diff
        let diffs = local_tree.diff_keys(&remote_tree);
        if diffs.is_empty() {
            info!("SYNC: already identical (no diff)");
            return Ok(());
        }

        // 4) Apply changes: local := remote
        let mut guard = self.store.lock().await;
        for k in diffs {
            if let Some(rv) = remote_map.get(&k) {
                // set / overwrite
                let _ = guard.set(k.clone(), rv.clone());
            } else {
                // missing remotely → delete local
                let _ = guard.delete(&k);
            }
        }

        info!("SYNC: done (local now matches remote)");
        Ok(())
    }

    /// Optional background loop (best-effort).
    pub async fn start_sync_loop(&mut self, host: String, port: u16) {
        let mut interval = time::interval(self.sync_interval);
        let addr = format!("{host}:{port}");
        loop {
            interval.tick().await;
            if let Err(e) = self.sync_once(host.as_str(), port).await {
                log::warn!("background sync with {} failed: {}", addr, e);
            }
        }
    }

    // ───────────── Snapshots ─────────────

    /// Build local Merkle snapshot from scan("") + get().
    async fn build_local_merkle_snapshot(&self) -> (MerkleTree, HashMap<String, String>) {
        let mut t = MerkleTree::new();
        let mut map = HashMap::new();

        let mut guard = self.store.lock().await;
        let keys = guard.scan(""); // empty prefix → all keys
        for k in keys {
            if let Some(v) = guard.get(&k) {
                t.insert(&k, &v);
                map.insert(k, v);
            }
        }
        drop(guard);

        (t, map)
    }

    /// Build remote Merkle snapshot by calling SCAN + GET over TCP.
    async fn build_remote_merkle_snapshot(
        &self,
        addr: &str,
    ) -> Result<(MerkleTree, HashMap<String, String>)> {
        let keys = self.read_remote_keys_via_scan(addr).await?;
        let mut t = MerkleTree::new();
        let mut map = HashMap::new();

        for k in keys {
            match self.read_remote_value_plain(addr, &k).await? {
                Some(v) => {
                    t.insert(&k, &v);
                    map.insert(k, v);
                }
                None => {
                    // Key disappeared between SCAN and GET; just skip.
                }
            }
        }

        Ok((t, map))
    }

    // ───────────── Wire I/O ─────────────

    /// SCAN (all keys): send "SCAN \r\n" → expect:
    ///  "KEYS <n>\r\n"
    ///   then n lines, each is one key.
    async fn read_remote_keys_via_scan(&self, addr: &str) -> Result<Vec<String>> {
        debug!("→ {} : SCAN", addr);
        let mut stream = TcpStream::connect(addr)
            .await
            .with_context(|| format!("connect {}", addr))?;
        stream.write_all(b"SCAN \r\n").await.context("write SCAN")?;

        let mut reader = BufReader::new(stream);

        // First line: "KEYS <n>\r\n"
        let mut header = String::new();
        let n = reader.read_line(&mut header).await?;
        if n == 0 {
            return Err(anyhow!("peer closed while reading SCAN header"));
        }
        let header = header.trim_end();
        let mut it = header.split_whitespace();
        if it.next() != Some("KEYS") {
            return Err(anyhow!("unexpected SCAN response: {}", header));
        }
        let count: usize = it
            .next()
            .ok_or_else(|| anyhow!("missing count after KEYS"))?
            .parse()
            .context("invalid count after KEYS")?;

        // Next <count> lines: keys
        let mut keys = Vec::with_capacity(count);
        for _ in 0..count {
            let mut line = String::new();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("peer closed while reading key list"));
            }
            keys.push(line.trim_end().to_string());
        }

        debug!("← {} : KEYS {}", addr, keys.len());
        Ok(keys)
    }

    /// GET one key: "GET <key>\r\n" → "VALUE <plain>\r\n" or "NOT_FOUND\r\n"
    async fn read_remote_value_plain(&self, addr: &str, key: &str) -> Result<Option<String>> {
        let cmd = format!("GET {key}\r\n");
        debug!("→ {} : {}", addr, cmd.trim_end());
        let mut stream = TcpStream::connect(addr)
            .await
            .with_context(|| format!("connect {}", addr))?;
        stream.write_all(cmd.as_bytes()).await.context("write GET")?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Err(anyhow!("peer closed on GET {}", key));
        }
        let line = line.trim_end();
        if line == "NOT_FOUND" {
            return Ok(None);
        }
        if let Some(rest) = line.strip_prefix("VALUE ") {
            return Ok(Some(rest.to_string()));
        }
        Err(anyhow!("unexpected GET response for {key}: {}", line))
    }
}
