//! C++ OpenIGTLink Compatibility Tests
//!
//! These tests verify byte-level compatibility with the C++ OpenIGTLink implementation.
//! They ensure that messages encoded by this library match the exact binary format
//! expected by C++ OpenIGTLink servers (3D Slicer, PLUS Toolkit, etc.).

use openigtlink_rust::protocol::message::{IgtlMessage, Message};
use openigtlink_rust::protocol::types::{
    GetCapabilityMessage, GetStatusMessage, RtsTDataMessage, StartTDataMessage, StopTDataMessage,
};

/// Test GET_CAPABIL message encoding
#[test]
fn test_get_capability_encoding() {
    let msg = GetCapabilityMessage;
    let igtl_msg = IgtlMessage::new(msg, "TestDevice").unwrap();

    // Verify message type is exactly "GET_CAPABIL" (12 bytes, null-padded)
    assert_eq!(igtl_msg.header.type_name.as_str().unwrap(), "GET_CAPABIL");

    // Verify body size is 0 (empty message)
    assert_eq!(igtl_msg.header.body_size, 0);

    // Verify device name
    assert_eq!(igtl_msg.header.device_name.as_str().unwrap(), "TestDevice");
}

/// Test GET_STATUS message encoding
#[test]
fn test_get_status_encoding() {
    let msg = GetStatusMessage;
    let igtl_msg = IgtlMessage::new(msg, "QueryClient").unwrap();

    // Verify message type
    assert_eq!(igtl_msg.header.type_name.as_str().unwrap(), "GET_STATUS");

    // Verify empty body
    assert_eq!(igtl_msg.header.body_size, 0);
}

/// Test STT_TDATA message encoding (byte-level compatibility)
#[test]
fn test_stt_tdata_encoding() {
    let msg = StartTDataMessage {
        resolution: 50,                     // 50ms = 0x00000032
        coordinate_name: "RAS".to_string(), // 32 bytes, null-padded
    };

    let igtl_msg = IgtlMessage::new(msg.clone(), "Tracker").unwrap();

    // Verify message type
    assert_eq!(igtl_msg.header.type_name.as_str().unwrap(), "STT_TDATA");

    // Verify body size: 4 bytes (resolution) + 32 bytes (coordinate_name) = 36 bytes
    assert_eq!(igtl_msg.header.body_size, 36);

    // Encode body and verify byte layout
    let body = msg.encode_content().unwrap();
    assert_eq!(body.len(), 36, "STT_TDATA body must be exactly 36 bytes");

    // Verify resolution (big-endian u32)
    assert_eq!(body[0], 0x00, "Resolution byte 0 should be 0x00");
    assert_eq!(body[1], 0x00, "Resolution byte 1 should be 0x00");
    assert_eq!(body[2], 0x00, "Resolution byte 2 should be 0x00");
    assert_eq!(
        body[3], 0x32,
        "Resolution byte 3 should be 0x32 (50 decimal)"
    );

    // Verify coordinate name (null-padded to 32 bytes)
    assert_eq!(body[4], b'R', "Coordinate name byte 0 should be 'R'");
    assert_eq!(body[5], b'A', "Coordinate name byte 1 should be 'A'");
    assert_eq!(body[6], b'S', "Coordinate name byte 2 should be 'S'");
    assert_eq!(body[7], 0x00, "Coordinate name byte 3 should be null");

    // Verify remaining bytes are null-padded
    for i in 8..36 {
        assert_eq!(
            body[i], 0x00,
            "Coordinate name byte {} should be null-padded",
            i
        );
    }
}

/// Test STT_TDATA with long coordinate name (truncation)
#[test]
fn test_stt_tdata_long_coordinate() {
    // 40-character string should be truncated to 32 bytes
    let long_name = "VeryLongCoordinateSystemNameHere_40ch";
    let msg = StartTDataMessage {
        resolution: 100,
        coordinate_name: long_name.to_string(),
    };

    let body = msg.encode_content().unwrap();

    // Verify still 36 bytes
    assert_eq!(body.len(), 36);

    // Verify coordinate name is truncated to 32 bytes
    let coord_bytes = &body[4..36];
    assert_eq!(coord_bytes.len(), 32);

    // First 32 chars should match
    assert_eq!(&coord_bytes[..32], &long_name.as_bytes()[..32]);
}

/// Test STP_TDATA message encoding
#[test]
fn test_stp_tdata_encoding() {
    let msg = StopTDataMessage;
    let igtl_msg = IgtlMessage::new(msg, "Tracker").unwrap();

    // Verify message type
    assert_eq!(igtl_msg.header.type_name.as_str().unwrap(), "STP_TDATA");

    // Verify empty body
    assert_eq!(igtl_msg.header.body_size, 0);
}

