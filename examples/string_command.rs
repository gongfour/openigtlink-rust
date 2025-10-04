//! STRING Message - Command/Control Example
//!
//! This example demonstrates using STRING messages for sending text-based
//! commands to control remote devices or systems.
//!
//! # Usage
//!
//! ```bash
//! # Send a series of control commands
//! cargo run --example string_command
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::IgtlClient;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::StringMessage;
use std::thread;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    println!("=== STRING Message: Device Command & Control ===\n");

    // Connect to server
    let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    println!("[INFO] Connected to OpenIGTLink server\n");

    // Scenario 1: System lifecycle commands
    println!("[SCENARIO 1] System Lifecycle Commands");
    println!("  Controlling imaging device startup/shutdown sequence...\n");

    send_command(&mut client, "INIT", "Initialize imaging system")?;
    thread::sleep(Duration::from_secs(1));

    send_command(&mut client, "START", "Begin image acquisition")?;
    thread::sleep(Duration::from_secs(2));

    send_command(&mut client, "STOP", "Stop image acquisition")?;
    thread::sleep(Duration::from_secs(1));

    send_command(&mut client, "SHUTDOWN", "Power down system")?;
    thread::sleep(Duration::from_secs(1));

    // Scenario 2: Configuration commands
    println!("\n[SCENARIO 2] Configuration Commands");
    println!("  Setting device parameters...\n");

    send_command(
        &mut client,
        "SET_GAIN 75",
        "Adjust image gain to 75%",
    )?;
    thread::sleep(Duration::from_millis(500));

    send_command(
        &mut client,
        "SET_FREQUENCY 5.0",
        "Set ultrasound frequency to 5 MHz",
    )?;
    thread::sleep(Duration::from_millis(500));

    send_command(
        &mut client,
        "SET_DEPTH 120",
        "Configure imaging depth to 120mm",
    )?;
    thread::sleep(Duration::from_millis(500));

    // Scenario 3: Query commands
    println!("\n[SCENARIO 3] Query Commands");
    println!("  Requesting device information...\n");

    send_command(&mut client, "GET_STATUS", "Request device status")?;
    thread::sleep(Duration::from_millis(500));

    send_command(&mut client, "GET_VERSION", "Query firmware version")?;
    thread::sleep(Duration::from_millis(500));

    send_command(&mut client, "GET_CAPABILITIES", "List device capabilities")?;

    println!("\n[INFO] All commands sent successfully");
    Ok(())
}

/// Send a text command via STRING message
///
/// # Arguments
/// * `client` - Connected OpenIGTLink client
/// * `command` - Command string to send
/// * `description` - Human-readable description
fn send_command(client: &mut IgtlClient, command: &str, description: &str) -> Result<()> {
    // Create STRING message with UTF-8 encoding
    let string_msg = StringMessage::utf8(command);

    println!("  → Sending: \"{}\"", command);
    println!("    Description: {}", description);
    println!("    Encoding: UTF-8 (MIBenum=106)");
    println!("    Length: {} bytes", string_msg.len());

    // Send message
    let msg = IgtlMessage::new(string_msg, "CommandInterface")?;
    client.send(&msg)?;

    println!("    ✓ Sent\n");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = StringMessage::utf8("START");
        assert_eq!(cmd.as_str(), "START");
        assert_eq!(cmd.encoding, 106); // UTF-8
    }

    #[test]
    fn test_command_with_parameters() {
        let cmd = StringMessage::utf8("SET_GAIN 75");
        assert_eq!(cmd.as_str(), "SET_GAIN 75");
        assert!(!cmd.is_empty());
    }

    #[test]
    fn test_ascii_encoding() {
        let cmd = StringMessage::new("INIT");
        assert_eq!(cmd.encoding, 3); // US-ASCII
    }

    #[test]
    fn test_command_length() {
        let cmd = StringMessage::utf8("GET_STATUS");
        assert_eq!(cmd.len(), 10);
    }
}
