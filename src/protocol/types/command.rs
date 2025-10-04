//! COMMAND message type implementation
//!
//! The COMMAND message type is used to transfer command strings structured in XML.
//! It provides command ID and name fields for referencing messages.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// Size of command name field
const COMMAND_NAME_SIZE: usize = 20;

/// COMMAND message containing command data with ID and name
///
/// # OpenIGTLink Specification
/// - Message type: "COMMAND"
/// - Body format: COMMAND_ID (uint32) + COMMAND_NAME (char[20]) + ENCODING (uint16) + LENGTH (uint32) + COMMAND (uint8[LENGTH])
/// - Character encoding: MIBenum value (default: 3 = US-ASCII)
#[derive(Debug, Clone, PartialEq)]
pub struct CommandMessage {
    /// Unique ID of this command
    pub command_id: u32,

    /// Name of the command (max 20 chars)
    pub command_name: String,

    /// Character encoding as MIBenum value
    /// Common values:
    /// - 3: US-ASCII (default)
    /// - 106: UTF-8
    pub encoding: u16,

    /// The command string (often XML)
    pub command: String,
}

impl CommandMessage {
    /// Create a new COMMAND message with US-ASCII encoding
    pub fn new(command_id: u32, command_name: impl Into<String>, command: impl Into<String>) -> Self {
        CommandMessage {
            command_id,
            command_name: command_name.into(),
            encoding: 3, // US-ASCII
            command: command.into(),
        }
    }

    /// Create a COMMAND message with UTF-8 encoding
    pub fn utf8(command_id: u32, command_name: impl Into<String>, command: impl Into<String>) -> Self {
        CommandMessage {
            command_id,
            command_name: command_name.into(),
            encoding: 106, // UTF-8
            command: command.into(),
        }
    }

    /// Create a COMMAND message with custom encoding
    pub fn with_encoding(
        command_id: u32,
        command_name: impl Into<String>,
        encoding: u16,
        command: impl Into<String>,
    ) -> Self {
        CommandMessage {
            command_id,
            command_name: command_name.into(),
            encoding,
            command: command.into(),
        }
    }

    /// Get the command string as a reference
    pub fn as_str(&self) -> &str {
        &self.command
    }
}

impl Message for CommandMessage {
    fn message_type() -> &'static str {
        "COMMAND"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let command_bytes = self.command.as_bytes();
        let command_len = command_bytes.len();

        let mut buf = Vec::with_capacity(4 + COMMAND_NAME_SIZE + 2 + 4 + command_len);

        // Encode COMMAND_ID (uint32)
        buf.put_u32(self.command_id);

        // Encode COMMAND_NAME (char[20])
        let mut name_bytes = [0u8; COMMAND_NAME_SIZE];
        let name_str = self.command_name.as_bytes();
        let copy_len = name_str.len().min(COMMAND_NAME_SIZE - 1);
        name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
        buf.extend_from_slice(&name_bytes);

        // Encode ENCODING (uint16)
        buf.put_u16(self.encoding);

        // Encode LENGTH (uint32)
        buf.put_u32(command_len as u32);

        // Encode COMMAND bytes
        buf.extend_from_slice(command_bytes);

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 4 + COMMAND_NAME_SIZE + 2 + 4 {
            return Err(IgtlError::InvalidSize {
                expected: 4 + COMMAND_NAME_SIZE + 2 + 4,
                actual: data.len(),
            });
        }

        // Decode COMMAND_ID
        let command_id = data.get_u32();

        // Decode COMMAND_NAME (char[20])
        let name_bytes = &data[..COMMAND_NAME_SIZE];
        data.advance(COMMAND_NAME_SIZE);

        let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(COMMAND_NAME_SIZE);
        let command_name = String::from_utf8(name_bytes[..name_len].to_vec())?;

        // Decode ENCODING
        let encoding = data.get_u16();

        // Decode LENGTH
        let length = data.get_u32() as usize;

        // Check remaining data size
        if data.len() < length {
            return Err(IgtlError::InvalidSize {
                expected: length,
                actual: data.len(),
            });
        }

        // Decode COMMAND
        let command_bytes = &data[..length];
        let command = String::from_utf8(command_bytes.to_vec())?;

        Ok(CommandMessage {
            command_id,
            command_name,
            encoding,
            command,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(CommandMessage::message_type(), "COMMAND");
    }

    #[test]
    fn test_new() {
        let msg = CommandMessage::new(1, "START", "<cmd>start</cmd>");
        assert_eq!(msg.command_id, 1);
        assert_eq!(msg.command_name, "START");
        assert_eq!(msg.encoding, 3);
        assert_eq!(msg.command, "<cmd>start</cmd>");
    }

    #[test]
    fn test_utf8() {
        let msg = CommandMessage::utf8(2, "STOP", "<cmd>停止</cmd>");
        assert_eq!(msg.encoding, 106);
    }

    #[test]
    fn test_with_encoding() {
        let msg = CommandMessage::with_encoding(3, "TEST", 42, "<test/>");
        assert_eq!(msg.encoding, 42);
    }

    #[test]
    fn test_as_str() {
        let msg = CommandMessage::new(1, "CMD", "test");
        assert_eq!(msg.as_str(), "test");
    }

    #[test]
    fn test_encode_simple() {
        let msg = CommandMessage::new(100, "START", "GO");
        let encoded = msg.encode_content().unwrap();

        // Check command ID (first 4 bytes)
        assert_eq!(u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]), 100);

        // Check encoding field (at offset 4 + 20 = 24)
        assert_eq!(u16::from_be_bytes([encoded[24], encoded[25]]), 3);

        // Check length field (at offset 26)
        assert_eq!(u32::from_be_bytes([encoded[26], encoded[27], encoded[28], encoded[29]]), 2);
    }

    #[test]
    fn test_roundtrip_simple() {
        let original = CommandMessage::new(42, "TEST", "Hello");
        let encoded = original.encode_content().unwrap();
        let decoded = CommandMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.command_id, original.command_id);
        assert_eq!(decoded.command_name, original.command_name);
        assert_eq!(decoded.encoding, original.encoding);
        assert_eq!(decoded.command, original.command);
    }

    #[test]
    fn test_roundtrip_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><command><action>start</action></command>"#;
        let original = CommandMessage::new(1, "XML_CMD", xml);
        let encoded = original.encode_content().unwrap();
        let decoded = CommandMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.command, xml);
    }

    #[test]
    fn test_roundtrip_utf8() {
        let original = CommandMessage::utf8(5, "日本語", "こんにちは世界");
        let encoded = original.encode_content().unwrap();
        let decoded = CommandMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.command_name, original.command_name);
        assert_eq!(decoded.command, original.command);
    }

    #[test]
    fn test_name_truncation() {
        let long_name = "ThisIsAVeryLongCommandNameThatExceedsTwentyCharacters";
        let msg = CommandMessage::new(1, long_name, "test");
        let encoded = msg.encode_content().unwrap();
        let decoded = CommandMessage::decode_content(&encoded).unwrap();

        assert!(decoded.command_name.len() < 20);
    }

    #[test]
    fn test_empty_command() {
        let msg = CommandMessage::new(0, "EMPTY", "");
        let encoded = msg.encode_content().unwrap();
        let decoded = CommandMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.command, "");
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 10]; // Too short
        let result = CommandMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_truncated_command() {
        let mut data = vec![0u8; 30]; // Header only
        // Set LENGTH to 10 at offset 26
        data[26..30].copy_from_slice(&10u32.to_be_bytes());
        // But don't provide the 10 bytes of command data

        let result = CommandMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
