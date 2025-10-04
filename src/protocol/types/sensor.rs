//! SENSOR message type implementation
//!
//! The SENSOR message type is used to transfer sensor readings including
//! position, velocity, acceleration, angle, and other sensor data.
//!
//! # Use Cases
//!
//! - **Force/Torque Sensors** - Haptic feedback during robotic surgery (6-axis F/T sensors)
//! - **IMU Sensors** - Inertial measurement units for tool orientation (accelerometer + gyroscope)
//! - **Pressure Sensors** - Blood pressure monitoring during procedures
//! - **Temperature Arrays** - Multi-point temperature monitoring in ablation procedures
//! - **Strain Gauges** - Measuring deformation in surgical instruments
//!
//! # Sensor Data Format
//!
//! The SENSOR message supports:
//! - **Multi-channel**: Up to 255 sensor channels per message
//! - **64-bit floats**: High-precision sensor readings
//! - **Unit encoding**: Standard SI unit codes (e.g., Newton, Meter/Second)
//! - **Status field**: Sensor health/validity indicator
//!
//! # Examples
//!
//! ## Sending 6-Axis Force/Torque Data
//!
//! ```no_run
//! use openigtlink_rust::protocol::types::SensorMessage;
//! use openigtlink_rust::io::IgtlClient;
//!
//! let mut client = IgtlClient::connect("127.0.0.1:18944")?;
//!
//! let mut sensor = SensorMessage::new();
//! sensor.set_device_name("ATI_ForceSensor");
//!
//! // 6-axis: Fx, Fy, Fz (forces) + Tx, Ty, Tz (torques)
//! let readings = vec![
//!     2.5,   // Fx (N)
//!     -1.2,  // Fy (N)
//!     5.8,   // Fz (N)
//!     0.15,  // Tx (Nm)
//!     -0.08, // Ty (Nm)
//!     0.22,  // Tz (Nm)
//! ];
//! sensor.set_data(readings);
//! sensor.set_unit(0x0101); // Newton + Newton-meter
//! sensor.set_status(1); // Sensor valid
//!
//! client.send(&sensor)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Receiving IMU Sensor Data
//!
//! ```no_run
//! use openigtlink_rust::io::IgtlServer;
//! use openigtlink_rust::protocol::types::SensorMessage;
//! use openigtlink_rust::protocol::message::Message;
//!
//! let server = IgtlServer::bind("0.0.0.0:18944")?;
//! let mut client_conn = server.accept()?;
//!
//! let message = client_conn.receive()?;
//!
//! if message.header.message_type == "SENSOR" {
//!     let sensor = SensorMessage::from_bytes(&message.body)?;
//!     println!("Sensor: {}", message.header.device_name);
//!     println!("Channels: {}", sensor.data.len());
//!
//!     // Typical IMU: 3 accel + 3 gyro = 6 channels
//!     if sensor.data.len() == 6 {
//!         println!("Acceleration: [{:.2}, {:.2}, {:.2}] m/s²",
//!                  sensor.data[0], sensor.data[1], sensor.data[2]);
//!         println!("Gyroscope: [{:.2}, {:.2}, {:.2}] rad/s",
//!                  sensor.data[3], sensor.data[4], sensor.data[5]);
//!     }
//! }
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```

use crate::protocol::message::Message;
use crate::error::{IgtlError, Result};
use bytes::{Buf, BufMut};

/// SENSOR message containing sensor data array
///
/// # OpenIGTLink Specification
/// - Message type: "SENSOR"
/// - Body format: LARRAY (uint8) + STATUS (uint8) + UNIT (uint64) + DATA (float64[LARRAY])
/// - Max array length: 255
#[derive(Debug, Clone, PartialEq)]
pub struct SensorMessage {
    /// Sensor status (reserved for future use)
    pub status: u8,

    /// Unit specification (64-bit field)
    /// See OpenIGTLink unit specification
    pub unit: u64,

    /// Sensor data array
    pub data: Vec<f64>,
}

impl SensorMessage {
    /// Create a new SENSOR message with data
    pub fn new(data: Vec<f64>) -> Result<Self> {
        if data.len() > 255 {
            return Err(IgtlError::BodyTooLarge {
                size: data.len(),
                max: 255,
            });
        }

        Ok(SensorMessage {
            status: 0,
            unit: 0,
            data,
        })
    }

    /// Create a SENSOR message with status and unit
    pub fn with_unit(status: u8, unit: u64, data: Vec<f64>) -> Result<Self> {
        if data.len() > 255 {
            return Err(IgtlError::BodyTooLarge {
                size: data.len(),
                max: 255,
            });
        }

        Ok(SensorMessage { status, unit, data })
    }

    /// Get the array length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data array is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Message for SensorMessage {
    fn message_type() -> &'static str {
        "SENSOR"
    }

    fn encode_content(&self) -> Result<Vec<u8>> {
        let larray = self.data.len();
        if larray > 255 {
            return Err(IgtlError::BodyTooLarge {
                size: larray,
                max: 255,
            });
        }

        let mut buf = Vec::with_capacity(10 + larray * 8);

        // Encode LARRAY (uint8)
        buf.put_u8(larray as u8);

        // Encode STATUS (uint8)
        buf.put_u8(self.status);

        // Encode UNIT (uint64)
        buf.put_u64(self.unit);

        // Encode DATA (float64[LARRAY])
        for &value in &self.data {
            buf.put_f64(value);
        }

        Ok(buf)
    }

    fn decode_content(mut data: &[u8]) -> Result<Self> {
        if data.len() < 10 {
            return Err(IgtlError::InvalidSize {
                expected: 10,
                actual: data.len(),
            });
        }

        // Decode LARRAY
        let larray = data.get_u8() as usize;

        // Decode STATUS
        let status = data.get_u8();

        // Decode UNIT
        let unit = data.get_u64();

        // Check remaining data size
        let expected_size = larray * 8;
        if data.len() < expected_size {
            return Err(IgtlError::InvalidSize {
                expected: expected_size,
                actual: data.len(),
            });
        }

        // Decode DATA
        let mut sensor_data = Vec::with_capacity(larray);
        for _ in 0..larray {
            sensor_data.push(data.get_f64());
        }

        Ok(SensorMessage {
            status,
            unit,
            data: sensor_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        assert_eq!(SensorMessage::message_type(), "SENSOR");
    }

    #[test]
    fn test_new() {
        let msg = SensorMessage::new(vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(msg.status, 0);
        assert_eq!(msg.unit, 0);
        assert_eq!(msg.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_with_unit() {
        let msg = SensorMessage::with_unit(1, 0x12345678, vec![1.0, 2.0]).unwrap();
        assert_eq!(msg.status, 1);
        assert_eq!(msg.unit, 0x12345678);
        assert_eq!(msg.data, vec![1.0, 2.0]);
    }

    #[test]
    fn test_len() {
        let msg = SensorMessage::new(vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(msg.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        let msg1 = SensorMessage::new(vec![]).unwrap();
        assert!(msg1.is_empty());

        let msg2 = SensorMessage::new(vec![1.0]).unwrap();
        assert!(!msg2.is_empty());
    }

    #[test]
    fn test_too_large() {
        let data = vec![0.0; 256];
        let result = SensorMessage::new(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_simple() {
        let msg = SensorMessage::new(vec![1.0, 2.0]).unwrap();
        let encoded = msg.encode_content().unwrap();

        // LARRAY (1 byte) + STATUS (1 byte) + UNIT (8 bytes) + DATA (2 * 8 bytes)
        assert_eq!(encoded.len(), 1 + 1 + 8 + 16);
        assert_eq!(encoded[0], 2); // LARRAY = 2
        assert_eq!(encoded[1], 0); // STATUS = 0
    }

    #[test]
    fn test_roundtrip() {
        let original = SensorMessage::with_unit(1, 0xABCDEF, vec![1.5, 2.5, 3.5]).unwrap();
        let encoded = original.encode_content().unwrap();
        let decoded = SensorMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.status, original.status);
        assert_eq!(decoded.unit, original.unit);
        assert_eq!(decoded.data, original.data);
    }

    #[test]
    fn test_empty_array() {
        let msg = SensorMessage::new(vec![]).unwrap();
        let encoded = msg.encode_content().unwrap();
        let decoded = SensorMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.data.len(), 0);
    }

    #[test]
    fn test_max_array() {
        let data = vec![1.0; 255];
        let msg = SensorMessage::new(data.clone()).unwrap();
        let encoded = msg.encode_content().unwrap();
        let decoded = SensorMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.data, data);
    }

    #[test]
    fn test_decode_invalid_size() {
        let data = vec![0u8; 5]; // Too short
        let result = SensorMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_truncated() {
        let mut data = vec![3, 0]; // LARRAY=3, STATUS=0
        data.extend_from_slice(&0u64.to_be_bytes()); // UNIT=0
        data.extend_from_slice(&1.0f64.to_be_bytes()); // Only 1 value instead of 3

        let result = SensorMessage::decode_content(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_big_endian_encoding() {
        let msg = SensorMessage::new(vec![1.0]).unwrap();
        let encoded = msg.encode_content().unwrap();

        // LARRAY = 1
        assert_eq!(encoded[0], 1);
        // STATUS = 0
        assert_eq!(encoded[1], 0);
        // UNIT = 0 (8 bytes, all zeros)
        assert_eq!(&encoded[2..10], &[0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
