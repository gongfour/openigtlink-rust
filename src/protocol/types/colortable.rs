//! COLORTABLE (Color Table) message type implementation
//!
//! The COLORTABLE message is used to transfer color lookup tables for
//! visualization of label images or other indexed data.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// Color table entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEntry {
    /// RGBA color values
    pub rgba: [u8; 4],
}

impl ColorEntry {
    /// Create a new color entry
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        ColorEntry { rgba: [r, g, b, a] }
    }

    /// Create a color entry from RGBA array
    pub fn from_rgba(rgba: [u8; 4]) -> Self {
        ColorEntry { rgba }
    }
}

/// Index type for color table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// Uint8 index (0-255)
    Uint8 = 3,
    /// Uint16 index (0-65535)
    Uint16 = 5,
}

impl IndexType {
    /// Get size in bytes
    pub fn size(&self) -> usize {
        match self {
            IndexType::Uint8 => 1,
            IndexType::Uint16 => 2,
        }
    }

    /// Create from scalar type value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            3 => Ok(IndexType::Uint8),
            5 => Ok(IndexType::Uint16),
            _ => Err(IgtlError::InvalidSize {
                expected: 3,
                actual: value as usize,
            }),
        }
    }
}

/// COLORTABLE message containing a color lookup table
///
/// # OpenIGTLink Specification
/// - Message type: "COLORTABLE"
/// - Format: INDEX_TYPE (uint8) + Reserved (uint8) + MAP (`rgba[n]`)
/// - INDEX_TYPE: 3=uint8, 5=uint16
/// - Number of colors determined by body size
#[derive(Debug, Clone, PartialEq)]
pub struct ColorTableMessage {
    /// Index type (uint8 or uint16)
    pub index_type: IndexType,
    /// Color map entries
    pub colors: Vec<ColorEntry>,
}

impl ColorTableMessage {
    /// Create a new color table message
    pub fn new(index_type: IndexType, colors: Vec<ColorEntry>) -> Self {
        ColorTableMessage { index_type, colors }
    }

    /// Create a uint8 color table (256 colors max)
    pub fn uint8(colors: Vec<ColorEntry>) -> Result<Self> {
        if colors.len() > 256 {
            return Err(IgtlError::InvalidSize {
                expected: 256,
                actual: colors.len(),
            });
        }
        Ok(ColorTableMessage {
            index_type: IndexType::Uint8,
            colors,
        })
    }

    /// Create a uint16 color table (65536 colors max)
    pub fn uint16(colors: Vec<ColorEntry>) -> Result<Self> {
        if colors.len() > 65536 {
            return Err(IgtlError::InvalidSize {
                expected: 65536,
                actual: colors.len(),
            });
        }
        Ok(ColorTableMessage {
            index_type: IndexType::Uint16,
            colors,
        })
    }

    /// Get number of colors
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Check if color table is empty
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Get color at index
    pub fn get(&self, index: usize) -> Option<ColorEntry> {
        self.colors.get(index).copied()
    }
}

impl Message for ColorTableMessage {
    fn message_type() -> &'static str {
        "COLORTABLE"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(2 + self.colors.len() * 4);

        // Encode INDEX_TYPE (uint8)
        buf.put_u8(self.index_type as u8);

        // Encode Reserved (uint8)
        buf.put_u8(0);

        // Encode color map
        for color in &self.colors {
            buf.extend_from_slice(&color.rgba);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(IgtlError::InvalidSize {
                expected: 2,
                actual: data.len(),
            });
        }

        // Decode INDEX_TYPE (uint8)
        let index_type = IndexType::from_u8(data.get_u8())?;

        // Decode Reserved (uint8)
        let _reserved = data.get_u8();

        // Decode color map
        let mut colors = Vec::new();
        while data.len() >= 4 {
            let rgba = [data.get_u8(), data.get_u8(), data.get_u8(), data.get_u8()];
            colors.push(ColorEntry { rgba });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(ColorTableMessage { index_type, colors })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(ColorTableMessage::message_type(), "COLORTABLE");
    }

