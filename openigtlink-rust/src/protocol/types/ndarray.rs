//! NDARRAY message type implementation
//!
//! The NDARRAY message type is used to transfer N-dimensional numerical arrays.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// Scalar data type for NDARRAY
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScalarType {
    Int8 = 2,
    Uint8 = 3,
    Int16 = 4,
    Uint16 = 5,
    Int32 = 6,
    Uint32 = 7,
    Float32 = 10,
    Float64 = 11,
}

impl ScalarType {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            2 => Ok(ScalarType::Int8),
            3 => Ok(ScalarType::Uint8),
            4 => Ok(ScalarType::Int16),
            5 => Ok(ScalarType::Uint16),
            6 => Ok(ScalarType::Int32),
            7 => Ok(ScalarType::Uint32),
            10 => Ok(ScalarType::Float32),
            11 => Ok(ScalarType::Float64),
            _ => Err(IgtlError::InvalidHeader(format!(
                "Invalid scalar type: {}",
                value
            ))),
        }
    }

    /// Get the size in bytes of this scalar type
    pub fn size(&self) -> usize {
        match self {
            ScalarType::Int8 | ScalarType::Uint8 => 1,
            ScalarType::Int16 | ScalarType::Uint16 => 2,
            ScalarType::Int32 | ScalarType::Uint32 | ScalarType::Float32 => 4,
            ScalarType::Float64 => 8,
        }
    }
}

/// NDARRAY message containing an N-dimensional numerical array
///
/// # OpenIGTLink Specification
/// - Message type: "NDARRAY"
/// - Body format: SCALAR_TYPE (uint8) + DIM (uint8) + SIZE (`uint16[DIM]`) + DATA (bytes)
/// - Data layout: Row-major order (C-style)
#[derive(Debug, Clone, PartialEq)]
pub struct NdArrayMessage {
    /// Scalar data type
    pub scalar_type: ScalarType,
    /// Array dimensions
    pub size: Vec<u16>,
    /// Raw array data in network byte order
    pub data: Vec<u8>,
}

impl NdArrayMessage {
    /// Create a new NDARRAY message
    pub fn new(scalar_type: ScalarType, size: Vec<u16>, data: Vec<u8>) -> Result<Self> {
        if size.is_empty() || size.len() > 255 {
            return Err(IgtlError::InvalidHeader(format!(
                "Invalid dimension count: {}",
                size.len()
            )));
        }

        // Calculate expected data size
        let expected_size: usize =
            size.iter().map(|&s| s as usize).product::<usize>() * scalar_type.size();

        if data.len() != expected_size {
            return Err(IgtlError::InvalidSize {
                expected: expected_size,
                actual: data.len(),
            });
        }

        Ok(NdArrayMessage {
            scalar_type,
            size,
            data,
        })
    }

    /// Create a 1D array
    pub fn new_1d(scalar_type: ScalarType, data: Vec<u8>) -> Result<Self> {
        let element_count = data.len() / scalar_type.size();
        Self::new(scalar_type, vec![element_count as u16], data)
    }

    /// Create a 2D array
    pub fn new_2d(scalar_type: ScalarType, rows: u16, cols: u16, data: Vec<u8>) -> Result<Self> {
        Self::new(scalar_type, vec![rows, cols], data)
    }

    /// Create a 3D array
    pub fn new_3d(
        scalar_type: ScalarType,
        dim1: u16,
        dim2: u16,
        dim3: u16,
        data: Vec<u8>,
    ) -> Result<Self> {
        Self::new(scalar_type, vec![dim1, dim2, dim3], data)
    }

    /// Get the number of dimensions
    pub fn ndim(&self) -> usize {
        self.size.len()
    }

    /// Get total number of elements
    pub fn element_count(&self) -> usize {
        self.size.iter().map(|&s| s as usize).product()
    }

    /// Get total data size in bytes
    pub fn data_size(&self) -> usize {
        self.data.len()
    }
}

impl Message for NdArrayMessage {
    fn message_type() -> &'static str {
        "NDARRAY"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let dim = self.size.len();
        if dim == 0 || dim > 255 {
            return Err(IgtlError::InvalidHeader(format!(
                "Invalid dimension count: {}",
                dim
            )));
        }

        let mut buf = Vec::with_capacity(2 + dim * 2 + self.data.len());

        // Encode SCALAR_TYPE (uint8)
        buf.put_u8(self.scalar_type as u8);

        // Encode DIM (uint8)
        buf.put_u8(dim as u8);

        // Encode SIZE (`uint16[DIM]`)
        for &s in &self.size {
            buf.put_u16(s);
        }

