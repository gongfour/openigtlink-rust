//! OpenIGTLink Version 3 Extended Header implementation
//!
//! The Extended Header is an optional component in OpenIGTLink Version 3 messages
//! that provides additional metadata and message identification capabilities.

use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// OpenIGTLink Version 3 Extended Header structure
///
/// The Extended Header is a fixed 12-byte structure (minimum) that appears
/// immediately after the standard 58-byte header when version >= 3.
///
/// # Structure (12 bytes minimum)
/// - extended_header_size (2 bytes) - Total size including this field and any additional fields
/// - metadata_header_size (2 bytes) - Number of metadata key-value pairs
/// - metadata_size (4 bytes) - Total metadata size in bytes
/// - message_id (4 bytes) - Unique message identifier
/// - additional_fields (variable) - Optional implementation-specific data
///
/// # C++ Compatibility
/// This structure matches the igtlMessageHeader.h ExtendedHeader format
/// from the OpenIGTLink C++ reference implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedHeader {
    /// Total size of extended header in bytes (including this field)
    /// Minimum value: 12 (for standard Extended Header)
    /// Larger values indicate additional implementation-specific fields
    pub extended_header_size: u16,

    /// Number of metadata key-value pairs that follow the message content
    /// This is a count, not a size in bytes
    pub metadata_header_size: u16,

    /// Total size of metadata section in bytes (not including this Extended Header)
    /// The metadata section appears at the end of the message body
    pub metadata_size: u32,

    /// Unique message identifier for this message
    /// Can be used for:
    /// - Request/response correlation
    /// - Message tracking and debugging
    /// - Transfer checkpointing and resumption
    pub message_id: u32,

    /// Additional implementation-specific fields (if extended_header_size > 12)
    /// These are optional and can contain custom protocol extensions
    pub additional_fields: Vec<u8>,
}

impl ExtendedHeader {
    /// Minimum Extended Header size (standard fields only)
    pub const MIN_SIZE: usize = 12;

    /// Create a new Extended Header with default values
    ///
    /// # Returns
    /// Extended Header with:
    /// - extended_header_size = 12 (minimum)
    /// - metadata_header_size = 0
    /// - metadata_size = 0
    /// - message_id = 0
    /// - no additional fields
    pub fn new() -> Self {
        ExtendedHeader {
            extended_header_size: Self::MIN_SIZE as u16,
            metadata_header_size: 0,
            metadata_size: 0,
            message_id: 0,
            additional_fields: Vec::new(),
        }
    }

    /// Create an Extended Header with metadata information
    ///
    /// # Arguments
    /// * `metadata_count` - Number of metadata key-value pairs
    /// * `metadata_size` - Total metadata size in bytes
    ///
    /// # Returns
    /// Extended Header configured for metadata
    pub fn with_metadata(metadata_count: u16, metadata_size: u32) -> Self {
        ExtendedHeader {
            extended_header_size: Self::MIN_SIZE as u16,
            metadata_header_size: metadata_count,
            metadata_size,
            message_id: 0,
            additional_fields: Vec::new(),
        }
    }

    /// Create an Extended Header with a message ID
    ///
    /// # Arguments
    /// * `message_id` - Unique message identifier
    ///
    /// # Returns
    /// Extended Header with the specified message ID
    pub fn with_message_id(message_id: u32) -> Self {
        ExtendedHeader {
            extended_header_size: Self::MIN_SIZE as u16,
            metadata_header_size: 0,
            metadata_size: 0,
            message_id,
            additional_fields: Vec::new(),
        }
    }

    /// Add additional implementation-specific fields
    ///
    /// # Arguments
    /// * `data` - Additional field data
    ///
    /// # Notes
    /// This automatically updates extended_header_size to include the additional data
    pub fn set_additional_fields(&mut self, data: Vec<u8>) {
        self.extended_header_size = (Self::MIN_SIZE + data.len()) as u16;
        self.additional_fields = data;
    }

