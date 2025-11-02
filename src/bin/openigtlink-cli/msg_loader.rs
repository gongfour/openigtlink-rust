use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::*;
use openigtlink_rust::protocol::AnyMessage;
use openigtlink_rust::error::{Result, IgtlError};
use serde_json::Value;
use std::fs;
use tracing::info;
use base64::{engine::general_purpose, Engine as _};

/// Load an OpenIGTLink message from a JSON file (supports all message types)
pub fn load_message_from_file(file_path: &str) -> Result<AnyMessage> {
    info!("Loading message from file: {}", file_path);
    let content = fs::read_to_string(file_path)?;
    load_message_from_json(&content)
}

/// Load an OpenIGTLink message from JSON string (supports all message types)
pub fn load_message_from_json(json_str: &str) -> Result<AnyMessage> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| {
        IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("JSON parse error: {}", e),
        ))
    })?;

    let device_name = value["device_name"]
        .as_str()
        .unwrap_or("DefaultDevice");

    let message_type = value["message_type"]
        .as_str()
        .ok_or_else(|| IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Missing 'message_type' field in JSON",
        )))?;

    let content = &value["content"];

    match message_type {
        // Core message types - fully supported
        "TRANSFORM" => {
            let matrix_arr = &value["transform"]["matrix"];
            let mut matrix = [[0.0f32; 4]; 4];

            for i in 0..4 {
                for j in 0..4 {
                    matrix[i][j] = matrix_arr[i][j]
                        .as_f64()
                        .unwrap_or(0.0) as f32;
                }
            }

            let transform = TransformMessage { matrix };
            let msg = IgtlMessage::new(transform, device_name)?;
            info!("✓ Loaded TRANSFORM message");
            Ok(AnyMessage::Transform(msg))
        }

        "STATUS" => {
            let code = content["code"].as_u64().unwrap_or(0) as u16;
            let status_string = content["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string();

            let status = StatusMessage {
                code,
                subcode: 0,
                error_name: String::new(),
                status_string,
            };
            let msg = IgtlMessage::new(status, device_name)?;
            info!("✓ Loaded STATUS message");
            Ok(AnyMessage::Status(msg))
        }

        "CAPABILITY" => {
            let types: Vec<String> = content["types"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            let capability = CapabilityMessage::new(types);
            let msg = IgtlMessage::new(capability, device_name)?;
            info!("✓ Loaded CAPABILITY message");
            Ok(AnyMessage::Capability(msg))
        }

        "STRING" => {
            let encoding = content["encoding"].as_u64().unwrap_or(3) as u16;
            let string_data = content["data"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let string_msg = StringMessage {
                encoding,
                string: string_data,
            };
            let msg = IgtlMessage::new(string_msg, device_name)?;
            info!("✓ Loaded STRING message");
            Ok(AnyMessage::String(msg))
        }

        "POSITION" => {
            let x = content["position"]["x"].as_f64().unwrap_or(0.0) as f32;
            let y = content["position"]["y"].as_f64().unwrap_or(0.0) as f32;
            let z = content["position"]["z"].as_f64().unwrap_or(0.0) as f32;

            let qx = content["quaternion"]["x"].as_f64().unwrap_or(0.0) as f32;
            let qy = content["quaternion"]["y"].as_f64().unwrap_or(0.0) as f32;
            let qz = content["quaternion"]["z"].as_f64().unwrap_or(0.0) as f32;
            let qw = content["quaternion"]["w"].as_f64().unwrap_or(1.0) as f32;

            let position = PositionMessage {
                position: [x, y, z],
                quaternion: [qx, qy, qz, qw],
            };
            let msg = IgtlMessage::new(position, device_name)?;
            info!("✓ Loaded POSITION message");
            Ok(AnyMessage::Position(msg))
        }

        "SENSOR" => {
            let status = content["status"].as_u64().unwrap_or(0) as u8;
            let unit = content["unit"].as_u64().unwrap_or(0);
            let data: Vec<f64> = content["data"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_f64())
                .collect();

            let sensor = SensorMessage {
                status,
                unit,
                data,
            };
            let msg = IgtlMessage::new(sensor, device_name)?;
            info!("✓ Loaded SENSOR message");
            Ok(AnyMessage::Sensor(msg))
        }

        // Complex message types - returning "not implemented" for now
        // These require deeper API knowledge to properly construct
        "IMAGE" | "QTDATA" | "TDATA" | "POINT" | "TRAJECTORY" | "NDARRAY" |
        "BIND" | "COLORTABLE" | "IMGMETA" | "LBMETA" | "POLYDATA" | "VIDEO" |
        "VIDEOMETA" | "COMMAND" => {
            info!("⚠ Message type '{}' loading not yet implemented", message_type);
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Loading '{}' messages from JSON not yet implemented. Supported types: TRANSFORM, STATUS, CAPABILITY, STRING, POSITION, SENSOR", message_type),
            )))
        }

        // Query messages
        "GET_TRANSFORM" | "GET_STATUS" | "GET_CAPABILITY" | "GET_IMAGE" |
        "GET_IMGMETA" | "GET_LBMETA" | "GET_POINT" | "GET_TDATA" => {
            info!("⚠ Query message type '{}' loading not yet implemented", message_type);
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Loading query '{}' messages not yet implemented", message_type),
            )))
        }

        // Response messages
        "RTS_TRANSFORM" | "RTS_STATUS" | "RTS_CAPABILITY" | "RTS_IMAGE" | "RTS_TDATA" => {
            info!("⚠ Response message type '{}' loading not yet implemented", message_type);
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Loading response '{}' messages not yet implemented", message_type),
            )))
        }

        // Streaming control messages
        "STT_TDATA" | "STP_TRANSFORM" | "STP_POSITION" | "STP_QTDATA" |
        "STP_TDATA" | "STP_IMAGE" | "STP_NDARRAY" => {
            info!("⚠ Streaming control message type '{}' loading not yet implemented", message_type);
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Loading streaming control '{}' messages not yet implemented", message_type),
            )))
        }

        _ => {
            Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown message type: {}", message_type),
            )))
        }
    }
}
