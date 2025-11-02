use std::net::SocketAddr;
use std::str::FromStr;
use openigtlink_rust::error::{IgtlError, Result};

/// Parse socket address from string
pub fn parse_addr(addr_str: &str) -> Result<SocketAddr> {
    SocketAddr::from_str(addr_str).map_err(|e| {
        IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid address '{}': {}", addr_str, e),
        ))
    })
}

/// Parse message types from comma-separated string
pub fn parse_message_types(types_str: &str) -> Vec<String> {
    if types_str.trim().to_lowercase() == "all" {
        vec![
            "TRANSFORM".to_string(),
            "IMAGE".to_string(),
            "STATUS".to_string(),
            "SENSOR".to_string(),
            "COMMAND".to_string(),
            "STRING".to_string(),
            "POSITION".to_string(),
            "QTDATA".to_string(),
            "TDATA".to_string(),
            "POINT".to_string(),
            "POLYDATA".to_string(),
            "TRAJECTORY".to_string(),
            "VIDEO".to_string(),
            "BIND".to_string(),
            "CAPABILITY".to_string(),
            "IMGMETA".to_string(),
            "VIDEOMETA".to_string(),
            "LBMETA".to_string(),
            "COLORTABLE".to_string(),
            "NDARRAY".to_string(),
        ]
    } else {
        types_str
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_addr() {
        let addr = parse_addr("127.0.0.1:18944").unwrap();
        assert_eq!(addr.port(), 18944);
    }

    #[test]
    fn test_parse_message_types_all() {
        let types = parse_message_types("all");
        assert_eq!(types.len(), 20);
    }

    #[test]
    fn test_parse_message_types_specific() {
        let types = parse_message_types("TRANSFORM,IMAGE,STATUS");
        assert_eq!(types, vec!["TRANSFORM", "IMAGE", "STATUS"]);
    }
}
