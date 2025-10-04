//! RTS (Ready-to-Send) response messages
//!
//! These are server responses to query messages (GET_*, STT_*, STP_*).
//! Different message types may have different RTS formats.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use crate::protocol::types::StatusMessage;

/// RTS_TDATA: Ready-to-send response for tracking data
///
/// # OpenIGTLink Specification
/// - Message type: "RTS_TDATA"
/// - Body size: 2 bytes (status code only) or can be StatusMessage format
/// - Encoding: u16 status code (0=error, 1=ok, big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct RtsTDataMessage {
    /// Status code: 0 = error, 1 = ok
    pub status: u16,
}

impl RtsTDataMessage {
    /// Create OK response
    pub fn ok() -> Self {
        Self { status: 1 }
    }

    /// Create error response
    pub fn error() -> Self {
        Self { status: 0 }
    }

    /// Create with specific status code
    pub fn new(status: u16) -> Self {
        Self { status }
    }
}

impl Message for RtsTDataMessage {
    fn message_type() -> &'static str {
        "RTS_TDATA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        Ok(self.status.to_be_bytes().to_vec())
    }

    fn decode_content(data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(IgtlError::InvalidSize {
                expected: 2,
                actual: data.len(),
            });
        }
        let status = u16::from_be_bytes([data[0], data[1]]);
        Ok(Self { status })
    }
}

// Type aliases for RTS messages that use StatusMessage format
/// RTS_CAPABILITY: Uses StatusMessage format
pub type RtsCapabilityMessage = StatusMessage;

/// RTS_STATUS: Uses StatusMessage format
pub type RtsStatusMessage = StatusMessage;

/// RTS_IMAGE: Uses StatusMessage format
pub type RtsImageMessage = StatusMessage;

/// RTS_TRANSFORM: Uses StatusMessage format
pub type RtsTransformMessage = StatusMessage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rts_tdata_message_type() {
        assert_eq!(RtsTDataMessage::message_type(), "RTS_TDATA");
    }

    #[test]
    fn test_rts_tdata_ok() {
        let msg = RtsTDataMessage::ok();
        assert_eq!(msg.status, 1);
    }

    #[test]
    fn test_rts_tdata_error() {
        let msg = RtsTDataMessage::error();
        assert_eq!(msg.status, 0);
    }

    #[test]
    fn test_rts_tdata_encoding_2_bytes() {
        let msg = RtsTDataMessage::ok();
        let encoded = msg.encode_content().unwrap();
        assert_eq!(encoded.len(), 2, "RTS_TDATA body should be 2 bytes");
    }

    #[test]
    fn test_rts_tdata_big_endian() {
        let msg = RtsTDataMessage::new(1);
        let encoded = msg.encode_content().unwrap();

        // status=1 in big-endian: 0x0001
        assert_eq!(encoded[0], 0x00);
        assert_eq!(encoded[1], 0x01);
    }

    #[test]
    fn test_rts_tdata_roundtrip() {
        let original = RtsTDataMessage::ok();
        let encoded = original.encode_content().unwrap();
        let decoded = RtsTDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.status, 1);
    }

    #[test]
    fn test_rts_tdata_custom_status() {
        let msg = RtsTDataMessage::new(42);
        let encoded = msg.encode_content().unwrap();
        let decoded = RtsTDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.status, 42);
    }

    #[test]
    fn test_rts_tdata_decode_short_buffer() {
        let short_data = vec![0u8; 1];
        let result = RtsTDataMessage::decode_content(&short_data);
        assert!(matches!(result, Err(IgtlError::InvalidSize { .. })));
    }

    #[test]
    fn test_type_aliases() {
        // Verify type aliases compile correctly
        let _cap: RtsCapabilityMessage = StatusMessage::ok("Ready");
        let _status: RtsStatusMessage = StatusMessage::ok("OK");
        let _image: RtsImageMessage = StatusMessage::ok("Ready");
        let _transform: RtsTransformMessage = StatusMessage::ok("Ready");
    }
}
