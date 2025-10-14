//! IMAGE message type implementation
//!
//! The IMAGE message is used to transfer 2D/3D medical image data.
//! This is one of the most commonly used OpenIGTLink message types.
//!
//! # Use Cases
//!
//! - **CT/MRI Image Transfer** - Sending volumetric medical images from scanner to workstation
//! - **Ultrasound Streaming** - Real-time 2D ultrasound image streaming during procedures
//! - **X-ray Images** - Transferring radiographic images for real-time guidance
//! - **Image-Guided Surgery** - Displaying pre-operative images in surgical navigation systems
//! - **Multi-Modal Registration** - Aligning images from different modalities (CT, MRI, PET)
//!
//! # Supported Image Types
//!
//! - **Scalar Types**: 8/16/32-bit integers, 32/64-bit floating point
//! - **Components**: 1 (grayscale), 3 (RGB), 4 (RGBA)
//! - **Dimensions**: 2D (single slice) or 3D (volume)
//! - **Coordinate Systems**: RAS (Right-Anterior-Superior), LPS (Left-Posterior-Superior)
//!
//! # Examples
//!
//! ## Sending a CT Image Slice
//!
//! ```no_run
//! use openigtlink_rust::protocol::types::{ImageMessage, ImageScalarType, CoordinateSystem};
//! use openigtlink_rust::protocol::message::IgtlMessage;
//! use openigtlink_rust::io::ClientBuilder;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .sync()
//!     .build()?;
//!
//! // Simulated CT data (512x512 16-bit)
//! let ct_data: Vec<u8> = vec![0; 512 * 512 * 2];
//!
//! let image = ImageMessage::new(
//!     ImageScalarType::Uint16,
//!     [512, 512, 1],
//!     ct_data
//! )?.with_coordinate(CoordinateSystem::LPS);
//!
//! let msg = IgtlMessage::new(image, "CTScanner")?;
//! client.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Receiving RGB Ultrasound Image
//!
//! ```no_run
//! use openigtlink_rust::io::IgtlServer;
//! use openigtlink_rust::protocol::types::ImageMessage;
//!
//! let server = IgtlServer::bind("0.0.0.0:18944")?;
//! let mut client_conn = server.accept()?;
//!
//! let message = client_conn.receive::<ImageMessage>()?;
//!
//! println!("Received image: {}x{}x{}",
//!          message.content.size[0], message.content.size[1], message.content.size[2]);
//! println!("Scalar type: {:?}", message.content.scalar_type);
//! println!("Components: {}", message.content.num_components);
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// Image scalar type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageScalarType {
    Int8 = 2,
    Uint8 = 3,
    Int16 = 4,
    Uint16 = 5,
    Int32 = 6,
    Uint32 = 7,
    Float32 = 10,
    Float64 = 11,
}

impl ImageScalarType {
    /// Get size in bytes
    pub fn size(&self) -> usize {
        match self {
            ImageScalarType::Int8 | ImageScalarType::Uint8 => 1,
            ImageScalarType::Int16 | ImageScalarType::Uint16 => 2,
            ImageScalarType::Int32 | ImageScalarType::Uint32 | ImageScalarType::Float32 => 4,
            ImageScalarType::Float64 => 8,
        }
    }

    /// Create from type value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            2 => Ok(ImageScalarType::Int8),
            3 => Ok(ImageScalarType::Uint8),
            4 => Ok(ImageScalarType::Int16),
            5 => Ok(ImageScalarType::Uint16),
            6 => Ok(ImageScalarType::Int32),
            7 => Ok(ImageScalarType::Uint32),
            10 => Ok(ImageScalarType::Float32),
            11 => Ok(ImageScalarType::Float64),
            _ => Err(IgtlError::InvalidSize {
                expected: 0,
                actual: value as usize,
            }),
        }
    }
}

/// Endianness type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Big = 1,
    Little = 2,
}

impl Endian {
    /// Create from endian value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Endian::Big),
            2 => Ok(Endian::Little),
            _ => Err(IgtlError::InvalidSize {
                expected: 1,
                actual: value as usize,
            }),
        }
    }
}

/// Coordinate system type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSystem {
    RAS = 1, // Right-Anterior-Superior (common in medical imaging)
    LPS = 2, // Left-Posterior-Superior
}

impl CoordinateSystem {
    /// Create from coordinate value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(CoordinateSystem::RAS),
            2 => Ok(CoordinateSystem::LPS),
            _ => Err(IgtlError::InvalidSize {
                expected: 1,
                actual: value as usize,
            }),
        }
    }
}

