//! OpenIGTLink message trait and structures
//!
//! This module defines the common interface that all message types must implement,
//! as well as the generic message wrapper structure.

use crate::error::Result;
use crate::protocol::header::Header;
use std::collections::HashMap;

/// Common interface for all OpenIGTLink message types
///
/// Each message type (TRANSFORM, IMAGE, STATUS, etc.) must implement this trait
/// to provide encoding/decoding functionality.
pub trait Message: Sized {
    /// Returns the message type name (e.g., "TRANSFORM", "IMAGE")
    ///
    /// This must match the OpenIGTLink protocol specification.
    fn message_type() -> &'static str;

    /// Encode message content to bytes
    ///
    /// # Returns
    /// Byte vector containing the encoded message content (without header)
    fn encode_content(&self) -> Result<Vec<u8>>;

    /// Decode message content from bytes
    ///
    /// # Arguments
    /// * `data` - Byte slice containing the message content (without header)
    ///
    /// # Returns
    /// Decoded message or error
    fn decode_content(data: &[u8]) -> Result<Self>;
}

/// Complete OpenIGTLink message structure
///
/// Wraps a specific message type with header, optional extended header,
/// and optional metadata.
///
/// # Type Parameters
/// * `T` - Message type that implements the `Message` trait
#[derive(Debug)]
pub struct IgtlMessage<T: Message> {
    /// Message header (58 bytes)
    pub header: Header,
    /// Extended header (Version 3 feature, optional)
    pub extended_header: Option<Vec<u8>>,
    /// Message content
    pub content: T,
    /// Metadata as key-value pairs (Version 3 feature, optional)
    pub metadata: Option<HashMap<String, String>>,
}

impl<T: Message> IgtlMessage<T> {
    /// Create a new message with the given content and device name
    ///
    /// # Arguments
    /// * `content` - Message content
    /// * `device_name` - Device name (max 20 characters)
    ///
    /// # Returns
    /// New message with generated header
    pub fn new(content: T, device_name: &str) -> Result<Self> {
        use std::time::{SystemTime, UNIX_EPOCH};
        use crate::protocol::header::{TypeName, DeviceName};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let content_bytes = content.encode_content()?;
        let body_size = content_bytes.len() as u64;

        let header = Header {
            version: 2, // Version 2 compatible
            type_name: TypeName::new(T::message_type())?,
            device_name: DeviceName::new(device_name)?,
            timestamp,
            body_size,
            crc: 0, // Will be calculated during encode
        };

        Ok(IgtlMessage {
            header,
            extended_header: None,
            content,
            metadata: None,
        })
    }

    /// Encode the complete message to bytes
    ///
    /// This will serialize: header + content
    /// (extended_header and metadata are optional and not yet implemented)
    ///
    /// # Returns
    /// Complete message as byte vector
    pub fn encode(&self) -> Result<Vec<u8>> {
        use crate::protocol::crc::calculate_crc;

        // 1. Encode content
        let content_bytes = self.content.encode_content()?;

        // 2. Update header with correct body_size and CRC
        let mut header = self.header.clone();
        header.body_size = content_bytes.len() as u64;
        header.crc = calculate_crc(&content_bytes);

        // 3. Combine header + content
        let mut buf = Vec::with_capacity(Header::SIZE + content_bytes.len());
        buf.extend_from_slice(&header.encode());
        buf.extend_from_slice(&content_bytes);

        Ok(buf)
    }

