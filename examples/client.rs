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
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
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
    let mut client = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .sync()
        .build()?;
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

fn test_transform(client: &mut SyncIgtlClient) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[TEST] Sending TRANSFORM message...");

    // Create a translation transform (10, 20, 30)
    let transform = TransformMessage::translation(10.0, 20.0, 30.0);
    println!("[SEND] Translation vector: (10.0, 20.0, 30.0)");
    println!("       Matrix (first row): [{:.2}, {:.2}, {:.2}, {:.2}]",
             transform.matrix[0][0], transform.matrix[0][1],
             transform.matrix[0][2], transform.matrix[0][3]);

    // Wrap in IgtlMessage and send
    let msg = IgtlMessage::new(transform, "ClientDevice")?;
    client.send(&msg)?;

    // Receive STATUS response
    let response: IgtlMessage<StatusMessage> = client.receive()?;
    println!("[RECV] STATUS response:");
    println!("       Code: {}", response.content.code);
    println!("       Name: '{}'", response.content.error_name);
    println!("       Message: '{}'", response.content.status_string);
    println!("✓ TRANSFORM test completed\n");

    Ok(())
}

fn test_status(client: &mut SyncIgtlClient) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[TEST] Sending STATUS message...");

    // Create an OK status message
    let status = StatusMessage::ok("Client test message");
    println!("[SEND] Code: {}, Message: '{}'", status.code, status.status_string);

    // Wrap in IgtlMessage and send
    let msg = IgtlMessage::new(status, "ClientDevice")?;
    client.send(&msg)?;

    // Receive CAPABILITY response
    let response: IgtlMessage<CapabilityMessage> = client.receive()?;
    println!("[RECV] CAPABILITY response:");
    println!("       Supported types ({}):", response.content.types.len());
    for (i, typ) in response.content.types.iter().enumerate() {
        println!("         {}. {}", i + 1, typ);
    }
    println!("✓ STATUS test completed\n");

    Ok(())
}

fn test_capability(client: &mut SyncIgtlClient) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[TEST] Sending CAPABILITY message...");

    // Create a CAPABILITY message with supported types
    let capability = CapabilityMessage::new(vec![
        "TRANSFORM".to_string(),
        "STATUS".to_string(),
        "CAPABILITY".to_string(),
    ]);
    println!("[SEND] Supported types ({}):", capability.types.len());
    for (i, typ) in capability.types.iter().enumerate() {
        println!("         {}. {}", i + 1, typ);
    }

    // Wrap in IgtlMessage and send
    let msg = IgtlMessage::new(capability, "ClientDevice")?;
    client.send(&msg)?;

    println!("[INFO] CAPABILITY sent, server will close connection");
    println!("✓ CAPABILITY test completed\n");

    Ok(())
}
