use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::TransformMessage;
use openigtlink_rust::error::Result;
use serde_json::{json, Value};
use std::fs;
use tracing::info;

/// Save received TRANSFORM messages to a JSON file
pub fn save_transform_messages(
    messages: &[IgtlMessage<TransformMessage>],
    file_path: &str,
) -> Result<()> {
    info!("Saving {} TRANSFORM message(s) to file: {}", messages.len(), file_path);

    let mut json_messages = Vec::new();

    for msg in messages {
        let device_name_str = msg.header.device_name.as_str().unwrap_or("Unknown");
        let transform_data = json!({
            "device_name": device_name_str,
            "timestamp": {
                "seconds": msg.header.timestamp.seconds,
                "fraction": msg.header.timestamp.fraction
            },
            "transform": {
                "matrix": msg.content.matrix
            }
        });
        json_messages.push(transform_data);
    }

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
    info!("✓ Saved TRANSFORM message(s) to: {}", file_path);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matching() {
        assert!(matches_filter("TRANSFORM", "all"));
        assert!(matches_filter("TRANSFORM", "TRANSFORM,IMAGE"));
        assert!(matches_filter("IMAGE", "TRANSFORM,IMAGE"));
        assert!(!matches_filter("STATUS", "TRANSFORM,IMAGE"));
        assert!(matches_filter("transform", "TRANSFORM")); // case insensitive
    }
}
