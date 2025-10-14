//! QTDATA (QuaternionTrackingData) message type implementation
//!
//! The QTDATA message type is used to transfer 3D positions and orientations
//! of surgical tools, markers, etc. using quaternions for orientation.

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// Instrument type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InstrumentType {
    Tracker = 1,
    Instrument6D = 2,
    Instrument3D = 3,
    Instrument5D = 4,
}

impl InstrumentType {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(InstrumentType::Tracker),
            2 => Ok(InstrumentType::Instrument6D),
            3 => Ok(InstrumentType::Instrument3D),
            4 => Ok(InstrumentType::Instrument5D),
            _ => Err(IgtlError::InvalidHeader(format!(
                "Invalid instrument type: {}",
                value
            ))),
        }
    }
}

/// Tracking data element with name, type, position and quaternion
#[derive(Debug, Clone, PartialEq)]
pub struct TrackingElement {
    /// Name/ID of the instrument/tracker (max 20 chars)
    pub name: String,
    /// Type of instrument
    pub instrument_type: InstrumentType,
    /// Position (x, y, z) in millimeters
    pub position: [f32; 3],
    /// Orientation as quaternion (qx, qy, qz, w)
    pub quaternion: [f32; 4],
}

impl TrackingElement {
    /// Create a new tracking element
    pub fn new(
        name: impl Into<String>,
        instrument_type: InstrumentType,
        position: [f32; 3],
        quaternion: [f32; 4],
    ) -> Self {
        TrackingElement {
            name: name.into(),
            instrument_type,
            position,
            quaternion,
        }
    }
}

/// QTDATA message containing multiple tracking elements
///
/// # OpenIGTLink Specification
/// - Message type: "QTDATA"
/// - Each element: NAME (`char[20]`) + TYPE (uint8) + Reserved (uint8) + POSITION (`float32[3]`) + QUATERNION (`float32[4]`)
/// - Element size: 20 + 1 + 1 + 12 + 16 = 50 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct QtDataMessage {
    /// List of tracking elements
    pub elements: Vec<TrackingElement>,
}

impl QtDataMessage {
    /// Create a new QTDATA message with elements
    pub fn new(elements: Vec<TrackingElement>) -> Self {
        QtDataMessage { elements }
    }

    /// Create an empty QTDATA message
    pub fn empty() -> Self {
        QtDataMessage {
            elements: Vec::new(),
        }
    }

    /// Add a tracking element
    pub fn add_element(&mut self, element: TrackingElement) {
        self.elements.push(element);
    }

