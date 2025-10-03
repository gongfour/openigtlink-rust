//! STATUS message type implementation
//!
//! The STATUS message type is used to notify the receiver about the current
//! status of the sender. It can contain status code, subcode, error name,
//! and a status string.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// STATUS message containing device status information
///
/// # OpenIGTLink Specification
/// - Message type: "STATUS"
/// - Body size: Variable (30 bytes minimum + status_string length + 1)
/// - Encoding:
///   - Code: u16 (2 bytes, big-endian)
///   - Subcode: i64 (8 bytes, big-endian)
///   - Error name: 20 bytes (null-padded)
///   - Status string: variable length (null-terminated)
#[derive(Debug, Clone, PartialEq)]
pub struct StatusMessage {
    /// Status code (0 = invalid, 1 = OK, others are device-specific)
    pub code: u16,
    /// Sub-code for additional status information
    pub subcode: i64,
    /// Error name (max 20 characters)
    pub error_name: String,
    /// Status message string
    pub status_string: String,
}

impl StatusMessage {
    /// Create a new STATUS message with OK status
    pub fn ok(status_string: &str) -> Self {
        StatusMessage {
            code: 1,
            subcode: 0,
            error_name: String::new(),
            status_string: status_string.to_string(),
        }
    }

    /// Create a new STATUS message with error status
    pub fn error(error_name: &str, status_string: &str) -> Self {
        StatusMessage {
            code: 0,
            subcode: 0,
            error_name: error_name.to_string(),
            status_string: status_string.to_string(),
        }
    }
}

impl Message for StatusMessage {
    fn message_type() -> &'static str {
        "STATUS"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Encode code (2 bytes, big-endian)
        buf.put_u16(self.code);

        // Encode subcode (8 bytes, big-endian)
        buf.put_i64(self.subcode);

        // Encode error_name (20 bytes, null-padded)
        let mut name_bytes = [0u8; 20];
        let name_len = self.error_name.len().min(20);
        if name_len > 0 {
            name_bytes[..name_len].copy_from_slice(&self.error_name.as_bytes()[..name_len]);
        }
        buf.extend_from_slice(&name_bytes);

        // Encode status_string (null-terminated)
        buf.extend_from_slice(self.status_string.as_bytes());
        buf.put_u8(0); // null terminator

        Ok(buf)
    }

    fn decode_content(data: &[u8]) -> Result<Self> {
        if data.len() < 31 {
            // Minimum: 2 + 8 + 20 + 1 = 31 bytes
            return Err(IgtlError::InvalidSize {
                expected: 31,
                actual: data.len(),
            });
        }

        let mut cursor = std::io::Cursor::new(data);

        // Decode code (2 bytes, big-endian)
        let code = cursor.get_u16();

        // Decode subcode (8 bytes, big-endian)
        let subcode = cursor.get_i64();

        // Decode error_name (20 bytes, null-padded)
        let mut name_bytes = [0u8; 20];
        cursor.copy_to_slice(&mut name_bytes);
        let error_name = String::from_utf8_lossy(&name_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Decode status_string (null-terminated)
        let remaining = &data[cursor.position() as usize..];
        let status_bytes: Vec<u8> = remaining
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect();

        let status_string = String::from_utf8(status_bytes)?;

        Ok(StatusMessage {
            code,
            subcode,
            error_name,
            status_string,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(StatusMessage::message_type(), "STATUS");
    }

    #[test]
    fn test_ok_status() {
        let status = StatusMessage::ok("Operation successful");
        assert_eq!(status.code, 1);
        assert_eq!(status.subcode, 0);
        assert_eq!(status.error_name, "");
        assert_eq!(status.status_string, "Operation successful");
    }

    #[test]
    fn test_error_status() {
        let status = StatusMessage::error("ERR_TIMEOUT", "Connection timeout");
        assert_eq!(status.code, 0);
        assert_eq!(status.error_name, "ERR_TIMEOUT");
        assert_eq!(status.status_string, "Connection timeout");
    }

    #[test]
    fn test_status_roundtrip() {
        let original = StatusMessage {
            code: 1,
            subcode: 42,
            error_name: "TestError".to_string(),
            status_string: "Test status message".to_string(),
        };

        let encoded = original.encode_content().unwrap();
        let decoded = StatusMessage::decode_content(&encoded).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_empty_strings() {
        let status = StatusMessage {
            code: 1,
            subcode: 0,
            error_name: String::new(),
            status_string: String::new(),
        };

        let encoded = status.encode_content().unwrap();
        let decoded = StatusMessage::decode_content(&encoded).unwrap();

        assert_eq!(status, decoded);
    }

    #[test]
    fn test_long_error_name_truncation() {
        let long_name = "ThisIsAVeryLongErrorNameThatExceeds20Characters";
        let status = StatusMessage {
            code: 0,
            subcode: 0,
            error_name: long_name.to_string(),
            status_string: "Error".to_string(),
        };

        let encoded = status.encode_content().unwrap();
        let decoded = StatusMessage::decode_content(&encoded).unwrap();

        // Should be truncated to 20 characters
        assert_eq!(decoded.error_name.len(), 20);
        assert_eq!(&decoded.error_name, &long_name[..20]);
    }

    #[test]
    fn test_null_padding() {
        let status = StatusMessage {
            code: 1,
            subcode: 0,
            error_name: "Short".to_string(),
            status_string: "OK".to_string(),
        };

        let encoded = status.encode_content().unwrap();

        // Check that error_name field is exactly 20 bytes
        // Offset: 2 (code) + 8 (subcode) = 10
        let name_field = &encoded[10..30];
        assert_eq!(name_field.len(), 20);

        // First 5 bytes should be "Short"
        assert_eq!(&name_field[0..5], b"Short");

        // Remaining should be null-padded
        for &byte in &name_field[5..] {
            assert_eq!(byte, 0);
        }
    }

    #[test]
    fn test_null_termination() {
        let status = StatusMessage::ok("Test");
        let encoded = status.encode_content().unwrap();

        // Last byte should be null terminator
        assert_eq!(encoded.last(), Some(&0));
    }

    #[test]
    fn test_decode_invalid_size() {
        let short_data = vec![0u8; 20];
        let result = StatusMessage::decode_content(&short_data);
        assert!(matches!(result, Err(IgtlError::InvalidSize { .. })));
    }

    #[test]
    fn test_big_endian_encoding() {
        let status = StatusMessage {
            code: 0x0102,
            subcode: 0x0102030405060708,
            error_name: String::new(),
            status_string: String::new(),
        };

        let encoded = status.encode_content().unwrap();

        // Verify big-endian encoding of code
        assert_eq!(encoded[0], 0x01);
        assert_eq!(encoded[1], 0x02);

        // Verify big-endian encoding of subcode
        assert_eq!(encoded[2], 0x01);
        assert_eq!(encoded[3], 0x02);
        assert_eq!(encoded[4], 0x03);
        assert_eq!(encoded[5], 0x04);
    }
}
