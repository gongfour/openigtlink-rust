//! Client Helper Methods Example
//!
//! This example demonstrates common query and streaming control patterns.
//! These patterns show how to build helper functions for common OpenIGTLink operations.
//!
//! # Usage
//!
//! ```bash
//! # Start a C++ OpenIGTLink server (3D Slicer, PLUS Toolkit, or the Rust server)
//! cargo run --example server
//!
//! # Then run this example (in another terminal)
//! cargo run --example client_helpers
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{
    CapabilityMessage, GetCapabilityMessage, RtsTDataMessage, StartTDataMessage, StopTDataMessage,
    TDataMessage,
};
use std::env;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let server_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:18944".to_string());

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Client Helper Patterns Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Connect to server
    println!("[1] Connecting to {}...", server_addr);
    let mut client = ClientBuilder::new().tcp(&server_addr).sync().build()?;
    println!("    ✓ Connected\n");

    // Helper Pattern 1: Request server capabilities
    println!("[2] Requesting server capabilities...");
    let capability = request_capability(&mut client)?;
    println!(
        "    ✓ Server supports {} message types:",
        capability.types.len()
    );
    for (i, msg_type) in capability.types.iter().enumerate() {
        println!("      {}. {}", i + 1, msg_type);
    }
    println!();

    // Helper Pattern 2: Start tracking stream
    println!("[3] Starting tracking stream (50ms resolution, RAS coordinates)...");
    let ack = start_tracking(&mut client, 50, "RAS")?;

    if ack.status == 1 {
        println!("    ✓ Server acknowledged (status: OK)\n");

        // Receive tracking data
        println!("[4] Receiving tracking data...");
        for i in 1..=10 {
            let tdata: IgtlMessage<TDataMessage> = client.receive()?;

            println!("    Sample {}/10:", i);
            println!("      Device: {}", tdata.header.device_name.as_str()?);
            println!("      Tools: {}", tdata.content.elements.len());

            for (j, element) in tdata.content.elements.iter().enumerate() {
                let x = element.matrix[0][3];
                let y = element.matrix[1][3];
                let z = element.matrix[2][3];

                println!(
                    "        {}. '{}': ({:.2}, {:.2}, {:.2}) mm",
                    j + 1,
                    element.name,
                    x,
                    y,
                    z
                );
            }
        }
        println!();

        // Helper Pattern 3: Stop tracking stream
        println!("[5] Stopping tracking stream...");
        stop_tracking(&mut client)?;
        println!("    ✓ Stream stopped\n");
    } else {
        println!("    ✗ Server rejected request (status: ERROR)\n");
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ Client helper patterns demo completed");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

/// Request server capabilities (helper function)
///
/// Sends a GET_CAPABIL query and receives the CAPABILITY response.
fn request_capability(client: &mut SyncIgtlClient) -> Result<CapabilityMessage> {
    let query = GetCapabilityMessage;
    let msg = IgtlMessage::new(query, "Client")?;
    client.send(&msg)?;

    let response: IgtlMessage<CapabilityMessage> = client.receive()?;
    Ok(response.content)
}

/// Start tracking data stream (helper function)
///
/// Sends a STT_TDATA message to request tracking data streaming and waits for
/// the server's RTS_TDATA acknowledgment.
fn start_tracking(
    client: &mut SyncIgtlClient,
    resolution: u32,
    coordinate_name: &str,
) -> Result<RtsTDataMessage> {
    let start_stream = StartTDataMessage {
        resolution,
        coordinate_name: coordinate_name.to_string(),
    };

    let msg = IgtlMessage::new(start_stream, "Client")?;
    client.send(&msg)?;

    let response: IgtlMessage<RtsTDataMessage> = client.receive()?;
    Ok(response.content)
}

/// Stop tracking data stream (helper function)
///
/// Sends a STP_TDATA message to stop the tracking data stream.
fn stop_tracking(client: &mut SyncIgtlClient) -> Result<()> {
    let stop_stream = StopTDataMessage;
    let msg = IgtlMessage::new(stop_stream, "Client")?;
    client.send(&msg)?;
    Ok(())
}
