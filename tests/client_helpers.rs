//! Integration tests for IgtlClient helper methods
//!
//! These tests verify the convenience methods for query and streaming control.

use openigtlink_rust::io::IgtlClient;

#[test]
fn test_helper_methods_compile() {
    // This test ensures the helper methods compile correctly
    // Actual integration testing requires a running server

    fn _test_request_capability(client: &mut IgtlClient) {
        let _capability = client.request_capability().unwrap();
    }

    fn _test_start_tracking(client: &mut IgtlClient) {
        let _ack = client.start_tracking(50, "RAS").unwrap();
    }

    fn _test_stop_tracking(client: &mut IgtlClient) {
        client.stop_tracking().unwrap();
    }
}

#[test]
fn test_helper_method_signatures() {
    // Verify method signatures match expected types
    use openigtlink_rust::protocol::types::{CapabilityMessage, RtsTDataMessage};

    fn _check_types(client: &mut IgtlClient) {
        // request_capability returns CapabilityMessage
        let _cap: CapabilityMessage = client.request_capability().unwrap();

        // start_tracking returns RtsTDataMessage
        let _rts: RtsTDataMessage = client.start_tracking(50, "RAS").unwrap();

        // stop_tracking returns ()
        let _unit: () = client.stop_tracking().unwrap();
    }
}

#[test]
fn test_coordinate_name_handling() {
    // Test that coordinate names are properly handled
    // (compilation test only, no server required)

    fn _test_various_coordinates(client: &mut IgtlClient) {
        // Common anatomical coordinate systems
        let _ = client.start_tracking(50, "RAS"); // Right-Anterior-Superior
        let _ = client.start_tracking(50, "LPS"); // Left-Posterior-Superior
        let _ = client.start_tracking(50, "IJK"); // Image indices

        // Short name
        let _ = client.start_tracking(50, "A");

        // Long name (will be truncated to 32 bytes in StartTDataMessage)
        let _ = client.start_tracking(50, "VeryLongCoordinateSystemNameThatExceeds32Characters");
    }
}

#[test]
fn test_resolution_values() {
    // Test various resolution (update interval) values
    // (compilation test only, no server required)

    fn _test_various_resolutions(client: &mut IgtlClient) {
        // High frequency (120 Hz)
        let _ = client.start_tracking(8, "RAS");

        // Medium frequency (60 Hz)
        let _ = client.start_tracking(16, "RAS");

        // Standard frequency (30 Hz)
        let _ = client.start_tracking(33, "RAS");

        // Low frequency (10 Hz)
        let _ = client.start_tracking(100, "RAS");

        // Very low frequency (1 Hz)
        let _ = client.start_tracking(1000, "RAS");
    }
}
