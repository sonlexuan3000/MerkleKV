# MerkleKV 🚀

A high-performance, distributed key-value store with self-healing replication, built in Rust.

[![CI/CD Status](https://img.shields.io/github/actions/workflow/status/your-username/merkledb/ci.yml?branch=main&style=for-the-badge)](https://github.com/your-username/merkledb/actions)
[![Crates.io](https://img.shields.io/crates/v/merkledb.svg?style=for-the-badge)](https://crates.io/crates/merkledb)
[![License](https://img.shields.io/badge/License-Apache_2.0_OR_MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

MerkleKV is an eventually-consistent, distributed key-value database designed for speed, reliability, and operational simplicity. It uses an MQTT broker for rapid, real-time update propagation and a sophisticated **Merkle tree** anti-entropy mechanism to efficiently detect and repair data inconsistencies in the background.

---

## ✨ Key Features

* **High Performance**: Built with Rust and the Tokio asynchronous runtime for low-latency and high-throughput operations.
* **Simple Text Protocol**: An easy-to-use, Memcached-like protocol for `SET`, `GET`, and `DEL` ... operations.
* **Fast Replication**: Updates are immediately published to an MQTT topic and broadcast to all peer nodes.
* **Efficient Synchronization**: Merkle trees allow nodes to verify data integrity by comparing a single root hash. Discrepancies are located with logarithmic time complexity ($O(\log n)$) without full data scans.
* **Self-Healing**: The anti-entropy mechanism runs periodically to automatically find and fix any data drift between replicas, ensuring the cluster converges to a consistent state.
* **Memory Safety**: Guarantees provided by the Rust compiler prevent common bugs like null pointer dereferencing and data races.

---

## 🏗️ Architecture

The system consists of a cluster of `MerkleKV` nodes. There is no leader node; all nodes are peers.

1.  **Write Path**: A client writes a value to any node. That node updates its local state and publishes the change to a shared MQTT topic.
2.  **Replication (Hot Path)**: All other nodes are subscribed to the topic and apply the change as soon as they receive the message.
3.  **Synchronization (Repair Path)**: In the background, each node periodically gossips with a random peer, comparing the root hash of their Merkle trees. If the hashes differ, they efficiently traverse the trees to find and repair the exact keys that are inconsistent.



---

## 🔧 Getting Started

### Prerequisites

* **Rust Toolchain**: Install via [rustup](https://rustup.rs/).
* **MQTT Broker**: A running instance of an MQTT broker like [Mosquitto](https://mosquitto.org/) or [EMQ X](https://www.emqx.io/).

### Installation & Running

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/AI-Decenter/MerkleKV.git 
    cd MerkleKV
    ```

2.  **Build the project:**
    ```bash
    cargo build --release
    ```

3.  **Configure your nodes:**
    Create a `config.toml` file for each node you want to run. See the configuration section below for an example.

4.  **Run a node:**
    ```bash
    cargo run --release -- --config /path/to/your/node1.toml
    ```
    To see replication in action, run multiple instances on different ports, each with its own configuration file.

---

## 📚 Usage (Client API)

You can interact with a `MerkleKV` node using any TCP client, like `netcat` or `telnet`. All commands must end with `\r\n`.

```bash
telnet localhost 7878
```bash
telnet localhost 7878
```
SET: Store a key-value pair.


```
SET user:100 jane_doe\r\n
OK
```
GET: Retrieve a value by its key.
```
GET user:100\r\n
jane_doe
```
DEL: Delete a key.
```
DEL user:100\r\n
DELETED
```
## ⚙️ Configuration
The server is configured using a TOML file.

Example node1.toml:

```Ini, TOML

# Unique identifier for this node
node_id = "node-alpha"

# TCP address for client connections
bind_address = "127.0.0.1"
bind_port = 7878
# Address of the MQTT broker for replication
mqtt_broker_address = "tcp://test.mosquitto.org:1883"

```
## 🗺️ Roadmap
We have an exciting future planned for MerkleKV! Here are some features we're looking to add:

[ ] KeyValue Node: Each node is an independent server instance, holding a complete replica of the dataset.
[ ] InMemory Storage Engine: The core component responsible for the actual key-value data storage and the management of the Merkle tree structure.
[ ] Client Protocol Listener: A TCP listener that parses and handles client commands (SET, GET, DEL) using a simple, text-based protocol.
[ ] Persistent Storage: Integrate Sled for on-disk persistence.

[ ] Replication Module: Manages the connection to the MQTT broker for publishing local changes and subscribing to updates from other nodes.

[ ] Advanced Conflict Resolution: Implement Vector Clocks for more robust conflict handling.

## 🙌 Contributing
Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated.

Please feel free to open an issue or submit a pull request. For major changes, please open an issue first to discuss what you would like to change.

## 📜 License
This project is licensed under either of:

Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