    /// Decode a complete message from bytes
    ///
    /// # Arguments
    /// * `data` - Byte slice containing the complete message
    ///
    /// # Returns
    /// Decoded message or error
    pub fn decode(data: &[u8]) -> Result<Self> {
        use crate::protocol::crc::calculate_crc;
        use crate::error::IgtlError;

        if data.len() < Header::SIZE {
            return Err(IgtlError::InvalidSize {
                expected: Header::SIZE,
                actual: data.len(),
            });
        }

        // 1. Parse header
        let header = Header::decode(&data[..Header::SIZE])?;

        // 2. Extract content
        let content_start = Header::SIZE;
        let content_end = content_start + header.body_size as usize;

        if data.len() < content_end {
            return Err(IgtlError::InvalidSize {
                expected: content_end,
                actual: data.len(),
            });
        }

        let content_bytes = &data[content_start..content_end];

        // 3. Verify CRC
        let calculated_crc = calculate_crc(content_bytes);
        if calculated_crc != header.crc {
            return Err(IgtlError::CrcMismatch {
                expected: header.crc,
                actual: calculated_crc,
            });
        }

        // 4. Decode content
        let content = T::decode_content(content_bytes)?;

        Ok(IgtlMessage {
            header,
            extended_header: None,
            content,
            metadata: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::{TransformMessage, StatusMessage, CapabilityMessage};

    // Mock message type for testing
    struct TestMessage {
        data: Vec<u8>,
    }

    impl Message for TestMessage {
        fn message_type() -> &'static str {
            "TEST"
        }

        fn encode_content(&self) -> Result<Vec<u8>> {
            Ok(self.data.clone())
        }

        fn decode_content(data: &[u8]) -> Result<Self> {
            Ok(TestMessage {
                data: data.to_vec(),
            })
        }
    }

    #[test]
    fn test_message_trait() {
        assert_eq!(TestMessage::message_type(), "TEST");
    }

    #[test]
    fn test_message_encode_decode() {
        let original = TestMessage {
            data: vec![1, 2, 3, 4, 5],
        };

        let encoded = original.encode_content().unwrap();
        let decoded = TestMessage::decode_content(&encoded).unwrap();

        assert_eq!(original.data, decoded.data);
    }

    #[test]
    fn test_full_message_roundtrip_transform() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded).unwrap();

        // Verify header fields
        assert_eq!(decoded.header.version, 2);
        assert_eq!(decoded.header.type_name.as_str().unwrap(), "TRANSFORM");
        assert_eq!(decoded.header.device_name.as_str().unwrap(), "TestDevice");
        assert_eq!(decoded.header.body_size, 48);

        // Verify content
        assert_eq!(decoded.content, transform);
    }

    #[test]
    fn test_full_message_roundtrip_status() {
        let status = StatusMessage::ok("Operation successful");
        let msg = IgtlMessage::new(status.clone(), "StatusDevice").unwrap();

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded).unwrap();

        assert_eq!(decoded.header.type_name.as_str().unwrap(), "STATUS");
        assert_eq!(decoded.content, status);
    }

    #[test]
    fn test_full_message_roundtrip_capability() {
        let capability = CapabilityMessage::new(vec![
            "TRANSFORM".to_string(),
            "STATUS".to_string(),
            "IMAGE".to_string(),
        ]);
        let msg = IgtlMessage::new(capability.clone(), "CapDevice").unwrap();

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<CapabilityMessage>::decode(&encoded).unwrap();

        assert_eq!(decoded.header.type_name.as_str().unwrap(), "CAPABILITY");
        assert_eq!(decoded.content, capability);
    }

    #[test]
    fn test_timestamp_reasonable() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        // Timestamp should be recent (within last year and not in future)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let one_year_ago = now - (365 * 24 * 60 * 60);

        assert!(msg.header.timestamp >= one_year_ago);
        assert!(msg.header.timestamp <= now + 1); // +1 for clock skew
    }

    #[test]
    fn test_crc_verification() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        let mut encoded = msg.encode().unwrap();

        // Corrupt the content
        let content_start = Header::SIZE;
        encoded[content_start] ^= 0xFF;

        // Should fail CRC check
        let result = IgtlMessage::<TransformMessage>::decode(&encoded);
        assert!(matches!(result, Err(crate::error::IgtlError::CrcMismatch { .. })));
    }

    #[test]
    fn test_message_size_calculation() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        let encoded = msg.encode().unwrap();

        // Total size should be: Header (58) + TRANSFORM content (48) = 106
        assert_eq!(encoded.len(), 106);
    }

    #[test]
    fn test_decode_short_buffer() {
        let short_data = vec![0u8; 30];
        let result = IgtlMessage::<TransformMessage>::decode(&short_data);
        assert!(matches!(result, Err(crate::error::IgtlError::InvalidSize { .. })));
    }
}
