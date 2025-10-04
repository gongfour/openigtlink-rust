//! Client Helper Methods Example
//!
//! This example demonstrates the convenient helper methods for query and streaming control.
//! These methods simplify common OpenIGTLink operations into single function calls.
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

use openigtlink_rust::io::IgtlClient;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::TDataMessage;
use std::env;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> openigtlink_rust::error::Result<()> {
    let server_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:18944".to_string());

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Client Helper Methods Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Connect to server
    println!("[1] Connecting to {}...", server_addr);
    let mut client = IgtlClient::connect(&server_addr)?;
    println!("    ✓ Connected\n");

    // Helper Method 1: Request server capabilities
    println!("[2] Requesting server capabilities...");
    let capability = client.request_capability()?;
    println!("    ✓ Server supports {} message types:", capability.types.len());
    for (i, msg_type) in capability.types.iter().enumerate() {
        println!("      {}. {}", i + 1, msg_type);
    }
    println!();

    // Helper Method 2: Start tracking stream
    println!("[3] Starting tracking stream (50ms resolution, RAS coordinates)...");
    let ack = client.start_tracking(50, "RAS")?;

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

        // Helper Method 3: Stop tracking stream
        println!("[5] Stopping tracking stream...");
        client.stop_tracking()?;
        println!("    ✓ Stream stopped\n");
    } else {
        println!("    ✗ Server rejected request (status: ERROR)\n");
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ Client helper methods demo completed");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}
