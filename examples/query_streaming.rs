//! Query and Streaming Control Example
//!
//! This example demonstrates how to use OpenIGTLink query and streaming control
//! messages to communicate with a C++ OpenIGTLink server (e.g., 3D Slicer, PLUS Toolkit).
//!
//! # Protocol Flow
//!
//! 1. **Query Server Capabilities** - Send GET_CAPABIL to discover supported message types
//! 2. **Request Streaming Start** - Send STT_TDATA to start tracking data stream
//! 3. **Receive Acknowledgment** - Server responds with RTS_TDATA (status code)
//! 4. **Receive Stream Data** - Server sends TDATA messages at specified resolution
//! 5. **Stop Streaming** - Send STP_TDATA to stop the stream
//!
//! # Usage
//!
//! ```bash
//! # Start a C++ OpenIGTLink server first (e.g., 3D Slicer with OpenIGTLink module)
//! # Then run this example:
//! cargo run --example query_streaming
//!
//! # Connect to custom server address
//! cargo run --example query_streaming -- 192.168.1.100:18944
//! ```
//!
//! # C++ Server Compatibility
//!
//! This example is designed to work with:
//! - 3D Slicer with OpenIGTLink module
//! - PLUS Toolkit tracking servers
//! - Any C++ OpenIGTLink v2/v3 compliant server

use openigtlink_rust::error::{IgtlError, Result};
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{
    CapabilityMessage, GetCapabilityMessage, RtsTDataMessage, StartTDataMessage,
    StopTDataMessage, TDataMessage,
};
use std::env;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        match e {
            IgtlError::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::ConnectionRefused => {
                eprintln!("\n[ERROR] Connection refused");
                eprintln!("\nPlease start a C++ OpenIGTLink server first:");
                eprintln!("  • 3D Slicer: Load OpenIGTLink module and start server");
                eprintln!("  • PLUS Toolkit: Run PlusServer with tracking configuration");
                eprintln!("\nOr run the companion server example:");
                eprintln!("  cargo run --example server\n");
            }
            _ => eprintln!("[ERROR] {}", e),
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse server address from command line or use default
    let server_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:18944".to_string());

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  OpenIGTLink Query & Streaming Control Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Step 1: Connect to C++ OpenIGTLink server
    println!("[1] Connecting to server at {}...", server_addr);
    let mut client = ClientBuilder::new()
        .tcp(&server_addr)
        .sync()
        .build()?;
    println!("    ✓ Connected successfully\n");

    // Step 2: Query server capabilities (GET_CAPABIL → CAPABILITY)
    println!("[2] Querying server capabilities...");
    query_capabilities(&mut client)?;

    // Step 3: Start tracking data streaming (STT_TDATA → RTS_TDATA)
    println!("[3] Starting tracking data stream...");
    start_tracking_stream(&mut client)?;

    // Step 4: Receive tracking data (TDATA messages)
    println!("[4] Receiving tracking data stream...");
    receive_tracking_data(&mut client)?;

    // Step 5: Stop tracking data streaming (STP_TDATA)
    println!("[5] Stopping tracking data stream...");
    stop_tracking_stream(&mut client)?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ Query and streaming control example completed");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

/// Query server capabilities using GET_CAPABIL message
fn query_capabilities(client: &mut SyncIgtlClient) -> Result<()> {
    // Send GET_CAPABIL query message
    let get_capability = GetCapabilityMessage;
    let msg = IgtlMessage::new(get_capability, "QueryClient")?;
    client.send(&msg)?;
    println!("    → Sent GET_CAPABIL query");

    // Receive CAPABILITY response
    let response: IgtlMessage<CapabilityMessage> = client.receive()?;
    println!("    ← Received CAPABILITY response");
    println!("      Supported message types ({}):", response.content.types.len());

    for (i, msg_type) in response.content.types.iter().enumerate() {
        println!("        {}. {}", i + 1, msg_type);
    }
    println!();

    Ok(())
}

/// Start tracking data streaming using STT_TDATA message
fn start_tracking_stream(client: &mut SyncIgtlClient) -> Result<()> {
    // Create STT_TDATA message
    // - resolution: 50ms (20 Hz update rate)
    // - coordinate_name: "RAS" (Right-Anterior-Superior anatomical coordinate system)
    let start_stream = StartTDataMessage {
        resolution: 50,                      // 50ms = 20 Hz
        coordinate_name: "RAS".to_string(),  // Anatomical coordinate system
    };

    let msg = IgtlMessage::new(start_stream, "QueryClient")?;
    client.send(&msg)?;
    println!("    → Sent STT_TDATA (resolution: 50ms, coordinate: RAS)");

    // Receive RTS_TDATA acknowledgment
    let response: IgtlMessage<RtsTDataMessage> = client.receive()?;
    println!("    ← Received RTS_TDATA acknowledgment");

    match response.content.status {
        1 => println!("      ✓ Server ready to send tracking data (status: OK)"),
        0 => {
            println!("      ✗ Server error (status: ERROR)");
            return Err(IgtlError::InvalidHeader(
                "Server rejected streaming request".into(),
            ));
        }
        code => {
            println!("      ? Unknown status code: {}", code);
        }
    }
    println!();

    Ok(())
}

/// Receive tracking data stream (TDATA messages)
fn receive_tracking_data(client: &mut SyncIgtlClient) -> Result<()> {
    const MAX_MESSAGES: usize = 10;

    println!("    Receiving {} TDATA messages...", MAX_MESSAGES);
    println!("    (Press Ctrl+C to stop early)\n");

    for i in 1..=MAX_MESSAGES {
        // Receive TDATA message
        let tdata: IgtlMessage<TDataMessage> = client.receive()?;

        println!("    [{}/{}] TDATA received:", i, MAX_MESSAGES);
        println!("            Device: {}", tdata.header.device_name.as_str()?);
        println!(
            "            Tracking tools: {}",
            tdata.content.elements.len()
        );

        // Display each tracking element
        for (j, element) in tdata.content.elements.iter().enumerate() {
            // Extract translation from matrix (last column: [m03, m13, m23])
            let tx = element.matrix[0][3];
            let ty = element.matrix[1][3];
            let tz = element.matrix[2][3];

            println!(
                "              {}. '{}': position ({:.2}, {:.2}, {:.2}) mm",
                j + 1,
                element.name,
                tx,
                ty,
                tz
            );
        }
        println!();

        // Simulate processing time
        std::thread::sleep(Duration::from_millis(50));
    }

    println!("    ✓ Successfully received {} tracking data messages\n", MAX_MESSAGES);

    Ok(())
}

/// Stop tracking data streaming using STP_TDATA message
fn stop_tracking_stream(client: &mut SyncIgtlClient) -> Result<()> {
    // Send STP_TDATA message
    let stop_stream = StopTDataMessage;
    let msg = IgtlMessage::new(stop_stream, "QueryClient")?;
    client.send(&msg)?;
    println!("    → Sent STP_TDATA");
    println!("    ✓ Streaming stopped\n");

    Ok(())
}