    /// Encode the Extended Header to bytes
    ///
    /// # Returns
    /// Byte vector containing the encoded Extended Header
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.extended_header_size as usize);

        // Extended header size (2 bytes, big-endian)
        buf.put_u16(self.extended_header_size);

        // Metadata header size / count (2 bytes, big-endian)
        buf.put_u16(self.metadata_header_size);

        // Metadata size in bytes (4 bytes, big-endian)
        buf.put_u32(self.metadata_size);

        // Message ID (4 bytes, big-endian)
        buf.put_u32(self.message_id);

        // Additional fields (if any)
        buf.extend_from_slice(&self.additional_fields);

        buf
    }

    /// Decode an Extended Header from bytes
    ///
    /// # Arguments
    /// * `data` - Byte slice containing at least 12 bytes
    ///
    /// # Returns
    /// Decoded Extended Header or error
    ///
    /// # Errors
    /// Returns error if data is less than 12 bytes or if extended_header_size
    /// is less than 12 or larger than available data
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < Self::MIN_SIZE {
            return Err(IgtlError::InvalidSize {
                expected: Self::MIN_SIZE,
                actual: data.len(),
            });
        }

        let mut cursor = std::io::Cursor::new(data);

        // Read extended header size (2 bytes, big-endian)
        let extended_header_size = cursor.get_u16();

        if (extended_header_size as usize) < Self::MIN_SIZE {
            return Err(IgtlError::InvalidHeader(format!(
                "Extended header size {} is less than minimum {}",
                extended_header_size,
                Self::MIN_SIZE
            )));
        }

        if (extended_header_size as usize) > data.len() {
            return Err(IgtlError::InvalidSize {
                expected: extended_header_size as usize,
                actual: data.len(),
            });
        }

        // Read metadata header size / count (2 bytes, big-endian)
        let metadata_header_size = cursor.get_u16();

        // Read metadata size (4 bytes, big-endian)
        let metadata_size = cursor.get_u32();

        // Read message ID (4 bytes, big-endian)
        let message_id = cursor.get_u32();

        // Read additional fields (if any)
        let additional_size = extended_header_size as usize - Self::MIN_SIZE;
        let mut additional_fields = vec![0u8; additional_size];
        if additional_size > 0 {
            cursor.copy_to_slice(&mut additional_fields);
        }

        Ok(ExtendedHeader {
            extended_header_size,
            metadata_header_size,
            metadata_size,
            message_id,
            additional_fields,
        })
    }

    /// Get the total size of this Extended Header
    pub fn size(&self) -> usize {
        self.extended_header_size as usize
    }

    /// Check if this Extended Header indicates metadata is present
    pub fn has_metadata(&self) -> bool {
        self.metadata_size > 0
    }

    /// Get the metadata size in bytes (size of metadata data)
    pub fn get_metadata_size(&self) -> usize {
        self.metadata_size as usize
    }

    /// Get the metadata header size in bytes (size of metadata structure definitions)
    pub fn get_metadata_header_size(&self) -> usize {
        self.metadata_header_size as usize
    }

    /// Get the number of metadata entries (deprecated - metadata_header_size is not entry count)
    #[deprecated(note = "Use get_metadata_header_size() instead - metadata_header_size is size in bytes, not count")]
    pub fn get_metadata_count(&self) -> usize {
        self.metadata_header_size as usize
    }

    /// Get the message ID
    pub fn get_message_id(&self) -> u32 {
        self.message_id
    }
}

