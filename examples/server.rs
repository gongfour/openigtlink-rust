//! OpenIGTLink Sync Server Example
//!
//! This example demonstrates how to create a simple synchronous OpenIGTLink server
//! that accepts client connections and handles different message types dynamically (AnyMessage).
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
use openigtlink_rust::protocol::types::{CapabilityMessage, StatusMessage};
use openigtlink_rust::protocol::AnyMessage;
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

    println!("=== OpenIGTLink Sync Server (AnyMessage) ===\n");

    // Bind server to address
    let server = IgtlServer::bind(&addr)?;
    println!("[INFO] Server listening on {}", addr);
    println!("[INFO] Press Ctrl+C to stop\n");

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
    println!("[CONNECT] Client connected: {}", peer);

    // Handle messages in a loop until client disconnects or sends CAPABILITY
    let mut message_count = 0;
    loop {
        // Receive any message type dynamically
        match conn.receive_any() {
            Ok(msg) => {
                message_count += 1;
                let msg_type = msg.message_type();
                let device_name = msg.device_name().unwrap_or("Unknown");

                println!("\n[RECV #{}] {} from '{}'", message_count, msg_type, device_name);

                // Handle different message types
                match msg {
                    AnyMessage::Transform(transform_msg) => {
                        println!("       Matrix (first row): [{:.2}, {:.2}, {:.2}, {:.2}]",
                            transform_msg.content.matrix[0][0],
                            transform_msg.content.matrix[0][1],
                            transform_msg.content.matrix[0][2],
                            transform_msg.content.matrix[0][3]
                        );

                        // Respond with STATUS(OK)
                        let status = StatusMessage::ok("Transform received");
                        let response = IgtlMessage::new(status, "Server")?;
                        conn.send(&response)?;
                        println!("[SEND] STATUS (OK) response");
                    }

                    AnyMessage::Status(status_msg) => {
                        println!("       Code: {}, Message: '{}'",
                            status_msg.content.code,
                            status_msg.content.status_string
                        );

                        // Respond with CAPABILITY
                        let capability = CapabilityMessage::new(vec![
                            "TRANSFORM".to_string(),
                            "STATUS".to_string(),
                            "CAPABILITY".to_string(),
                            "IMAGE".to_string(),
                            "POSITION".to_string(),
                        ]);
                        let response = IgtlMessage::new(capability, "Server")?;
                        conn.send(&response)?;
                        println!("[SEND] CAPABILITY response");
                    }

                    AnyMessage::Capability(cap_msg) => {
                        println!("       Client capabilities ({} types):", cap_msg.content.types.len());
                        for (i, typ) in cap_msg.content.types.iter().enumerate() {
                            println!("         {}. {}", i + 1, typ);
                        }
                        println!("\n[INFO] Session completed, closing connection");
                        break;
                    }

                    AnyMessage::Image(img_msg) => {
                        println!("       Image: {}x{}x{}, {} bytes",
                            img_msg.content.size[0],
                            img_msg.content.size[1],
                            img_msg.content.size[2],
                            img_msg.content.data.len()
                        );

                        let status = StatusMessage::ok("Image received");
                        let response = IgtlMessage::new(status, "Server")?;
                        conn.send(&response)?;
                        println!("[SEND] STATUS (OK) response");
                    }

                    AnyMessage::Position(pos_msg) => {
                        println!("       Position: ({:.2}, {:.2}, {:.2})",
                            pos_msg.content.position[0],
                            pos_msg.content.position[1],
                            pos_msg.content.position[2]
                        );

                        let status = StatusMessage::ok("Position received");
                        let response = IgtlMessage::new(status, "Server")?;
                        conn.send(&response)?;
                        println!("[SEND] STATUS (OK) response");
                    }

                    _ => {
                        println!("       (No specific handler, echoing acknowledgment)");
                        let status = StatusMessage::ok(&format!("{} received", msg_type));
                        let response = IgtlMessage::new(status, "Server")?;
                        conn.send(&response)?;
                        println!("[SEND] STATUS (OK) response");
                    }
                }
            }
            Err(e) => {
                eprintln!("\n[WARN] Connection closed or error: {}", e);
                break;
            }
        }
    }

    println!("[DISCONNECT] Client disconnected: {}\n", peer);
    Ok(())
}
