//! OpenIGTLink protocol header implementation
//!
//! The header is a fixed 58-byte structure that precedes every OpenIGTLink message.

use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut, BytesMut};

/// Type-safe wrapper for message type name (12 bytes, null-padded)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName([u8; 12]);

impl TypeName {
    /// Create a new TypeName from a string
    pub fn new(name: &str) -> Result<Self> {
        if name.len() > 12 {
            return Err(IgtlError::InvalidHeader(format!(
                "Type name too long: {} bytes (max: 12)",
                name.len()
            )));
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
            return Err(IgtlError::InvalidHeader(format!(
                "Device name too long: {} bytes (max: 20)",
                name.len()
            )));
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

/// High-precision timestamp for OpenIGTLink messages
///
/// The timestamp field in OpenIGTLink is a 64-bit value where:
/// - Upper 32 bits: seconds since Unix epoch (UTC)
/// - Lower 32 bits: fractional seconds in nanoseconds / 2^32
///
/// This provides nanosecond-level precision, critical for real-time tracking
/// at 1000 Hz (1ms intervals).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    /// Seconds since Unix epoch (1970-01-01 00:00:00 UTC)
    pub seconds: u32,
    /// Fractional seconds as a 32-bit value (nanoseconds * 2^32 / 1_000_000_000)
    pub fraction: u32,
}

impl Timestamp {
    /// Create a new timestamp from seconds and fraction
    ///
    /// # Arguments
    /// * `seconds` - Seconds since Unix epoch
    /// * `fraction` - Fractional seconds (0x00000000 to 0xFFFFFFFF represents 0.0 to ~1.0)
    pub fn new(seconds: u32, fraction: u32) -> Self {
        Timestamp { seconds, fraction }
    }

    /// Create a timestamp representing the current time
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::protocol::header::Timestamp;
    ///
    /// let ts = Timestamp::now();
    /// assert!(ts.seconds > 0);
    /// ```
    pub fn now() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        let seconds = now.as_secs() as u32;
        // Convert nanoseconds to fraction: fraction = (nanoseconds * 2^32) / 10^9
        let nanos = now.subsec_nanos();
        let fraction = ((nanos as u64) * 0x1_0000_0000 / 1_000_000_000) as u32;

        Timestamp { seconds, fraction }
    }

    /// Create a zero timestamp (no timestamp)
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::protocol::header::Timestamp;
    ///
    /// let ts = Timestamp::zero();
    /// assert_eq!(ts.to_u64(), 0);
    /// ```
    pub fn zero() -> Self {
        Timestamp {
            seconds: 0,
            fraction: 0,
        }
    }

    /// Convert to OpenIGTLink wire format (u64)
    ///
    /// Upper 32 bits: seconds, Lower 32 bits: fraction
    pub fn to_u64(self) -> u64 {
        ((self.seconds as u64) << 32) | (self.fraction as u64)
    }

    /// Create from OpenIGTLink wire format (u64)
    ///
    /// Upper 32 bits: seconds, Lower 32 bits: fraction
    pub fn from_u64(value: u64) -> Self {
        Timestamp {
            seconds: (value >> 32) as u32,
            fraction: (value & 0xFFFFFFFF) as u32,
        }
    }

    /// Convert to nanoseconds since Unix epoch
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::protocol::header::Timestamp;
    ///
    /// let ts = Timestamp::new(1000, 0x80000000); // 1000.5 seconds
    /// assert_eq!(ts.to_nanos(), 1_000_500_000_000);
    /// ```
    pub fn to_nanos(self) -> u64 {
        let sec_nanos = (self.seconds as u64) * 1_000_000_000;
        let frac_nanos = ((self.fraction as u64) * 1_000_000_000) / 0x1_0000_0000;
        sec_nanos + frac_nanos
    }

    /// Create from nanoseconds since Unix epoch
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::protocol::header::Timestamp;
    ///
    /// let ts = Timestamp::from_nanos(1_000_500_000_000); // 1000.5 seconds
    /// assert_eq!(ts.seconds, 1000);
    /// // Fraction should be approximately 0x80000000 (0.5)
    /// assert!((ts.fraction as i64 - 0x80000000_i64).abs() < 100);
    /// ```
    pub fn from_nanos(nanos: u64) -> Self {
        let seconds = (nanos / 1_000_000_000) as u32;
        let remaining_nanos = (nanos % 1_000_000_000) as u32;
        let fraction = ((remaining_nanos as u64) * 0x1_0000_0000 / 1_000_000_000) as u32;

        Timestamp { seconds, fraction }
    }

    /// Convert to floating-point seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::protocol::header::Timestamp;
    ///
    /// let ts = Timestamp::new(1000, 0x80000000); // 1000.5 seconds
    /// assert!((ts.to_f64() - 1000.5).abs() < 0.0001);
    /// ```
    pub fn to_f64(self) -> f64 {
        let frac_f64 = (self.fraction as f64) / (u32::MAX as f64 + 1.0);
        (self.seconds as f64) + frac_f64
    }
}

/// OpenIGTLink message header (58 bytes fixed size)
///
/// # Header Structure (all numerical values in big-endian)
/// - Version: u16 (2 bytes)
/// - Type: `char[12]` (12 bytes, null-padded)
/// - Device Name: `char[20]` (20 bytes, null-padded)
/// - Timestamp: u64 (8 bytes) - high 32 bits: seconds, low 32 bits: fraction
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
    /// High-precision timestamp (nanosecond resolution)
    pub timestamp: Timestamp,
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

