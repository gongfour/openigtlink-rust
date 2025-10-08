//! Demonstration of logging capabilities using the tracing crate
//!
//! This example shows how to enable and configure logging for OpenIGTLink operations.
//!
//! # Running with different log levels
//!
//! ```bash
//! # Show all logs (trace, debug, info, warn, error)
//! RUST_LOG=trace cargo run --example logging_demo
//!
//! # Show only info and above
//! RUST_LOG=info cargo run --example logging_demo
//!
//! # Show debug logs only for openigtlink_rust
//! RUST_LOG=openigtlink_rust=debug cargo run --example logging_demo
//!
//! # Show trace logs for io module only
//! RUST_LOG=openigtlink_rust::io=trace cargo run --example logging_demo
//! ```

use openigtlink_rust::{
    io::{builder::ClientBuilder, IgtlServer},
    protocol::{message::IgtlMessage, types::StatusMessage},
};
use std::thread;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with environment filter
    // This allows control via RUST_LOG environment variable
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("=== OpenIGTLink Logging Demo ===");
    info!("This example demonstrates structured logging throughout the library");

    // Spawn server in a separate thread
    let server_thread = thread::spawn(|| {
        info!("Server thread started");

        // Bind server - this will log binding and listening
        let server = IgtlServer::bind("127.0.0.1:18944").expect("Failed to bind server");
        info!("Server ready to accept connections");

        // Accept one client - this will log client connection
        let mut conn = server.accept().expect("Failed to accept connection");

        // Receive a message - this will log header reception, body reception, and decoding
        let msg: IgtlMessage<StatusMessage> =
            conn.receive().expect("Failed to receive message");

        info!("Server received status: {}", msg.content.status_string);

        // Send a response - this will log message sending
        let response = StatusMessage::ok("Message received successfully");
        let response_msg = IgtlMessage::new(response, "Server").expect("Failed to create message");
        conn.send(&response_msg).expect("Failed to send response");

        info!("Server thread completed");
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(100));

    info!("Client starting connection");

    // Connect to server - this will log connection attempt and success
    let mut client = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .sync()
        .build()?;

    info!("Client connected, preparing to send message");

    // Demonstrate CRC setting logging
    client.set_verify_crc(false); // This will log a warning
    client.set_verify_crc(true); // This will log an info message

    // Send a message - this will log message details
    let status = StatusMessage::ok("Hello from client");
    let msg = IgtlMessage::new(status, "Client")?;
    client.send(&msg)?;

    info!("Client sent message, waiting for response");

    // Receive response - this will log reception details
    let response: IgtlMessage<StatusMessage> = client.receive()?;

    info!("Client received response: {}", response.content.status_string);

    // Wait for server thread to complete
    server_thread.join().expect("Server thread panicked");

    info!("=== Demo completed successfully ===");
    info!("Try running with different RUST_LOG values to see different log levels:");
    info!("  RUST_LOG=trace cargo run --example logging_demo    # All logs");
    info!("  RUST_LOG=debug cargo run --example logging_demo    # Debug and above");
    info!("  RUST_LOG=info cargo run --example logging_demo     # Info and above");

    Ok(())
}
