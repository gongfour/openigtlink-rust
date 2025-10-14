//! IMGMETA (ImageMeta) message type implementation
//!
//! The IMGMETA message is used to transfer image metadata not available in IMAGE messages,
//! such as patient information, modality, etc.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// Image metadata element
#[derive(Debug, Clone, PartialEq)]
pub struct ImageMetaElement {
    /// Name or description of the image (max 64 chars)
    pub name: String,
    /// ID to query the IMAGE (max 20 chars)
    pub id: String,
    /// Modality (e.g., "CT", "MRI") (max 32 chars)
    pub modality: String,
    /// Patient name (max 64 chars)
    pub patient_name: String,
    /// Patient ID (max 64 chars)
    pub patient_id: String,
    /// Scan timestamp
    pub timestamp: u64,
    /// Number of pixels in each direction (RI, RJ, RK)
    pub size: [u16; 3],
    /// Scalar type (same as IMAGE: 3=uint8, 5=uint16, etc.)
    pub scalar_type: u8,
}

impl ImageMetaElement {
    /// Create a new image metadata element
    pub fn new(
        name: impl Into<String>,
        id: impl Into<String>,
        modality: impl Into<String>,
    ) -> Self {
        ImageMetaElement {
            name: name.into(),
            id: id.into(),
            modality: modality.into(),
            patient_name: String::new(),
            patient_id: String::new(),
            timestamp: 0,
            size: [0, 0, 0],
            scalar_type: 3, // Default to uint8
        }
    }

    /// Set patient information
    pub fn with_patient(
        mut self,
        patient_name: impl Into<String>,
        patient_id: impl Into<String>,
    ) -> Self {
        self.patient_name = patient_name.into();
        self.patient_id = patient_id.into();
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set image size
    pub fn with_size(mut self, size: [u16; 3]) -> Self {
        self.size = size;
        self
    }

    /// Set scalar type
    pub fn with_scalar_type(mut self, scalar_type: u8) -> Self {
        self.scalar_type = scalar_type;
        self
    }
}

/// IMGMETA message containing multiple image metadata elements
///
/// # OpenIGTLink Specification
/// - Message type: "IMGMETA"
/// - Each element: NAME (`char[64]`) + ID (`char[20]`) + MODALITY (`char[32]`) + PATIENT_NAME (`char[64]`) + PATIENT_ID (`char[64]`) + TIMESTAMP (uint64) + SIZE (`uint16[3]`) + SCALAR_TYPE (uint8) + Reserved (uint8)
/// - Element size: 64 + 20 + 32 + 64 + 64 + 8 + 6 + 1 + 1 = 260 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct ImgMetaMessage {
    /// List of image metadata elements
    pub images: Vec<ImageMetaElement>,
}

impl ImgMetaMessage {
    /// Create a new IMGMETA message
    pub fn new(images: Vec<ImageMetaElement>) -> Self {
        ImgMetaMessage { images }
    }

    /// Create an empty IMGMETA message
    pub fn empty() -> Self {
        ImgMetaMessage { images: Vec::new() }
    }

    /// Add an image metadata element
    pub fn add_image(&mut self, image: ImageMetaElement) {
        self.images.push(image);
    }

    /// Get number of images
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Check if message has no images
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

impl Message for ImgMetaMessage {
    fn message_type() -> &'static str {
        "IMGMETA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.images.len() * 260);

        for img in &self.images {
            // Encode NAME (`char[64]`)
            let mut name_bytes = [0u8; 64];
            let name_str = img.name.as_bytes();
            let copy_len = name_str.len().min(63);
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);

            // Encode ID (`char[20]`)
            let mut id_bytes = [0u8; 20];
            let id_str = img.id.as_bytes();
            let copy_len = id_str.len().min(19);
            id_bytes[..copy_len].copy_from_slice(&id_str[..copy_len]);
            buf.extend_from_slice(&id_bytes);

            // Encode MODALITY (`char[32]`)
            let mut modality_bytes = [0u8; 32];
            let modality_str = img.modality.as_bytes();
            let copy_len = modality_str.len().min(31);
            modality_bytes[..copy_len].copy_from_slice(&modality_str[..copy_len]);
            buf.extend_from_slice(&modality_bytes);

            // Encode PATIENT_NAME (`char[64]`)
            let mut patient_name_bytes = [0u8; 64];
            let patient_name_str = img.patient_name.as_bytes();
            let copy_len = patient_name_str.len().min(63);
            patient_name_bytes[..copy_len].copy_from_slice(&patient_name_str[..copy_len]);
            buf.extend_from_slice(&patient_name_bytes);

            // Encode PATIENT_ID (`char[64]`)
            let mut patient_id_bytes = [0u8; 64];
            let patient_id_str = img.patient_id.as_bytes();
            let copy_len = patient_id_str.len().min(63);
            patient_id_bytes[..copy_len].copy_from_slice(&patient_id_str[..copy_len]);
            buf.extend_from_slice(&patient_id_bytes);

            // Encode TIMESTAMP (uint64)
            buf.put_u64(img.timestamp);

            // Encode SIZE (uint16[3])
            for &s in &img.size {
                buf.put_u16(s);
            }

