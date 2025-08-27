Extend Protocol with Memcached-like Commands and Statistical Actions
The current protocol in MerkleKV only supports basic operations: GET, SET, and DELETE. To make the system more powerful and feature-complete, we should extend the protocol with additional Memcached-like commands and statistical operations.
Requirements
1. Add Numeric Operations
    • INC <key> [amount] - Increment a numeric value (default increment: 1)
    • DEC <key> [amount] - Decrement a numeric value (default decrement: 1)
    • APPEND <key> <value> - Append value to existing string
    • PREPEND <key> <value> - Prepend value to existing string
2. Add Bulk Operations
    • MGET <key1> <key2> ... <keyN> - Get multiple keys in one command
    • MSET <key1> <value1> <key2> <value2> ... - Set multiple key-value pairs
    • TRUNCATE - Clear all keys/values in the store
3. Add Statistical Commands
    • STATS - Return general server statistics (connections, operations, memory usage)
    • INFO - Return detailed server information (version, uptime, config)
    • PING - Simple health check command
4. Add Server Management
    • VERSION - Return server version
    • FLUSH - Force replication of pending changes
    • SHUTDOWN - Gracefully shut down the server
Implementation Guidelines
    1. Update the Command enum in protocol.rs to include new command variants
    2. Extend the parser logic to handle the new commands
    3. Implement handlers for each new command in server.rs
    4. Add comprehensive tests for each new command
    5. Update documentation to reflect the new protocol capabilities
Technical Considerations
    • All numeric operations should validate that values are valid numbers
    • Statistical operations should collect metrics from various parts of the system
    • Consider backward compatibility with existing clients
    • Update relevant integration tests
m
Medium - This enhancement will significantly improve the functionality of MerkleKV without affecting the current core functionality."
