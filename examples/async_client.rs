//! OpenIGTLink Async Client Example
//!
//! This example demonstrates how to create an asynchronous OpenIGTLink client
//! using ClientBuilder and dynamic message handling (AnyMessage).
//!
//! # Usage
//!
//! ```bash
//! # Start async server first
//! cargo run --example async_server
//!
//! # Then run async client (in another terminal)
//! cargo run --example async_client
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::ClientBuilder;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{CapabilityMessage, StatusMessage, TransformMessage};
use openigtlink_rust::protocol::AnyMessage;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    println!("=== OpenIGTLink Async Client (AnyMessage) ===\n");

    // Connect to server using ClientBuilder
    println!("[INFO] Connecting to 127.0.0.1:18944...");
    let mut client = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .async_mode()
        .build()
        .await?;
    println!("[INFO] Connected to server\n");

    // Test 1: Send TRANSFORM message
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[TEST 1] Sending TRANSFORM message...");
    let transform = TransformMessage::translation(10.0, 20.0, 30.0);
    println!("[SEND] Translation: (10.0, 20.0, 30.0)");
    let msg = IgtlMessage::new(transform, "AsyncClient")?;
    client.send(&msg).await?;

    // Receive response dynamically
    let response = client.receive_any().await?;
    println!("[RECV] {} from '{}'", response.message_type(), response.device_name()?);

    match response {
        AnyMessage::Status(status_msg) => {
            println!("       Status: {} - {}", status_msg.content.code, status_msg.content.status_string);
        }
        _ => println!("       Unexpected message type"),
    }
    println!("✓ Test 1 completed\n");

    // Test 2: Send STATUS message
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[TEST 2] Sending STATUS message...");
    let status = StatusMessage::ok("Async client test message");
    println!("[SEND] Status: {}", status.status_string);
    let msg = IgtlMessage::new(status, "AsyncClient")?;
    client.send(&msg).await?;

    // Receive response dynamically
    let response = client.receive_any().await?;
    println!("[RECV] {} from '{}'", response.message_type(), response.device_name()?);

    match response {
        AnyMessage::Capability(cap_msg) => {
            println!("       Server capabilities ({} types):", cap_msg.content.types.len());
            for (i, typ) in cap_msg.content.types.iter().enumerate() {
                println!("         {}. {}", i + 1, typ);
            }
        }
        _ => println!("       Unexpected message type"),
    }
    println!("✓ Test 2 completed\n");

    // Test 3: Send CAPABILITY message
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[TEST 3] Sending CAPABILITY message...");
    let capability = CapabilityMessage::new(vec![
        "TRANSFORM".to_string(),
        "STATUS".to_string(),
        "CAPABILITY".to_string(),
    ]);
    println!("[SEND] Client capabilities: {} types", capability.types.len());
    let msg = IgtlMessage::new(capability, "AsyncClient")?;
    client.send(&msg).await?;
    println!("✓ Test 3 completed\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[INFO] All tests completed successfully");
    println!("[INFO] Connection will close automatically");

    Ok(())
}
