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

        // Check for invalid characters (tabs, newlines within the command)
        if input.contains('\t') {
            return Err(anyhow!("Invalid character: tab character not allowed"));
        }
        if input.contains('\n') && input.trim() != input.trim_end_matches('\n') {
            return Err(anyhow!("Invalid character: newline character not allowed"));
        }

        // Split command into parts - for SET we need to split into exactly 3 parts
        // to allow spaces in values. For GET/DELETE, we can split normally.
        let first_space = input.find(' ');
        
        if first_space.is_none() {
            // Single word command
            match input.to_uppercase().as_str() {
                "GET" | "SET" | "DELETE" | "DEL" => {
                    return Err(anyhow!("{} command requires arguments", input.to_uppercase()));
                }
                _ => return Err(anyhow!("Unknown command: {}", input)),
            }
        }

        let command = &input[..first_space.unwrap()];
        let rest = &input[first_space.unwrap() + 1..];

        // Parse command based on the first word (case-insensitive)
        match command.to_uppercase().as_str() {
            "GET" => {
                if rest.is_empty() {
                    return Err(anyhow!("GET command requires a key"));
                }
                if rest.contains(' ') {
                    return Err(anyhow!("GET command accepts only one argument"));
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
                Ok(Command::Delete {
                    key: rest.to_string(),
                })
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

        // Test extra arguments validation
        assert!(protocol.parse("GET key extra_arg").is_err()); // Too many args for GET
        assert!(protocol.parse("DELETE key extra_arg").is_err()); // Too many args for DELETE
        assert!(protocol.parse("DEL key extra_arg").is_err()); // Too many args for DEL
        // Note: SET can have spaces in values, so "SET key value extra_arg" is valid
        
        // Test invalid characters
        assert!(protocol.parse("GET\tkey").is_err()); // Tab character
        assert!(protocol.parse("GET\nkey").is_err()); // Newline character
    }
}
