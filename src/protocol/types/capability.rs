//! CAPABILITY message type implementation
//!
//! The CAPABILITY message type is used to notify the receiver about
//! the types of messages supported by the sender.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// CAPABILITY message containing list of supported message types
///
/// # OpenIGTLink Specification
/// - Message type: "CAPABILITY"
/// - Body size: Variable (4 bytes + sum of type name lengths + null terminators)
/// - Encoding:
///   - Number of types: u32 (4 bytes, big-endian)
///   - Type names: null-terminated strings
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityMessage {
    /// List of supported message type names
    pub types: Vec<String>,
}

impl CapabilityMessage {
    /// Create a new CAPABILITY message with the given type list
    pub fn new(types: Vec<String>) -> Self {
        CapabilityMessage { types }
    }

    /// Create an empty CAPABILITY message
    pub fn empty() -> Self {
        CapabilityMessage { types: Vec::new() }
    }
}

impl Message for CapabilityMessage {
    fn message_type() -> &'static str {
        "CAPABILITY"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Encode number of types (4 bytes, big-endian)
        buf.put_u32(self.types.len() as u32);

        // Encode each type name (null-terminated)
        for type_name in &self.types {
            buf.extend_from_slice(type_name.as_bytes());
            buf.put_u8(0); // null terminator
        }

        Ok(buf)
    }

    fn decode_content(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(IgtlError::InvalidSize {
                expected: 4,
                actual: data.len(),
            });
        }

        let mut cursor = std::io::Cursor::new(data);

        // Decode number of types (4 bytes, big-endian)
        let count = cursor.get_u32() as usize;

        let mut types = Vec::with_capacity(count);
        let remaining = &data[cursor.position() as usize..];
        let mut pos = 0;

        for _ in 0..count {
            if pos >= remaining.len() {
                return Err(IgtlError::InvalidHeader(
                    "Unexpected end of data while decoding capability types".to_string(),
                ));
            }

            // Find null terminator
            let end = remaining[pos..]
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| {
                    IgtlError::InvalidHeader("Missing null terminator in capability type".to_string())
                })?;

            // Extract type name
            let type_bytes = &remaining[pos..pos + end];
            let type_name = String::from_utf8(type_bytes.to_vec())?;
            types.push(type_name);

            pos += end + 1; // +1 for null terminator
        }

        Ok(CapabilityMessage { types })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(CapabilityMessage::message_type(), "CAPABILITY");
    }

    #[test]
    fn test_empty_capability() {
        let capability = CapabilityMessage::empty();
        assert_eq!(capability.types.len(), 0);
    }

    #[test]
    fn test_new_capability() {
        let types = vec!["TRANSFORM".to_string(), "IMAGE".to_string()];
        let capability = CapabilityMessage::new(types.clone());
        assert_eq!(capability.types, types);
    }

    #[test]
    fn test_capability_roundtrip() {
        let original = CapabilityMessage {
            types: vec![
                "TRANSFORM".to_string(),
                "IMAGE".to_string(),
                "STATUS".to_string(),
            ],
        };

        let encoded = original.encode_content().unwrap();
        let decoded = CapabilityMessage::decode_content(&encoded).unwrap();

        assert_eq!(original.types, decoded.types);
    }

    #[test]
    fn test_empty_list() {
        let capability = CapabilityMessage { types: Vec::new() };

        let encoded = capability.encode_content().unwrap();
        assert_eq!(encoded.len(), 4); // Just the count field

        let decoded = CapabilityMessage::decode_content(&encoded).unwrap();
        assert_eq!(decoded.types.len(), 0);
    }

    #[test]
    fn test_single_type() {
        let capability = CapabilityMessage {
            types: vec!["TRANSFORM".to_string()],
        };

        let encoded = capability.encode_content().unwrap();
        let decoded = CapabilityMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.types.len(), 1);
        assert_eq!(decoded.types[0], "TRANSFORM");
    }

    #[test]
    fn test_multiple_types() {
        let capability = CapabilityMessage {
            types: vec![
                "TRANSFORM".to_string(),
                "IMAGE".to_string(),
                "STATUS".to_string(),
                "POSITION".to_string(),
                "CAPABILITY".to_string(),
            ],
        };

        let encoded = capability.encode_content().unwrap();
        let decoded = CapabilityMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.types, capability.types);
    }

    #[test]
    fn test_null_termination() {
        let capability = CapabilityMessage {
            types: vec!["TEST".to_string()],
        };

        let encoded = capability.encode_content().unwrap();

        // Should have: 4 bytes (count) + 4 bytes ("TEST") + 1 byte (null) = 9 bytes
        assert_eq!(encoded.len(), 9);

        // Check null terminator
        assert_eq!(encoded[8], 0);
    }

    #[test]
    fn test_big_endian_count() {
        let capability = CapabilityMessage {
            types: vec!["A".to_string(), "B".to_string()],
        };

        let encoded = capability.encode_content().unwrap();

        // Verify big-endian encoding of count (2)
        assert_eq!(encoded[0], 0x00);
        assert_eq!(encoded[1], 0x00);
        assert_eq!(encoded[2], 0x00);
        assert_eq!(encoded[3], 0x02);
    }

    #[test]
    fn test_decode_invalid_size() {
        let short_data = vec![0u8; 2];
        let result = CapabilityMessage::decode_content(&short_data);
        assert!(matches!(result, Err(IgtlError::InvalidSize { .. })));
    }

    #[test]
    fn test_decode_missing_null_terminator() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes()); // count = 1
        data.extend_from_slice(b"TEST"); // no null terminator

        let result = CapabilityMessage::decode_content(&data);
        assert!(matches!(result, Err(IgtlError::InvalidHeader(_))));
    }

    #[test]
    fn test_decode_unexpected_end() {
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_be_bytes()); // count = 2
        data.extend_from_slice(b"TEST\0"); // only one type

        let result = CapabilityMessage::decode_content(&data);
        assert!(matches!(result, Err(IgtlError::InvalidHeader(_))));
    }
}
