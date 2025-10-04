//! TRAJECTORY message type implementation
//!
//! The TRAJECTORY message type is used to transfer information about 3D trajectories,
//! often used for surgical planning and guidance.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// Trajectory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TrajectoryType {
    /// Trajectory with only entry point
    EntryOnly = 1,
    /// Trajectory with only target point
    TargetOnly = 2,
    /// Trajectory with both entry and target points
    EntryAndTarget = 3,
}

impl TrajectoryType {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(TrajectoryType::EntryOnly),
            2 => Ok(TrajectoryType::TargetOnly),
            3 => Ok(TrajectoryType::EntryAndTarget),
            _ => Err(IgtlError::InvalidHeader(format!(
                "Invalid trajectory type: {}",
                value
            ))),
        }
    }
}

/// Trajectory element with entry/target points
#[derive(Debug, Clone, PartialEq)]
pub struct TrajectoryElement {
    /// Name or description (max 64 chars)
    pub name: String,
    /// Group name (e.g., "Trajectory") (max 32 chars)
    pub group_name: String,
    /// Type of trajectory
    pub trajectory_type: TrajectoryType,
    /// Color in RGBA (0-255)
    pub rgba: [u8; 4],
    /// Entry point (X1, Y1, Z1) in millimeters
    pub entry_point: [f32; 3],
    /// Target point (X2, Y2, Z2) in millimeters
    pub target_point: [f32; 3],
    /// Diameter in millimeters (can be 0)
    pub diameter: f32,
    /// Owner image/sliceset ID (max 20 chars)
    pub owner_image: String,
}

impl TrajectoryElement {
    /// Create a trajectory with entry and target points
    pub fn new(
        name: impl Into<String>,
        group_name: impl Into<String>,
        entry_point: [f32; 3],
        target_point: [f32; 3],
    ) -> Self {
        TrajectoryElement {
            name: name.into(),
            group_name: group_name.into(),
            trajectory_type: TrajectoryType::EntryAndTarget,
            rgba: [255, 255, 255, 255],
            entry_point,
            target_point,
            diameter: 0.0,
            owner_image: String::new(),
        }
    }

    /// Create trajectory with only entry point
    pub fn entry_only(
        name: impl Into<String>,
        group_name: impl Into<String>,
        entry_point: [f32; 3],
    ) -> Self {
        TrajectoryElement {
            name: name.into(),
            group_name: group_name.into(),
            trajectory_type: TrajectoryType::EntryOnly,
            rgba: [255, 255, 255, 255],
            entry_point,
            target_point: [0.0, 0.0, 0.0],
            diameter: 0.0,
            owner_image: String::new(),
        }
    }

    /// Create trajectory with only target point
    pub fn target_only(
        name: impl Into<String>,
        group_name: impl Into<String>,
        target_point: [f32; 3],
    ) -> Self {
        TrajectoryElement {
            name: name.into(),
            group_name: group_name.into(),
            trajectory_type: TrajectoryType::TargetOnly,
            rgba: [255, 255, 255, 255],
            entry_point: [0.0, 0.0, 0.0],
            target_point,
            diameter: 0.0,
            owner_image: String::new(),
        }
    }

    /// Set color
    pub fn with_color(mut self, rgba: [u8; 4]) -> Self {
        self.rgba = rgba;
        self
    }

    /// Set diameter
    pub fn with_diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    /// Set owner image
    pub fn with_owner(mut self, owner_image: impl Into<String>) -> Self {
        self.owner_image = owner_image.into();
        self
    }
}

/// TRAJECTORY message containing multiple trajectory elements
///
/// # OpenIGTLink Specification
/// - Message type: "TRAJ"
/// - Each element: NAME (char[64]) + GROUP_NAME (char[32]) + TYPE (uint8) + Reserved (uint8) + RGBA (uint8[4]) + Entry (float32[3]) + Target (float32[3]) + DIAMETER (float32) + OWNER_IMAGE (char[20])
/// - Element size: 64 + 32 + 1 + 1 + 4 + 12 + 12 + 4 + 20 = 150 bytes
#[derive(Debug, Clone, PartialEq)]
pub struct TrajectoryMessage {
    /// List of trajectory elements
    pub trajectories: Vec<TrajectoryElement>,
}

