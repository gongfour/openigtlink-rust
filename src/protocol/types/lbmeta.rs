//! LBMETA (LabelMeta) message type implementation
//!
//! The LBMETA message is used to transfer label/segmentation metadata not available
//! in LABEL messages, such as label names, colors, and owner information.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// Label metadata element
#[derive(Debug, Clone, PartialEq)]
pub struct LabelMetaElement {
    /// Name or description of the label (max 64 chars)
    pub name: String,
    /// ID to query the LABEL (max 20 chars)
    pub id: String,
    /// Label value (intensity value in the label image)
    pub label: u8,
    /// RGBA color for visualization
    pub rgba: [u8; 4],
    /// Number of pixels in each direction (RI, RJ, RK)
    pub size: [u16; 3],
    /// Owner image ID (max 20 chars)
    pub owner: String,
}

impl LabelMetaElement {
    /// Create a new label metadata element
    pub fn new(name: impl Into<String>, id: impl Into<String>, label: u8) -> Self {
        LabelMetaElement {
            name: name.into(),
            id: id.into(),
            label,
            rgba: [255, 255, 255, 255], // Default to white
            size: [0, 0, 0],
            owner: String::new(),
        }
    }

    /// Set RGBA color
    pub fn with_rgba(mut self, rgba: [u8; 4]) -> Self {
        self.rgba = rgba;
        self
    }

    /// Set label size
    pub fn with_size(mut self, size: [u16; 3]) -> Self {
        self.size = size;
        self
    }

    /// Set owner image
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = owner.into();
        self
    }
}

/// LBMETA message containing multiple label metadata elements
///
/// # OpenIGTLink Specification
/// - Message type: "LBMETA"
/// - Each element: NAME (`char[64]`) + ID (`char[20]`) + LABEL (uint8) + Reserved (uint8) + RGBA (`uint8[4]`) + SIZE (`uint16[3]`) + OWNER (`char[20]`)
/// - Element size: 64 + 20 + 1 + 1 + 4 + 6 + 20 = 116 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct LbMetaMessage {
    /// List of label metadata elements
    pub labels: Vec<LabelMetaElement>,
}

impl LbMetaMessage {
    /// Create a new LBMETA message
    pub fn new(labels: Vec<LabelMetaElement>) -> Self {
        LbMetaMessage { labels }
    }

    /// Create an empty LBMETA message
    pub fn empty() -> Self {
        LbMetaMessage { labels: Vec::new() }
    }

    /// Add a label metadata element
    pub fn add_label(&mut self, label: LabelMetaElement) {
        self.labels.push(label);
    }

    /// Get number of labels
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Check if message has no labels
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

impl Message for LbMetaMessage {
    fn message_type() -> &'static str {
        "LBMETA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.labels.len() * 116);

        for label in &self.labels {
            // Encode NAME (`char[64]`)
            let mut name_bytes = [0u8; 64];
            let name_str = label.name.as_bytes();
            let copy_len = name_str.len().min(63);
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);

            // Encode ID (`char[20]`)
            let mut id_bytes = [0u8; 20];
            let id_str = label.id.as_bytes();
            let copy_len = id_str.len().min(19);
            id_bytes[..copy_len].copy_from_slice(&id_str[..copy_len]);
            buf.extend_from_slice(&id_bytes);

            // Encode LABEL (uint8)
            buf.put_u8(label.label);

            // Encode Reserved (uint8)
            buf.put_u8(0);

            // Encode RGBA (`uint8[4]`)
            buf.extend_from_slice(&label.rgba);

            // Encode SIZE (`uint16[3]`)
            for &s in &label.size {
                buf.put_u16(s);
            }

            // Encode OWNER (`char[20]`)
            let mut owner_bytes = [0u8; 20];
            let owner_str = label.owner.as_bytes();
            let copy_len = owner_str.len().min(19);
            owner_bytes[..copy_len].copy_from_slice(&owner_str[..copy_len]);
            buf.extend_from_slice(&owner_bytes);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        let mut labels = Vec::new();

