//! STATUS Message - System Monitoring Example
//!
//! This example demonstrates using STATUS messages to report device health,
//! errors, and operational state to monitoring systems.
//!
//! # Usage
//!
//! ```bash
//! # Simulate device status reporting
//! cargo run --example status_monitor
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::StatusMessage;
use std::thread;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    println!("=== STATUS Message: Device Monitoring ===\n");

    // Connect to server
    let mut client = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .sync()
        .build()?;
    println!("[INFO] Connected to monitoring server\n");

    // Simulate device lifecycle with status updates
    println!("[SCENARIO 1] Normal Operation");
    println!("  Simulating successful device operation...\n");

    send_status_ok(&mut client, "System initialized successfully")?;
    thread::sleep(Duration::from_secs(1));

    send_status_ok(&mut client, "Imaging device ready")?;
    thread::sleep(Duration::from_secs(1));

    send_status_ok(&mut client, "Acquiring images at 30 FPS")?;
    thread::sleep(Duration::from_secs(2));

    // Scenario 2: Warning conditions
    println!("\n[SCENARIO 2] Warning Conditions");
    println!("  Simulating non-critical warnings...\n");

    send_status_warning(&mut client, "High CPU usage: 85%")?;
    thread::sleep(Duration::from_secs(1));

    send_status_warning(&mut client, "Temperature elevated: 68°C")?;
    thread::sleep(Duration::from_secs(1));

    send_status_warning(&mut client, "Network latency: 150ms")?;
    thread::sleep(Duration::from_secs(1));

    // Scenario 3: Error conditions
    println!("\n[SCENARIO 3] Error Conditions");
    println!("  Simulating critical errors...\n");

    send_status_error(
        &mut client,
        "ERR_TIMEOUT",
        "Connection timeout after 5000ms",
    )?;
    thread::sleep(Duration::from_secs(1));

    send_status_error(
        &mut client,
        "ERR_CRC",
        "CRC checksum mismatch in frame #1247",
    )?;
    thread::sleep(Duration::from_secs(1));

    send_status_error(
        &mut client,
        "ERR_HARDWARE",
        "Camera sensor failure - requires service",
    )?;
    thread::sleep(Duration::from_secs(1));

    // Recovery
    println!("\n[SCENARIO 4] Error Recovery");
    println!("  System recovering from errors...\n");

    send_status_ok(&mut client, "Connection re-established")?;
    thread::sleep(Duration::from_secs(1));

    send_status_ok(&mut client, "All systems operational")?;

    println!("\n[INFO] Status monitoring simulation complete");
    Ok(())
}

/// Send OK status (code=1)
fn send_status_ok(client: &mut SyncIgtlClient, message: &str) -> Result<()> {
    let status = StatusMessage::ok(message);

    println!("  ✓ OK: {}", message);
    println!("    Code: {} (OK)", status.code);
    println!("    Subcode: {}", status.subcode);

    let msg = IgtlMessage::new(status, "DeviceMonitor")?;
    client.send(&msg)?;

    Ok(())
}

/// Send warning status (code=2, custom)
fn send_status_warning(client: &mut SyncIgtlClient, message: &str) -> Result<()> {
    let status = StatusMessage {
        code: 2, // Custom warning code
        subcode: 0,
        error_name: "WARNING".to_string(),
        status_string: message.to_string(),
    };

    println!("  ⚠ WARNING: {}", message);
    println!("    Code: {} (Warning)", status.code);
    println!("    Error Name: {}", status.error_name);

    let msg = IgtlMessage::new(status, "DeviceMonitor")?;
    client.send(&msg)?;

    Ok(())
}

/// Send error status (code=0)
fn send_status_error(client: &mut SyncIgtlClient, error_name: &str, message: &str) -> Result<()> {
    let status = StatusMessage::error(error_name, message);

    println!("  ✗ ERROR: {}", message);
    println!("    Code: {} (Error)", status.code);
    println!("    Error Name: {}", error_name);
    println!("    Subcode: {}", status.subcode);

    let msg = IgtlMessage::new(status, "DeviceMonitor")?;
    client.send(&msg)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_status() {
        let status = StatusMessage::ok("Test OK");
        assert_eq!(status.code, 1);
        assert_eq!(status.status_string, "Test OK");
        assert_eq!(status.error_name, "");
    }

    #[test]
    fn test_error_status() {
        let status = StatusMessage::error("ERR_TEST", "Test error");
        assert_eq!(status.code, 0);
        assert_eq!(status.error_name, "ERR_TEST");
        assert_eq!(status.status_string, "Test error");
    }

    #[test]
    fn test_warning_status() {
        let status = StatusMessage {
            code: 2,
            subcode: 0,
            error_name: "WARNING".to_string(),
            status_string: "Test warning".to_string(),
        };
        assert_eq!(status.code, 2);
        assert_eq!(status.error_name, "WARNING");
    }

    #[test]
    fn test_status_with_subcode() {
        let status = StatusMessage {
            code: 1,
            subcode: 42,
            error_name: String::new(),
            status_string: "Custom subcode".to_string(),
        };
        assert_eq!(status.subcode, 42);
    }
}
