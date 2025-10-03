//! STRING message type implementation
//!
//! The STRING message type is used for transferring character strings.
//! It supports strings up to 65535 bytes with configurable character encoding.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// STRING message containing a text string with encoding information
///
/// # OpenIGTLink Specification
/// - Message type: "STRING"
/// - Body format: ENCODING (uint16) + LENGTH (uint16) + STRING (uint8[LENGTH])
/// - Encoding: MIBenum value (default: 3 = US-ASCII)
/// - Max length: 65535 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct StringMessage {
    /// Character encoding as MIBenum value
    ///
    /// Common values:
    /// - 3: US-ASCII (ANSI-X3.4-1968) - recommended
    /// - 106: UTF-8
    /// See: <http://www.iana.org/assignments/character-sets>
    pub encoding: u16,

    /// The text content
    pub string: String,
}

impl StringMessage {
    /// Create a new STRING message with US-ASCII encoding (default)
    pub fn new(string: impl Into<String>) -> Self {
        StringMessage {
            encoding: 3, // US-ASCII
            string: string.into(),
        }
    }

    /// Create a STRING message with UTF-8 encoding
    pub fn utf8(string: impl Into<String>) -> Self {
        StringMessage {
            encoding: 106, // UTF-8
            string: string.into(),
        }
    }

    /// Create a STRING message with custom encoding
    pub fn with_encoding(encoding: u16, string: impl Into<String>) -> Self {
        StringMessage {
            encoding,
            string: string.into(),
        }
    }

    /// Get the string content as a reference
    pub fn as_str(&self) -> &str {
        &self.string
    }

    /// Get the length of the string in bytes
    pub fn len(&self) -> usize {
        self.string.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }
}

impl Message for StringMessage {
    fn message_type() -> &'static str {
        "STRING"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let string_bytes = self.string.as_bytes();
        let length = string_bytes.len();

        if length > 65535 {
            return Err(IgtlError::BodyTooLarge {
                size: length,
                max: 65535,
            });
        }

        let mut buf = Vec::with_capacity(4 + length);

        // Encode ENCODING (uint16)
        buf.put_u16(self.encoding);

        // Encode LENGTH (uint16)
        buf.put_u16(length as u16);

        // Encode STRING bytes
        buf.extend_from_slice(string_bytes);

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(IgtlError::InvalidSize {
                expected: 4,
                actual: data.len(),
            });
        }

        // Decode ENCODING
        let encoding = data.get_u16();

        // Decode LENGTH
        let length = data.get_u16() as usize;

        // Check remaining data size
        if data.len() < length {
            return Err(IgtlError::InvalidSize {
                expected: length,
                actual: data.len(),
            });
        }

        // Decode STRING
        let string_bytes = &data[..length];
        let string = String::from_utf8(string_bytes.to_vec())?;

        Ok(StringMessage { encoding, string })
    }
}

impl From<&str> for StringMessage {
    fn from(s: &str) -> Self {
        StringMessage::new(s)
    }
}

impl From<String> for StringMessage {
    fn from(s: String) -> Self {
        StringMessage::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(StringMessage::message_type(), "STRING");
    }

    #[test]
    fn test_new() {
        let msg = StringMessage::new("Hello");
        assert_eq!(msg.encoding, 3); // US-ASCII
        assert_eq!(msg.string, "Hello");
    }

    #[test]
    fn test_utf8() {
        let msg = StringMessage::utf8("こんにちは");
        assert_eq!(msg.encoding, 106); // UTF-8
        assert_eq!(msg.string, "こんにちは");
    }

    #[test]
    fn test_with_encoding() {
        let msg = StringMessage::with_encoding(42, "Test");
        assert_eq!(msg.encoding, 42);
        assert_eq!(msg.string, "Test");
    }

    #[test]
    fn test_as_str() {
        let msg = StringMessage::new("Test");
        assert_eq!(msg.as_str(), "Test");
    }

    #[test]
    fn test_len() {
        let msg = StringMessage::new("Hello");
        assert_eq!(msg.len(), 5);
    }

    #[test]
    fn test_is_empty() {
        let msg1 = StringMessage::new("");
        assert!(msg1.is_empty());

        let msg2 = StringMessage::new("test");
        assert!(!msg2.is_empty());
    }

    #[test]
    fn test_encode_simple() {
        let msg = StringMessage::new("Test");
        let encoded = msg.encode_content().unwrap();

        // Check header: encoding (2 bytes) + length (2 bytes)
        assert_eq!(encoded[0..2], [0, 3]); // Encoding = 3 (US-ASCII)
        assert_eq!(encoded[2..4], [0, 4]); // Length = 4
        assert_eq!(&encoded[4..], b"Test");
    }

    #[test]
    fn test_roundtrip_ascii() {
        let original = StringMessage::new("Hello, World!");
        let encoded = original.encode_content().unwrap();
        let decoded = StringMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.encoding, original.encoding);
        assert_eq!(decoded.string, original.string);
    }

    #[test]
    fn test_roundtrip_utf8() {
        let original = StringMessage::utf8("Hello 世界 🌍");
        let encoded = original.encode_content().unwrap();
        let decoded = StringMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.encoding, original.encoding);
        assert_eq!(decoded.string, original.string);
    }

    #[test]
    fn test_empty_string() {
        let msg = StringMessage::new("");
        let encoded = msg.encode_content().unwrap();
        let decoded = StringMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.string, "");
        assert_eq!(encoded.len(), 4); // Only header, no content
    }

    #[test]
    fn test_max_length() {
        let long_string = "A".repeat(65535);
        let msg = StringMessage::new(long_string.clone());
        let encoded = msg.encode_content().unwrap();
        let decoded = StringMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.string, long_string);
    }

    #[test]
    fn test_too_long() {
        let too_long = "A".repeat(65536);
        let msg = StringMessage::new(too_long);
        let result = msg.encode_content();

        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0, 3]; // Only 2 bytes, need at least 4
        let result = StringMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_truncated() {
        let mut data = vec![0, 3, 0, 10]; // Encoding=3, Length=10
        data.extend_from_slice(b"Short"); // Only 5 bytes instead of 10

        let result = StringMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str() {
        let msg: StringMessage = "Test".into();
        assert_eq!(msg.string, "Test");
        assert_eq!(msg.encoding, 3);
    }

    #[test]
    fn test_from_string() {
        let s = String::from("Test");
        let msg: StringMessage = s.into();
        assert_eq!(msg.string, "Test");
        assert_eq!(msg.encoding, 3);
    }

    #[test]
    fn test_big_endian_encoding() {
        let msg = StringMessage::new("X");
        let encoded = msg.encode_content().unwrap();

        // Encoding = 3: should be [0x00, 0x03] in big-endian
        assert_eq!(encoded[0], 0x00);
        assert_eq!(encoded[1], 0x03);

        // Length = 1: should be [0x00, 0x01] in big-endian
        assert_eq!(encoded[2], 0x00);
        assert_eq!(encoded[3], 0x01);
    }
}
