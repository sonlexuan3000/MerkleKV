//! # Protocol Parser
//!
//! This module implements a simple text-based protocol for client-server communication.
//! The protocol supports basic operations, numeric operations, and string operations.
//!
//! ## Protocol Format
//!
//! Commands are text-based and terminated with line endings:
//!
//! ### Basic Operations
//! - `GET <key>` - Retrieve a value by key
//! - `SET <key> <value>` - Store a key-value pair  
//! - `DEL <key>` or `DELETE <key>` - Delete a key
//!
//! ### Numeric Operations
//! - `INC <key> [amount]` - Increment a numeric value (default: 1)
//! - `DEC <key> [amount]` - Decrement a numeric value (default: 1)
//!
//! ### String Operations
//! - `APPEND <key> <value>` - Append value to existing string
//! - `PREPEND <key> <value>` - Prepend value to existing string
//!
//! ### Bulk Operations
//! - `MGET <key1> <key2> ... <keyN>` - Get multiple keys in one command
//! - `MSET <key1> <value1> <key2> <value2> ...` - Set multiple key-value pairs
//! - `TRUNCATE` - Clear all keys/values in the store
//!
//! ### Statistical Commands
//! - `STATS` - Return general server statistics (connections, operations, memory usage)
//! - `INFO` - Return detailed server information (version, uptime, config)
//! - `PING` - Simple health check command
//!
//! ## Example Usage
//! ```
//! GET user:123
//! SET user:123 john_doe
//! DELETE user:123
//! INC counter
//! INC counter 5
//! DEC counter 2
//! APPEND greeting " World!"
//! PREPEND greeting "Hello,"
//! MGET user:123 user:456 user:789
//! MSET user:123 john_doe user:456 jane_smith
//! TRUNCATE
//! ```
//!
//! ## Response Format
//! - Success responses: `VALUE <data>`, `OK`
//! - Error responses: `ERROR <message>`, `NOT_FOUND`

use anyhow::{anyhow, Result};

/// Represents the different commands that clients can send to the server.
///
/// Each command variant contains the necessary data to execute the operation.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncOptions {
    pub full: bool,
    pub verify: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Retrieve a value by its key
    Get {
        /// The key to look up
        key: String,
    },

    /// Store a key-value pair
    Set {
        /// The key to store
        key: String,
        /// The value to associate with the key
        value: String,
    },

    /// Delete a key-value pair
    Delete {
        /// The key to delete
        key: String,
    },
    /// Ping the server with an optional message
    Ping {
        /// The message to include in the ping response
        message: String,
    },
    /// Echo the provided message back to the client
    Echo {
        /// The message to echo back
        message: String,
    },
    /// Check if a key exists
    Exists {
        /// The key to check for existence
        keys: Vec<String>,
    },
    /// Scan for keys matching a prefix
    Scan {
        /// The prefix to scan for
        prefix: String,
    },
    /// Hash a key (not implemented)
    Hash {
        /// The key to hash
        pattern: Option<String>
    },
    /// Increment a numeric value
    Increment {
        /// The key to increment
        key: String,
        /// The amount to increment by (default: 1)
        amount: Option<i64>,
    },

    /// Decrement a numeric value
    Decrement {
        /// The key to decrement
        key: String,
        /// The amount to decrement by (default: 1)
        amount: Option<i64>,
    },

    /// Append a value to an existing string
    Append {
        /// The key to append to
        key: String,
        /// The value to append
        value: String,
    },

    /// Prepend a value to an existing string
    Prepend {
        /// The key to prepend to
        key: String,
        /// The value to prepend
        value: String,
    },

    /// Get multiple keys in one command
    MultiGet {
        /// The keys to look up
        keys: Vec<String>,
    },

    /// Set multiple key-value pairs
    MultiSet {
        /// The key-value pairs to store
        pairs: Vec<(String, String)>,
    },

    Sync {
        host: String,    
        port: u16,
        options: SyncOptions,
    },
    /// Clear all keys/values in the store
    Truncate,
    
    /// Return general server statistics (connections, operations, memory usage)
    Stats,
    
    /// Return detailed server information (version, uptime, config)
    Info,

    /// Return the current keystore size
    Dbsize,

    /// Return server version
    Version,
    
    /// Force replication of pending changes
    Flushdb,
    
    /// Gracefully shut down the server
    Shutdown,

    /// Get memory usage    
    Memory,

    /// List connected clients
    Clientlist,
}

