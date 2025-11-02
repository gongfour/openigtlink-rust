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
        // Complex message types - save with minimal info
        AnyMessage::Image(msg) => {
            json!({
                "message_type": "IMAGE",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "timestamp": {
                    "seconds": msg.header.timestamp.seconds,
                    "fraction": msg.header.timestamp.fraction
                },
                "note": "IMAGE message received but full details not saved"
            })
        }
        AnyMessage::QtData(msg) => {
            json!({
                "message_type": "QTDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("QTDATA message with {} tracking elements", msg.content.elements.len())
            })
        }
        AnyMessage::TData(msg) => {
            json!({
                "message_type": "TDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("TDATA message with {} tracking data elements", msg.content.elements.len())
            })
        }
        AnyMessage::Point(msg) => {
            json!({
                "message_type": "POINT",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("POINT message with {} points", msg.content.points.len())
            })
        }
        AnyMessage::Trajectory(msg) => {
            json!({
                "message_type": "TRAJECTORY",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("TRAJECTORY message with {} trajectories", msg.content.trajectories.len())
            })
        }
        AnyMessage::NdArray(msg) => {
            json!({
                "message_type": "NDARRAY",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("NDARRAY message with {} dimensions", msg.content.size.len())
            })
        }
        AnyMessage::Bind(msg) => {
            json!({
                "message_type": "BIND",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("BIND message with {} entries", msg.content.entries.len())
            })
        }
        AnyMessage::ColorTable(msg) => {
            json!({
                "message_type": "COLORTABLE",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("COLORTABLE message with {} colors", msg.content.colors.len())
            })
        }
        AnyMessage::ImgMeta(msg) => {
            json!({
                "message_type": "IMGMETA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("IMGMETA message with {} images", msg.content.images.len())
            })
        }
        AnyMessage::LbMeta(msg) => {
            json!({
                "message_type": "LBMETA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("LBMETA message with {} labels", msg.content.labels.len())
            })
        }
        AnyMessage::PolyData(msg) => {
            json!({
                "message_type": "POLYDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("POLYDATA message with {} points", msg.content.points.len())
            })
        }
        AnyMessage::Video(msg) => {
            json!({
                "message_type": "VIDEO",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("VIDEO message: {}x{} pixels", msg.content.width, msg.content.height)
            })
        }
        AnyMessage::VideoMeta(msg) => {
            json!({
                "message_type": "VIDEOMETA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "content": {
                    "width": msg.content.width,
                    "height": msg.content.height,
                    "framerate": msg.content.framerate,
                    "bitrate": msg.content.bitrate
                }
            })
        }
        AnyMessage::Command(msg) => {
            json!({
                "message_type": "COMMAND",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "content": {
                    "command_id": msg.content.command_id,
                    "command_name": &msg.content.command_name,
                    "encoding": msg.content.encoding
                }
            })
        }

        // Query messages
        AnyMessage::GetTransform(msg) => {
            json!({
                "message_type": "GET_TRANSFORM",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_TRANSFORM query received"
            })
        }
        AnyMessage::GetStatus(msg) => {
            json!({
                "message_type": "GET_STATUS",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_STATUS query received"
            })
        }
        AnyMessage::GetCapability(msg) => {
            json!({
                "message_type": "GET_CAPABILITY",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_CAPABILITY query received"
            })
        }
        AnyMessage::GetImage(msg) => {
            json!({
                "message_type": "GET_IMAGE",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_IMAGE query received"
            })
        }
        AnyMessage::GetImgMeta(msg) => {
            json!({
                "message_type": "GET_IMGMETA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_IMGMETA query received"
            })
        }
        AnyMessage::GetLbMeta(msg) => {
            json!({
                "message_type": "GET_LBMETA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_LBMETA query received"
            })
        }
        AnyMessage::GetPoint(msg) => {
            json!({
                "message_type": "GET_POINT",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_POINT query received"
            })
        }
        AnyMessage::GetTData(msg) => {
            json!({
                "message_type": "GET_TDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "GET_TDATA query received"
            })
        }

        // Response messages
        AnyMessage::RtsTransform(msg) => {
            json!({
                "message_type": "RTS_TRANSFORM",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "RTS_TRANSFORM response received"
            })
        }
        AnyMessage::RtsStatus(msg) => {
            json!({
                "message_type": "RTS_STATUS",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "RTS_STATUS response received"
            })
        }
        AnyMessage::RtsCapability(msg) => {
            json!({
                "message_type": "RTS_CAPABILITY",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "RTS_CAPABILITY response received"
            })
        }
        AnyMessage::RtsImage(msg) => {
            json!({
                "message_type": "RTS_IMAGE",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "RTS_IMAGE response received"
            })
        }
        AnyMessage::RtsTData(msg) => {
            json!({
                "message_type": "RTS_TDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "RTS_TDATA response received"
            })
        }

        // Streaming control messages
        AnyMessage::StartTData(msg) => {
            json!({
                "message_type": "STT_TDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "STT_TDATA streaming start received"
            })
        }
        AnyMessage::StopTransform(msg) => {
            json!({
                "message_type": "STP_TRANSFORM",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "STP_TRANSFORM streaming stop received"
            })
        }
        AnyMessage::StopPosition(msg) => {
            json!({
                "message_type": "STP_POSITION",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "STP_POSITION streaming stop received"
            })
        }
        AnyMessage::StopQtData(msg) => {
            json!({
                "message_type": "STP_QTDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "STP_QTDATA streaming stop received"
            })
        }
        AnyMessage::StopTData(msg) => {
            json!({
                "message_type": "STP_TDATA",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "STP_TDATA streaming stop received"
            })
        }
        AnyMessage::StopImage(msg) => {
            json!({
                "message_type": "STP_IMAGE",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "STP_IMAGE streaming stop received"
            })
        }
        AnyMessage::StopNdArray(msg) => {
            json!({
                "message_type": "STP_NDARRAY",
                "device_name": msg.header.device_name.as_str().unwrap_or("Unknown"),
                "note": "STP_NDARRAY streaming stop received"
            })
        }

        // Unknown message type
        AnyMessage::Unknown { header, body } => {
            json!({
                "message_type": header.type_name.as_str().unwrap_or("UNKNOWN"),
                "device_name": header.device_name.as_str().unwrap_or("Unknown"),
                "note": format!("Unknown message type with {} bytes body", body.len())
            })
        }

        _ => {
            json!({
                "message_type": message.message_type(),
                "device_name": "Unknown",
                "note": "Message type received"
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
