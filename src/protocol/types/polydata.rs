//! POLYDATA message type implementation
//!
//! The POLYDATA message is used to transfer 3D polygon/mesh data for surgical navigation,
//! visualization of anatomical structures, or surgical planning.

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// Attribute type for polygon data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Point = 0,
    Cell = 1,
}

impl AttributeType {
    /// Create from type value
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(AttributeType::Point),
            1 => Ok(AttributeType::Cell),
            _ => Err(IgtlError::InvalidSize {
                expected: 0,
                actual: value as usize,
            }),
        }
    }
}

/// Attribute data for points or cells
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    /// Attribute type (point or cell)
    pub attr_type: AttributeType,
    /// Number of components per attribute
    pub num_components: u8,
    /// Attribute name (max 64 chars)
    pub name: String,
    /// Attribute data (length = n_points/cells * num_components)
    pub data: Vec<f32>,
}

impl Attribute {
    /// Create a new attribute
    pub fn new(
        attr_type: AttributeType,
        num_components: u8,
        name: impl Into<String>,
        data: Vec<f32>,
    ) -> Self {
        Attribute {
            attr_type,
            num_components,
            name: name.into(),
            data,
        }
    }
}

/// POLYDATA message for 3D polygon/mesh data
///
/// # OpenIGTLink Specification
/// - Message type: "POLYDATA"
/// - Format: Points + Vertices + Lines + Polygons + Triangle Strips + Attributes
/// - Complex variable-length structure
#[derive(Debug, Clone, PartialEq)]
pub struct PolyDataMessage {
    /// 3D points (x, y, z)
    pub points: Vec<[f32; 3]>,
    /// Vertex indices
    pub vertices: Vec<u32>,
    /// Line connectivity (first element = count, followed by indices)
    pub lines: Vec<u32>,
    /// Polygon connectivity (first element = count, followed by indices)
    pub polygons: Vec<u32>,
    /// Triangle strip connectivity
    pub triangle_strips: Vec<u32>,
    /// Attribute data
    pub attributes: Vec<Attribute>,
}

impl PolyDataMessage {
    /// Create a new POLYDATA message
    pub fn new(points: Vec<[f32; 3]>) -> Self {
        PolyDataMessage {
            points,
            vertices: Vec::new(),
            lines: Vec::new(),
            polygons: Vec::new(),
            triangle_strips: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Add vertices
    pub fn with_vertices(mut self, vertices: Vec<u32>) -> Self {
        self.vertices = vertices;
        self
    }

    /// Add lines
    pub fn with_lines(mut self, lines: Vec<u32>) -> Self {
        self.lines = lines;
        self
    }

    /// Add polygons
    pub fn with_polygons(mut self, polygons: Vec<u32>) -> Self {
        self.polygons = polygons;
        self
    }

    /// Add triangle strips
    pub fn with_triangle_strips(mut self, strips: Vec<u32>) -> Self {
        self.triangle_strips = strips;
        self
    }

    /// Add attribute
    pub fn add_attribute(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }

    /// Get number of points
    pub fn num_points(&self) -> usize {
        self.points.len()
    }
}

impl Message for PolyDataMessage {
    fn message_type() -> &'static str {
        "POLYDATA"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Encode number of points (uint32)
        buf.put_u32(self.points.len() as u32);

        // Encode points
        for point in &self.points {
            for &coord in point {
                buf.put_f32(coord);
            }
        }

        // Encode vertices (uint32 count + data)
        buf.put_u32(self.vertices.len() as u32);
        for &v in &self.vertices {
            buf.put_u32(v);
        }

        // Encode lines
        buf.put_u32(self.lines.len() as u32);
        for &l in &self.lines {
            buf.put_u32(l);
        }

        // Encode polygons
        buf.put_u32(self.polygons.len() as u32);
        for &p in &self.polygons {
            buf.put_u32(p);
        }

        // Encode triangle strips
        buf.put_u32(self.triangle_strips.len() as u32);
        for &t in &self.triangle_strips {
            buf.put_u32(t);
        }

        // Encode number of attributes
        buf.put_u32(self.attributes.len() as u32);

        // Encode each attribute
        for attr in &self.attributes {
            // Attribute type (uint8)
            buf.put_u8(attr.attr_type as u8);

            // Number of components (uint8)
            buf.put_u8(attr.num_components);

            // Name (char[64])
            let mut name_bytes = [0u8; 64];
            let name_str = attr.name.as_bytes();
            let copy_len = name_str.len().min(63);
            name_bytes[..copy_len].copy_from_slice(&name_str[..copy_len]);
            buf.extend_from_slice(&name_bytes);

            // Data size (uint32)
            buf.put_u32(attr.data.len() as u32);

            // Data (float32[])
            for &val in &attr.data {
                buf.put_f32(val);
            }
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(IgtlError::InvalidSize {
                expected: 4,
                actual: data.len(),
            });
        }

        // Decode number of points
        let num_points = data.get_u32() as usize;

        // Decode points
        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            if data.remaining() < 12 {
                return Err(IgtlError::InvalidSize {
                    expected: 12,
                    actual: data.remaining(),
                });
            }
            points.push([data.get_f32(), data.get_f32(), data.get_f32()]);
        }

        // Decode vertices
        let num_vertices = data.get_u32() as usize;
        let mut vertices = Vec::with_capacity(num_vertices);
        for _ in 0..num_vertices {
            vertices.push(data.get_u32());
        }

        // Decode lines
        let num_lines = data.get_u32() as usize;
        let mut lines = Vec::with_capacity(num_lines);
        for _ in 0..num_lines {
            lines.push(data.get_u32());
        }

        // Decode polygons
        let num_polygons = data.get_u32() as usize;
        let mut polygons = Vec::with_capacity(num_polygons);
        for _ in 0..num_polygons {
            polygons.push(data.get_u32());
        }

        // Decode triangle strips
        let num_strips = data.get_u32() as usize;
        let mut triangle_strips = Vec::with_capacity(num_strips);
        for _ in 0..num_strips {
            triangle_strips.push(data.get_u32());
        }

        // Decode attributes
        let num_attributes = data.get_u32() as usize;
        let mut attributes = Vec::with_capacity(num_attributes);

        for _ in 0..num_attributes {
            // Attribute type
            let attr_type = AttributeType::from_u8(data.get_u8())?;

            // Number of components
            let num_components = data.get_u8();

            // Name (char[64])
            let name_bytes = &data[..64];
            data.advance(64);
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(64);
            let name = String::from_utf8(name_bytes[..name_len].to_vec())?;

            // Data size
            let data_size = data.get_u32() as usize;

            // Data
            let mut attr_data = Vec::with_capacity(data_size);
            for _ in 0..data_size {
                attr_data.push(data.get_f32());
            }

            attributes.push(Attribute {
                attr_type,
                num_components,
                name,
                data: attr_data,
            });
        }

        if !data.is_empty() {
            return Err(IgtlError::InvalidSize {
                expected: 0,
                actual: data.remaining(),
            });
        }

        Ok(PolyDataMessage {
            points,
            vertices,
            lines,
            polygons,
            triangle_strips,
            attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(PolyDataMessage::message_type(), "POLYDATA");
    }

    #[test]
    fn test_new() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let poly = PolyDataMessage::new(points.clone());
        assert_eq!(poly.num_points(), 3);
        assert_eq!(poly.points, points);
    }

    #[test]
    fn test_with_polygons() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let poly = PolyDataMessage::new(points)
            .with_polygons(vec![3, 0, 1, 2]); // Triangle with 3 vertices

        assert_eq!(poly.polygons, vec![3, 0, 1, 2]);
    }

