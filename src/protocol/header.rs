//! OpenIGTLink protocol header implementation
//!
//! The header is a fixed 58-byte structure that precedes every OpenIGTLink message.

use bytes::{Buf, BufMut, BytesMut};
use crate::error::{IgtlError, Result};

/// Type-safe wrapper for message type name (12 bytes, null-padded)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName([u8; 12]);

impl TypeName {
    /// Create a new TypeName from a string
    pub fn new(name: &str) -> Result<Self> {
        if name.len() > 12 {
            return Err(IgtlError::InvalidHeader(
                format!("Type name too long: {} bytes (max: 12)", name.len())
            ));
        }
        let mut bytes = [0u8; 12];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Ok(TypeName(bytes))
    }

    /// Get the type name as a string (trimming null bytes)
    pub fn as_str(&self) -> Result<&str> {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(12);
        std::str::from_utf8(&self.0[..len])
            .map_err(|_| IgtlError::InvalidHeader("Invalid UTF-8 in type name".to_string()))
    }
}

impl From<[u8; 12]> for TypeName {
    fn from(bytes: [u8; 12]) -> Self {
        TypeName(bytes)
    }
}

/// Type-safe wrapper for device name (20 bytes, null-padded)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceName([u8; 20]);

impl DeviceName {
    /// Create a new DeviceName from a string
    pub fn new(name: &str) -> Result<Self> {
        if name.len() > 20 {
            return Err(IgtlError::InvalidHeader(
                format!("Device name too long: {} bytes (max: 20)", name.len())
            ));
        }
        let mut bytes = [0u8; 20];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        Ok(DeviceName(bytes))
    }

    /// Get the device name as a string (trimming null bytes)
    pub fn as_str(&self) -> Result<&str> {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(20);
        std::str::from_utf8(&self.0[..len])
            .map_err(|_| IgtlError::InvalidHeader("Invalid UTF-8 in device name".to_string()))
    }
}

impl From<[u8; 20]> for DeviceName {
    fn from(bytes: [u8; 20]) -> Self {
        DeviceName(bytes)
    }
}

/// OpenIGTLink message header (58 bytes fixed size)
///
/// # Header Structure (all numerical values in big-endian)
/// - Version: u16 (2 bytes)
/// - Type: char[12] (12 bytes, null-padded)
/// - Device Name: char[20] (20 bytes, null-padded)
/// - Timestamp: u64 (8 bytes)
/// - Body Size: u64 (8 bytes)
/// - CRC: u64 (8 bytes)
#[derive(Debug, Clone)]
pub struct Header {
    /// Protocol version number (2 for version 2 and 3)
    pub version: u16,
    /// Message type name
    pub type_name: TypeName,
    /// Unique device name
    pub device_name: DeviceName,
    /// Timestamp or 0 if unused (seconds since epoch)
    pub timestamp: u64,
    /// Size of the body in bytes
    pub body_size: u64,
    /// 64-bit CRC for body data
    pub crc: u64,
}

impl Header {
    /// Header size in bytes
    pub const SIZE: usize = 58;

    /// Decode a header from a byte slice
    ///
    /// # Arguments
    /// * `buf` - Byte slice containing at least 58 bytes
    ///
    /// # Returns
    /// Decoded header or error if buffer is too short
    pub fn decode(buf: &[u8]) -> Result<Self> {
        if buf.len() < Self::SIZE {
            return Err(IgtlError::InvalidSize {
                expected: Self::SIZE,
                actual: buf.len(),
            });
        }

        let mut cursor = std::io::Cursor::new(buf);

        // Read version (2 bytes, big-endian)
        let version = cursor.get_u16();

        // Read type name (12 bytes)
        let mut type_bytes = [0u8; 12];
        cursor.copy_to_slice(&mut type_bytes);
        let type_name = TypeName::from(type_bytes);

        // Read device name (20 bytes)
        let mut device_bytes = [0u8; 20];
        cursor.copy_to_slice(&mut device_bytes);
        let device_name = DeviceName::from(device_bytes);

        // Read timestamp (8 bytes, big-endian)
        let timestamp = cursor.get_u64();

        // Read body size (8 bytes, big-endian)
        let body_size = cursor.get_u64();

        // Read CRC (8 bytes, big-endian)
        let crc = cursor.get_u64();

        Ok(Header {
            version,
            type_name,
            device_name,
            timestamp,
            body_size,
            crc,
        })
    }

    /// Encode the header into a byte vector
    ///
    /// # Returns
    /// 58-byte vector containing the encoded header
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(Self::SIZE);

        // Write version (2 bytes, big-endian)
        buf.put_u16(self.version);

        // Write type name (12 bytes)
        buf.put_slice(&self.type_name.0);

        // Write device name (20 bytes)
        buf.put_slice(&self.device_name.0);

        // Write timestamp (8 bytes, big-endian)
        buf.put_u64(self.timestamp);

        // Write body size (8 bytes, big-endian)
        buf.put_u64(self.body_size);

        // Write CRC (8 bytes, big-endian)
        buf.put_u64(self.crc);

        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_name_creation() {
        let name = TypeName::new("TRANSFORM").unwrap();
        assert_eq!(name.as_str().unwrap(), "TRANSFORM");
    }

    #[test]
    fn test_type_name_too_long() {
        let result = TypeName::new("VERY_LONG_TYPE_NAME");
        assert!(result.is_err());
    }

    #[test]
    fn test_device_name_creation() {
        let name = DeviceName::new("TestDevice").unwrap();
        assert_eq!(name.as_str().unwrap(), "TestDevice");
    }

    #[test]
    fn test_header_size() {
        assert_eq!(Header::SIZE, 58);
    }

    #[test]
    fn test_header_roundtrip() {
        let original = Header {
            version: 2,
            type_name: TypeName::new("TRANSFORM").unwrap(),
            device_name: DeviceName::new("TestDevice").unwrap(),
            timestamp: 1234567890,
            body_size: 48,
            crc: 0xDEADBEEFCAFEBABE,
        };

        let encoded = original.encode();
        assert_eq!(encoded.len(), Header::SIZE);

        let decoded = Header::decode(&encoded).unwrap();
        assert_eq!(decoded.version, original.version);
        assert_eq!(decoded.type_name, original.type_name);
        assert_eq!(decoded.device_name, original.device_name);
        assert_eq!(decoded.timestamp, original.timestamp);
        assert_eq!(decoded.body_size, original.body_size);
        assert_eq!(decoded.crc, original.crc);
    }

    #[test]
    fn test_header_decode_short_buffer() {
        let short_buf = vec![0u8; 30];
        let result = Header::decode(&short_buf);
        assert!(matches!(result, Err(IgtlError::InvalidSize { .. })));
    }

    #[test]
    fn test_big_endian_encoding() {
        let header = Header {
            version: 0x0102,
            type_name: TypeName::new("TEST").unwrap(),
            device_name: DeviceName::new("DEV").unwrap(),
            timestamp: 0x0102030405060708,
            body_size: 0x090A0B0C0D0E0F10,
            crc: 0x1112131415161718,
        };

        let encoded = header.encode();

        // Verify big-endian encoding of version
        assert_eq!(encoded[0], 0x01);
        assert_eq!(encoded[1], 0x02);

        // Verify big-endian encoding of timestamp (at offset 34)
        assert_eq!(encoded[34], 0x01);
        assert_eq!(encoded[35], 0x02);
        assert_eq!(encoded[36], 0x03);
        assert_eq!(encoded[37], 0x04);
    }
}