            // Encode SCALAR_TYPE (uint8)
            buf.put_u8(img.scalar_type);

            // Encode Reserved (uint8)
            buf.put_u8(0);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        let mut images = Vec::new();

        while data.len() >= 260 {
            // Decode NAME (`char[64]`)
            let name_bytes = &data[..64];
            data.advance(64);
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(64);
            let name = String::from_utf8(name_bytes[..name_len].to_vec())?;

            // Decode ID (`char[20]`)
            let id_bytes = &data[..20];
            data.advance(20);
            let id_len = id_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let id = String::from_utf8(id_bytes[..id_len].to_vec())?;

            // Decode MODALITY (`char[32]`)
            let modality_bytes = &data[..32];
            data.advance(32);
            let modality_len = modality_bytes.iter().position(|&b| b == 0).unwrap_or(32);
            let modality = String::from_utf8(modality_bytes[..modality_len].to_vec())?;

            // Decode PATIENT_NAME (`char[64]`)
            let patient_name_bytes = &data[..64];
            data.advance(64);
            let patient_name_len = patient_name_bytes
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(64);
            let patient_name = String::from_utf8(patient_name_bytes[..patient_name_len].to_vec())?;

            // Decode PATIENT_ID (`char[64]`)
            let patient_id_bytes = &data[..64];
            data.advance(64);
            let patient_id_len = patient_id_bytes.iter().position(|&b| b == 0).unwrap_or(64);
            let patient_id = String::from_utf8(patient_id_bytes[..patient_id_len].to_vec())?;

            // Decode TIMESTAMP (uint64)
            let timestamp = data.get_u64();

            // Decode SIZE (uint16[3])
            let size = [data.get_u16(), data.get_u16(), data.get_u16()];

            // Decode SCALAR_TYPE (uint8)
            let scalar_type = data.get_u8();

            // Decode Reserved (uint8)
            let _reserved = data.get_u8();

            images.push(ImageMetaElement {
                name,
                id,
                modality,
                patient_name,
                patient_id,
                timestamp,
                size,
                scalar_type,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(ImgMetaMessage { images })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(ImgMetaMessage::message_type(), "IMGMETA");
    }

    #[test]
    fn test_empty() {
        let msg = ImgMetaMessage::empty();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn test_new() {
        let elem = ImageMetaElement::new("Image1", "IMG001", "CT");
        assert_eq!(elem.name, "Image1");
        assert_eq!(elem.id, "IMG001");
        assert_eq!(elem.modality, "CT");
    }

    #[test]
    fn test_with_patient() {
        let elem =
            ImageMetaElement::new("Image1", "IMG001", "MRI").with_patient("John Doe", "P12345");
        assert_eq!(elem.patient_name, "John Doe");
        assert_eq!(elem.patient_id, "P12345");
    }

    #[test]
    fn test_with_size() {
        let elem = ImageMetaElement::new("Image1", "IMG001", "CT").with_size([512, 512, 128]);
        assert_eq!(elem.size, [512, 512, 128]);
    }

    #[test]
    fn test_add_image() {
        let mut msg = ImgMetaMessage::empty();
        msg.add_image(ImageMetaElement::new("Image1", "IMG001", "CT"));
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_encode_single() {
        let elem = ImageMetaElement::new("TestImage", "TEST001", "CT");
        let msg = ImgMetaMessage::new(vec![elem]);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 260);
    }

    #[test]
    fn test_roundtrip() {
        let original = ImgMetaMessage::new(vec![
            ImageMetaElement::new("CTScan1", "CT001", "CT")
                .with_patient("Jane Smith", "P67890")
                .with_timestamp(1234567890)
                .with_size([512, 512, 200])
                .with_scalar_type(5), // uint16
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = ImgMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.images.len(), 1);
        assert_eq!(decoded.images[0].name, "CTScan1");
        assert_eq!(decoded.images[0].id, "CT001");
        assert_eq!(decoded.images[0].modality, "CT");
        assert_eq!(decoded.images[0].patient_name, "Jane Smith");
        assert_eq!(decoded.images[0].patient_id, "P67890");
        assert_eq!(decoded.images[0].timestamp, 1234567890);
        assert_eq!(decoded.images[0].size, [512, 512, 200]);
        assert_eq!(decoded.images[0].scalar_type, 5);
    }

    #[test]
    fn test_roundtrip_multiple() {
        let original = ImgMetaMessage::new(vec![
            ImageMetaElement::new("CT1", "CT001", "CT"),
            ImageMetaElement::new("MRI1", "MRI001", "MRI"),
            ImageMetaElement::new("US1", "US001", "Ultrasound"),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = ImgMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.images.len(), 3);
        assert_eq!(decoded.images[0].modality, "CT");
        assert_eq!(decoded.images[1].modality, "MRI");
        assert_eq!(decoded.images[2].modality, "Ultrasound");
    }

    #[test]
    fn test_empty_message() {
        let msg = ImgMetaMessage::empty();
        let encoded = msg.encode_content().unwrap();
        let decoded = ImgMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.images.len(), 0);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 259]; // One byte short
        let result = ImgMetaMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