impl Default for ExtendedHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_extended_header() {
        let ext_header = ExtendedHeader::new();
        assert_eq!(ext_header.extended_header_size, 12);
        assert_eq!(ext_header.metadata_header_size, 0);
        assert_eq!(ext_header.metadata_size, 0);
        assert_eq!(ext_header.message_id, 0);
        assert!(ext_header.additional_fields.is_empty());
    }

    #[test]
    fn test_with_metadata() {
        let ext_header = ExtendedHeader::with_metadata(5, 128);
        assert_eq!(ext_header.metadata_header_size, 5);
        assert_eq!(ext_header.metadata_size, 128);
        assert!(ext_header.has_metadata());
        assert_eq!(ext_header.get_metadata_count(), 5);
        assert_eq!(ext_header.get_metadata_size(), 128);
    }

    #[test]
    fn test_with_message_id() {
        let ext_header = ExtendedHeader::with_message_id(12345);
        assert_eq!(ext_header.message_id, 12345);
        assert_eq!(ext_header.get_message_id(), 12345);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = ExtendedHeader {
            extended_header_size: 12,
            metadata_header_size: 3,
            metadata_size: 256,
            message_id: 98765,
            additional_fields: Vec::new(),
        };

        let encoded = original.encode();
        assert_eq!(encoded.len(), 12);

        let decoded = ExtendedHeader::decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_decode_with_additional_fields() {
        let mut original = ExtendedHeader::new();
        original.set_additional_fields(vec![0xAA, 0xBB, 0xCC, 0xDD]);

        assert_eq!(original.extended_header_size, 16); // 12 + 4

        let encoded = original.encode();
        assert_eq!(encoded.len(), 16);

        let decoded = ExtendedHeader::decode(&encoded).unwrap();
        assert_eq!(decoded, original);
        assert_eq!(decoded.additional_fields, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_decode_too_small() {
        let data = vec![0u8; 10];
        let result = ExtendedHeader::decode(&data);
        assert!(matches!(result, Err(IgtlError::InvalidSize { .. })));
    }

    #[test]
    fn test_decode_invalid_size() {
        let mut data = vec![0u8; 12];
        // Set extended_header_size to 8 (less than minimum 12)
        data[0] = 0;
        data[1] = 8;

        let result = ExtendedHeader::decode(&data);
        assert!(matches!(result, Err(IgtlError::InvalidHeader(_))));
    }

    #[test]
    fn test_size_methods() {
        let ext_header = ExtendedHeader::with_metadata(2, 64);
        assert_eq!(ext_header.size(), 12);
        assert!(ext_header.has_metadata());
    }

    #[test]
    fn test_big_endian_encoding() {
        let ext_header = ExtendedHeader {
            extended_header_size: 0x1234,
            metadata_header_size: 0x5678,
            metadata_size: 0x9ABCDEF0,
            message_id: 0x11223344,
            additional_fields: Vec::new(),
        };

        let encoded = ext_header.encode();

        // Verify big-endian encoding
        assert_eq!(encoded[0], 0x12);
        assert_eq!(encoded[1], 0x34);
        assert_eq!(encoded[2], 0x56);
        assert_eq!(encoded[3], 0x78);
        assert_eq!(encoded[4], 0x9A);
        assert_eq!(encoded[5], 0xBC);
        assert_eq!(encoded[6], 0xDE);
        assert_eq!(encoded[7], 0xF0);
        assert_eq!(encoded[8], 0x11);
        assert_eq!(encoded[9], 0x22);
        assert_eq!(encoded[10], 0x33);
        assert_eq!(encoded[11], 0x44);
    }

    #[test]
    fn test_real_world_example() {
        // Example from actual OpenIGTLink message
        let data = vec![
            0x00, 0x0C, // extended_header_size = 12
            0x00, 0x01, // metadata_header_size = 1 (1 entry)
            0x00, 0x00, 0x00, 0x14, // metadata_size = 20 bytes
            0x00, 0x00, 0x00, 0x00, // message_id = 0
        ];

        let ext_header = ExtendedHeader::decode(&data).unwrap();
        assert_eq!(ext_header.extended_header_size, 12);
        assert_eq!(ext_header.metadata_header_size, 1);
        assert_eq!(ext_header.metadata_size, 20);
        assert_eq!(ext_header.message_id, 0);
        assert!(ext_header.has_metadata());
    }
}
