use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
}

pub struct Protocol;

impl Protocol {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> Result<Command> {
        let input = input.trim();
        let parts: Vec<&str> = input.splitn(3, ' ').collect();

        if parts.is_empty() {
            return Err(anyhow!("Empty command"));
        }

        match parts[0].to_uppercase().as_str() {
            "GET" => {
                if parts.len() < 2 {
                    return Err(anyhow!("GET command requires a key"));
                }
                Ok(Command::Get {
                    key: parts[1].to_string(),
                })
            }
            "SET" => {
                if parts.len() < 3 {
                    return Err(anyhow!("SET command requires a key and value"));
                }
                Ok(Command::Set {
                    key: parts[1].to_string(),
                    value: parts[2].to_string(),
                })
            }
            "DEL" | "DELETE" => {
                if parts.len() < 2 {
                    return Err(anyhow!("DELETE command requires a key"));
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
        assert_eq!(result, Command::Get { key: "test_key".to_string() });
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
        assert_eq!(result, Command::Delete { key: "test_key".to_string() });
    }

    #[test]
    fn test_parse_error() {
        let protocol = Protocol::new();
        assert!(protocol.parse("").is_err());
        assert!(protocol.parse("UNKNOWN_COMMAND").is_err());
        assert!(protocol.parse("GET").is_err());
        assert!(protocol.parse("SET key").is_err());
        assert!(protocol.parse("DELETE").is_err());
    }
}
