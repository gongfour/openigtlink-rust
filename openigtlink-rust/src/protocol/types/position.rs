//! POSITION message type implementation
//!
//! The POSITION message type is used to transfer position and orientation information.
//! The data consists of a 3D position vector and a quaternion for orientation.
//! This format is 19% smaller than TRANSFORM and ideal for high frame-rate tracking data.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// POSITION message containing position and quaternion orientation
///
/// # OpenIGTLink Specification
/// - Message type: "POSITION" (alias: "QTRANS")
/// - Body size: 28 bytes (7 × 4-byte floats)
/// - Encoding: Big-endian
/// - Position in millimeters, orientation as quaternion
#[derive(Debug, Clone, PartialEq)]
pub struct PositionMessage {
    /// Position in millimeters
    pub position: [f32; 3], // X, Y, Z

    /// Orientation as quaternion
    ///
    /// Quaternion components in order: [ox, oy, oz, w]
    /// where q = w + ox*i + oy*j + oz*k
    pub quaternion: [f32; 4], // OX, OY, OZ, W
}

impl PositionMessage {
    /// Create a new POSITION message at origin with identity orientation
    pub fn identity() -> Self {
        PositionMessage {
            position: [0.0, 0.0, 0.0],
            quaternion: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
        }
    }

    /// Create a new POSITION message with specified position and identity orientation
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        PositionMessage {
            position: [x, y, z],
            quaternion: [0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Create a new POSITION message with position and quaternion
    pub fn with_quaternion(position: [f32; 3], quaternion: [f32; 4]) -> Self {
        PositionMessage {
            position,
            quaternion,
        }
    }

    /// Get position coordinates
    pub fn get_position(&self) -> (f32, f32, f32) {
        (self.position[0], self.position[1], self.position[2])
    }

    /// Get quaternion components
    pub fn get_quaternion(&self) -> (f32, f32, f32, f32) {
        (
            self.quaternion[0],
            self.quaternion[1],
            self.quaternion[2],
            self.quaternion[3],
        )
    }
}

impl Message for PositionMessage {
    fn message_type() -> &'static str {
        "POSITION"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(28);

        // Encode position (X, Y, Z)
        for &coord in &self.position {
            buf.put_f32(coord);
        }

        // Encode quaternion (OX, OY, OZ, W)
        for &comp in &self.quaternion {
            buf.put_f32(comp);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 28 {
            return Err(IgtlError::InvalidSize {
                expected: 28,
                actual: data.len(),
            });
        }

        // Decode position
        let position = [data.get_f32(), data.get_f32(), data.get_f32()];

        // Decode quaternion
        let quaternion = [
            data.get_f32(),
            data.get_f32(),
            data.get_f32(),
            data.get_f32(),
        ];

        Ok(PositionMessage {
            position,
            quaternion,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(PositionMessage::message_type(), "POSITION");
    }

    #[test]
    fn test_identity() {
        let pos = PositionMessage::identity();
        assert_eq!(pos.position, [0.0, 0.0, 0.0]);
        assert_eq!(pos.quaternion, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_new() {
        let pos = PositionMessage::new(10.0, 20.0, 30.0);
        assert_eq!(pos.position, [10.0, 20.0, 30.0]);
        assert_eq!(pos.quaternion, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_with_quaternion() {
        let pos = PositionMessage::with_quaternion([1.0, 2.0, 3.0], [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(pos.position, [1.0, 2.0, 3.0]);
        assert_eq!(pos.quaternion, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn test_encode_size() {
        let pos = PositionMessage::new(1.0, 2.0, 3.0);
        let encoded = pos.encode_content().unwrap();
        assert_eq!(encoded.len(), 28);
    }

    #[test]
    fn test_position_roundtrip() {
        let original = PositionMessage::with_quaternion(
            [100.5, 200.25, 300.125],
            [0.1, 0.2, 0.3, 0.9274], // Approximately normalized
        );

        let encoded = original.encode_content().unwrap();
        let decoded = PositionMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.position, original.position);
        assert_eq!(decoded.quaternion, original.quaternion);
    }

    #[test]
    fn test_big_endian_encoding() {
        let pos = PositionMessage::new(1.0, 0.0, 0.0);
        let encoded = pos.encode_content().unwrap();

        // First float (1.0) should be: 0x3F800000 in big-endian
        assert_eq!(encoded[0], 0x3F);
        assert_eq!(encoded[1], 0x80);
        assert_eq!(encoded[2], 0x00);
        assert_eq!(encoded[3], 0x00);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 20]; // Wrong size
        let result = PositionMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_position() {
        let pos = PositionMessage::new(10.0, 20.0, 30.0);
        let (x, y, z) = pos.get_position();
        assert_eq!((x, y, z), (10.0, 20.0, 30.0));
    }

    #[test]
    fn test_get_quaternion() {
        let pos = PositionMessage::with_quaternion([0.0, 0.0, 0.0], [0.1, 0.2, 0.3, 0.4]);
        let (ox, oy, oz, w) = pos.get_quaternion();
        assert_eq!((ox, oy, oz, w), (0.1, 0.2, 0.3, 0.4));
    }
}