    #[test]
    fn test_color_entry() {
        let color = ColorEntry::new(255, 0, 0, 255);
        assert_eq!(color.rgba, [255, 0, 0, 255]);
    }

    #[test]
    fn test_from_rgba() {
        let color = ColorEntry::from_rgba([0, 255, 0, 128]);
        assert_eq!(color.rgba, [0, 255, 0, 128]);
    }

    #[test]
    fn test_index_type_size() {
        assert_eq!(IndexType::Uint8.size(), 1);
        assert_eq!(IndexType::Uint16.size(), 2);
    }

    #[test]
    fn test_uint8_table() {
        let colors = vec![
            ColorEntry::new(255, 0, 0, 255),
            ColorEntry::new(0, 255, 0, 255),
            ColorEntry::new(0, 0, 255, 255),
        ];
        let table = ColorTableMessage::uint8(colors).unwrap();
        assert_eq!(table.index_type, IndexType::Uint8);
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn test_uint8_overflow() {
        let colors = vec![ColorEntry::new(0, 0, 0, 255); 257];
        let result = ColorTableMessage::uint8(colors);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_color() {
        let colors = vec![
            ColorEntry::new(255, 0, 0, 255),
            ColorEntry::new(0, 255, 0, 255),
        ];
        let table = ColorTableMessage::uint8(colors).unwrap();

        assert_eq!(table.get(0).unwrap().rgba, [255, 0, 0, 255]);
        assert_eq!(table.get(1).unwrap().rgba, [0, 255, 0, 255]);
        assert!(table.get(2).is_none());
    }

    #[test]
    fn test_encode() {
        let colors = vec![ColorEntry::new(255, 0, 0, 255)];
        let table = ColorTableMessage::uint8(colors).unwrap();
        let encoded = table.encode_content().unwrap();

        // 2 bytes header + 4 bytes color = 6 bytes
        assert_eq!(encoded.len(), 6);
        assert_eq!(encoded[0], 3); // INDEX_TYPE = uint8
    }

    #[test]
    fn test_roundtrip_uint8() {
        let original = ColorTableMessage::uint8(vec![
            ColorEntry::new(255, 0, 0, 255),
            ColorEntry::new(0, 255, 0, 255),
            ColorEntry::new(0, 0, 255, 255),
            ColorEntry::new(255, 255, 0, 255),
        ])
        .unwrap();

        let encoded = original.encode_content().unwrap();
        let decoded = ColorTableMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.index_type, IndexType::Uint8);
        assert_eq!(decoded.colors.len(), 4);
        assert_eq!(decoded.colors[0].rgba, [255, 0, 0, 255]);
        assert_eq!(decoded.colors[1].rgba, [0, 255, 0, 255]);
        assert_eq!(decoded.colors[2].rgba, [0, 0, 255, 255]);
        assert_eq!(decoded.colors[3].rgba, [255, 255, 0, 255]);
    }

    #[test]
    fn test_roundtrip_uint16() {
        let original = ColorTableMessage::uint16(vec![
            ColorEntry::new(128, 128, 128, 255),
            ColorEntry::new(64, 64, 64, 128),
        ])
        .unwrap();

        let encoded = original.encode_content().unwrap();
        let decoded = ColorTableMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.index_type, IndexType::Uint16);
        assert_eq!(decoded.colors.len(), 2);
    }

    #[test]
    fn test_empty_table() {
        let table = ColorTableMessage::uint8(vec![]).unwrap();
        assert!(table.is_empty());

        let encoded = table.encode_content().unwrap();
        let decoded = ColorTableMessage::decode_content(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 5]; // 2 header + 3 bytes (incomplete color)
        let result = ColorTableMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_index_type() {
        let data = vec![99, 0, 255, 0, 0, 255]; // Invalid index type
        let result = ColorTableMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
