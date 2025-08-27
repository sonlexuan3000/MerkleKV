//! # Protocol Parser
//!
//! This module implements a simple text-based protocol for client-server communication.
//! The protocol supports three basic operations: GET, SET, and DELETE.
//!
//! ## Protocol Format
//!
//! Commands are text-based and terminated with line endings:
//!
//! - `GET <key>` - Retrieve a value by key
//! - `SET <key> <value>` - Store a key-value pair  
//! - `DEL <key>` or `DELETE <key>` - Delete a key
//!
//! ## Example Usage
//! ```
//! GET user:123
//! SET user:123 john_doe
//! DELETE user:123
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
        // Split into at most 3 parts: command, key, and value (for SET operations)
        let parts: Vec<&str> = input.splitn(3, ' ').collect();

        if parts.is_empty() {
            return Err(anyhow!("Empty command"));
        }

        // Parse command based on the first word (case-insensitive)
        match parts[0].to_uppercase().as_str() {
            "GET" => {
                if parts.len() < 2 {
                    return Err(anyhow!("GET command requires a key"));
                }
                if parts[1].is_empty() {
                    return Err(anyhow!("GET command key cannot be empty"));
                }
                Ok(Command::Get {
                    key: parts[1].to_string(),
                })
            }
            "SET" => {
                if parts.len() < 3 {
                    return Err(anyhow!("SET command requires a key and value"));
                }
                if parts[1].is_empty() {
                    return Err(anyhow!("SET command key cannot be empty"));
                }
                Ok(Command::Set {
                    key: parts[1].to_string(),
                    value: parts[2].to_string(),
                })
            }
            // Support both "DEL" and "DELETE" for convenience
            "DEL" | "DELETE" => {
                if parts.len() < 2 {
                    return Err(anyhow!("DELETE command requires a key"));
                }
                if parts[1].is_empty() {
                    return Err(anyhow!("DELETE command key cannot be empty"));
                }
                Ok(Command::Delete {
                    key: parts[1].to_string(),
                })
            }
            _ => Err(anyhow!("Unknown command: {}", parts[0])),
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
    fn test_parse_error() {
        let protocol = Protocol::new();

        // Test various error conditions
        assert!(protocol.parse("").is_err()); // Empty command
        assert!(protocol.parse("UNKNOWN_COMMAND").is_err()); // Unknown command
        assert!(protocol.parse("GET").is_err()); // Missing key for GET
        assert!(protocol.parse("SET key").is_err()); // Missing value for SET
        assert!(protocol.parse("DELETE").is_err()); // Missing key for DELETE
        
        // Test empty key validation
        assert!(protocol.parse("GET ").is_err()); // Empty key for GET
        assert!(protocol.parse("SET  value").is_err()); // Empty key for SET
        assert!(protocol.parse("DELETE ").is_err()); // Empty key for DELETE
    }
}