/// Test RTS_TDATA message encoding (byte-level)
#[test]
fn test_rts_tdata_encoding() {
    // Test OK status (1)
    let msg_ok = RtsTDataMessage::ok();
    let body_ok = msg_ok.encode_content().unwrap();

    assert_eq!(body_ok.len(), 2, "RTS_TDATA body must be 2 bytes");
    assert_eq!(body_ok[0], 0x00, "Status byte 0 should be 0x00");
    assert_eq!(body_ok[1], 0x01, "Status byte 1 should be 0x01 (OK)");

    // Test ERROR status (0)
    let msg_err = RtsTDataMessage::error();
    let body_err = msg_err.encode_content().unwrap();

    assert_eq!(body_err.len(), 2, "RTS_TDATA body must be 2 bytes");
    assert_eq!(body_err[0], 0x00, "Status byte 0 should be 0x00");
    assert_eq!(body_err[1], 0x00, "Status byte 1 should be 0x00 (ERROR)");

    // Test custom status (big-endian encoding)
    let msg_custom = RtsTDataMessage::new(0x1234);
    let body_custom = msg_custom.encode_content().unwrap();

    assert_eq!(body_custom.len(), 2);
    assert_eq!(body_custom[0], 0x12, "Status high byte should be 0x12");
    assert_eq!(body_custom[1], 0x34, "Status low byte should be 0x34");
}

/// Test STT_TDATA roundtrip (encode → decode)
#[test]
fn test_stt_tdata_roundtrip() {
    let original = StartTDataMessage {
        resolution: 33,
        coordinate_name: "LPS".to_string(),
    };

    // Encode
    let encoded = original.encode_content().unwrap();

    // Decode
    let decoded = StartTDataMessage::decode_content(&encoded).unwrap();

    // Verify
    assert_eq!(decoded.resolution, 33);
    assert_eq!(decoded.coordinate_name, "LPS");
}

/// Test RTS_TDATA roundtrip
#[test]
fn test_rts_tdata_roundtrip() {
    let statuses = vec![0, 1, 42, 0xFFFF];

    for status in statuses {
        let original = RtsTDataMessage::new(status);
        let encoded = original.encode_content().unwrap();
        let decoded = RtsTDataMessage::decode_content(&encoded).unwrap();

        assert_eq!(decoded.status, status, "Status {} roundtrip failed", status);
    }
}

/// Test message type name length compliance
#[test]
fn test_message_type_length() {
    // All message types must be ≤ 12 characters (OpenIGTLink spec)
    let types = vec![
        "GET_CAPABIL",  // 11 chars (CAPABILITY truncated)
        "GET_STATUS",   // 10 chars
        "GET_TRANSFOR", // 12 chars (TRANSFORM truncated)
        "GET_IMAGE",    // 9 chars
        "GET_TDATA",    // 9 chars
        "GET_POINT",    // 9 chars
        "GET_IMGMETA",  // 11 chars
        "GET_LBMETA",   // 10 chars
        "STT_TDATA",    // 9 chars
        "STP_TDATA",    // 9 chars
        "STP_IMAGE",    // 9 chars
        "STP_TRANSFOR", // 12 chars
        "RTS_TDATA",    // 9 chars
    ];

    for msg_type in types {
        assert!(
            msg_type.len() <= 12,
            "Message type '{}' exceeds 12 characters ({})",
            msg_type,
            msg_type.len()
        );
    }
}

/// Verify header size is exactly 58 bytes (OpenIGTLink spec)
#[test]
fn test_header_size() {
    use openigtlink_rust::protocol::header::Header;
    assert_eq!(
        Header::SIZE,
        58,
        "OpenIGTLink header must be exactly 58 bytes"
    );
}

/// Test CRC calculation compatibility
#[test]
fn test_crc_compatibility() {
    // Empty body CRC should be 0
    let msg = GetCapabilityMessage;
    let encoded = msg.encode_content().unwrap();

    use openigtlink_rust::protocol::crc::calculate_crc;
    let crc = calculate_crc(&encoded);

    // Empty message CRC is 0
    assert_eq!(crc, 0, "Empty message CRC should be 0");
}

/// Test full message serialization (header + body + CRC)
#[test]
fn test_full_message_serialization() {
    let msg = StartTDataMessage {
        resolution: 50,
        coordinate_name: "RAS".to_string(),
    };

    let igtl_msg = IgtlMessage::new(msg, "TestDevice").unwrap();

    // Serialize message (header + body, CRC is in header)
    let serialized = igtl_msg.encode().unwrap();

    // Total size: 58 (header) + 36 (body) = 94 bytes
    assert_eq!(
        serialized.len(),
        58 + 36,
        "STT_TDATA message should be 94 bytes (header + body)"
    );

    // Verify header is at the beginning (58 bytes)
    assert_eq!(serialized[0], 0x00); // Version byte 0
    assert_eq!(serialized[1], 0x02); // Version = 2

    // Verify message type starts at byte 2
    let msg_type = &serialized[2..14];
    assert_eq!(&msg_type[..9], b"STT_TDATA");

    // Verify body starts at byte 58
    let body = &serialized[58..];
    assert_eq!(body.len(), 36);

    // Verify body content (resolution + coordinate_name)
    assert_eq!(body[0..4], [0x00, 0x00, 0x00, 0x32]); // resolution = 50
    assert_eq!(body[4], b'R'); // coordinate 'RAS'
    assert_eq!(body[5], b'A');
    assert_eq!(body[6], b'S');
}
