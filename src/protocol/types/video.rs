//! VIDEO message type implementation
//!
//! The VIDEO message is used to transfer video frame data for real-time visualization
//! during image-guided procedures.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// Video codec type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecType {
    H264 = 1,
    VP9 = 2,
    HEVC = 3,
    MJPEG = 4,
    Raw = 5,
}

impl CodecType {
    /// Create from codec value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(CodecType::H264),
            2 => Ok(CodecType::VP9),
            3 => Ok(CodecType::HEVC),
            4 => Ok(CodecType::MJPEG),
            5 => Ok(CodecType::Raw),
            _ => Err(IgtlError::InvalidSize {
                expected: 1,
                actual: value as usize,
            }),
        }
    }
}

/// VIDEO message for video frame data
///
/// # OpenIGTLink Specification
/// - Message type: "VIDEO"
/// - Format: CODEC (uint8) + WIDTH (uint16) + HEIGHT (uint16) + Reserved (uint8) + Frame data
/// - Header size: 6 bytes + variable frame data
#[derive(Debug, Clone, PartialEq)]
pub struct VideoMessage {
    /// Video codec type
    pub codec: CodecType,
    /// Frame width in pixels
    pub width: u16,
    /// Frame height in pixels
    pub height: u16,
    /// Encoded frame data
    pub frame_data: Vec<u8>,
}

impl VideoMessage {
    /// Create a new VIDEO message
    pub fn new(
        codec: CodecType,
        width: u16,
        height: u16,
        frame_data: Vec<u8>,
    ) -> Self {
        VideoMessage {
            codec,
            width,
            height,
            frame_data,
        }
    }

    /// Get frame size in bytes
    pub fn frame_size(&self) -> usize {
        self.frame_data.len()
    }

    /// Check if frame is empty
    pub fn is_empty(&self) -> bool {
        self.frame_data.is_empty()
    }
}

impl Message for VideoMessage {
    fn message_type() -> &'static str {
        "VIDEO"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(6 + self.frame_data.len());

        // Encode CODEC (uint8)
        buf.put_u8(self.codec as u8);

        // Encode WIDTH (uint16)
        buf.put_u16(self.width);

        // Encode HEIGHT (uint16)
        buf.put_u16(self.height);

        // Encode Reserved (uint8)
        buf.put_u8(0);

        // Encode frame data
        buf.extend_from_slice(&self.frame_data);

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(IgtlError::InvalidSize {
                expected: 6,
                actual: data.len(),
            });
        }

        // Decode CODEC (uint8)
        let codec = CodecType::from_u8(data.get_u8())?;

        // Decode WIDTH (uint16)
        let width = data.get_u16();

        // Decode HEIGHT (uint16)
        let height = data.get_u16();

        // Decode Reserved (uint8)
        let _reserved = data.get_u8();

        // Decode frame data (remaining bytes)
        let frame_data = data.to_vec();

        Ok(VideoMessage {
            codec,
            width,
            height,
            frame_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(VideoMessage::message_type(), "VIDEO");
    }

    #[test]
    fn test_new() {
        let frame = vec![0u8; 1920 * 1080 * 3]; // RGB frame
        let msg = VideoMessage::new(CodecType::Raw, 1920, 1080, frame);

        assert_eq!(msg.width, 1920);
        assert_eq!(msg.height, 1080);
        assert_eq!(msg.codec, CodecType::Raw);
    }

    #[test]
    fn test_frame_size() {
        let frame = vec![0u8; 1024];
        let msg = VideoMessage::new(CodecType::H264, 640, 480, frame);

        assert_eq!(msg.frame_size(), 1024);
    }

    #[test]
    fn test_is_empty() {
        let msg = VideoMessage::new(CodecType::MJPEG, 320, 240, vec![]);
        assert!(msg.is_empty());
    }

    #[test]
    fn test_encode() {
        let frame = vec![1, 2, 3, 4, 5];
        let msg = VideoMessage::new(CodecType::H264, 100, 100, frame);
        let encoded = msg.encode_content().unwrap();

        // 6 bytes header + 5 bytes data = 11 bytes
        assert_eq!(encoded.len(), 11);
        assert_eq!(encoded[0], CodecType::H264 as u8);
    }

    #[test]
    fn test_roundtrip_h264() {
        let original = VideoMessage::new(
            CodecType::H264,
            1920,
            1080,
            vec![0xFF; 1000], // Simulated compressed data
        );

        let encoded = original.encode_content().unwrap();
        let decoded = VideoMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.codec, CodecType::H264);
        assert_eq!(decoded.width, 1920);
        assert_eq!(decoded.height, 1080);
        assert_eq!(decoded.frame_data.len(), 1000);
    }

    #[test]
    fn test_roundtrip_raw() {
        let original = VideoMessage::new(
            CodecType::Raw,
            640,
            480,
            vec![128u8; 640 * 480 * 3],
        );

        let encoded = original.encode_content().unwrap();
        let decoded = VideoMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.codec, CodecType::Raw);
        assert_eq!(decoded.width, 640);
        assert_eq!(decoded.height, 480);
    }

    #[test]
    fn test_decode_invalid_header() {
        let data = vec![0u8; 5]; // Too short
        let result = VideoMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_codec() {
        let data = vec![99, 0, 100, 0, 100, 0]; // Invalid codec
        let result = VideoMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
