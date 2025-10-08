//! VIDEOMETA (Video Metadata) message type implementation
//!
//! The VIDEOMETA message is used to transfer video stream metadata such as
//! codec parameters, framerate, and bitrate information.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

use super::video::CodecType;

/// VIDEOMETA message for video stream metadata
///
/// # OpenIGTLink Specification
/// - Message type: "VIDEOMETA"
/// - Format: CODEC (uint8) + WIDTH (uint16) + HEIGHT (uint16) + FRAMERATE (uint8) + BITRATE (uint32) + Reserved (uint16)
/// - Size: 12 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoMetaMessage {
    /// Video codec type
    pub codec: CodecType,
    /// Frame width in pixels
    pub width: u16,
    /// Frame height in pixels
    pub height: u16,
    /// Frames per second
    pub framerate: u8,
    /// Bitrate in kbps
    pub bitrate: u32,
}

impl VideoMetaMessage {
    /// Create a new VIDEOMETA message
    pub fn new(codec: CodecType, width: u16, height: u16, framerate: u8, bitrate: u32) -> Self {
        VideoMetaMessage {
            codec,
            width,
            height,
            framerate,
            bitrate,
        }
    }

    /// Create with common HD 1080p settings
    pub fn hd1080(codec: CodecType, framerate: u8, bitrate: u32) -> Self {
        VideoMetaMessage {
            codec,
            width: 1920,
            height: 1080,
            framerate,
            bitrate,
        }
    }

    /// Create with common HD 720p settings
    pub fn hd720(codec: CodecType, framerate: u8, bitrate: u32) -> Self {
        VideoMetaMessage {
            codec,
            width: 1280,
            height: 720,
            framerate,
            bitrate,
        }
    }

    /// Create with SD settings
    pub fn sd(codec: CodecType, framerate: u8, bitrate: u32) -> Self {
        VideoMetaMessage {
            codec,
            width: 640,
            height: 480,
            framerate,
            bitrate,
        }
    }

    /// Get total pixels per frame
    pub fn pixels_per_frame(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Get estimated bandwidth in bytes per second
    pub fn bandwidth_bps(&self) -> u32 {
        self.bitrate * 1000 / 8 // Convert kbps to bytes/sec
    }
}

impl Message for VideoMetaMessage {
    fn message_type() -> &'static str {
        "VIDEOMETA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(12);

        // Encode CODEC (uint8)
        buf.put_u8(self.codec as u8);

        // Encode WIDTH (uint16)
        buf.put_u16(self.width);

        // Encode HEIGHT (uint16)
        buf.put_u16(self.height);

        // Encode FRAMERATE (uint8)
        buf.put_u8(self.framerate);

        // Encode BITRATE (uint32)
        buf.put_u32(self.bitrate);

        // Encode Reserved (uint16)
        buf.put_u16(0);

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() != 12 {
            return Err(IgtlError::InvalidSize {
                expected: 12,
                actual: data.len(),
            });
        }

        // Decode CODEC (uint8)
        let codec = CodecType::from_u8(data.get_u8())?;

        // Decode WIDTH (uint16)
        let width = data.get_u16();

        // Decode HEIGHT (uint16)
        let height = data.get_u16();

        // Decode FRAMERATE (uint8)
        let framerate = data.get_u8();

        // Decode BITRATE (uint32)
        let bitrate = data.get_u32();

        // Decode Reserved (uint16)
        let _reserved = data.get_u16();

        Ok(VideoMetaMessage {
            codec,
            width,
            height,
            framerate,
            bitrate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(VideoMetaMessage::message_type(), "VIDEOMETA");
    }

    #[test]
    fn test_new() {
        let meta = VideoMetaMessage::new(CodecType::H264, 1920, 1080, 30, 5000);

        assert_eq!(meta.codec, CodecType::H264);
        assert_eq!(meta.width, 1920);
        assert_eq!(meta.height, 1080);
        assert_eq!(meta.framerate, 30);
        assert_eq!(meta.bitrate, 5000);
    }

    #[test]
    fn test_hd1080() {
        let meta = VideoMetaMessage::hd1080(CodecType::H264, 60, 10000);

        assert_eq!(meta.width, 1920);
        assert_eq!(meta.height, 1080);
        assert_eq!(meta.framerate, 60);
    }

    #[test]
    fn test_hd720() {
        let meta = VideoMetaMessage::hd720(CodecType::VP9, 30, 3000);

        assert_eq!(meta.width, 1280);
        assert_eq!(meta.height, 720);
    }

    #[test]
    fn test_sd() {
        let meta = VideoMetaMessage::sd(CodecType::MJPEG, 25, 1000);

        assert_eq!(meta.width, 640);
        assert_eq!(meta.height, 480);
    }

    #[test]
    fn test_pixels_per_frame() {
        let meta = VideoMetaMessage::new(CodecType::H264, 100, 100, 30, 1000);
        assert_eq!(meta.pixels_per_frame(), 10000);
    }

    #[test]
    fn test_bandwidth_bps() {
        let meta = VideoMetaMessage::new(CodecType::H264, 1920, 1080, 30, 8000);
        // 8000 kbps = 8000 * 1000 / 8 = 1000000 bytes/sec
        assert_eq!(meta.bandwidth_bps(), 1000000);
    }

    #[test]
    fn test_encode() {
        let meta = VideoMetaMessage::new(CodecType::H264, 1920, 1080, 30, 5000);
        let encoded = meta.encode_content().unwrap();

        assert_eq!(encoded.len(), 12);
        assert_eq!(encoded[0], CodecType::H264 as u8);
    }

    #[test]
    fn test_roundtrip() {
        let original = VideoMetaMessage::new(CodecType::H264, 1920, 1080, 30, 5000);

        let encoded = original.encode_content().unwrap();
        let decoded = VideoMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.codec, original.codec);
        assert_eq!(decoded.width, original.width);
        assert_eq!(decoded.height, original.height);
        assert_eq!(decoded.framerate, original.framerate);
        assert_eq!(decoded.bitrate, original.bitrate);
    }

    #[test]
    fn test_roundtrip_vp9() {
        let original = VideoMetaMessage::hd720(CodecType::VP9, 60, 4000);

        let encoded = original.encode_content().unwrap();
        let decoded = VideoMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.codec, CodecType::VP9);
        assert_eq!(decoded.width, 1280);
        assert_eq!(decoded.height, 720);
        assert_eq!(decoded.framerate, 60);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 11]; // One byte short
        let result = VideoMetaMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_too_long() {
        let data = vec![0u8; 13]; // One byte too long
        let result = VideoMetaMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_codec() {
        let mut data = vec![0u8; 12];
        data[0] = 99; // Invalid codec
        let result = VideoMetaMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