/// Protocol parser that converts text commands into structured Command enums.
///
/// This parser is stateless and can be safely shared across threads.

pub struct Protocol;

impl Protocol {
    /// Create a new protocol parser instance.
    ///
    /// # Returns
    /// * `Protocol` - A new parser instance
    pub fn new() -> Self {
        Self
    }

    /// Parse a text command into a structured Command enum.
    ///
    /// The parser is case-insensitive for command names and handles both
    /// "DEL" and "DELETE" for deletion operations.
    ///
    /// # Arguments
    /// * `input` - The text command to parse (e.g., "GET mykey")
    ///
    /// # Returns
    /// * `Result<Command>` - Parsed command or error if invalid syntax
    ///
    /// # Errors
    /// Returns an error if:
    /// - The input is empty
    /// - The command is not recognized
    /// - Required arguments are missing
    /// - Too many arguments are provided
    ///
    /// # Example
    /// ```rust
    /// let protocol = Protocol::new();
    /// let cmd = protocol.parse("SET user:123 john_doe")?;
    /// match cmd {
    ///     Command::Set { key, value } => println!("Setting {} = {}", key, value),
    ///     _ => {}
    /// }
    /// ```
    pub fn parse(&self, input: &str) -> Result<Command> {
        let input = input.trim();
        
        // Check for empty input
        if input.is_empty() {
            return Err(anyhow!("Empty command"));
        }
        
        // Split command into parts - for SET we need to split into exactly 3 parts
        // to allow spaces in values. For GET/DELETE, we can split normally.
        let first_space = input.find(' ');
        
        if first_space.is_none() {
            // Single word command - check for invalid characters in command only
            if input.contains('\t') {
                return Err(anyhow!("Invalid character: tab character not allowed in command"));
            }
            if input.contains('\n') {
                return Err(anyhow!("Invalid character: newline character not allowed in command"));
            }
            
            match input.to_uppercase().as_str() {
                "GET" | "SET" | "DELETE" | "DEL" | "ECHO" | "EXISTS" | "SYNC" => {
                    return Err(anyhow!("{} command requires arguments", input.to_uppercase()));
                }
                "TRUNCATE" => return Ok(Command::Truncate),
                "STATS" => return Ok(Command::Stats),
                "INFO" => return Ok(Command::Info),
                "VERSION" => return Ok(Command::Version),
                "FLUSHDB" => return Ok(Command::Flushdb),
                "MEMORY" => return Ok(Command::Memory),
                "SCAN" => return Ok(Command::Scan { prefix: String::new() }),
                "HASH" => return Ok(Command::Hash { pattern: None }),
                "CLIENT" => return Ok(Command::Clientlist),
                "PING" => return Ok(Command::Ping { message: String::new() }),
                "SHUTDOWN" => return Ok(Command::Shutdown),
                "DBSIZE" => return Ok(Command::Dbsize),
                _ => return Err(anyhow!("Unknown command: {}", input)),
            }
        }

        let command = &input[..first_space.unwrap()];
        let rest = &input[first_space.unwrap() + 1..];

        // Check for invalid characters in command only
        if command.contains('\t') {
            return Err(anyhow!("Invalid character: tab character not allowed in command"));
        }
        if command.contains('\n') {
            return Err(anyhow!("Invalid character: newline character not allowed in command"));
        }

        // Parse command based on the first word (case-insensitive)
        match command.to_uppercase().as_str() {
            "GET" => {
                if rest.is_empty() {
                    return Err(anyhow!("GET command requires a key"));
                }
                if rest.contains(' ') {
                    return Err(anyhow!("GET command accepts only one argument"));
                }
                // Check for invalid characters in key
                if rest.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if rest.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                Ok(Command::Get {
                    key: rest.to_string(),
                })
            }
            "SET" => {
                let second_space = rest.find(' ');
                if second_space.is_none() {
                    return Err(anyhow!("SET command requires a key and value"));
                }
                let key = &rest[..second_space.unwrap()];
                let value = &rest[second_space.unwrap() + 1..];
                
                if key.is_empty() {
                    return Err(anyhow!("SET command key cannot be empty"));
                }
                
                // Check for invalid characters in key only (tabs allowed in values; newlines reserved for CRLF framing)
                if key.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if key.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                
                // Newlines are forbidden in values due to CRLF protocol framing
                if value.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in value"));
                }
                
                Ok(Command::Set {
                    key: key.to_string(),
                    value: value.to_string(),
                })
            }
            // Support both "DEL" and "DELETE" for convenience
            "DEL" | "DELETE" => {
                if rest.is_empty() {
                    return Err(anyhow!("DELETE command requires a key"));
                }
                if rest.contains(' ') {
                    return Err(anyhow!("DELETE command accepts only one argument"));
                }
                // Check for invalid characters in key
                if rest.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if rest.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                Ok(Command::Delete {
                    key: rest.to_string(),
                })
            }
            "DBSIZE" => {
                if !rest.is_empty() {
                    return Err(anyhow!("DBSIZE command does not accept any arguments"));
                }
                Ok(Command::Dbsize)
            }
            "PING" => {
                // Allow optional message after PING
                if rest.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in message"));
                }
                if rest.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in message"));
                }
                Ok(Command::Ping {
                    message: rest.to_string(),
                })
            }
            "ECHO" => {
                // Require a message after ECHO
                if rest.is_empty() {
                    return Err(anyhow!("ECHO command requires a message"));
                }
                if rest.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in message"));
                }
                if rest.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in message"));
                }
                Ok(Command::Echo {
                    message: rest.to_string(),
                })
            }
            "EXISTS" => {
                if rest.is_empty() {
                    return Err(anyhow!("EXISTS command requires at least one key"));
                }
                
                // Extract all keys
                let keys: Vec<String> = rest.split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                if keys.is_empty() {
                    return Err(anyhow!("EXISTS command requires at least one key"));
                }

                // Check for invalid characters in all keys
                for key in &keys {
                    if key.contains('\t') {
                        return Err(anyhow!("Invalid character: tab character not allowed in key"));
                    }
                    if key.contains('\n') {
                        return Err(anyhow!("Invalid character: newline character not allowed in key"));
                    }
                }

                Ok(Command::Exists { keys })
            }
            "SYNC" => {
                // Syntax: SYNC <host> <port> [--full] [--verify]
                // Examples:
                //   SYNC 192.168.1.10 7878
                //   SYNC example.com 7878 --full --verify
                //   SYNC [::1] 7878 --verify

                if rest.is_empty() {
                    return Err(anyhow!("SYNC requires arguments: <host> <port> [--full] [--verify]"));
                }

                // Split by ASCII whitespace
                let mut it = rest.split_whitespace();

                // --- host ---
                let host = it
                    .next()
                    .ok_or_else(|| anyhow!("SYNC requires <host> as the first argument"))?
                    .to_string();

                // Basic protocol hygiene: forbid TAB/NEWLINE inside host token
                if host.contains('\t') || host.contains('\n') {
                    return Err(anyhow!("Invalid character in host: tabs/newlines are not allowed"));
                }
                // (Optional hardening: uncomment to strictly validate DNS hostname or IPv6 literal)
                // if !(is_valid_hostname(&host) || is_ipv6_literal(&host)) {
                //     return Err(anyhow!("Invalid host format"));
                // }

                // --- port ---
                let port_str = it
                    .next()
                    .ok_or_else(|| anyhow!("SYNC requires <port> as the second argument"))?;
                if port_str.contains('\t') || port_str.contains('\n') {
                    return Err(anyhow!("Invalid character in port: tabs/newlines are not allowed"));
                }

                // TCP/UDP ports are 16-bit unsigned integers: 0..=65535 (IANA) 
                // We parse to u16 to enforce the range.
                let port: u16 = port_str
                    .parse()
                    .map_err(|_| anyhow!("Invalid port: must be an integer in 0..=65535"))?;

                // --- options ---
                // Supported flags: --full, --verify (each at most once)
                let mut opt_full = false;
                let mut opt_verify = false;

                for tok in it {
                    if tok.contains('\t') || tok.contains('\n') {
                        return Err(anyhow!("Invalid character in option: tabs/newlines are not allowed"));
                    }
                    match tok {
                        "--full" => {
                            if opt_full {
                                return Err(anyhow!("Duplicate option: --full"));
                            }
                            opt_full = true;
                        }
                        "--verify" => {
                            if opt_verify {
                                return Err(anyhow!("Duplicate option: --verify"));
                            }
                            opt_verify = true;
                        }
                        _ => {
                            // Unknown flag → hard error to surface typos early
                            return Err(anyhow!(format!("Unknown option: {}", tok)));
                        }
                    }
                }

                // Map to your command enum. Adjust the variant name/shape to your codebase.
                // Example:
                //   Command::Sync { host, port, options: SyncOptions { full: opt_full, verify: opt_verify } }
                Ok(Command::Sync {
                    host,
                    port,
                    options: SyncOptions {
                        full: opt_full,
                        verify: opt_verify,
                    },
                })
            }
            "HASH" => {
                if rest.contains(' ') {
                    return Err(anyhow!("HASH command accepts only one argument"));
                }
                // Check for invalid characters in key
                if rest.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if rest.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                Ok(Command::Hash {
                    pattern: Some(rest.to_string()),
                })
            }
            "MEMORY" => {
                if !rest.is_empty() {
                    return Err(anyhow!("MEMORY command does not accept any arguments"));
                }
                Ok(Command::Memory)
            }
            "CLIENT" => {
                let mut it = rest.split_whitespace();
                let sub = it.next().unwrap_or("").to_ascii_uppercase();
                match sub.as_str() {
                    "LIST" => Ok(Command::Clientlist),
                    _ => Err(anyhow::anyhow!("Unknown CLIENT subcommand")),
                }
            }
            "SCAN" => {
                if rest.contains(' ') {
                    return Err(anyhow!("SCAN command accepts only one argument"));
                }
                // Check for invalid characters in prefix
                if rest.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in prefix"));
                }
                if rest.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in prefix"));
                }
                Ok(Command::Scan {
                    prefix: rest.to_string(),
                })
            }
            "INC" => {
                if rest.is_empty() {
                    return Err(anyhow!("INC command requires a key"));
                }
                
                // Split the rest into key and optional amount
                let parts: Vec<&str> = rest.split_whitespace().collect();
                
                // Check if what appears to be the key is actually a number
                if parts[0].parse::<i64>().is_ok() && parts.len() == 1 {
                    return Err(anyhow!("INC command requires a key"));
                }
                
                // Check for invalid characters in key
                if parts[0].contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if parts[0].contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                
                // Parse optional amount parameter
                let amount = if parts.len() > 1 {
                    match parts[1].parse::<i64>() {
                        Ok(val) => Some(val),
                        Err(_) => return Err(anyhow!("INC command amount must be a valid number")),
                    }
                } else {
                    None // Default increment of 1 will be applied
                };
                
                Ok(Command::Increment {
                    key: parts[0].to_string(),
                    amount,
                })
            }
            "DEC" => {
                if rest.is_empty() {
                    return Err(anyhow!("DEC command requires a key"));
                }
                
                // Split the rest into key and optional amount
                let parts: Vec<&str> = rest.split_whitespace().collect();
                
                // Check if what appears to be the key is actually a number
                if parts[0].parse::<i64>().is_ok() && parts.len() == 1 {
                    return Err(anyhow!("DEC command requires a key"));
                }
                
                // Check for invalid characters in key
                if parts[0].contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if parts[0].contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                
                // Parse optional amount parameter
                let amount = if parts.len() > 1 {
                    match parts[1].parse::<i64>() {
                        Ok(val) => Some(val),
                        Err(_) => return Err(anyhow!("DEC command amount must be a valid number")),
                    }
                } else {
                    None // Default decrement of 1 will be applied
                };
                
                Ok(Command::Decrement {
                    key: parts[0].to_string(),
                    amount,
                })
            }
            "APPEND" => {
                let second_space = rest.find(' ');
                if second_space.is_none() {
                    return Err(anyhow!("APPEND command requires a key and value"));
                }
                let key = &rest[..second_space.unwrap()];
                let value = &rest[second_space.unwrap() + 1..];
                
                if key.is_empty() {
                    return Err(anyhow!("APPEND command key cannot be empty"));
                }
                
                // Check for invalid characters in key only (tabs allowed in values; newlines reserved for CRLF framing)
                if key.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if key.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                
                // Newlines are forbidden in values due to CRLF protocol framing
                if value.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in value"));
                }
                
                // Allow empty values for APPEND
                
                Ok(Command::Append {
                    key: key.to_string(),
                    value: value.to_string(),
                })
            }
            "PREPEND" => {
                let second_space = rest.find(' ');
                if second_space.is_none() {
                    return Err(anyhow!("PREPEND command requires a key and value"));
                }
                let key = &rest[..second_space.unwrap()];
                let value = &rest[second_space.unwrap() + 1..];
                
                if key.is_empty() {
                    return Err(anyhow!("PREPEND command key cannot be empty"));
                }
                
                // Check for invalid characters in key only (tabs allowed in values; newlines reserved for CRLF framing)
                if key.contains('\t') {
                    return Err(anyhow!("Invalid character: tab character not allowed in key"));
                }
                if key.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in key"));
                }
                
                // Newlines are forbidden in values due to CRLF protocol framing
                if value.contains('\n') {
                    return Err(anyhow!("Invalid character: newline character not allowed in value"));
                }
                
                // Allow empty values for PREPEND
                
                Ok(Command::Prepend {
                    key: key.to_string(),
                    value: value.to_string(),
                })
            }
            "MGET" => {
                if rest.is_empty() {
                    return Err(anyhow!("MGET command requires at least one key"));
                }
                
                // Extract all keys
                let keys: Vec<String> = rest.split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                
                if keys.is_empty() {
                    return Err(anyhow!("MGET command requires at least one key"));
                }
                
                // Check for invalid characters in all keys
                for key in &keys {
                    if key.contains('\t') {
                        return Err(anyhow!("Invalid character: tab character not allowed in key"));
                    }
                    if key.contains('\n') {
                        return Err(anyhow!("Invalid character: newline character not allowed in key"));
                    }
                }
                
                Ok(Command::MultiGet { keys })
            }
            "MSET" => {
                if rest.is_empty() {
                    return Err(anyhow!("MSET command requires at least one key-value pair"));
                }
                
                // Extract all parts
                let args: Vec<&str> = rest.split_whitespace().collect();
                
                // We need an even number of parts for key-value pairs
                if args.len() % 2 != 0 {
                    return Err(anyhow!("MSET command requires an even number of arguments (key-value pairs)"));
                }
                
                let mut pairs = Vec::new();
                let mut i = 0;
                while i < args.len() {
                    let key = args[i].to_string();
                    let value = args[i + 1].to_string();
                    
                    // Check for invalid characters in key only (values can contain control characters)
                    if key.contains('\t') {
                        return Err(anyhow!("Invalid character: tab character not allowed in key"));
                    }
                    if key.contains('\n') {
                        return Err(anyhow!("Invalid character: newline character not allowed in key"));
                    }
                    
                    pairs.push((key, value));
                    i += 2;
                }
                
                if pairs.is_empty() {
                    return Err(anyhow!("MSET command requires at least one key-value pair"));
                }
                
                Ok(Command::MultiSet { pairs })
            }
            "FLUSHDB" => {
                Ok(Command::Flushdb)
            }
            "MEMORY" => {
                Ok(Command::Memory)
            }
            "TRUNCATE" => {
                Ok(Command::Truncate)
            }
            "STATS" => {
                Ok(Command::Stats)
            }
            "INFO" => {
                Ok(Command::Info)
            }
            _ => Err(anyhow!("Unknown command: {}", command)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get() {
        let protocol = Protocol::new();
        let result = protocol.parse("GET test_key").unwrap();
        assert_eq!(
            result,
            Command::Get {
                key: "test_key".to_string()
            }
        );
    }

    #[test]
    fn test_parse_set() {
        let protocol = Protocol::new();
        let result = protocol.parse("SET test_key test_value").unwrap();
        assert_eq!(
            result,
            Command::Set {
                key: "test_key".to_string(),
                value: "test_value".to_string()
            }
        );
        
        // Test SET with value containing spaces
        let result = protocol.parse("SET key value with spaces").unwrap();
        assert_eq!(
            result,
            Command::Set {
                key: "key".to_string(),
                value: "value with spaces".to_string()
            }
        );
    }

    #[test]
    fn test_parse_delete() {
        let protocol = Protocol::new();
        let result = protocol.parse("DELETE test_key").unwrap();
        assert_eq!(
            result,
            Command::Delete {
                key: "test_key".to_string()
            }
        );
    }

    #[test]
    fn test_parse_scan() {
        let protocol = Protocol::new();
        let result = protocol.parse("SCAN test_prefix").unwrap();
        assert_eq!(
            result,
            Command::Scan {
                prefix: "test_prefix".to_string()
            }
        );
        // Test SCAN with empty prefix
        assert!(protocol.parse("SCAN").is_err());
        // Test SCAN with spaces in prefix
        assert!(protocol.parse("SCAN test prefix").is_err());
    }
    #[test]
    fn test_parse_ping() {
        let protocol = Protocol::new();
        let result = protocol.parse("PING Hello").unwrap();
        assert_eq!(
            result,
            Command::Ping {
                message: "Hello".to_string()
            }
        );
        
        // Test PING with no message
        let result = protocol.parse("PING").unwrap();
        assert_eq!(
            result,
            Command::Ping {
                message: "".to_string()
            }
        );
    }
    #[test]
    fn test_parse_echo() {
        let protocol = Protocol::new();
        let result = protocol.parse("ECHO Hello, World!").unwrap();
        assert_eq!(
            result,
            Command::Echo {
                message: "Hello, World!".to_string()
            }
        );
        
        // Test ECHO with no message (should error)
        assert!(protocol.parse("ECHO").is_err());
    }
    #[test]
    fn test_parse_dbsize() {
        let protocol = Protocol::new();
        let result = protocol.parse("DBSIZE").unwrap();
        assert_eq!(result, Command::Dbsize);
        
        // Test DBSIZE with extra arguments (should error)
        assert!(protocol.parse("DBSIZE extra_arg").is_err());
    }
    #[test]
    fn test_parse_exists() {
        let protocol = Protocol::new();
        
        // Test with a single key
        let result = protocol.parse("EXISTS key1").unwrap();
        assert_eq!(
            result,
            Command::Exists {
                keys: vec!["key1".to_string()]
            }
        );
        
        // Test with multiple keys
        let result = protocol.parse("EXISTS key1 key2 key3").unwrap();
        assert_eq!(
            result,
            Command::Exists {
                keys: vec!["key1".to_string(), "key2".to_string(), "key3".to_string()]
            }
        );
        
        // Test with no keys (should error)
        assert!(protocol.parse("EXISTS").is_err());
    }
    #[test]
    fn test_parse_memory() {
        let protocol = Protocol::new();
        let result = protocol.parse("MEMORY").unwrap();
        assert_eq!(result, Command::Memory);
        
        // Test MEMORY with extra arguments (should error)
        assert!(protocol.parse("MEMORY extra_arg").is_err());
    }
    #[test]
    fn test_parse_clientlist() {
        let protocol = Protocol::new();
        let result = protocol.parse("CLIENT LIST").unwrap();
        assert_eq!(result, Command::Clientlist);
        
        // Test CLIENT with unknown subcommand (should error)
        assert!(protocol.parse("CLIENT UNKNOWN").is_err());
        
        // Test CLIENT with no subcommand (should error)
        assert!(protocol.parse("CLIENT").is_err());
    }
    #[test]
    fn test_parse_increment() {
        let protocol = Protocol::new();
        
        // Test default increment (no amount specified)
        let result = protocol.parse("INC counter").unwrap();
        assert_eq!(
            result,
            Command::Increment {
                key: "counter".to_string(),
                amount: None
            }
        );
        
        // Test with specific amount
        let result = protocol.parse("INC counter 5").unwrap();
        assert_eq!(
            result,
            Command::Increment {
                key: "counter".to_string(),
                amount: Some(5)
            }
        );
        
        // Test with negative amount
        let result = protocol.parse("INC counter -3").unwrap();
        assert_eq!(
            result,
            Command::Increment {
                key: "counter".to_string(),
                amount: Some(-3)
            }
        );
    }

    #[test]
    fn test_parse_decrement() {
        let protocol = Protocol::new();
        
        // Test default decrement (no amount specified)
        let result = protocol.parse("DEC counter").unwrap();
        assert_eq!(
            result,
            Command::Decrement {
                key: "counter".to_string(),
                amount: None
            }
        );
        
        // Test with specific amount
        let result = protocol.parse("DEC counter 5").unwrap();
        assert_eq!(
            result,
            Command::Decrement {
                key: "counter".to_string(),
                amount: Some(5)
            }
        );
    }

    #[test]
    fn test_parse_append() {
        let protocol = Protocol::new();
        let result = protocol.parse("APPEND key_name suffix_value").unwrap();
        assert_eq!(
            result,
            Command::Append {
                key: "key_name".to_string(),
                value: "suffix_value".to_string()
            }
        );
    }

    #[test]
    fn test_parse_prepend() {
        let protocol = Protocol::new();
        let result = protocol.parse("PREPEND key_name prefix_value").unwrap();
        assert_eq!(
            result,
            Command::Prepend {
                key: "key_name".to_string(),
                value: "prefix_value".to_string()
            }
        );
    }

    #[test]
    fn test_parse_mget() {
        let protocol = Protocol::new();
        
        // Test with a single key
        let result = protocol.parse("MGET key1").unwrap();
        assert_eq!(
            result,
            Command::MultiGet {
                keys: vec!["key1".to_string()]
            }
        );
        
        // Test with multiple keys
        let result = protocol.parse("MGET key1 key2 key3").unwrap();
        assert_eq!(
            result,
            Command::MultiGet {
                keys: vec!["key1".to_string(), "key2".to_string(), "key3".to_string()]
            }
        );
    }
    
    #[test]
    fn test_parse_mset() {
        let protocol = Protocol::new();
        
        // Test with a single key-value pair
        let result = protocol.parse("MSET key1 value1").unwrap();
        assert_eq!(
            result,
            Command::MultiSet {
                pairs: vec![("key1".to_string(), "value1".to_string())]
            }
        );
        
        // Test with multiple key-value pairs
        let result = protocol.parse("MSET key1 value1 key2 value2 key3 value3").unwrap();
        assert_eq!(
            result,
            Command::MultiSet {
                pairs: vec![
                    ("key1".to_string(), "value1".to_string()),
                    ("key2".to_string(), "value2".to_string()),
                    ("key3".to_string(), "value3".to_string())
                ]
            }
        );
    }
    
    #[test]
    fn test_parse_truncate() {
        let protocol = Protocol::new();
        let result = protocol.parse("TRUNCATE").unwrap();
        assert_eq!(result, Command::Truncate);
    }
    
    #[test]
    fn test_parse_stats() {
        let protocol = Protocol::new();
        let result = protocol.parse("STATS").unwrap();
        assert_eq!(result, Command::Stats);
    }
    
    #[test]
    fn test_parse_info() {
        let protocol = Protocol::new();
        let result = protocol.parse("INFO").unwrap();
        assert_eq!(result, Command::Info);
    }
    
    #[test]
    fn test_parse_ping() {
        let protocol = Protocol::new();
        let result = protocol.parse("PING").unwrap();
        assert_eq!(result, Command::Ping);
    }
    
    #[test]
    fn test_parse_version() {
        let protocol = Protocol::new();
        let result = protocol.parse("VERSION").unwrap();
        assert_eq!(result, Command::Version);
    }
    
    #[test]
    fn test_parse_flushdb() {
        let protocol = Protocol::new();
        let result = protocol.parse("FLUSHDB").unwrap();
        assert_eq!(result, Command::Flushdb);
    }
    
    #[test]
    fn test_parse_shutdown() {
        let protocol = Protocol::new();
        let result = protocol.parse("SHUTDOWN").unwrap();
        assert_eq!(result, Command::Shutdown);
    }

    #[test]
    fn test_parse_error() {
        let protocol = Protocol::new();

        // Test various error conditions
        assert!(protocol.parse("").is_err()); // Empty command
        assert!(protocol.parse("UNKNOWN_COMMAND").is_err()); // Unknown command
        assert!(protocol.parse("GET").is_err()); // Missing key for GET
        assert!(protocol.parse("SET key").is_err()); // Missing value for SET
        assert!(protocol.parse("DELETE").is_err()); // Missing key for DELETE
        
        // Test numeric operation errors
        assert!(protocol.parse("INC").is_err()); // Missing key for INC
        assert!(protocol.parse("DEC").is_err()); // Missing key for DEC
        assert!(protocol.parse("INC  5").is_err()); // Empty key for INC
        assert!(protocol.parse("INC counter abc").is_err()); // Invalid amount for INC
        
        // Test string operation errors
        assert!(protocol.parse("APPEND").is_err()); // Missing key for APPEND
        assert!(protocol.parse("PREPEND").is_err()); // Missing key for PREPEND
        assert!(protocol.parse("APPEND key").is_err()); // Missing value for APPEND
        assert!(protocol.parse("PREPEND key").is_err()); // Missing value for PREPEND
        
        // Test bulk operation errors
        assert!(protocol.parse("MGET").is_err()); // Missing keys for MGET
        assert!(protocol.parse("MSET").is_err()); // Missing key-value pairs for MSET
        assert!(protocol.parse("MSET key").is_err()); // Odd number of arguments for MSET
        assert!(protocol.parse("MSET key1 value1 key2").is_err()); // Odd number of arguments for MSET
        
        // Test extra arguments validation
        assert!(protocol.parse("GET key extra_arg").is_err()); // Too many args for GET
        assert!(protocol.parse("DELETE key extra_arg").is_err()); // Too many args for DELETE
        assert!(protocol.parse("DEL key extra_arg").is_err()); // Too many args for DEL
        
        // Test invalid characters
        assert!(protocol.parse("GET\tkey").is_err()); // Tab character
        assert!(protocol.parse("GET\nkey").is_err()); // Newline character
    }
}
