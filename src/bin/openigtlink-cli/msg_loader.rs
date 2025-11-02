use openigtlink_rust::protocol::types::TransformMessage;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::error::Result;
use serde_json::Value;
use std::fs;
use tracing::info;

/// Load a TRANSFORM message from a JSON file (simplified Phase 2)
pub fn load_transform_from_file(file_path: &str) -> Result<IgtlMessage<TransformMessage>> {
    info!("Loading TRANSFORM message from file: {}", file_path);

    let content = fs::read_to_string(file_path)?;
    load_transform_from_json(&content)
}

/// Load a TRANSFORM message from JSON string
pub fn load_transform_from_json(json_str: &str) -> Result<IgtlMessage<TransformMessage>> {
    let value: Value = serde_json::from_str(json_str).map_err(|e| {
        openigtlink_rust::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("JSON parse error: {}", e),
        ))
    })?;

    let device_name = value["device_name"]
        .as_str()
        .unwrap_or("DefaultDevice");

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

    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transform() {
        let json = r#"{
            "device_name": "Tool1",
            "transform": {
                "matrix": [
                    [1.0, 0.0, 0.0, 100.0],
                    [0.0, 1.0, 0.0, 50.0],
                    [0.0, 0.0, 1.0, 200.0],
                    [0.0, 0.0, 0.0, 1.0]
                ]
            }
        }"#;

        let result = load_transform_from_json(json);
        assert!(result.is_ok());
    }
}
