//! Streaming control messages (STT_*, STP_*)
//!
//! - STT_*: Start streaming messages
//! - STP_*: Stop streaming messages

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

use super::impl_empty_query;

// STP (Stop) messages - all have empty body
impl_empty_query!(StopTDataMessage, "STP_TDATA");
impl_empty_query!(StopImageMessage, "STP_IMAGE");
impl_empty_query!(StopTransformMessage, "STP_TRANSFOR");
impl_empty_query!(StopPositionMessage, "STP_POSITION");
impl_empty_query!(StopQtDataMessage, "STP_QTDATA");
impl_empty_query!(StopNdArrayMessage, "STP_NDARRAY");

// STT_TDATA: Start tracking data streaming
/// Start tracking data streaming message
///
/// # OpenIGTLink Specification
/// - Message type: "STT_TDATA"
/// - Body size: 36 bytes (fixed)
/// - Encoding:
///   - resolution: u32 (4 bytes, big-endian) - streaming interval in milliseconds
///   - coordinate_name: 32 bytes (null-padded) - coordinate system name
///
/// # C++ Compatibility
/// Matches igtl_stt_tdata structure:
/// ```c
/// typedef struct {
///   igtl_uint32 resolution;   // 4 bytes
///   char coord_name[32];      // 32 bytes
/// } igtl_stt_tdata;           // total: 36 bytes
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct StartTDataMessage {
    /// Streaming interval in milliseconds (e.g., 50ms = 20Hz)
    pub resolution: u32,
    /// Coordinate system name (max 32 characters)
    pub coordinate_name: String,
}

impl StartTDataMessage {
    /// Create a new StartTDataMessage
    pub fn new(resolution: u32, coordinate_name: impl Into<String>) -> Self {
        Self {
            resolution,
            coordinate_name: coordinate_name.into(),
        }
    }
}

impl Message for StartTDataMessage {
    fn message_type() -> &'static str {
        "STT_TDATA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(36);

        // Encode resolution (4 bytes, big-endian)
        buf.put_u32(self.resolution);

        // Encode coordinate_name (32 bytes, null-padded)
        let mut coord_bytes = [0u8; 32];
        let name_bytes = self.coordinate_name.as_bytes();
        let len = name_bytes.len().min(32);
        coord_bytes[..len].copy_from_slice(&name_bytes[..len]);
        buf.extend_from_slice(&coord_bytes);

        Ok(buf)
    }

    fn decode_content(data: &[u8]) -> Result<Self> {
        if data.len() < 36 {
            return Err(IgtlError::InvalidSize {
                expected: 36,
                actual: data.len(),
            });
        }

        let mut cursor = std::io::Cursor::new(data);

        // Decode resolution (4 bytes, big-endian)
        let resolution = cursor.get_u32();

        // Decode coordinate_name (32 bytes, null-terminated)
        let coord_bytes = &data[4..36];
        let len = coord_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(32);
        let coordinate_name = String::from_utf8(coord_bytes[..len].to_vec())
            .map_err(|_| IgtlError::InvalidHeader("Invalid UTF-8 in coordinate name".to_string()))?;

        Ok(Self {
            resolution,
            coordinate_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::message::Message;

    #[test]
    fn test_stop_tdata_message_type() {
        assert_eq!(StopTDataMessage::message_type(), "STP_TDATA");
    }

    #[test]
    fn test_stop_image_empty_body() {
        let msg = StopImageMessage;
        let encoded = msg.encode_content().unwrap();
        assert_eq!(encoded.len(), 0);
    }

    #[test]
    fn test_all_stop_messages_type_names() {
        // Verify all type names are ≤12 characters
        assert!(StopTDataMessage::message_type().len() <= 12);
        assert!(StopImageMessage::message_type().len() <= 12);
        assert!(StopTransformMessage::message_type().len() <= 12);
        assert!(StopPositionMessage::message_type().len() <= 12);
        assert!(StopQtDataMessage::message_type().len() <= 12);
        assert!(StopNdArrayMessage::message_type().len() <= 12);
    }

    // StartTDataMessage tests
    #[test]
    fn test_start_tdata_message_type() {
        assert_eq!(StartTDataMessage::message_type(), "STT_TDATA");
    }

    #[test]
    fn test_start_tdata_encoding_36_bytes() {
        let msg = StartTDataMessage::new(50, "Patient");
        let encoded = msg.encode_content().unwrap();
        assert_eq!(encoded.len(), 36, "Body should be exactly 36 bytes");
    }

    #[test]
    fn test_start_tdata_big_endian_resolution() {
        let msg = StartTDataMessage::new(50, "Patient");
        let encoded = msg.encode_content().unwrap();

        // Verify big-endian encoding of resolution (50 = 0x00000032)
        assert_eq!(encoded[0], 0x00);
        assert_eq!(encoded[1], 0x00);
        assert_eq!(encoded[2], 0x00);
        assert_eq!(encoded[3], 0x32);
    }

    #[test]
    fn test_start_tdata_coordinate_name_padding() {
        let msg = StartTDataMessage::new(100, "Patient");
        let encoded = msg.encode_content().unwrap();

        // Verify coordinate name encoding
        assert_eq!(&encoded[4..11], b"Patient");
        // Verify null padding
        assert_eq!(encoded[11], 0);
        assert_eq!(encoded[35], 0); // Last byte should be null
    }

    #[test]
    fn test_start_tdata_roundtrip() {
        let original = StartTDataMessage::new(50, "Patient");
        let encoded = original.encode_content().unwrap();
        let decoded = StartTDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.resolution, 50);
        assert_eq!(decoded.coordinate_name, "Patient");
    }

    #[test]
    fn test_start_tdata_long_coordinate_name() {
        // Test name truncation at 32 characters
        let long_name = "ThisIsAVeryLongCoordinateNameThatExceeds32Characters";
        let msg = StartTDataMessage::new(100, long_name);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 36);

        let decoded = StartTDataMessage::decode_content(&encoded).unwrap();
        assert_eq!(decoded.coordinate_name.len(), 32); // Truncated to 32
    }

    #[test]
    fn test_start_tdata_empty_coordinate_name() {
        let msg = StartTDataMessage::new(50, "");
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 36);

        let decoded = StartTDataMessage::decode_content(&encoded).unwrap();
        assert_eq!(decoded.resolution, 50);
        assert_eq!(decoded.coordinate_name, "");
    }

    #[test]
    fn test_start_tdata_c_compat() {
        // C++ compatibility test
        let msg = StartTDataMessage::new(50, "Patient");
        let encoded = msg.encode_content().unwrap();

        // Verify structure matches C++ igtl_stt_tdata
        // resolution: 4 bytes (big-endian)
        let resolution = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
        assert_eq!(resolution, 50);

        // coord_name: 32 bytes (null-padded)
        assert_eq!(&encoded[4..11], b"Patient");
        assert_eq!(encoded[11], 0); // null terminator
    }
}