    /// Get number of tracking elements
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if message has no elements
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl Message for QtDataMessage {
    fn message_type() -> &'static str {
        "QTDATA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.elements.len() * 50);

        for element in &self.elements {
            // Encode NAME (`char[20]`)
            let mut name_bytes = [0u8; 20];
            let name_str = element.name.as_bytes();
            let copy_len = name_str.len().min(19); // Reserve 1 byte for null terminator
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);

            // Encode TYPE (uint8)
            buf.put_u8(element.instrument_type as u8);

            // Encode Reserved (uint8)
            buf.put_u8(0);

            // Encode POSITION (`float32[3]`)
            for &coord in &element.position {
                buf.put_f32(coord);
            }

            // Encode QUATERNION (`float32[4]`)
            for &comp in &element.quaternion {
                buf.put_f32(comp);
            }
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        let mut elements = Vec::new();

        while data.len() >= 50 {
            // Decode NAME (`char[20]`)
            let name_bytes = &data[..20];
            data.advance(20);

            // Find null terminator or use full length
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let name = String::from_utf8(name_bytes[..name_len].to_vec())?;

            // Decode TYPE (uint8)
            let instrument_type = InstrumentType::from_u8(data.get_u8())?;

            // Decode Reserved (uint8)
            let _reserved = data.get_u8();

            // Decode POSITION (`float32[3]`)
            let position = [data.get_f32(), data.get_f32(), data.get_f32()];

            // Decode QUATERNION (`float32[4]`)
            let quaternion = [
                data.get_f32(),
                data.get_f32(),
                data.get_f32(),
                data.get_f32(),
            ];

            elements.push(TrackingElement {
                name,
                instrument_type,
                position,
                quaternion,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(QtDataMessage { elements })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(QtDataMessage::message_type(), "QTDATA");
    }

    #[test]
    fn test_instrument_type() {
        assert_eq!(InstrumentType::Tracker as u8, 1);
        assert_eq!(InstrumentType::Instrument6D as u8, 2);
        assert_eq!(InstrumentType::Instrument3D as u8, 3);
        assert_eq!(InstrumentType::Instrument5D as u8, 4);
    }

    #[test]
    fn test_empty() {
        let msg = QtDataMessage::empty();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn test_new() {
        let elem = TrackingElement::new(
            "Tool1",
            InstrumentType::Instrument6D,
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0],
        );
        let msg = QtDataMessage::new(vec![elem]);
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_add_element() {
        let mut msg = QtDataMessage::empty();
        msg.add_element(TrackingElement::new(
            "Tool1",
            InstrumentType::Tracker,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ));
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_encode_single_element() {
        let elem = TrackingElement::new(
            "Tool",
            InstrumentType::Instrument6D,
            [10.0, 20.0, 30.0],
            [0.1, 0.2, 0.3, 0.9],
        );
        let msg = QtDataMessage::new(vec![elem]);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 50);
        // Check TYPE field
        assert_eq!(encoded[20], 2); // Instrument6D
                                    // Check Reserved field
        assert_eq!(encoded[21], 0);
    }

    #[test]
    fn test_roundtrip_single() {
        let original = QtDataMessage::new(vec![TrackingElement::new(
            "Tracker1",
            InstrumentType::Tracker,
            [100.5, 200.5, 300.5],
            [0.1, 0.2, 0.3, 0.9],
        )]);

        let encoded = original.encode_content().unwrap();
        let decoded = QtDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.elements.len(), 1);
        assert_eq!(decoded.elements[0].name, "Tracker1");
        assert_eq!(decoded.elements[0].instrument_type, InstrumentType::Tracker);
        assert_eq!(decoded.elements[0].position, [100.5, 200.5, 300.5]);
        assert_eq!(decoded.elements[0].quaternion, [0.1, 0.2, 0.3, 0.9]);
    }

    #[test]
    fn test_roundtrip_multiple() {
        let original = QtDataMessage::new(vec![
            TrackingElement::new(
                "Tool1",
                InstrumentType::Instrument6D,
                [1.0, 2.0, 3.0],
                [0.0, 0.0, 0.0, 1.0],
            ),
            TrackingElement::new(
                "Tool2",
                InstrumentType::Instrument3D,
                [4.0, 5.0, 6.0],
                [0.1, 0.2, 0.3, 0.9],
            ),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = QtDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.elements.len(), 2);
        assert_eq!(decoded.elements[0].name, "Tool1");
        assert_eq!(decoded.elements[1].name, "Tool2");
    }

    #[test]
    fn test_name_truncation() {
        let long_name = "ThisIsAVeryLongNameThatExceedsTwentyCharacters";
        let elem = TrackingElement::new(
            long_name,
            InstrumentType::Tracker,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        );
        let msg = QtDataMessage::new(vec![elem]);

        let encoded = msg.encode_content().unwrap();
        let decoded = QtDataMessage::decode_content(&encoded).unwrap();

        // Name should be truncated to 19 chars (20th is null terminator)
        assert!(decoded.elements[0].name.len() <= 19);
    }

    #[test]
    fn test_empty_message() {
        let msg = QtDataMessage::empty();
        let encoded = msg.encode_content().unwrap();
        let decoded = QtDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.elements.len(), 0);
        assert_eq!(encoded.len(), 0);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 49]; // One byte short of a complete element
        let result = QtDataMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
