//! TDATA (TrackingData) message type implementation
//!
//! The TDATA message type is used to transfer 3D positions and orientations
//! of surgical tools, markers, etc. using transformation matrices.
//!
//! # Use Cases
//!
//! - **Surgical Tool Tracking** - Real-time tracking of multiple instruments (scalpel, probe, drill)
//! - **Optical Tracking Systems** - NDI Polaris, Atracsys, or similar optical trackers
//! - **Electromagnetic Tracking** - Aurora or Ascension electromagnetic trackers
//! - **Surgical Navigation** - Displaying tool positions relative to patient anatomy
//! - **Multi-Tool Coordination** - Tracking multiple tools simultaneously in robotic surgery
//!
//! # TDATA vs TRANSFORM
//!
//! - **TDATA**: Array of named transforms, efficient for tracking multiple tools
//! - **TRANSFORM**: Single 4x4 matrix, simpler but requires multiple messages
//!
//! Use TDATA when tracking ≥2 tools to reduce network overhead.
//!
//! # Examples
//!
//! ## Tracking Multiple Surgical Instruments
//!
//! ```no_run
//! use openigtlink_rust::protocol::types::{TDataMessage, TrackingDataElement, TrackingInstrumentType};
//! use openigtlink_rust::protocol::message::IgtlMessage;
//! use openigtlink_rust::io::ClientBuilder;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .sync()
//!     .build()?;
//!
//! // Tool 1: Scalpel
//! let scalpel = TrackingDataElement::with_translation(
//!     "Scalpel",
//!     TrackingInstrumentType::Instrument5D,
//!     100.0,  // X translation: 100mm
//!     50.0,   // Y translation: 50mm
//!     200.0   // Z translation: 200mm
//! );
//!
//! // Tool 2: Probe
//! let probe = TrackingDataElement::with_translation(
//!     "Probe",
//!     TrackingInstrumentType::Instrument6D,
//!     150.0,
//!     75.0,
//!     180.0
//! );
//!
//! let tdata = TDataMessage::new(vec![scalpel, probe]);
//! let msg = IgtlMessage::new(tdata, "OpticalTracker")?;
//! client.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Receiving Tracking Data at 60Hz
//!
//! ```no_run
//! use openigtlink_rust::io::IgtlServer;
//! use openigtlink_rust::protocol::types::TDataMessage;
//!
//! let server = IgtlServer::bind("0.0.0.0:18944")?;
//! let mut client_conn = server.accept()?;
//!
//! loop {
//!     let message = client_conn.receive::<TDataMessage>()?;
//!
//!     for element in &message.content.elements {
//!         let x = element.matrix[0][3];
//!         let y = element.matrix[1][3];
//!         let z = element.matrix[2][3];
//!
//!         println!("{}: position = ({:.2}, {:.2}, {:.2}) mm",
//!                  element.name, x, y, z);
//!     }
//! }
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// Instrument type for tracking data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TrackingInstrumentType {
    Tracker = 1,
    Instrument6D = 2,
    Instrument3D = 3,
    Instrument5D = 4,
}

impl TrackingInstrumentType {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(TrackingInstrumentType::Tracker),
            2 => Ok(TrackingInstrumentType::Instrument6D),
            3 => Ok(TrackingInstrumentType::Instrument3D),
            4 => Ok(TrackingInstrumentType::Instrument5D),
            _ => Err(IgtlError::InvalidHeader(format!(
                "Invalid tracking instrument type: {}",
                value
            ))),
        }
    }
}

/// Tracking data element with name, type, and transformation matrix
#[derive(Debug, Clone, PartialEq)]
pub struct TrackingDataElement {
    /// Name/ID of the instrument/tracker (max 20 chars)
    pub name: String,
    /// Type of instrument
    pub instrument_type: TrackingInstrumentType,
    /// Upper 3x4 portion of 4x4 transformation matrix (row-major)
    /// Last row [0, 0, 0, 1] is implicit
    pub matrix: [[f32; 4]; 3],
}

impl TrackingDataElement {
    /// Create a new tracking data element
    pub fn new(
        name: impl Into<String>,
        instrument_type: TrackingInstrumentType,
        matrix: [[f32; 4]; 3],
    ) -> Self {
        TrackingDataElement {
            name: name.into(),
            instrument_type,
            matrix,
        }
    }

    /// Create an identity transformation
    pub fn identity(name: impl Into<String>, instrument_type: TrackingInstrumentType) -> Self {
        TrackingDataElement {
            name: name.into(),
            instrument_type,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        }
    }

    /// Create with translation only
    pub fn with_translation(
        name: impl Into<String>,
        instrument_type: TrackingInstrumentType,
        x: f32,
        y: f32,
        z: f32,
    ) -> Self {
        TrackingDataElement {
            name: name.into(),
            instrument_type,
            matrix: [
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
            ],
        }
    }
}

/// TDATA message containing multiple tracking data elements
///
/// # OpenIGTLink Specification
/// - Message type: "TDATA"
/// - Each element: NAME (char[20]) + TYPE (uint8) + Reserved (uint8) + MATRIX (float32[12])
/// - Element size: 20 + 1 + 1 + 48 = 70 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct TDataMessage {
    /// List of tracking data elements
    pub elements: Vec<TrackingDataElement>,
}

impl TDataMessage {
    /// Create a new TDATA message with elements
    pub fn new(elements: Vec<TrackingDataElement>) -> Self {
        TDataMessage { elements }
    }