/// IMAGE message for 2D/3D medical image data
///
/// # OpenIGTLink Specification
/// - Message type: "IMAGE"
/// - Header: VERSION (uint16) + NUM_COMPONENTS (uint8) + SCALAR_TYPE (uint8) + ENDIAN (uint8) + COORD (uint8) + SIZE (`uint16[3]`) + MATRIX (`float32[12]`)
/// - Header size: 2 + 1 + 1 + 1 + 1 + 6 + 48 = 60 bytes
/// - Followed by image data
#[derive(Debug, Clone, PartialEq)]
pub struct ImageMessage {
    /// Protocol version (should be 1 or 2)
    pub version: u16,
    /// Number of components (1=scalar, 3=RGB, 4=RGBA)
    pub num_components: u8,
    /// Scalar type
    pub scalar_type: ImageScalarType,
    /// Endianness of image data
    pub endian: Endian,
    /// Coordinate system
    pub coordinate: CoordinateSystem,
    /// Image size [columns, rows, slices]
    pub size: [u16; 3],
    /// 4x3 transformation matrix (stored row-major, upper 3x4 of 4x4 matrix)
    pub matrix: [[f32; 4]; 3],
    /// Image data (raw bytes)
    pub data: Vec<u8>,
}

impl ImageMessage {
    /// Create a new IMAGE message
    pub fn new(scalar_type: ImageScalarType, size: [u16; 3], data: Vec<u8>) -> Result<Self> {
        let num_components = 1;
        let expected_size = (size[0] as usize)
            * (size[1] as usize)
            * (size[2] as usize)
            * (num_components as usize)
            * scalar_type.size();

        if data.len() != expected_size {
            return Err(IgtlError::InvalidSize {
                expected: expected_size,
                actual: data.len(),
            });
        }

        Ok(ImageMessage {
            version: 1,
            num_components,
            scalar_type,
            endian: Endian::Big,
            coordinate: CoordinateSystem::RAS,
            size,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
            data,
        })
    }

    /// Create with RGB components
    pub fn rgb(scalar_type: ImageScalarType, size: [u16; 3], data: Vec<u8>) -> Result<Self> {
        let num_components = 3;
        let expected_size = (size[0] as usize)
            * (size[1] as usize)
            * (size[2] as usize)
            * (num_components as usize)
            * scalar_type.size();

        if data.len() != expected_size {
            return Err(IgtlError::InvalidSize {
                expected: expected_size,
                actual: data.len(),
            });
        }

        Ok(ImageMessage {
            version: 1,
            num_components,
            scalar_type,
            endian: Endian::Big,
            coordinate: CoordinateSystem::RAS,
            size,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
            data,
        })
    }

    /// Set transformation matrix
    pub fn with_matrix(mut self, matrix: [[f32; 4]; 3]) -> Self {
        self.matrix = matrix;
        self
    }

    /// Set coordinate system
    pub fn with_coordinate(mut self, coordinate: CoordinateSystem) -> Self {
        self.coordinate = coordinate;
        self
    }

    /// Get total number of pixels
    pub fn num_pixels(&self) -> usize {
        (self.size[0] as usize) * (self.size[1] as usize) * (self.size[2] as usize)
    }
}

impl Message for ImageMessage {
    fn message_type() -> &'static str {
        "IMAGE"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(60 + self.data.len());

        // Encode VERSION (uint16)
        buf.put_u16(self.version);

        // Encode NUM_COMPONENTS (uint8)
        buf.put_u8(self.num_components);

        // Encode SCALAR_TYPE (uint8)
        buf.put_u8(self.scalar_type as u8);

        // Encode ENDIAN (uint8)
        buf.put_u8(self.endian as u8);

        // Encode COORD (uint8)
        buf.put_u8(self.coordinate as u8);

        // Encode SIZE (`uint16[3]`)
        for &s in &self.size {
            buf.put_u16(s);
        }

        // Encode MATRIX (`float32[12]`) - row-major order
        for row in &self.matrix {
            for &val in row {
                buf.put_f32(val);
            }
        }

        // Encode image data
        buf.extend_from_slice(&self.data);

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 60 {
            return Err(IgtlError::InvalidSize {
                expected: 60,
                actual: data.len(),
            });
        }

        // Decode VERSION (uint16)
        let version = data.get_u16();

        // Decode NUM_COMPONENTS (uint8)
        let num_components = data.get_u8();

        // Decode SCALAR_TYPE (uint8)
        let scalar_type = ImageScalarType::from_u8(data.get_u8())?;

        // Decode ENDIAN (uint8)
        let endian = Endian::from_u8(data.get_u8())?;

        // Decode COORD (uint8)
        let coordinate = CoordinateSystem::from_u8(data.get_u8())?;

        // Decode SIZE (`uint16[3]`)
        let size = [data.get_u16(), data.get_u16(), data.get_u16()];

        // Decode MATRIX (`float32[12]`)
        let mut matrix = [[0.0f32; 4]; 3];
        for row in &mut matrix {
            for val in row {
                *val = data.get_f32();
            }
        }

