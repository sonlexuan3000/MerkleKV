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


### Components 

KeyValue Node: Each node is an independent server instance, holding a complete replica of the dataset.

Storage Engine: The core component responsible for the actual key-value data storage and the management of the Merkle tree structure.

Client Protocol Listener: A TCP listener that parses and handles client commands (SET, GET, DEL) using a simple, text-based protocol.

Replication Module: Manages the connection to the MQTT broker for publishing local changes and subscribing to updates from other nodes.

Anti-Entropy Module: A background process that periodically initiates a Merkle tree comparison with other peer nodes to detect and repair any data inconsistencies.

MQTT Broker: An external message broker (e.g., Mosquitto, EMQ X) that facilitates the pub/sub communication for update propagation.
 
---

### 2. Core Data Structure: The Merkle Tree

The Merkle tree is the foundation of our efficient data verification strategy. It allows us to verify the integrity of the entire dataset by comparing a single hash, and to rapidly locate inconsistencies if they exist.

Tree Construction:

All keys in the store are sorted lexicographically.

Each (key, value) pair is hashed to form a leaf node in the tree (e.g., using hash(key + value)).

Adjacent nodes are then concatenated and hashed together to form a parent node.

This process is repeated recursively until a single root hash is generated, which represents the state of the entire dataset.

Synchronization Process:

Node A requests the root hash from Node B.

If the root hashes match, the data is considered in-sync, and the process ends.

If the hashes differ, Node A requests the children of the root node from Node B.

Node A compares the received child hashes with its own. By identifying the mismatched child hash, it knows which branch of the tree contains the inconsistency.

This process repeats, traversing down the tree only along the divergent paths. The search complexity is O(
logn).

Once a leaf node is identified as different, Node A has found the inconsistent key. It then requests the correct value from Node B and updates its local store.

## 🔧 Getting Started

### Prerequisites

* **Rust Toolchain**: Install via [rustup](https://rustup.rs/).
* **MQTT Broker**: A running instance of an MQTT broker like [Mosquitto](https://mosquitto.org/) 

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
## 🗺️ Roadmap & Implementation Issues

We have an exciting future planned for MerkleKV! Below is a comprehensive issue list for implementing the key features and architecture components described above.

### Phase 1: Core Foundation
**Priority: High** - Essential components for basic functionality

- [ ] **Issue #1: Project Structure & Build System**
  - Set up Cargo workspace and project structure
  - Configure dependencies (tokio, serde, toml, etc.)
  - Set up CI/CD pipeline and testing framework
  - Create basic configuration system

- [ ] **Issue #2: Storage Engine Foundation**
  - Implement in-memory key-value storage with HashMap
  - Add thread-safe access patterns using RwLock/Mutex
  - Create basic CRUD operations (Create, Read, Update, Delete)
  - Implement key-value serialization/deserialization

- [ ] **Issue #3: Merkle Tree Implementation**
  - Design and implement Merkle tree data structure
  - Add lexicographical key sorting functionality
  - Implement hash computation for leaf and internal nodes
  - Create root hash calculation and tree traversal methods

### Phase 2: Client Interface
**Priority: High** - User-facing functionality

- [ ] **Issue #4: TCP Protocol Listener**
  - Implement asynchronous TCP server using Tokio
  - Create command parser for SET, GET, DEL operations
  - Add protocol validation and error handling
  - Implement response formatting and client communication

- [ ] **Issue #5: Client Command Processing**
  - Integrate TCP listener with storage engine
  - Add command execution logic for all operations
  - Implement proper error responses and status codes
  - Add basic logging and monitoring capabilities

### Phase 3: Distributed System Core
**Priority: High** - Replication and consistency

- [ ] **Issue #6: MQTT Integration**
  - Integrate MQTT client library (rumqttmqtt or similar)
  - Implement connection management and reconnection logic
  - Create message publishing for local changes
  - Add subscription handling for remote updates

- [ ] **Issue #7: Replication Module**
  - Design change event format and serialization
  - Implement change publishing on local operations
  - Add remote change application logic
  - Handle basic conflict resolution (last-write-wins)

- [ ] **Issue #8: Anti-Entropy Mechanism**
  - Implement periodic peer discovery and selection
  - Create Merkle tree comparison protocol
  - Add efficient tree traversal for inconsistency detection
  - Implement repair operations for divergent data

### Phase 4: Advanced Features
**Priority: Medium** - Performance and reliability improvements

- [ ] **Issue #9: Configuration Management**
  - Implement TOML configuration parsing
  - Add runtime configuration validation
  - Create configuration hot-reloading capability
  - Add environment variable override support

- [ ] **Issue #10: Persistent Storage Backend**
  - Integrate Sled embedded database for disk persistence
  - Implement write-ahead logging (WAL) for durability
  - Add database recovery and initialization logic
  - Create storage engine abstraction layer

- [ ] **Issue #11: Performance Optimizations**
  - Implement connection pooling and reuse
  - Add batching for multiple operations
  - Optimize Merkle tree updates and caching
  - Add metrics collection and performance monitoring

### Phase 5: Production Readiness
**Priority: Low** - Enhanced robustness and features

- [ ] **Issue #12: Advanced Conflict Resolution**
  - Implement Vector Clocks for causality tracking
  - Add sophisticated conflict detection algorithms
  - Create conflict resolution strategies (manual/automatic)
  - Add conflict history and audit trails

- [ ] **Issue #13: Monitoring & Observability**
  - Implement structured logging with tracing
  - Add Prometheus metrics endpoint
  - Create health check endpoints
  - Add distributed tracing support

- [ ] **Issue #14: Security & Authentication**
  - Add TLS support for client connections
  - Implement MQTT authentication mechanisms
  - Add access control and authorization
  - Create secure configuration management

- [ ] **Issue #15: Documentation & Tooling**
  - Create comprehensive API documentation
  - Add deployment guides and examples
  - Create client libraries for popular languages
  - Add benchmarking and testing tools

## 🙌 Contributing
Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated.

Please feel free to open an issue or submit a pull request. For major changes, please open an issue first to discuss what you would like to change.

## 📜 License
This project is licensed under either of:

Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

