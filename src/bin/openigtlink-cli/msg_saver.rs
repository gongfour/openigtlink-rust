use openigtlink_rust::protocol::AnyMessage;
use openigtlink_rust::error::Result;
use serde_json::{json, Value};
use std::fs;
use tracing::info;

/// Save received OpenIGTLink messages to a JSON file (supports all message types)
pub fn save_message(
    message: &AnyMessage,
    file_path: &str,
) -> Result<()> {
    info!("Saving message to file: {}", file_path);

    let json_data = match message {
        AnyMessage::Transform(msg) => {
            json!({
                "message_type": "TRANSFORM",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "timestamp": {
                    "seconds": msg.header.timestamp.seconds,
                    "fraction": msg.header.timestamp.fraction
                },
                "transform": {
                    "matrix": msg.content.matrix
                }
            })
        }
        AnyMessage::Status(msg) => {
            json!({
                "message_type": "STATUS",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "timestamp": {
                    "seconds": msg.header.timestamp.seconds,
                    "fraction": msg.header.timestamp.fraction
                },
                "content": {
                    "code": msg.content.code,
                    "message": msg.content.status_string
                }
            })
        }
        AnyMessage::Capability(msg) => {
            json!({
                "message_type": "CAPABILITY",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "timestamp": {
                    "seconds": msg.header.timestamp.seconds,
                    "fraction": msg.header.timestamp.fraction
                },
                "content": {
                    "types": msg.content.types
                }
            })
        }
        AnyMessage::String(msg) => {
            json!({
                "message_type": "STRING",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "timestamp": {
                    "seconds": msg.header.timestamp.seconds,
                    "fraction": msg.header.timestamp.fraction
                },
                "content": {
                    "encoding": msg.content.encoding,
                    "data": msg.content.string
                }
            })
        }
        AnyMessage::Position(msg) => {
            json!({
                "message_type": "POSITION",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "timestamp": {
                    "seconds": msg.header.timestamp.seconds,
                    "fraction": msg.header.timestamp.fraction
                },
                "content": {
                    "position": {
                        "x": msg.content.position[0],
                        "y": msg.content.position[1],
                        "z": msg.content.position[2]
                    },
                    "quaternion": {
                        "x": msg.content.quaternion[0],
                        "y": msg.content.quaternion[1],
                        "z": msg.content.quaternion[2],
                        "w": msg.content.quaternion[3]
                    }
                }
            })
        }
        AnyMessage::Sensor(msg) => {
            json!({
                "message_type": "SENSOR",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "timestamp": {
                    "seconds": msg.header.timestamp.seconds,
                    "fraction": msg.header.timestamp.fraction
                },
                "content": {
                    "status": msg.content.status,
                    "unit": msg.content.unit,
                    "data": msg.content.data
                }
            })
        }
        _ => {
            json!({
                "message_type": message.message_type(),
                "device_name": "Unknown",
                "note": "Complex message type - convert to string representation"
            })
        }
    };

    let json_str = serde_json::to_string_pretty(&json_data).map_err(|e| {
        openigtlink_rust::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("JSON serialization error: {}", e),
        ))
    })?;

    fs::write(file_path, json_str)?;
    info!("✓ Saved message to: {}", file_path);

    Ok(())
}

/// Save multiple received messages to a JSON file
pub fn save_messages(
    messages: &[AnyMessage],
    file_path: &str,
) -> Result<()> {
    info!("Saving {} message(s) to file: {}", messages.len(), file_path);

    let json_messages: Vec<Value> = messages
        .iter()
        .filter_map(|msg| {
            match msg {
                AnyMessage::Transform(m) => Some(json!({
                    "message_type": "TRANSFORM",
                    "device_name": m.header.device_name.as_str().unwrap_or("Unknown"),
                    "transform": { "matrix": m.content.matrix }
                })),
                AnyMessage::Status(m) => Some(json!({
                    "message_type": "STATUS",
                    "device_name": m.header.device_name.as_str().unwrap_or("Unknown"),
                    "content": { "code": m.content.code, "message": m.content.status_string }
                })),
                AnyMessage::Capability(m) => Some(json!({
                    "message_type": "CAPABILITY",
                    "device_name": m.header.device_name.as_str().unwrap_or("Unknown"),
                    "content": { "types": &m.content.types }
                })),
                AnyMessage::String(m) => Some(json!({
                    "message_type": "STRING",
                    "device_name": m.header.device_name.as_str().unwrap_or("Unknown"),
                    "content": {
                        "encoding": m.content.encoding,
                        "data": &m.content.string
                    }
                })),
                AnyMessage::Position(m) => Some(json!({
                    "message_type": "POSITION",
                    "device_name": m.header.device_name.as_str().unwrap_or("Unknown"),
                    "content": {
                        "position": {
                            "x": m.content.position[0],
                            "y": m.content.position[1],
                            "z": m.content.position[2]
                        },
                        "quaternion": {
                            "x": m.content.quaternion[0],
                            "y": m.content.quaternion[1],
                            "z": m.content.quaternion[2],
                            "w": m.content.quaternion[3]
                        }
                    }
                })),
                AnyMessage::Sensor(m) => Some(json!({
                    "message_type": "SENSOR",
                    "device_name": m.header.device_name.as_str().unwrap_or("Unknown"),
                    "content": {
                        "status": m.content.status,
                        "unit": m.content.unit,
                        "data": &m.content.data
                    }
                })),
                _ => None,
            }
        })
        .collect();

    let output = if json_messages.len() == 1 {
        json_messages[0].clone()
    } else {
        Value::Array(json_messages)
    };

    let json_str = serde_json::to_string_pretty(&output).map_err(|e| {
        openigtlink_rust::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("JSON serialization error: {}", e),
        ))
    })?;

    fs::write(file_path, json_str)?;
    info!("✓ Saved {} message(s) to: {}", messages.len(), file_path);

    Ok(())
}

/// Check if a message type string matches a filter
/// Returns true if the message type should be included
pub fn matches_filter(message_type: &str, filter_str: &str) -> bool {
    if filter_str == "all" {
        return true;
    }

    filter_str
        .split(',')
        .map(|s| s.trim())
        .any(|t| t.to_uppercase() == message_type.to_uppercase())
}
