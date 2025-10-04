//! BIND message type implementation
//!
//! The BIND message is used to bind multiple OpenIGTLink messages into a single message.
//! This allows grouping related messages together for synchronized transmission.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::Buf;

/// Child message entry in BIND message
#[derive(Debug, Clone, PartialEq)]
pub struct BindEntry {
    /// Message type (max 12 chars)
    pub message_type: String,
    /// Device name (max 20 chars)
    pub device_name: String,
}

impl BindEntry {
    /// Create a new bind entry
    pub fn new(message_type: impl Into<String>, device_name: impl Into<String>) -> Self {
        BindEntry {
            message_type: message_type.into(),
            device_name: device_name.into(),
        }
    }
}

/// BIND message for grouping multiple messages
///
/// # OpenIGTLink Specification
/// - Message type: "BIND"
/// - Format: (TYPE (char[12]) + NAME (char[20])) * n
/// - Each entry: 32 bytes
/// - Number of child messages determined by body size / 32
#[derive(Debug, Clone, PartialEq)]
pub struct BindMessage {
    /// List of child message entries
    pub entries: Vec<BindEntry>,
}

impl BindMessage {
    /// Create a new BIND message
    pub fn new(entries: Vec<BindEntry>) -> Self {
        BindMessage { entries }
    }

    /// Create an empty BIND message
    pub fn empty() -> Self {
        BindMessage { entries: Vec::new() }
    }

    /// Add a child message entry
    pub fn add_entry(&mut self, entry: BindEntry) {
        self.entries.push(entry);
    }

    /// Add a child message by type and name
    pub fn add(&mut self, message_type: impl Into<String>, device_name: impl Into<String>) {
        self.entries.push(BindEntry::new(message_type, device_name));
    }

    /// Get number of child messages
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if message has no children
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Message for BindMessage {
    fn message_type() -> &'static str {
        "BIND"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.entries.len() * 32);

        for entry in &self.entries {
            // Encode TYPE (char[12])
            let mut type_bytes = [0u8; 12];
            let type_str = entry.message_type.as_bytes();
            let copy_len = type_str.len().min(12);
            type_bytes[..copy_len].copy_from_slice(&type_str[..copy_len]);
            buf.extend_from_slice(&type_bytes);

            // Encode NAME (char[20])
            let mut name_bytes = [0u8; 20];
            let name_str = entry.device_name.as_bytes();
            let copy_len = name_str.len().min(20);
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        let mut entries = Vec::new();

        while data.len() >= 32 {
            // Decode TYPE (char[12])
            let type_bytes = &data[..12];
            data.advance(12);
            let type_len = type_bytes.iter().position(|&b| b == 0).unwrap_or(12);
            let message_type = String::from_utf8(type_bytes[..type_len].to_vec())?;

            // Decode NAME (char[20])
            let name_bytes = &data[..20];
            data.advance(20);
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let device_name = String::from_utf8(name_bytes[..name_len].to_vec())?;

            entries.push(BindEntry {
                message_type,
                device_name,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(BindMessage { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(BindMessage::message_type(), "BIND");
    }

    #[test]
    fn test_empty() {
        let msg = BindMessage::empty();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn test_new_entry() {
        let entry = BindEntry::new("TRANSFORM", "Device1");
        assert_eq!(entry.message_type, "TRANSFORM");
        assert_eq!(entry.device_name, "Device1");
    }

    #[test]
    fn test_add_entry() {
        let mut msg = BindMessage::empty();
        msg.add_entry(BindEntry::new("STATUS", "Device2"));
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_add() {
        let mut msg = BindMessage::empty();
        msg.add("TRANSFORM", "Device1");
        msg.add("STATUS", "Device2");
        assert_eq!(msg.len(), 2);
    }

    #[test]
    fn test_encode_single() {
        let msg = BindMessage::new(vec![
            BindEntry::new("TRANSFORM", "Device1"),
        ]);
        let encoded = msg.encode_content().unwrap();

        // Each entry is 32 bytes
        assert_eq!(encoded.len(), 32);
    }

    #[test]
    fn test_encode_multiple() {
        let msg = BindMessage::new(vec![
            BindEntry::new("TRANSFORM", "Device1"),
            BindEntry::new("STATUS", "Device2"),
            BindEntry::new("POSITION", "Device3"),
        ]);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 96); // 3 * 32
    }

    #[test]
    fn test_roundtrip_single() {
        let original = BindMessage::new(vec![
            BindEntry::new("TRANSFORM", "SurgicalTool"),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = BindMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.entries[0].message_type, "TRANSFORM");
        assert_eq!(decoded.entries[0].device_name, "SurgicalTool");
    }

    #[test]
    fn test_roundtrip_multiple() {
        let original = BindMessage::new(vec![
            BindEntry::new("TRANSFORM", "Device1"),
            BindEntry::new("STATUS", "Device2"),
            BindEntry::new("POSITION", "Device3"),
            BindEntry::new("SENSOR", "Device4"),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = BindMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.entries.len(), 4);
        assert_eq!(decoded.entries[0].message_type, "TRANSFORM");
        assert_eq!(decoded.entries[1].message_type, "STATUS");
        assert_eq!(decoded.entries[2].message_type, "POSITION");
        assert_eq!(decoded.entries[3].message_type, "SENSOR");
    }

    #[test]
    fn test_empty_message() {
        let msg = BindMessage::empty();
        let encoded = msg.encode_content().unwrap();
        let decoded = BindMessage::decode_content(&encoded).unwrap();

        assert!(decoded.is_empty());
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 31]; // One byte short
        let result = BindMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_long_names_truncated() {
        let msg = BindMessage::new(vec![
            BindEntry::new("VERYLONGMESSAGETYPE", "VERYLONGDEVICENAMEOVER20CHARS"),
        ]);
        let encoded = msg.encode_content().unwrap();
        let decoded = BindMessage::decode_content(&encoded).unwrap();

        // Should be truncated to 12 and 20 chars respectively
        assert!(decoded.entries[0].message_type.len() <= 12);
        assert!(decoded.entries[0].device_name.len() <= 20);
    }
}