impl TrajectoryMessage {
    /// Create a new TRAJECTORY message
    pub fn new(trajectories: Vec<TrajectoryElement>) -> Self {
        TrajectoryMessage { trajectories }
    }

    /// Create an empty TRAJECTORY message
    pub fn empty() -> Self {
        TrajectoryMessage {
            trajectories: Vec::new(),
        }
    }

    /// Add a trajectory element
    pub fn add_trajectory(&mut self, trajectory: TrajectoryElement) {
        self.trajectories.push(trajectory);
    }

    /// Get number of trajectories
    pub fn len(&self) -> usize {
        self.trajectories.len()
    }

    /// Check if message has no trajectories
    pub fn is_empty(&self) -> bool {
        self.trajectories.is_empty()
    }
}

impl Message for TrajectoryMessage {
    fn message_type() -> &'static str {
        "TRAJ"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.trajectories.len() * 150);

        for traj in &self.trajectories {
            // Encode NAME (char[64])
            let mut name_bytes = [0u8; 64];
            let name_str = traj.name.as_bytes();
            let copy_len = name_str.len().min(63);
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);

            // Encode GROUP_NAME (char[32])
            let mut group_bytes = [0u8; 32];
            let group_str = traj.group_name.as_bytes();
            let copy_len = group_str.len().min(31);
            group_bytes[..copy_len].copy_from_slice(&group_str[..copy_len]);
            buf.extend_from_slice(&group_bytes);

            // Encode TYPE (uint8)
            buf.put_u8(traj.trajectory_type as u8);

            // Encode Reserved (uint8)
            buf.put_u8(0);

            // Encode RGBA (uint8[4])
            buf.extend_from_slice(&traj.rgba);

            // Encode Entry point (float32[3])
            for &coord in &traj.entry_point {
                buf.put_f32(coord);
            }

            // Encode Target point (float32[3])
            for &coord in &traj.target_point {
                buf.put_f32(coord);
            }

            // Encode DIAMETER (float32)
            buf.put_f32(traj.diameter);

            // Encode OWNER_IMAGE (char[20])
            let mut owner_bytes = [0u8; 20];
            let owner_str = traj.owner_image.as_bytes();
            let copy_len = owner_str.len().min(19);
            owner_bytes[..copy_len].copy_from_slice(&owner_str[..copy_len]);
            buf.extend_from_slice(&owner_bytes);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        let mut trajectories = Vec::new();

        while data.len() >= 150 {
            // Decode NAME (char[64])
            let name_bytes = &data[..64];
            data.advance(64);
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(64);
            let name = String::from_utf8(name_bytes[..name_len].to_vec())?;

            // Decode GROUP_NAME (char[32])
            let group_bytes = &data[..32];
            data.advance(32);
            let group_len = group_bytes.iter().position(|&b| b == 0).unwrap_or(32);
            let group_name = String::from_utf8(group_bytes[..group_len].to_vec())?;

            // Decode TYPE (uint8)
            let trajectory_type = TrajectoryType::from_u8(data.get_u8())?;

            // Decode Reserved (uint8)
            let _reserved = data.get_u8();

            // Decode RGBA (uint8[4])
            let rgba = [data.get_u8(), data.get_u8(), data.get_u8(), data.get_u8()];

            // Decode Entry point (float32[3])
            let entry_point = [data.get_f32(), data.get_f32(), data.get_f32()];

            // Decode Target point (float32[3])
            let target_point = [data.get_f32(), data.get_f32(), data.get_f32()];

            // Decode DIAMETER (float32)
            let diameter = data.get_f32();

            // Decode OWNER_IMAGE (char[20])
            let owner_bytes = &data[..20];
            data.advance(20);
            let owner_len = owner_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let owner_image = String::from_utf8(owner_bytes[..owner_len].to_vec())?;

            trajectories.push(TrajectoryElement {
                name,
                group_name,
                trajectory_type,
                rgba,
                entry_point,
                target_point,
                diameter,
                owner_image,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.len(),
            });
        }

        Ok(TrajectoryMessage { trajectories })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(TrajectoryMessage::message_type(), "TRAJ");
    }

    #[test]
    fn test_trajectory_type() {
        assert_eq!(TrajectoryType::EntryOnly as u8, 1);
        assert_eq!(TrajectoryType::TargetOnly as u8, 2);
        assert_eq!(TrajectoryType::EntryAndTarget as u8, 3);
    }

    #[test]
    fn test_empty() {
        let msg = TrajectoryMessage::empty();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
    }

    #[test]
    fn test_new() {
        let traj = TrajectoryElement::new(
            "Traj1",
            "Trajectory",
            [0.0, 0.0, 0.0],
            [100.0, 100.0, 100.0],
        );
        assert_eq!(traj.trajectory_type, TrajectoryType::EntryAndTarget);
    }

    #[test]
    fn test_entry_only() {
        let traj = TrajectoryElement::entry_only("Entry", "Trajectory", [10.0, 20.0, 30.0]);
        assert_eq!(traj.trajectory_type, TrajectoryType::EntryOnly);
        assert_eq!(traj.entry_point, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_target_only() {
        let traj = TrajectoryElement::target_only("Target", "Trajectory", [50.0, 60.0, 70.0]);
        assert_eq!(traj.trajectory_type, TrajectoryType::TargetOnly);
        assert_eq!(traj.target_point, [50.0, 60.0, 70.0]);
    }

    #[test]
    fn test_with_color() {
        let traj = TrajectoryElement::new("T1", "Trajectory", [0.0; 3], [1.0; 3])
            .with_color([255, 0, 0, 255]);
        assert_eq!(traj.rgba, [255, 0, 0, 255]);
    }

    #[test]
    fn test_with_diameter() {
        let traj = TrajectoryElement::new("T1", "Trajectory", [0.0; 3], [1.0; 3])
            .with_diameter(5.5);
        assert_eq!(traj.diameter, 5.5);
    }

    #[test]
    fn test_add_trajectory() {
        let mut msg = TrajectoryMessage::empty();
        msg.add_trajectory(TrajectoryElement::new("T1", "Trajectory", [0.0; 3], [1.0; 3]));
        assert_eq!(msg.len(), 1);
    }

    #[test]
    fn test_encode_single() {
        let traj = TrajectoryElement::new("Test", "Trajectory", [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let msg = TrajectoryMessage::new(vec![traj]);
        let encoded = msg.encode_content().unwrap();

        assert_eq!(encoded.len(), 150);
    }

    #[test]
    fn test_roundtrip() {
        let original = TrajectoryMessage::new(vec![
            TrajectoryElement::new("Traj1", "Planning", [0.0, 0.0, 0.0], [100.0, 100.0, 100.0])
                .with_color([255, 0, 0, 255])
                .with_diameter(2.5)
                .with_owner("Image1"),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = TrajectoryMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.trajectories.len(), 1);
        assert_eq!(decoded.trajectories[0].name, "Traj1");
        assert_eq!(decoded.trajectories[0].group_name, "Planning");
        assert_eq!(decoded.trajectories[0].rgba, [255, 0, 0, 255]);
        assert_eq!(decoded.trajectories[0].diameter, 2.5);
        assert_eq!(decoded.trajectories[0].owner_image, "Image1");
    }

    #[test]
    fn test_roundtrip_multiple() {
        let original = TrajectoryMessage::new(vec![
            TrajectoryElement::entry_only("Entry1", "Trajectory", [10.0, 20.0, 30.0]),
            TrajectoryElement::target_only("Target1", "Trajectory", [40.0, 50.0, 60.0]),
            TrajectoryElement::new("Both1", "Trajectory", [0.0; 3], [100.0; 3]),
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = TrajectoryMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.trajectories.len(), 3);
        assert_eq!(decoded.trajectories[0].trajectory_type, TrajectoryType::EntryOnly);
        assert_eq!(decoded.trajectories[1].trajectory_type, TrajectoryType::TargetOnly);
        assert_eq!(decoded.trajectories[2].trajectory_type, TrajectoryType::EntryAndTarget);
    }

    #[test]
    fn test_empty_message() {
        let msg = TrajectoryMessage::empty();
        let encoded = msg.encode_content().unwrap();
        let decoded = TrajectoryMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.trajectories.len(), 0);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 149]; // One byte short
        let result = TrajectoryMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