        // Read timestamp (8 bytes, big-endian) - convert from u64 to Timestamp
        let timestamp_u64 = cursor.get_u64();
        let timestamp = Timestamp::from_u64(timestamp_u64);

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

        // Write timestamp (8 bytes, big-endian) - convert Timestamp to u64
        buf.put_u64(self.timestamp.to_u64());

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
    fn test_timestamp_now() {
        let ts = Timestamp::now();
        assert!(ts.seconds > 0);
        assert!(ts.to_f64() > 0.0);
    }

    #[test]
    fn test_timestamp_zero() {
        let ts = Timestamp::zero();
        assert_eq!(ts.seconds, 0);
        assert_eq!(ts.fraction, 0);
        assert_eq!(ts.to_u64(), 0);
    }

    #[test]
    fn test_timestamp_conversion() {
        let ts = Timestamp::new(1000, 0x80000000); // 1000.5 seconds
        assert_eq!(ts.seconds, 1000);
        assert_eq!(ts.fraction, 0x80000000);

        // Test to_nanos
        let nanos = ts.to_nanos();
        assert_eq!(nanos, 1_000_500_000_000);

        // Test from_nanos roundtrip
        let ts2 = Timestamp::from_nanos(nanos);
        assert_eq!(ts2.seconds, ts.seconds);
        // Allow small rounding error in fraction
        assert!((ts2.fraction as i64 - ts.fraction as i64).abs() < 100);

        // Test to_f64
        let f = ts.to_f64();
        assert!((f - 1000.5).abs() < 0.0001);
    }

    #[test]
    fn test_timestamp_u64_roundtrip() {
        let original = Timestamp::new(1234567890, 0xABCDEF12);
        let u64_val = original.to_u64();
        let restored = Timestamp::from_u64(u64_val);

        assert_eq!(restored.seconds, original.seconds);
        assert_eq!(restored.fraction, original.fraction);
    }

    #[test]
    fn test_header_roundtrip() {
        let original = Header {
            version: 2,
            type_name: TypeName::new("TRANSFORM").unwrap(),
            device_name: DeviceName::new("TestDevice").unwrap(),
            timestamp: Timestamp::new(1234567890, 0x12345678),
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
            timestamp: Timestamp::from_u64(0x0102030405060708),
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