        // Encode DATA
        buf.extend_from_slice(&self.data);

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(IgtlError::InvalidSize {
                expected: 2,
                actual: data.len(),
            });
        }

        // Decode SCALAR_TYPE
        let scalar_type = ScalarType::from_u8(data.get_u8())?;

        // Decode DIM
        let dim = data.get_u8() as usize;

        if dim == 0 {
            return Err(IgtlError::InvalidHeader(
                "Dimension cannot be zero".to_string(),
            ));
        }

        // Check we have enough data for SIZE array
        if data.len() < dim * 2 {
            return Err(IgtlError::InvalidSize {
                expected: dim * 2,
                actual: data.len(),
            });
        }

        // Decode SIZE
        let mut size = Vec::with_capacity(dim);
        for _ in 0..dim {
            size.push(data.get_u16());
        }

        // Calculate expected data size
        let expected_data_size: usize =
            size.iter().map(|&s| s as usize).product::<usize>() * scalar_type.size();

        if data.len() < expected_data_size {
            return Err(IgtlError::InvalidSize {
                expected: expected_data_size,
                actual: data.len(),
            });
        }

        // Decode DATA
        let array_data = data[..expected_data_size].to_vec();

        Ok(NdArrayMessage {
            scalar_type,
            size,
            data: array_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(NdArrayMessage::message_type(), "NDARRAY");
    }

    #[test]
    fn test_scalar_type_size() {
        assert_eq!(ScalarType::Int8.size(), 1);
        assert_eq!(ScalarType::Uint8.size(), 1);
        assert_eq!(ScalarType::Int16.size(), 2);
        assert_eq!(ScalarType::Uint16.size(), 2);
        assert_eq!(ScalarType::Int32.size(), 4);
        assert_eq!(ScalarType::Uint32.size(), 4);
        assert_eq!(ScalarType::Float32.size(), 4);
        assert_eq!(ScalarType::Float64.size(), 8);
    }

    #[test]
    fn test_new_1d() {
        let data = vec![1u8, 2, 3, 4];
        let msg = NdArrayMessage::new_1d(ScalarType::Uint8, data).unwrap();

        assert_eq!(msg.ndim(), 1);
        assert_eq!(msg.size[0], 4);
        assert_eq!(msg.element_count(), 4);
    }

    #[test]
    fn test_new_2d() {
        let data = vec![0u8; 12]; // 3x4 matrix of uint8
        let msg = NdArrayMessage::new_2d(ScalarType::Uint8, 3, 4, data).unwrap();

        assert_eq!(msg.ndim(), 2);
        assert_eq!(msg.size, vec![3, 4]);
        assert_eq!(msg.element_count(), 12);
    }

    #[test]
    fn test_new_3d() {
        let data = vec![0u8; 24]; // 2x3x4 array of uint8
        let msg = NdArrayMessage::new_3d(ScalarType::Uint8, 2, 3, 4, data).unwrap();

        assert_eq!(msg.ndim(), 3);
        assert_eq!(msg.size, vec![2, 3, 4]);
        assert_eq!(msg.element_count(), 24);
    }

    #[test]
    fn test_invalid_data_size() {
        let data = vec![0u8; 10]; // Wrong size for 3x4 array
        let result = NdArrayMessage::new_2d(ScalarType::Uint8, 3, 4, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_1d_uint8() {
        let data = vec![1u8, 2, 3];
        let msg = NdArrayMessage::new_1d(ScalarType::Uint8, data).unwrap();
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded[0], 3); // SCALAR_TYPE = Uint8
        assert_eq!(encoded[1], 1); // DIM = 1
        assert_eq!(u16::from_be_bytes([encoded[2], encoded[3]]), 3); // SIZE[0] = 3
        assert_eq!(&encoded[4..], &[1, 2, 3]);
    }

    #[test]
    fn test_roundtrip_1d() {
        let original_data = vec![10u8, 20, 30, 40];
        let original = NdArrayMessage::new_1d(ScalarType::Uint8, original_data.clone()).unwrap();

        let encoded = original.encode_content().unwrap();
        let decoded = NdArrayMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.scalar_type, ScalarType::Uint8);
        assert_eq!(decoded.size, vec![4]);
        assert_eq!(decoded.data, original_data);
    }

    #[test]
    fn test_roundtrip_2d() {
        let data = vec![1u8, 2, 3, 4, 5, 6]; // 2x3 matrix
        let original = NdArrayMessage::new_2d(ScalarType::Uint8, 2, 3, data.clone()).unwrap();

        let encoded = original.encode_content().unwrap();
        let decoded = NdArrayMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.size, vec![2, 3]);
        assert_eq!(decoded.data, data);
    }

    #[test]
    fn test_roundtrip_float32() {
        // Create 2x2 float32 array
        let mut data = Vec::new();
        for val in [1.0f32, 2.0, 3.0, 4.0] {
            data.extend_from_slice(&val.to_be_bytes());
        }

        let original = NdArrayMessage::new_2d(ScalarType::Float32, 2, 2, data.clone()).unwrap();
        let encoded = original.encode_content().unwrap();
        let decoded = NdArrayMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.scalar_type, ScalarType::Float32);
        assert_eq!(decoded.size, vec![2, 2]);
        assert_eq!(decoded.data, data);
    }

    #[test]
    fn test_decode_invalid_header() {
        let data = vec![0u8]; // Too short
        let result = NdArrayMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_truncated_data() {
        let mut data = vec![3u8, 1]; // SCALAR_TYPE=Uint8, DIM=1
        data.extend_from_slice(&5u16.to_be_bytes()); // SIZE[0]=5
        data.extend_from_slice(&[1, 2, 3]); // Only 3 bytes instead of 5

        let result = NdArrayMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