    /// Create an empty TDATA message
    pub fn empty() -> Self {
        TDataMessage {
            elements: Vec::new(),
        }
    }

    /// Add a tracking element
    pub fn add_element(&mut self, element: TrackingDataElement) {
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

impl Message for TDataMessage {
    fn message_type() -> &'static str {
        "TDATA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.elements.len() * 70);

        for element in &self.elements {
            // Encode NAME (char[20])
            let mut name_bytes = [0u8; 20];
            let name_str = element.name.as_bytes();
            let copy_len = name_str.len().min(19);
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);

            // Encode TYPE (uint8)
            buf.put_u8(element.instrument_type as u8);

            // Encode Reserved (uint8)
            buf.put_u8(0);

            // Encode MATRIX (float32[12]) - upper 3x4 portion
            for row in &element.matrix {
                for &val in row {
                    buf.put_f32(val);
                }
            }
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        let mut elements = Vec::new();

        while data.len() >= 70 {
            // Decode NAME (char[20])
            let name_bytes = &data[..20];
            data.advance(20);

            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let name = String::from_utf8(name_bytes[..name_len].to_vec())?;

            // Decode TYPE (uint8)
            let instrument_type = TrackingInstrumentType::from_u8(data.get_u8())?;

            // Decode Reserved (uint8)
            let _reserved = data.get_u8();

            // Decode MATRIX (float32[12])
            let mut matrix = [[0.0f32; 4]; 3];
            for row in &mut matrix {
                for val in row {
                    *val = data.get_f32();
                }
            }

            elements.push(TrackingDataElement {
                name,
                instrument_type,
                matrix,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(TDataMessage { elements })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(TDataMessage::message_type(), "TDATA");
    }

    #[test]
    fn test_instrument_type() {
        assert_eq!(TrackingInstrumentType::Tracker as u8, 1);
        assert_eq!(TrackingInstrumentType::Instrument6D as u8, 2);
        assert_eq!(TrackingInstrumentType::Instrument3D as u8, 3);
        assert_eq!(TrackingInstrumentType::Instrument5D as u8, 4);
    }

    #[test]
    fn test_empty() {
        let msg = TDataMessage::empty();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn test_identity() {
        let elem = TrackingDataElement::identity("Tool1", TrackingInstrumentType::Instrument6D);
        assert_eq!(elem.matrix[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(elem.matrix[1], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(elem.matrix[2], [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_with_translation() {
        let elem = TrackingDataElement::with_translation(
            "Tool1",
            TrackingInstrumentType::Tracker,
            10.0,
            20.0,
            30.0,
        );
        assert_eq!(elem.matrix[0][3], 10.0);
        assert_eq!(elem.matrix[1][3], 20.0);
        assert_eq!(elem.matrix[2][3], 30.0);
    }

    #[test]
    fn test_add_element() {
        let mut msg = TDataMessage::empty();
        msg.add_element(TrackingDataElement::identity(
            "Tool1",
            TrackingInstrumentType::Tracker,
        ));
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_encode_single_element() {
        let elem = TrackingDataElement::identity("Test", TrackingInstrumentType::Instrument6D);
        let msg = TDataMessage::new(vec![elem]);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 70);
        // Check TYPE field
        assert_eq!(encoded[20], 2); // Instrument6D
        // Check Reserved field
        assert_eq!(encoded[21], 0);
    }

    #[test]
    fn test_roundtrip_single() {
        let matrix = [
            [1.0, 0.0, 0.0, 10.0],
            [0.0, 1.0, 0.0, 20.0],
            [0.0, 0.0, 1.0, 30.0],
        ];

        let original = TDataMessage::new(vec![TrackingDataElement::new(
            "Tracker1",
            TrackingInstrumentType::Tracker,
            matrix,
        )]);

        let encoded = original.encode_content().unwrap();
        let decoded = TDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.elements.len(), 1);
        assert_eq!(decoded.elements[0].name, "Tracker1");
        assert_eq!(
            decoded.elements[0].instrument_type,
            TrackingInstrumentType::Tracker
        );
        assert_eq!(decoded.elements[0].matrix, matrix);
    }

    #[test]
    fn test_roundtrip_multiple() {
        let original = TDataMessage::new(vec![
            TrackingDataElement::identity("Tool1", TrackingInstrumentType::Instrument6D),
            TrackingDataElement::with_translation(
                "Tool2",
                TrackingInstrumentType::Instrument3D,
                5.0,
                10.0,
                15.0,
            ),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = TDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.elements.len(), 2);
        assert_eq!(decoded.elements[0].name, "Tool1");
        assert_eq!(decoded.elements[1].name, "Tool2");
    }

    #[test]
    fn test_name_truncation() {
        let long_name = "ThisIsAVeryLongNameThatExceedsTwentyCharacters";
        let elem = TrackingDataElement::identity(long_name, TrackingInstrumentType::Tracker);
        let msg = TDataMessage::new(vec![elem]);

        let encoded = msg.encode_content().unwrap();
        let decoded = TDataMessage::decode_content(&encoded).unwrap();

        assert!(decoded.elements[0].name.len() <= 19);
    }

    #[test]
    fn test_empty_message() {
        let msg = TDataMessage::empty();
        let encoded = msg.encode_content().unwrap();
        let decoded = TDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.elements.len(), 0);
        assert_eq!(encoded.len(), 0);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 69]; // One byte short
        let result = TDataMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
