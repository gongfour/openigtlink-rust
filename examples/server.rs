//! OpenIGTLink Server Example
//!
//! This example demonstrates how to create a simple OpenIGTLink server
//! that accepts client connections and handles different message types.
//!
//! # Usage
//!
//! ```bash
//! # Start server on default port 18944
//! cargo run --example server
//!
//! # Start server on custom port
//! cargo run --example server 12345
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{IgtlConnection, IgtlServer};
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
    // Parse port from command line arguments (default: 18944)
    let port = env::args().nth(1).unwrap_or_else(|| "18944".to_string());

    let addr = format!("127.0.0.1:{}", port);

    // Bind server to address
    let server = IgtlServer::bind(&addr)?;
    println!("Server listening on {}", addr);
    println!("Press Ctrl+C to stop\n");

    // Accept client connections in a loop
    loop {
        match server.accept() {
            Ok(conn) => {
                if let Err(e) = handle_client(conn) {
                    eprintln!("[ERROR] Client handler failed: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_client(mut conn: IgtlConnection) -> Result<()> {
    let peer = conn.peer_addr()?;
    println!("[INFO] Client connected: {}\n", peer);

    // Handle messages in a loop until client disconnects or sends CAPABILITY
    loop {
        // Try to receive TRANSFORM message
        if let Ok(msg) = conn.receive::<TransformMessage>() {
            let device_name = msg.header.device_name.as_str().unwrap_or("Unknown");
            println!("[RECV] TRANSFORM from device '{}'", device_name);
            println!(
                "       Matrix (first row): [{:.2}, {:.2}, {:.2}, {:.2}]",
                msg.content.matrix[0][0],
                msg.content.matrix[0][1],
                msg.content.matrix[0][2],
                msg.content.matrix[0][3]
            );

            // Respond with STATUS(OK)
            let status = StatusMessage::ok("Transform received");
            let response = IgtlMessage::new(status, "Server")?;
            conn.send(&response)?;
            println!("[SEND] STATUS (OK) response\n");
            continue;
        }

        // Try to receive STATUS message
        if let Ok(msg) = conn.receive::<StatusMessage>() {
            let device_name = msg.header.device_name.as_str().unwrap_or("Unknown");
            println!("[RECV] STATUS from device '{}'", device_name);
            println!(
                "       Code: {}, Name: '{}', Message: '{}'",
                msg.content.code, msg.content.error_name, msg.content.status_string
            );

            // Respond with CAPABILITY
            let capability = CapabilityMessage::new(vec![
                "TRANSFORM".to_string(),
                "STATUS".to_string(),
                "CAPABILITY".to_string(),
            ]);
            let response = IgtlMessage::new(capability, "Server")?;
            conn.send(&response)?;
            println!("[SEND] CAPABILITY response\n");
            continue;
        }

        // Try to receive CAPABILITY message
        if let Ok(msg) = conn.receive::<CapabilityMessage>() {
            let device_name = msg.header.device_name.as_str().unwrap_or("Unknown");
            println!("[RECV] CAPABILITY from device '{}'", device_name);
            println!("       Supported types ({}):", msg.content.types.len());
            for (i, typ) in msg.content.types.iter().enumerate() {
                println!("         {}. {}", i + 1, typ);
            }
            println!("\n[INFO] Client session completed, closing connection\n");
            break;
        }

        // If no known message type could be decoded, break
        eprintln!("[WARN] Unknown message type or connection closed");
        break;
    }

    Ok(())
}
