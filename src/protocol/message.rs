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
        use crate::protocol::header::{DeviceName, Timestamp, TypeName};

        let timestamp = Timestamp::now();

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

    /// Set extended header data (Version 3 feature)
    ///
    /// When extended header is set, the message version is automatically upgraded to 3.
    ///
    /// # Arguments
    /// * `data` - Extended header data as byte vector
    ///
    /// # Examples
    /// ```no_run
    /// # use openigtlink_rust::protocol::{IgtlMessage, types::TransformMessage};
    /// let transform = TransformMessage::identity();
    /// let mut msg = IgtlMessage::new(transform, "Device").unwrap();
    /// msg.set_extended_header(vec![0x01, 0x02, 0x03, 0x04]);
    /// ```
    pub fn set_extended_header(&mut self, data: Vec<u8>) {
        self.extended_header = Some(data);
        // Upgrade to version 3 when extended header is used
        if self.header.version < 3 {
            self.header.version = 3;
        }
    }

    /// Get extended header data reference (Version 3 feature)
    ///
    /// # Returns
    /// Optional reference to extended header bytes
    pub fn get_extended_header(&self) -> Option<&[u8]> {
        self.extended_header.as_deref()
    }

    /// Remove extended header and optionally downgrade to Version 2
    pub fn clear_extended_header(&mut self) {
        self.extended_header = None;
        // Downgrade to version 2 if no version 3 features are used
        if self.metadata.is_none() && self.header.version == 3 {
            self.header.version = 2;
        }
    }

    /// Set metadata key-value pairs (Version 3 feature)
    ///
    /// When metadata is set, the message version is automatically upgraded to 3.
    ///
    /// # Arguments
    /// * `metadata` - HashMap of key-value pairs
    ///
    /// # Examples
    /// ```no_run
    /// # use openigtlink_rust::protocol::{IgtlMessage, types::TransformMessage};
    /// # use std::collections::HashMap;
    /// let transform = TransformMessage::identity();
    /// let mut msg = IgtlMessage::new(transform, "Device").unwrap();
    /// let mut metadata = HashMap::new();
    /// metadata.insert("priority".to_string(), "high".to_string());
    /// msg.set_metadata(metadata);
    /// ```
    pub fn set_metadata(&mut self, metadata: HashMap<String, String>) {
        self.metadata = Some(metadata);
        // Upgrade to version 3 when metadata is used
        if self.header.version < 3 {
            self.header.version = 3;
        }
    }

    /// Add a single metadata key-value pair (Version 3 feature)
    ///
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - Metadata value
    pub fn add_metadata(&mut self, key: String, value: String) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
            if self.header.version < 3 {
                self.header.version = 3;
            }
        }
        self.metadata.as_mut().unwrap().insert(key, value);
    }

    /// Get metadata reference (Version 3 feature)
    ///
    /// # Returns
    /// Optional reference to metadata HashMap
    pub fn get_metadata(&self) -> Option<&HashMap<String, String>> {
        self.metadata.as_ref()
    }

    /// Remove metadata and optionally downgrade to Version 2
    pub fn clear_metadata(&mut self) {
        self.metadata = None;
        // Downgrade to version 2 if no version 3 features are used
        if self.extended_header.is_none() && self.header.version == 3 {
            self.header.version = 2;
        }
    }

    /// Encode the complete message to bytes
    ///
    /// Version 2 format: Header (58) + Content
    /// Version 3 format: Header (58) + ExtHdrSize (2) + ExtHdr (var) + Content + Metadata (var)
    ///
    /// Metadata format (Version 3):
    /// - MetadataSize (2 bytes, big-endian)
    /// - For each pair:
    ///   - KeySize (2 bytes)
    ///   - Key (KeySize bytes, UTF-8)
    ///   - ValueSize (2 bytes)
    ///   - Value (ValueSize bytes, UTF-8)
    ///
    /// # Returns
    /// Complete message as byte vector
    pub fn encode(&self) -> Result<Vec<u8>> {
        use crate::protocol::crc::calculate_crc;

        // 1. Encode content
        let content_bytes = self.content.encode_content()?;

        // 2. Encode metadata if present
        let metadata_bytes = if self.header.version >= 3 && self.metadata.is_some() {
            let metadata = self.metadata.as_ref().unwrap();
            let mut meta_buf = Vec::new();

            // Metadata header size (2 bytes)
            let meta_header_size = (metadata.len() as u16).to_be_bytes();
            meta_buf.extend_from_slice(&meta_header_size);

            // Each key-value pair
            for (key, value) in metadata.iter() {
                // Key size and data
                let key_bytes = key.as_bytes();
                meta_buf.extend_from_slice(&(key_bytes.len() as u16).to_be_bytes());
                meta_buf.extend_from_slice(key_bytes);

                // Value size and data
                let value_bytes = value.as_bytes();
                meta_buf.extend_from_slice(&(value_bytes.len() as u16).to_be_bytes());
                meta_buf.extend_from_slice(value_bytes);
            }

            meta_buf
        } else {
            Vec::new()
        };

        // 3. Build body based on version
        let body_bytes = if self.header.version >= 3 && self.extended_header.is_some() {
            // Version 3 with extended header
            let ext_header = self.extended_header.as_ref().unwrap();
            let ext_header_size = ext_header.len() as u16;

            let mut body = Vec::with_capacity(
                2 + ext_header.len() + content_bytes.len() + metadata_bytes.len(),
            );
            // Extended header size (2 bytes, big-endian)
            body.extend_from_slice(&ext_header_size.to_be_bytes());
            // Extended header data
            body.extend_from_slice(ext_header);
            // Content
            body.extend_from_slice(&content_bytes);
            // Metadata
            body.extend_from_slice(&metadata_bytes);

            body
        } else if self.header.version >= 3 && !metadata_bytes.is_empty() {
            // Version 3 without extended header but with metadata
            let mut body = Vec::with_capacity(2 + content_bytes.len() + metadata_bytes.len());
            // Extended header size = 0
            body.extend_from_slice(&0u16.to_be_bytes());
            // Content
            body.extend_from_slice(&content_bytes);
            // Metadata
            body.extend_from_slice(&metadata_bytes);

            body
        } else {
            // Version 2 or Version 3 without extended header/metadata
            content_bytes
        };

        // 4. Update header with correct body_size and CRC
        let mut header = self.header.clone();
        header.body_size = body_bytes.len() as u64;
        header.crc = calculate_crc(&body_bytes);

        // 5. Combine header + body
        let mut buf = Vec::with_capacity(Header::SIZE + body_bytes.len());
        buf.extend_from_slice(&header.encode());
        buf.extend_from_slice(&body_bytes);

        Ok(buf)
    }

    /// Decode a complete message from bytes with CRC verification
    ///
    /// Automatically detects Version 2 or Version 3 format based on header.
    ///
    /// # Arguments
    /// * `data` - Byte slice containing the complete message
    ///
    /// # Returns
    /// Decoded message or error
    pub fn decode(data: &[u8]) -> Result<Self> {
        Self::decode_with_options(data, true)
    }

    /// Decode a complete message from bytes with optional CRC verification
    ///
    /// Allows skipping CRC verification for performance in trusted environments.
    ///
    /// # Arguments
    /// * `data` - Byte slice containing the complete message
    /// * `verify_crc` - Whether to verify CRC (true = verify, false = skip)
    ///
    /// # Returns
    /// Decoded message or error
    ///
    /// # Safety
    /// Disabling CRC verification (`verify_crc = false`) should only be done in
    /// trusted environments where data corruption is unlikely (e.g., loopback, local network).
    /// Using this in production over unreliable networks may lead to silent data corruption.
    ///
    /// # Examples
    /// ```no_run
    /// # use openigtlink_rust::protocol::{IgtlMessage, types::TransformMessage};
    /// # let data = vec![0u8; 106];
    /// // Decode with CRC verification (recommended)
    /// let msg = IgtlMessage::<TransformMessage>::decode_with_options(&data, true)?;
    ///
    /// // Decode without CRC verification (use with caution)
    /// let msg_fast = IgtlMessage::<TransformMessage>::decode_with_options(&data, false)?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn decode_with_options(data: &[u8], verify_crc: bool) -> Result<Self> {
        use crate::error::IgtlError;
        use crate::protocol::crc::calculate_crc;

        if data.len() < Header::SIZE {
            return Err(IgtlError::InvalidSize {
                expected: Header::SIZE,
                actual: data.len(),
            });
        }

        // 1. Parse header
        let header = Header::decode(&data[..Header::SIZE])?;

        // 2. Extract body
        let body_start = Header::SIZE;
        let body_end = body_start + header.body_size as usize;

        if data.len() < body_end {
            return Err(IgtlError::InvalidSize {
                expected: body_end,
                actual: data.len(),
            });
        }

        let body_bytes = &data[body_start..body_end];

        // 3. Verify CRC (if requested)
        if verify_crc {
            let calculated_crc = calculate_crc(body_bytes);
            if calculated_crc != header.crc {
                return Err(IgtlError::CrcMismatch {
                    expected: header.crc,
                    actual: calculated_crc,
                });
            }
        }

        // 4. Parse body based on version
        let (extended_header, remaining_bytes, has_ext_header_field) =
            if header.version >= 3 && body_bytes.len() >= 2 {
                // Try to parse as Version 3 with extended header
                let ext_header_size = u16::from_be_bytes([body_bytes[0], body_bytes[1]]) as usize;

                if ext_header_size > 0 && body_bytes.len() >= 2 + ext_header_size {
                    // Version 3 with non-empty extended header
                    let ext_header_data = body_bytes[2..2 + ext_header_size].to_vec();
                    let content_start = 2 + ext_header_size;
                    (Some(ext_header_data), &body_bytes[content_start..], true)
                } else if ext_header_size == 0 && body_bytes.len() >= 2 {
                    // Version 3 with empty extended header (size field = 0)
                    (Some(Vec::new()), &body_bytes[2..], true)
                } else {
                    // Invalid extended header size
                    return Err(IgtlError::InvalidSize {
                        expected: 2 + ext_header_size,
                        actual: body_bytes.len(),
                    });
                }
            } else {
                // Version 2 - entire body is content
                (None, body_bytes, false)
            };

        // 5. Try to determine content size and parse metadata (Version 3 only)
        let (content_bytes, metadata) = if header.version >= 3 && has_ext_header_field {
            // For Version 3 with ext_header_size field (whether 0 or not)
            // We need to figure out where content ends and metadata begins
            // Strategy: Try to decode content. If successful, re-encode to find actual size.

            // First, try decoding the entire remaining_bytes as content
            match T::decode_content(remaining_bytes) {
                Ok(content) => {
                    // Content decoded successfully
                    // Re-encode to find actual content size
                    let actual_content_size = content.encode_content()?.len();

                    if remaining_bytes.len() > actual_content_size {
                        // There's metadata after the content
                        let content_part = &remaining_bytes[..actual_content_size];
                        let metadata_part = &remaining_bytes[actual_content_size..];

                        // Parse metadata
                        let parsed_metadata = Self::decode_metadata(metadata_part)?;
                        (content_part, parsed_metadata)
                    } else {
                        // No metadata
                        (remaining_bytes, None)
                    }
                }
                Err(IgtlError::InvalidSize { expected, .. })
                    if remaining_bytes.len() > expected =>
                {
                    // Content decode failed due to size mismatch (fixed-size message with metadata)
                    // This means we have: Content (expected bytes) + Metadata (rest)
                    let content_part = &remaining_bytes[..expected];
                    let metadata_part = &remaining_bytes[expected..];

                    // Verify content decodes correctly
                    if T::decode_content(content_part).is_ok() {
                        // Parse metadata
                        let parsed_metadata = Self::decode_metadata(metadata_part)?;
                        (content_part, parsed_metadata)
                    } else {
                        // Content still doesn't decode - treat all as content
                        (remaining_bytes, None)
                    }
                }
                Err(_) => {
                    // Some other error - treat all as content
                    (remaining_bytes, None)
                }
            }
        } else {
            // Version 2 - no metadata
            (remaining_bytes, None)
        };

        // 6. Decode content
        let content = T::decode_content(content_bytes)?;

        Ok(IgtlMessage {
            header,
            extended_header,
            content,
            metadata,
        })
    }

    /// Decode metadata from bytes (helper function)
    fn decode_metadata(data: &[u8]) -> Result<Option<HashMap<String, String>>> {
        use crate::error::IgtlError;

        if data.len() < 2 {
            return Ok(None);
        }

        let metadata_count = u16::from_be_bytes([data[0], data[1]]) as usize;

        if metadata_count == 0 {
            return Ok(None);
        }

        let mut metadata = HashMap::new();
        let mut offset = 2;

        for _ in 0..metadata_count {
            // Read key size
            if offset + 2 > data.len() {
                return Err(IgtlError::InvalidSize {
                    expected: offset + 2,
                    actual: data.len(),
                });
            }
            let key_size = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            // Read key
            if offset + key_size > data.len() {
                return Err(IgtlError::InvalidSize {
                    expected: offset + key_size,
                    actual: data.len(),
                });
            }
            let key = String::from_utf8(data[offset..offset + key_size].to_vec())?;
            offset += key_size;

            // Read value size
            if offset + 2 > data.len() {
                return Err(IgtlError::InvalidSize {
                    expected: offset + 2,
                    actual: data.len(),
                });
            }
            let value_size = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            // Read value
            if offset + value_size > data.len() {
                return Err(IgtlError::InvalidSize {
                    expected: offset + value_size,
                    actual: data.len(),
                });
            }
            let value = String::from_utf8(data[offset..offset + value_size].to_vec())?;
            offset += value_size;

            metadata.insert(key, value);
        }

        Ok(Some(metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::{CapabilityMessage, StatusMessage, TransformMessage};

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
            .as_secs() as u32;

        let one_year_ago = now - (365 * 24 * 60 * 60);

        assert!(msg.header.timestamp.seconds >= one_year_ago);
        assert!(msg.header.timestamp.seconds <= now + 1); // +1 for clock skew
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
        assert!(matches!(
            result,
            Err(crate::error::IgtlError::CrcMismatch { .. })
        ));
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
        assert!(matches!(
            result,
            Err(crate::error::IgtlError::InvalidSize { .. })
        ));
    }

    // Version 3 Extended Header Tests

    #[test]
    fn test_extended_header_set_get() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        // Initially no extended header
        assert_eq!(msg.get_extended_header(), None);
        assert_eq!(msg.header.version, 2);

        // Set extended header
        let ext_header = vec![0x01, 0x02, 0x03, 0x04];
        msg.set_extended_header(ext_header.clone());

        // Should upgrade to version 3
        assert_eq!(msg.header.version, 3);
        assert_eq!(msg.get_extended_header(), Some(ext_header.as_slice()));
    }

    #[test]
    fn test_extended_header_clear() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        // Set extended header
        msg.set_extended_header(vec![0x01, 0x02, 0x03, 0x04]);
        assert_eq!(msg.header.version, 3);

        // Clear extended header
        msg.clear_extended_header();
        assert_eq!(msg.get_extended_header(), None);
        // Should downgrade to version 2 (no metadata either)
        assert_eq!(msg.header.version, 2);
    }

    #[test]
    fn test_version3_encode_decode_with_extended_header() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        // Add extended header
        let ext_header = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        msg.set_extended_header(ext_header.clone());

        // Encode
        let encoded = msg.encode().unwrap();

        // Decode
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded).unwrap();

        // Verify version
        assert_eq!(decoded.header.version, 3);

        // Verify extended header
        assert_eq!(decoded.get_extended_header(), Some(ext_header.as_slice()));

        // Verify content
        assert_eq!(decoded.content, transform);
    }

    #[test]
    fn test_version3_encode_decode_without_extended_header() {
        let status = StatusMessage::ok("Test message");
        let msg = IgtlMessage::new(status.clone(), "TestDevice").unwrap();

        // Encode as Version 2
        let encoded = msg.encode().unwrap();

        // Decode
        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded).unwrap();

        // Should remain version 2
        assert_eq!(decoded.header.version, 2);
        assert_eq!(decoded.get_extended_header(), None);
        assert_eq!(decoded.content, status);
    }

    #[test]
    fn test_extended_header_body_size_calculation() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        // Add extended header
        let ext_header = vec![0x01, 0x02, 0x03, 0x04]; // 4 bytes
        msg.set_extended_header(ext_header);

        let encoded = msg.encode().unwrap();

        // Total size should be:
        // Header (58) + ExtHdrSize (2) + ExtHdr (4) + TRANSFORM content (48) = 112
        assert_eq!(encoded.len(), 112);

        // Verify body_size in header includes extended header overhead
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded).unwrap();
        assert_eq!(decoded.header.body_size, 2 + 4 + 48); // ExtHdrSize + ExtHdr + Content
    }

    #[test]
    fn test_extended_header_empty() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        // Set empty extended header
        msg.set_extended_header(vec![]);

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded).unwrap();

        // Should still be version 3
        assert_eq!(decoded.header.version, 3);
        // Extended header should be empty
        assert_eq!(decoded.get_extended_header(), Some(&[] as &[u8]));
        // Content should be intact
        assert_eq!(decoded.content, transform);
    }

    #[test]
    fn test_extended_header_large() {
        let status = StatusMessage::ok("Test");
        let mut msg = IgtlMessage::new(status.clone(), "TestDevice").unwrap();

        // Create large extended header (1 KB)
        let ext_header = vec![0xAB; 1024];
        msg.set_extended_header(ext_header.clone());

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded).unwrap();

        assert_eq!(decoded.header.version, 3);
        assert_eq!(decoded.get_extended_header(), Some(ext_header.as_slice()));
        assert_eq!(decoded.content, status);
    }

    #[test]
    fn test_version3_crc_includes_extended_header() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        msg.set_extended_header(vec![0x01, 0x02, 0x03, 0x04]);

        let mut encoded = msg.encode().unwrap();

        // Corrupt extended header
        encoded[Header::SIZE + 2] ^= 0xFF; // First byte of extended header

        // Should fail CRC check
        let result = IgtlMessage::<TransformMessage>::decode(&encoded);
        assert!(matches!(
            result,
            Err(crate::error::IgtlError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn test_backward_compatibility_version2() {
        // Create a Version 2 message
        let capability = CapabilityMessage::new(vec!["TRANSFORM".to_string()]);
        let msg = IgtlMessage::new(capability.clone(), "Device").unwrap();

        assert_eq!(msg.header.version, 2);

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<CapabilityMessage>::decode(&encoded).unwrap();

        // Should decode correctly as Version 2
        assert_eq!(decoded.header.version, 2);
        assert_eq!(decoded.get_extended_header(), None);
        assert_eq!(decoded.content, capability);
    }

    // Version 3 Metadata Tests

    #[test]
    fn test_metadata_set_get() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        // Initially no metadata
        assert_eq!(msg.get_metadata(), None);
        assert_eq!(msg.header.version, 2);

        // Set metadata
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("sequence".to_string(), "42".to_string());
        msg.set_metadata(metadata.clone());

        // Should upgrade to version 3
        assert_eq!(msg.header.version, 3);
        assert_eq!(msg.get_metadata(), Some(&metadata));
    }

    #[test]
    fn test_metadata_add() {
        let status = StatusMessage::ok("Test");
        let mut msg = IgtlMessage::new(status, "TestDevice").unwrap();

        assert_eq!(msg.header.version, 2);

        // Add metadata one by one
        msg.add_metadata("key1".to_string(), "value1".to_string());
        assert_eq!(msg.header.version, 3);

        msg.add_metadata("key2".to_string(), "value2".to_string());

        let metadata = msg.get_metadata().unwrap();
        assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_metadata_clear() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        // Set metadata
        msg.add_metadata("test".to_string(), "value".to_string());
        assert_eq!(msg.header.version, 3);

        // Clear metadata
        msg.clear_metadata();
        assert_eq!(msg.get_metadata(), None);
        // Should downgrade to version 2 (no extended header either)
        assert_eq!(msg.header.version, 2);
    }

    #[test]
    fn test_version3_encode_decode_with_metadata() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        // Add metadata
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("timestamp".to_string(), "123456".to_string());
        msg.set_metadata(metadata.clone());

        // Encode
        let encoded = msg.encode().unwrap();

        // Decode
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded).unwrap();

        // Verify version
        assert_eq!(decoded.header.version, 3);

        // Verify metadata
        let decoded_metadata = decoded.get_metadata().unwrap();
        assert_eq!(decoded_metadata.get("priority"), Some(&"high".to_string()));
        assert_eq!(
            decoded_metadata.get("timestamp"),
            Some(&"123456".to_string())
        );

        // Verify content
        assert_eq!(decoded.content, transform);
    }

    #[test]
    fn test_version3_with_extended_header_and_metadata() {
        let status = StatusMessage::ok("Test message");
        let mut msg = IgtlMessage::new(status.clone(), "TestDevice").unwrap();

        // Add both extended header and metadata
        msg.set_extended_header(vec![0xAA, 0xBB, 0xCC, 0xDD]);
        msg.add_metadata("key1".to_string(), "value1".to_string());
        msg.add_metadata("key2".to_string(), "value2".to_string());

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded).unwrap();

        // Verify version
        assert_eq!(decoded.header.version, 3);

        // Verify extended header
        let expected_ext_header: &[u8] = &[0xAA, 0xBB, 0xCC, 0xDD];
        assert_eq!(decoded.get_extended_header(), Some(expected_ext_header));

        // Verify metadata
        let metadata = decoded.get_metadata().unwrap();
        assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));

        // Verify content
        assert_eq!(decoded.content, status);
    }

    #[test]
    fn test_metadata_empty() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        // Set empty metadata
        msg.set_metadata(HashMap::new());

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded).unwrap();

        // Should not have metadata (empty HashMap)
        assert_eq!(decoded.get_metadata(), None);
        assert_eq!(decoded.content, transform);
    }

    #[test]
    fn test_metadata_utf8_values() {
        let status = StatusMessage::ok("Test");
        let mut msg = IgtlMessage::new(status.clone(), "TestDevice").unwrap();

        // Add UTF-8 metadata
        msg.add_metadata("name".to_string(), "日本語".to_string());
        msg.add_metadata("emoji".to_string(), "🎉✨".to_string());

        let encoded = msg.encode().unwrap();
        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded).unwrap();

        let metadata = decoded.get_metadata().unwrap();
        assert_eq!(metadata.get("name"), Some(&"日本語".to_string()));
        assert_eq!(metadata.get("emoji"), Some(&"🎉✨".to_string()));
    }

    // CRC Verification Tests

    #[test]
    fn test_decode_with_crc_verification_enabled() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        let encoded = msg.encode().unwrap();

        // Should decode successfully with CRC verification
        let decoded = IgtlMessage::<TransformMessage>::decode_with_options(&encoded, true).unwrap();
        assert_eq!(decoded.content, transform);
    }

    #[test]
    fn test_decode_with_crc_verification_disabled() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        let mut encoded = msg.encode().unwrap();

        // Corrupt the data
        encoded[Header::SIZE] ^= 0xFF;

        // Should fail with CRC verification enabled
        let result_with_crc = IgtlMessage::<TransformMessage>::decode_with_options(&encoded, true);
        assert!(matches!(
            result_with_crc,
            Err(crate::error::IgtlError::CrcMismatch { .. })
        ));

        // Should succeed with CRC verification disabled (even with corrupted data)
        let result_without_crc =
            IgtlMessage::<TransformMessage>::decode_with_options(&encoded, false);
        assert!(result_without_crc.is_ok());
    }

    #[test]
    fn test_decode_default_uses_crc_verification() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();

        let mut encoded = msg.encode().unwrap();

        // Corrupt the data
        encoded[Header::SIZE] ^= 0xFF;

        // Default decode() should verify CRC and fail
        let result = IgtlMessage::<TransformMessage>::decode(&encoded);
        assert!(matches!(
            result,
            Err(crate::error::IgtlError::CrcMismatch { .. })
        ));
    }

    #[test]
    fn test_crc_skip_performance() {
        // This test demonstrates that skipping CRC works correctly
        let status = StatusMessage::ok("Performance test");
        let msg = IgtlMessage::new(status.clone(), "TestDevice").unwrap();

        let encoded = msg.encode().unwrap();

        // Both should decode to the same content (when data is not corrupted)
        let decoded_with_crc =
            IgtlMessage::<StatusMessage>::decode_with_options(&encoded, true).unwrap();
        let decoded_without_crc =
            IgtlMessage::<StatusMessage>::decode_with_options(&encoded, false).unwrap();

        assert_eq!(decoded_with_crc.content, decoded_without_crc.content);
        assert_eq!(decoded_with_crc.content, status);
    }

    #[test]
    fn test_version3_crc_skip_with_extended_header() {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform.clone(), "TestDevice").unwrap();

        msg.set_extended_header(vec![0x01, 0x02, 0x03, 0x04]);

        let encoded = msg.encode().unwrap();

        // Should work with CRC disabled
        let decoded =
            IgtlMessage::<TransformMessage>::decode_with_options(&encoded, false).unwrap();
        assert_eq!(decoded.header.version, 3);
        let expected: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        assert_eq!(decoded.get_extended_header(), Some(expected));
        assert_eq!(decoded.content, transform);
    }

    #[test]
    fn test_version3_crc_skip_with_metadata() {
        let status = StatusMessage::ok("Test");
        let mut msg = IgtlMessage::new(status.clone(), "TestDevice").unwrap();

        msg.add_metadata("key1".to_string(), "value1".to_string());
        msg.add_metadata("key2".to_string(), "value2".to_string());

        let encoded = msg.encode().unwrap();

        // Should work with CRC disabled
        let decoded = IgtlMessage::<StatusMessage>::decode_with_options(&encoded, false).unwrap();
        assert_eq!(decoded.header.version, 3);

        let metadata = decoded.get_metadata().unwrap();
        assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));
        assert_eq!(decoded.content, status);
    }
}
