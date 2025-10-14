//! TRANSFORM message type implementation
//!
//! The TRANSFORM message type is used to transfer a 4x4 homogeneous
//! transformation matrix. Only the upper 3x4 portion is transmitted
//! (48 bytes = 12 floats), as the last row is always [0, 0, 0, 1].

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// TRANSFORM message containing a 4x4 homogeneous transformation matrix
///
/// # OpenIGTLink Specification
/// - Message type: "TRANSFORM"
/// - Body size: 48 bytes (12 × 4-byte floats)
/// - Encoding: 3×4 matrix in row-major order, big-endian
/// - Last row [0, 0, 0, 1] is implicit and not transmitted
#[derive(Debug, Clone, PartialEq)]
pub struct TransformMessage {
    /// 4x4 transformation matrix
    ///
    /// The matrix represents a homogeneous transformation with:
    /// - Upper-left 3x3: rotation matrix
    /// - Upper-right 3x1: translation vector
    /// - Bottom row: [0, 0, 0, 1] (implicit)
    pub matrix: [[f32; 4]; 4],
}

impl TransformMessage {
    /// Create a new identity transformation
    pub fn identity() -> Self {
        TransformMessage {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Create a transformation with only translation
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        TransformMessage {
            matrix: [
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl Message for TransformMessage {
    fn message_type() -> &'static str {
        "TRANSFORM"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(48);

        // Encode 3x4 matrix (12 floats) in column-major order, big-endian
        // Order: R11, R21, R31, R12, R22, R32, R13, R23, R33, TX, TY, TZ
        for col in 0..4 {
            for row in 0..3 {
                buf.put_f32(self.matrix[row][col]);
            }
        }

        // Verify encoded size
        if buf.len() != 48 {
            return Err(IgtlError::InvalidSize {
                expected: 48,
                actual: buf.len(),
            });
        }

        Ok(buf)
    }

    fn decode_content(data: &[u8]) -> Result<Self> {
        if data.len() != 48 {
            return Err(IgtlError::InvalidSize {
                expected: 48,
                actual: data.len(),
            });
        }

        let mut cursor = std::io::Cursor::new(data);
        let mut matrix = [[0.0f32; 4]; 4];

        // Decode 3x4 matrix from column-major order, big-endian
        // Order: R11, R21, R31, R12, R22, R32, R13, R23, R33, TX, TY, TZ
        for col in 0..4 {
            for row in matrix.iter_mut().take(3) {
                row[col] = cursor.get_f32();
            }
        }

        // Set implicit last row
        matrix[3] = [0.0, 0.0, 0.0, 1.0];

        Ok(TransformMessage { matrix })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(TransformMessage::message_type(), "TRANSFORM");
    }

    #[test]
    fn test_identity() {
        let identity = TransformMessage::identity();
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((identity.matrix[i][j] - expected).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_translation() {
        let trans = TransformMessage::translation(10.0, 20.0, 30.0);
        assert!((trans.matrix[0][3] - 10.0).abs() < 1e-6);
        assert!((trans.matrix[1][3] - 20.0).abs() < 1e-6);
        assert!((trans.matrix[2][3] - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform_roundtrip() {
        let original = TransformMessage {
            matrix: [
                [1.0, 0.0, 0.0, 10.0],
                [0.0, 1.0, 0.0, 20.0],
                [0.0, 0.0, 1.0, 30.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        let encoded = original.encode_content().unwrap();
        assert_eq!(encoded.len(), 48);

        let decoded = TransformMessage::decode_content(&encoded).unwrap();

        // Compare matrices with floating-point tolerance
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (original.matrix[i][j] - decoded.matrix[i][j]).abs() < 1e-6,
                    "Matrix mismatch at [{}, {}]: {} != {}",
                    i,
                    j,
                    original.matrix[i][j],
                    decoded.matrix[i][j]
                );
            }
        }
    }

    #[test]
    fn test_encode_size() {
        let transform = TransformMessage::identity();
        let encoded = transform.encode_content().unwrap();
        assert_eq!(encoded.len(), 48);
    }

    #[test]
    fn test_decode_invalid_size() {
        let short_data = vec![0u8; 40];
        let result = TransformMessage::decode_content(&short_data);
        assert!(matches!(result, Err(IgtlError::InvalidSize { .. })));

        let long_data = vec![0u8; 50];
        let result = TransformMessage::decode_content(&long_data);
        assert!(matches!(result, Err(IgtlError::InvalidSize { .. })));
    }

    #[test]
    fn test_big_endian_encoding() {
        // Create a transform with known values
        let mut transform = TransformMessage::identity();
        transform.matrix[0][0] = 1.5f32; // Known bit pattern

        let encoded = transform.encode_content().unwrap();

        // 1.5 in IEEE 754 big-endian: 0x3FC00000
        assert_eq!(encoded[0], 0x3F);
        assert_eq!(encoded[1], 0xC0);
        assert_eq!(encoded[2], 0x00);
        assert_eq!(encoded[3], 0x00);
    }

    #[test]
    fn test_last_row_implicit() {
        let transform = TransformMessage {
            matrix: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        let encoded = transform.encode_content().unwrap();
        let decoded = TransformMessage::decode_content(&encoded).unwrap();

        // Verify last row is always [0, 0, 0, 1]
        assert_eq!(decoded.matrix[3], [0.0, 0.0, 0.0, 1.0]);
    }
}