        while data.len() >= 116 {
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

            // Decode LABEL (uint8)
            let label = data.get_u8();

            // Decode Reserved (uint8)
            let _reserved = data.get_u8();

            // Decode RGBA (`uint8[4]`)
            let rgba = [data.get_u8(), data.get_u8(), data.get_u8(), data.get_u8()];

            // Decode SIZE (`uint16[3]`)
            let size = [data.get_u16(), data.get_u16(), data.get_u16()];

            // Decode OWNER (`char[20]`)
            let owner_bytes = &data[..20];
            data.advance(20);
            let owner_len = owner_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let owner = String::from_utf8(owner_bytes[..owner_len].to_vec())?;

            labels.push(LabelMetaElement {
                name,
                id,
                label,
                rgba,
                size,
                owner,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(LbMetaMessage { labels })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(LbMetaMessage::message_type(), "LBMETA");
    }

    #[test]
    fn test_empty() {
        let msg = LbMetaMessage::empty();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn test_new() {
        let elem = LabelMetaElement::new("Liver", "LBL001", 1);
        assert_eq!(elem.name, "Liver");
        assert_eq!(elem.id, "LBL001");
        assert_eq!(elem.label, 1);
        assert_eq!(elem.rgba, [255, 255, 255, 255]);
    }

    #[test]
    fn test_with_rgba() {
        let elem = LabelMetaElement::new("Heart", "LBL002", 2).with_rgba([255, 0, 0, 255]);
        assert_eq!(elem.rgba, [255, 0, 0, 255]);
    }

    #[test]
    fn test_with_size() {
        let elem = LabelMetaElement::new("Kidney", "LBL003", 3).with_size([256, 256, 100]);
        assert_eq!(elem.size, [256, 256, 100]);
    }

    #[test]
    fn test_add_label() {
        let mut msg = LbMetaMessage::empty();
        msg.add_label(LabelMetaElement::new("Liver", "LBL001", 1));
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_encode_single() {
        let elem = LabelMetaElement::new("TestLabel", "TEST001", 1);
        let msg = LbMetaMessage::new(vec![elem]);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 116);
    }

    #[test]
    fn test_roundtrip() {
        let original = LbMetaMessage::new(vec![LabelMetaElement::new("Liver", "LBL001", 1)
            .with_rgba([139, 69, 19, 255])
            .with_size([512, 512, 200])
            .with_owner("CT001")]);

        let encoded = original.encode_content().unwrap();
        let decoded = LbMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.labels.len(), 1);
        assert_eq!(decoded.labels[0].name, "Liver");
        assert_eq!(decoded.labels[0].id, "LBL001");
        assert_eq!(decoded.labels[0].label, 1);
        assert_eq!(decoded.labels[0].rgba, [139, 69, 19, 255]);
        assert_eq!(decoded.labels[0].size, [512, 512, 200]);
        assert_eq!(decoded.labels[0].owner, "CT001");
    }

    #[test]
    fn test_roundtrip_multiple() {
        let original = LbMetaMessage::new(vec![
            LabelMetaElement::new("Liver", "LBL001", 1).with_rgba([139, 69, 19, 255]),
            LabelMetaElement::new("Heart", "LBL002", 2).with_rgba([255, 0, 0, 255]),
            LabelMetaElement::new("Kidney", "LBL003", 3).with_rgba([0, 255, 0, 255]),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = LbMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.labels.len(), 3);
        assert_eq!(decoded.labels[0].name, "Liver");
        assert_eq!(decoded.labels[1].name, "Heart");
        assert_eq!(decoded.labels[2].name, "Kidney");
    }

    #[test]
    fn test_empty_message() {
        let msg = LbMetaMessage::empty();
        let encoded = msg.encode_content().unwrap();
        let decoded = LbMetaMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.labels.len(), 0);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 115]; // One byte short
        let result = LbMetaMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
