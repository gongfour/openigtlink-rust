//! OpenIGTLink Client Example
//!
//! This example demonstrates how to create a simple OpenIGTLink client
//! that connects to a server and sends different message types.
//!
//! # Usage
//!
//! ```bash
//! # Test all message types sequentially
//! cargo run --example client
//!
//! # Test specific message type
//! cargo run --example client transform
//! cargo run --example client status
//! cargo run --example client capability
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::IgtlClient;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{CapabilityMessage, StatusMessage, TransformMessage};
use std::env;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let scenario = parse_scenario();

    // Connect to server
    let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    println!("[INFO] Connected to server\n");

    // Execute test scenario
    match scenario.as_str() {
        "all" => {
            test_transform(&mut client)?;
            test_status(&mut client)?;
            test_capability(&mut client)?;
        }
        "transform" => test_transform(&mut client)?,
        "status" => test_status(&mut client)?,
        "capability" => test_capability(&mut client)?,
        _ => unreachable!(),
    }

    println!("[INFO] All tests completed successfully");
    Ok(())
}

fn parse_scenario() -> String {
    let scenario = env::args().nth(1).unwrap_or_else(|| "all".to_string());

    match scenario.as_str() {
        "all" | "transform" | "status" | "capability" => scenario,
        _ => {
            eprintln!("Usage: {} [all|transform|status|capability]", env::args().next().unwrap());
            eprintln!("\nAvailable scenarios:");
            eprintln!("  all        - Test all message types sequentially (default)");
            eprintln!("  transform  - Test TRANSFORM message only");
            eprintln!("  status     - Test STATUS message only");
            eprintln!("  capability - Test CAPABILITY message only");
            std::process::exit(1);
        }
    }
}

fn test_transform(_client: &mut IgtlClient) -> Result<()> {
    println!("[TEST] TRANSFORM message test");
    // Implementation will be added in next task
    Ok(())
}

fn test_status(_client: &mut IgtlClient) -> Result<()> {
    println!("[TEST] STATUS message test");
    // Implementation will be added in next task
    Ok(())
}

fn test_capability(_client: &mut IgtlClient) -> Result<()> {
    println!("[TEST] CAPABILITY message test");
    // Implementation will be added in next task
    Ok(())
}