        // Decode image data (remaining bytes)
        let image_data = data.to_vec();

        // Validate data size
        let expected_size = (size[0] as usize)
            * (size[1] as usize)
            * (size[2] as usize)
            * (num_components as usize)
            * scalar_type.size();

        if image_data.len() != expected_size {
            return Err(IgtlError::InvalidSize {
                expected: expected_size,
                actual: image_data.len(),
            });
        }

        Ok(ImageMessage {
            version,
            num_components,
            scalar_type,
            endian,
            coordinate,
            size,
            matrix,
            data: image_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(ImageMessage::message_type(), "IMAGE");
    }

    #[test]
    fn test_scalar_type_size() {
        assert_eq!(ImageScalarType::Uint8.size(), 1);
        assert_eq!(ImageScalarType::Uint16.size(), 2);
        assert_eq!(ImageScalarType::Float32.size(), 4);
        assert_eq!(ImageScalarType::Float64.size(), 8);
    }

    #[test]
    fn test_new_2d() {
        let size = [256, 256, 1];
        let data_size = 256 * 256;
        let data = vec![0u8; data_size];

        let img = ImageMessage::new(ImageScalarType::Uint8, size, data).unwrap();
        assert_eq!(img.size, size);
        assert_eq!(img.num_components, 1);
    }

    #[test]
    fn test_new_3d() {
        let size = [128, 128, 64];
        let data_size = 128 * 128 * 64;
        let data = vec![0u8; data_size];

        let img = ImageMessage::new(ImageScalarType::Uint8, size, data).unwrap();
        assert_eq!(img.size, size);
        assert_eq!(img.num_pixels(), 128 * 128 * 64);
    }

    #[test]
    fn test_rgb() {
        let size = [100, 100, 1];
        let data_size = 100 * 100 * 3; // 3 components
        let data = vec![0u8; data_size];

        let img = ImageMessage::rgb(ImageScalarType::Uint8, size, data).unwrap();
        assert_eq!(img.num_components, 3);
    }

    #[test]
    fn test_invalid_data_size() {
        let size = [10, 10, 1];
        let data = vec![0u8; 50]; // Too small

        let result = ImageMessage::new(ImageScalarType::Uint8, size, data);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_matrix() {
        let size = [10, 10, 1];
        let data = vec![0u8; 100];
        let matrix = [
            [2.0, 0.0, 0.0, 10.0],
            [0.0, 2.0, 0.0, 20.0],
            [0.0, 0.0, 1.0, 0.0],
        ];

        let img = ImageMessage::new(ImageScalarType::Uint8, size, data)
            .unwrap()
            .with_matrix(matrix);

        assert_eq!(img.matrix[0][3], 10.0);
        assert_eq!(img.matrix[1][3], 20.0);
    }

    #[test]
    fn test_encode_decode_small() {
        let size = [4, 4, 1];
        let data = vec![128u8; 16];

        let original = ImageMessage::new(ImageScalarType::Uint8, size, data).unwrap();
        let encoded = original.encode_content().unwrap();
        let decoded = ImageMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.version, original.version);
        assert_eq!(decoded.num_components, original.num_components);
        assert_eq!(decoded.scalar_type, original.scalar_type);
        assert_eq!(decoded.size, original.size);
        assert_eq!(decoded.data, original.data);
    }

    #[test]
    fn test_roundtrip_uint16() {
        let size = [8, 8, 2];
        let data_size = 8 * 8 * 2 * 2; // uint16 = 2 bytes
        let data = vec![0u8; data_size];

        let original = ImageMessage::new(ImageScalarType::Uint16, size, data).unwrap();
        let encoded = original.encode_content().unwrap();
        let decoded = ImageMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.scalar_type, ImageScalarType::Uint16);
        assert_eq!(decoded.size, size);
        assert_eq!(decoded.data.len(), data_size);
    }

    #[test]
    fn test_roundtrip_with_matrix() {
        let size = [5, 5, 1];
        let data = vec![255u8; 25];
        let matrix = [
            [1.0, 0.0, 0.0, 5.0],
            [0.0, 1.0, 0.0, 10.0],
            [0.0, 0.0, 1.0, 15.0],
        ];

        let original = ImageMessage::new(ImageScalarType::Uint8, size, data)
            .unwrap()
            .with_matrix(matrix)
            .with_coordinate(CoordinateSystem::LPS);

        let encoded = original.encode_content().unwrap();
        let decoded = ImageMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.coordinate, CoordinateSystem::LPS);
        assert_eq!(decoded.matrix, matrix);
    }

    #[test]
    fn test_decode_invalid_header() {
        let data = vec![0u8; 50]; // Too short
        let result = ImageMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
