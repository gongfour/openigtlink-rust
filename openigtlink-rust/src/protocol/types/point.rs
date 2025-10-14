//! POINT message type implementation
//!
//! The POINT message type is used to transfer information about fiducials,
//! which are often used in surgical planning and navigation.
//!
//! # Use Cases
//!
//! - **Surgical Navigation** - Fiducial markers for patient-to-image registration
//! - **Biopsy Planning** - Target points for needle insertion
//! - **Tumor Localization** - Marking tumor boundaries in pre-operative images
//! - **Anatomical Landmarks** - Identifying critical structures (nerves, vessels)
//! - **Treatment Verification** - Comparing planned vs. actual positions
//!
//! # Point Attributes
//!
//! Each point contains:
//! - **3D Position (x, y, z)** - Coordinates in mm
//! - **Name** - Identifier (e.g., "Fiducial_1", "TumorCenter")
//! - **Group** - Logical grouping (e.g., "Fiducials", "Targets")
//! - **Color (RGBA)** - Visualization color
//! - **Diameter** - Size for rendering (mm)
//! - **Owner** - Associated image/coordinate frame
//!
//! # Examples
//!
//! ## Registering Fiducial Points for Navigation
//!
//! ```no_run
//! use openigtlink_rust::protocol::types::{PointMessage, PointElement};
//! use openigtlink_rust::protocol::message::IgtlMessage;
//! use openigtlink_rust::io::ClientBuilder;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .sync()
//!     .build()?;
//!
//! // Fiducial 1: Nasion (nose bridge)
//! let fid1 = PointElement::with_details(
//!     "Nasion",
//!     "Fiducials",
//!     [255, 0, 0, 255],        // Red
//!     [0.0, 85.0, -30.0],      // x, y, z in mm
//!     5.0,                      // 5mm sphere
//!     "CTImage"
//! );
//!
//! // Fiducial 2: Left ear
//! let fid2 = PointElement::with_details(
//!     "LeftEar",
//!     "Fiducials",
//!     [0, 255, 0, 255],        // Green
//!     [-75.0, 0.0, -20.0],
//!     5.0,
//!     "CTImage"
//! );
//!
//! // Fiducial 3: Right ear
//! let fid3 = PointElement::with_details(
//!     "RightEar",
//!     "Fiducials",
//!     [0, 0, 255, 255],        // Blue
//!     [75.0, 0.0, -20.0],
//!     5.0,
//!     "CTImage"
//! );
//!
//! let point_msg = PointMessage::new(vec![fid1, fid2, fid3]);
//! let msg = IgtlMessage::new(point_msg, "NavigationSystem")?;
//! client.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Receiving Biopsy Target Points
//!
//! ```no_run
//! use openigtlink_rust::io::IgtlServer;
//! use openigtlink_rust::protocol::types::PointMessage;
//!
//! let server = IgtlServer::bind("0.0.0.0:18944")?;
//! let mut client_conn = server.accept()?;
//!
//! let message = client_conn.receive::<PointMessage>()?;
//!
//! println!("Received {} points", message.content.points.len());
//!
//! for (i, point) in message.content.points.iter().enumerate() {
//!     println!("\nPoint {}: {}", i + 1, point.name);
//!     println!("  Group: {}", point.group);
//!     println!("  Position: ({:.2}, {:.2}, {:.2}) mm",
//!              point.position[0], point.position[1], point.position[2]);
//!     println!("  Color: RGB({}, {}, {})",
//!              point.rgba[0], point.rgba[1], point.rgba[2]);
//!     println!("  Diameter: {:.2} mm", point.diameter);
//! }
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```

use crate::error::{IgtlError, Result};
use crate::protocol::message::Message;
use bytes::{Buf, BufMut};

/// Point/fiducial data element
#[derive(Debug, Clone, PartialEq)]
pub struct PointElement {
    /// Name or description of the point (max 64 chars)
    pub name: String,
    /// Group name (e.g., "Labeled Point", "Landmark", "Fiducial") (max 32 chars)
    pub group: String,
    /// Color in RGBA (0-255)
    pub rgba: [u8; 4],
    /// Coordinate of the point in millimeters
    pub position: [f32; 3],
    /// Diameter of the point in millimeters (can be 0)
    pub diameter: f32,
    /// ID of the owner image/sliceset (max 20 chars)
    pub owner: String,
}

impl PointElement {
    /// Create a new point element
    pub fn new(name: impl Into<String>, group: impl Into<String>, position: [f32; 3]) -> Self {
        PointElement {
            name: name.into(),
            group: group.into(),
            rgba: [255, 255, 255, 255], // White, fully opaque
            position,
            diameter: 0.0,
            owner: String::new(),
        }
    }

    /// Create a point with color
    pub fn with_color(
        name: impl Into<String>,
        group: impl Into<String>,
        rgba: [u8; 4],
        position: [f32; 3],
    ) -> Self {
        PointElement {
            name: name.into(),
            group: group.into(),
            rgba,
            position,
            diameter: 0.0,
            owner: String::new(),
        }
    }

    /// Create a point with all fields
    pub fn with_details(
        name: impl Into<String>,
        group: impl Into<String>,
        rgba: [u8; 4],
        position: [f32; 3],
        diameter: f32,
        owner: impl Into<String>,
    ) -> Self {
        PointElement {
            name: name.into(),
            group: group.into(),
            rgba,
            position,
            diameter,
            owner: owner.into(),
        }
    }
}

/// POINT message containing multiple fiducial points
///
/// # OpenIGTLink Specification
/// - Message type: "POINT"
/// - Each element: NAME (`char[64]`) + GROUP (`char[32]`) + RGBA (`uint8[4]`) + XYZ (`float32[3]`) + DIAMETER (float32) + OWNER (`char[20]`)
/// - Element size: 64 + 32 + 4 + 12 + 4 + 20 = 136 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct PointMessage {
    /// List of point elements
    pub points: Vec<PointElement>,
}

impl PointMessage {
    /// Create a new POINT message with points
    pub fn new(points: Vec<PointElement>) -> Self {
        PointMessage { points }
    }

    /// Create an empty POINT message
    pub fn empty() -> Self {
        PointMessage { points: Vec::new() }
    }

    /// Add a point element
    pub fn add_point(&mut self, point: PointElement) {
        self.points.push(point);
    }

    /// Get number of points
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if message has no points
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Message for PointMessage {
    fn message_type() -> &'static str {
        "POINT"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.points.len() * 136);

        for point in &self.points {
            // Encode NAME (`char[64]`)
            let mut name_bytes = [0u8; 64];
            let name_str = point.name.as_bytes();
            let copy_len = name_str.len().min(63);
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);

            // Encode GROUP (`char[32]`)
            let mut group_bytes = [0u8; 32];
            let group_str = point.group.as_bytes();
            let copy_len = group_str.len().min(31);
            group_bytes[..copy_len].copy_from_slice(&group_str[..copy_len]);
            buf.extend_from_slice(&group_bytes);

            // Encode RGBA (`uint8[4]`)
            buf.extend_from_slice(&point.rgba);

            // Encode XYZ (`float32[3]`)
            for &coord in &point.position {
                buf.put_f32(coord);
            }

            // Encode DIAMETER (float32)
            buf.put_f32(point.diameter);

            // Encode OWNER (`char[20]`)
            let mut owner_bytes = [0u8; 20];
            let owner_str = point.owner.as_bytes();
            let copy_len = owner_str.len().min(19);
            owner_bytes[..copy_len].copy_from_slice(&owner_str[..copy_len]);
            buf.extend_from_slice(&owner_bytes);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        let mut points = Vec::new();

        while data.len() >= 136 {
            // Decode NAME (`char[64]`)
            let name_bytes = &data[..64];
            data.advance(64);
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(64);
            let name = String::from_utf8(name_bytes[..name_len].to_vec())?;

            // Decode GROUP (`char[32]`)
            let group_bytes = &data[..32];
            data.advance(32);
            let group_len = group_bytes.iter().position(|&b| b == 0).unwrap_or(32);
            let group = String::from_utf8(group_bytes[..group_len].to_vec())?;

            // Decode RGBA (`uint8[4]`)
            let rgba = [data.get_u8(), data.get_u8(), data.get_u8(), data.get_u8()];

            // Decode XYZ (`float32[3]`)
            let position = [data.get_f32(), data.get_f32(), data.get_f32()];

            // Decode DIAMETER (float32)
            let diameter = data.get_f32();

            // Decode OWNER (`char[20]`)
            let owner_bytes = &data[..20];
            data.advance(20);
            let owner_len = owner_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let owner = String::from_utf8(owner_bytes[..owner_len].to_vec())?;

            points.push(PointElement {
                name,
                group,
                rgba,
                position,
                diameter,
                owner,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(PointMessage { points })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(PointMessage::message_type(), "POINT");
    }

    #[test]
    fn test_empty() {
        let msg = PointMessage::empty();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn test_new_point() {
        let point = PointElement::new("Fiducial1", "Landmark", [10.0, 20.0, 30.0]);
        assert_eq!(point.name, "Fiducial1");
        assert_eq!(point.group, "Landmark");
        assert_eq!(point.position, [10.0, 20.0, 30.0]);
        assert_eq!(point.rgba, [255, 255, 255, 255]);
    }

    #[test]
    fn test_point_with_color() {
        let point =
            PointElement::with_color("Point1", "Fiducial", [255, 0, 0, 255], [1.0, 2.0, 3.0]);
        assert_eq!(point.rgba, [255, 0, 0, 255]);
    }

    #[test]
    fn test_add_point() {
        let mut msg = PointMessage::empty();
        msg.add_point(PointElement::new("P1", "Landmark", [0.0, 0.0, 0.0]));
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_encode_single_point() {
        let point = PointElement::new("Test", "Fiducial", [1.0, 2.0, 3.0]);
        let msg = PointMessage::new(vec![point]);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 136);
    }

    #[test]
    fn test_roundtrip_single() {
        let original = PointMessage::new(vec![PointElement::with_details(
            "Fiducial1",
            "Landmark",
            [255, 128, 64, 255],
            [100.5, 200.5, 300.5],
            5.0,
            "Image1",
        )]);

        let encoded = original.encode_content().unwrap();
        let decoded = PointMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.points.len(), 1);
        assert_eq!(decoded.points[0].name, "Fiducial1");
        assert_eq!(decoded.points[0].group, "Landmark");
        assert_eq!(decoded.points[0].rgba, [255, 128, 64, 255]);
        assert_eq!(decoded.points[0].position, [100.5, 200.5, 300.5]);
        assert_eq!(decoded.points[0].diameter, 5.0);
        assert_eq!(decoded.points[0].owner, "Image1");
    }

    #[test]
    fn test_roundtrip_multiple() {
        let original = PointMessage::new(vec![
            PointElement::new("P1", "Landmark", [1.0, 2.0, 3.0]),
            PointElement::new("P2", "Fiducial", [4.0, 5.0, 6.0]),
            PointElement::new("P3", "Target", [7.0, 8.0, 9.0]),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = PointMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.points.len(), 3);
        assert_eq!(decoded.points[0].name, "P1");
        assert_eq!(decoded.points[1].name, "P2");
        assert_eq!(decoded.points[2].name, "P3");
    }

    #[test]
    fn test_name_truncation() {
        let long_name = "A".repeat(100);
        let point = PointElement::new(&long_name, "Group", [0.0, 0.0, 0.0]);
        let msg = PointMessage::new(vec![point]);

        let encoded = msg.encode_content().unwrap();
        let decoded = PointMessage::decode_content(&encoded).unwrap();

        assert!(decoded.points[0].name.len() <= 63);
    }

    #[test]
    fn test_empty_message() {
        let msg = PointMessage::empty();
        let encoded = msg.encode_content().unwrap();
        let decoded = PointMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.points.len(), 0);
        assert_eq!(encoded.len(), 0);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 135]; // One byte short
        let result = PointMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_color_values() {
        let point =
            PointElement::with_color("ColorTest", "Test", [128, 64, 32, 200], [0.0, 0.0, 0.0]);
        let msg = PointMessage::new(vec![point]);

        let encoded = msg.encode_content().unwrap();
        let decoded = PointMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.points[0].rgba, [128, 64, 32, 200]);
    }
}
