//! Integration tests for IgtlClient helper methods
//!
//! These tests verify the convenience methods for query and streaming control.

use openigtlink_rust::io::SyncIgtlClient;

// NOTE: These tests are disabled because the helper methods (request_capability,
// start_tracking, stop_tracking) have not yet been implemented on SyncIgtlClient.
// They will be re-enabled when the helper methods are added.

/*
#[test]
#[ignore = "Helper methods not yet implemented"]
fn test_helper_methods_compile() {
    // This test ensures the helper methods compile correctly
    // Actual integration testing requires a running server
    // TODO: Implement helper methods on SyncIgtlClient

    fn _test_request_capability(client: &mut SyncIgtlClient) {
        let _capability = client.request_capability().unwrap();
    }

    fn _test_start_tracking(client: &mut SyncIgtlClient) {
        let _ack = client.start_tracking(50, "RAS").unwrap();
    }

    fn _test_stop_tracking(client: &mut SyncIgtlClient) {
        client.stop_tracking().unwrap();
    }
}

#[test]
#[ignore = "Helper methods not yet implemented"]
fn test_helper_method_signatures() {
    // Verify method signatures match expected types
    // TODO: Implement helper methods on SyncIgtlClient
    use openigtlink_rust::protocol::types::{CapabilityMessage, RtsTDataMessage};

    fn _check_types(client: &mut SyncIgtlClient) {
        // request_capability returns CapabilityMessage
        let _cap: CapabilityMessage = client.request_capability().unwrap();

        // start_tracking returns RtsTDataMessage
        let _rts: RtsTDataMessage = client.start_tracking(50, "RAS").unwrap();

        // stop_tracking returns ()
        let _unit: () = client.stop_tracking().unwrap();
    }
}

#[test]
#[ignore = "Helper methods not yet implemented"]
fn test_coordinate_name_handling() {
    // Test that coordinate names are properly handled
    // (compilation test only, no server required)
    // TODO: Implement helper methods on SyncIgtlClient

    fn _test_various_coordinates(client: &mut SyncIgtlClient) {
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
#[ignore = "Helper methods not yet implemented"]
fn test_resolution_values() {
    // Test various resolution (update interval) values
    // (compilation test only, no server required)
    // TODO: Implement helper methods on SyncIgtlClient

    fn _test_various_resolutions(client: &mut SyncIgtlClient) {
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
*/