    #[test]
    fn test_add_attribute() {
        let points = vec![[0.0, 0.0, 0.0]];
        let mut poly = PolyDataMessage::new(points);

        let attr = Attribute::new(
            AttributeType::Point,
            3,
            "Normals",
            vec![0.0, 0.0, 1.0],
        );
        poly.add_attribute(attr);

        assert_eq!(poly.attributes.len(), 1);
    }

    #[test]
    fn test_encode_simple() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let poly = PolyDataMessage::new(points);
        let encoded = poly.encode_content().unwrap();

        // Should contain: point count (4) + points (24) + 5 counts (20) = 48 bytes minimum
        assert!(encoded.len() >= 48);
    }

    #[test]
    fn test_roundtrip_points_only() {
        let original = PolyDataMessage::new(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]);

        let encoded = original.encode_content().unwrap();
        let decoded = PolyDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.num_points(), 4);
        assert_eq!(decoded.points, original.points);
    }

    #[test]
    fn test_roundtrip_with_polygons() {
        let original = PolyDataMessage::new(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ])
        .with_polygons(vec![3, 0, 1, 2]);

        let encoded = original.encode_content().unwrap();
        let decoded = PolyDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.polygons, vec![3, 0, 1, 2]);
    }

    #[test]
    fn test_roundtrip_with_attribute() {
        let mut original = PolyDataMessage::new(vec![[0.0, 0.0, 0.0]]);

        original.add_attribute(Attribute::new(
            AttributeType::Point,
            3,
            "Normals",
            vec![0.0, 0.0, 1.0],
        ));

        let encoded = original.encode_content().unwrap();
        let decoded = PolyDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.attributes.len(), 1);
        assert_eq!(decoded.attributes[0].name, "Normals");
        assert_eq!(decoded.attributes[0].num_components, 3);
        assert_eq!(decoded.attributes[0].data, vec![0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_roundtrip_complex() {
        let mut original = PolyDataMessage::new(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ])
        .with_polygons(vec![4, 0, 1, 2, 3])
        .with_lines(vec![2, 0, 1]);

        original.add_attribute(Attribute::new(
            AttributeType::Cell,
            1,
            "Quality",
            vec![0.95],
        ));

        let encoded = original.encode_content().unwrap();
        let decoded = PolyDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.num_points(), 4);
        assert_eq!(decoded.polygons, vec![4, 0, 1, 2, 3]);
        assert_eq!(decoded.lines, vec![2, 0, 1]);
        assert_eq!(decoded.attributes.len(), 1);
    }

    #[test]
    fn test_decode_invalid() {
        let data = vec![0u8; 2]; // Too short
        let result = PolyDataMessage::decode_content(&data);
        assert!(result.is_err());
    }
}
