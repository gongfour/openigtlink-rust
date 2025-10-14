//! OpenIGTLink Async Server Example
//!
//! This example demonstrates how to create an asynchronous OpenIGTLink server
//! using Tokio that can handle multiple concurrent client connections efficiently.
//! Uses AnyMessage for dynamic message handling.
//!
//! # Usage
//!
//! ```bash
//! # Start the async server
//! cargo run --example async_server
//!
//! # In separate terminals, connect multiple clients:
//! cargo run --example async_client
//! ```
//!
//! # Features
//!
//! - Tokio async runtime for high concurrency
//! - Separate task per client connection
//! - Dynamic message handling (AnyMessage)
//! - Graceful client disconnection handling
//! - Real-time connection statistics
//! - No blocking operations

use openigtlink_rust::error::Result;
use openigtlink_rust::io::AsyncIgtlServer;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{CapabilityMessage, StatusMessage};
use openigtlink_rust::protocol::AnyMessage;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{interval, Duration};

static CLIENT_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() {
    if let Err(e) = run_server().await {
        eprintln!("[ERROR] Server failed: {}", e);
        std::process::exit(1);
    }
}

async fn run_server() -> Result<()> {
    let addr = "127.0.0.1:18944";
    let server = AsyncIgtlServer::bind(addr).await?;

    println!("=== OpenIGTLink Async Server (AnyMessage) ===\n");
    println!("[INFO] Configuration:");
    println!("  Address: {}", addr);
    println!("  Runtime: Tokio (async/await)");
    println!("  Concurrency: Unlimited clients");
    println!("  I/O Mode: Non-blocking");
    println!("  Message Handling: Dynamic (AnyMessage)\n");

    let active_clients = Arc::new(AtomicUsize::new(0));

    // Spawn statistics reporter
    let stats_clients = active_clients.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            let count = stats_clients.load(Ordering::Relaxed);
            if count > 0 {
                println!("[STATS] Active clients: {}", count);
            }
        }
    });

    println!("[INFO] Server ready, waiting for connections...\n");

    loop {
        let mut conn = server.accept().await?;
        let client_id = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
        let clients = active_clients.clone();

        println!("[CONNECT] Client #{} connected", client_id);
        clients.fetch_add(1, Ordering::Relaxed);

        // Spawn a new task for each client
        tokio::spawn(async move {
            let result = handle_client(&mut conn, client_id).await;
            clients.fetch_sub(1, Ordering::Relaxed);

            match result {
                Ok(_) => println!("[DISCONNECT] Client #{} disconnected gracefully", client_id),
                Err(e) => eprintln!("[ERROR] Client #{} error: {}", client_id, e),
            }
        });
    }
}

/// Handle a single client connection asynchronously
async fn handle_client(
    conn: &mut openigtlink_rust::io::AsyncIgtlConnection,
    client_id: usize,
) -> Result<()> {
    println!("  [#{}] Handler started", client_id);

    let mut message_count = 0;

    loop {
        // Receive any message type dynamically
        match conn.receive_any().await {
            Ok(msg) => {
                message_count += 1;
                let msg_type = msg.message_type();
                let device_name = msg.device_name().unwrap_or("Unknown");

                println!(
                    "  [#{}] Message #{}: {} from '{}'",
                    client_id, message_count, msg_type, device_name
                );

                // Handle different message types
                match msg {
                    AnyMessage::Transform(transform_msg) => {
                        println!(
                            "       Matrix (first row): [{:.2}, {:.2}, {:.2}, {:.2}]",
                            transform_msg.content.matrix[0][0],
                            transform_msg.content.matrix[0][1],
                            transform_msg.content.matrix[0][2],
                            transform_msg.content.matrix[0][3]
                        );

                        // Send acknowledgment
                        let status = StatusMessage::ok(&format!(
                            "Transform #{} processed by client #{}",
                            message_count, client_id
                        ));
                        let response = IgtlMessage::new(status, "AsyncServer")?;
                        conn.send(&response).await?;
                        println!("  [#{}] Sent STATUS acknowledgment", client_id);
                    }

                    AnyMessage::Status(status_msg) => {
                        println!(
                            "       Code: {}, Message: '{}'",
                            status_msg.content.code, status_msg.content.status_string
                        );

                        // Respond with CAPABILITY
                        let capability = CapabilityMessage::new(vec![
                            "TRANSFORM".to_string(),
                            "STATUS".to_string(),
                            "CAPABILITY".to_string(),
                            "IMAGE".to_string(),
                            "POSITION".to_string(),
                        ]);
                        let response = IgtlMessage::new(capability, "AsyncServer")?;
                        conn.send(&response).await?;
                        println!("  [#{}] Sent CAPABILITY response", client_id);
                    }

                    AnyMessage::Capability(cap_msg) => {
                        println!(
                            "       Client capabilities ({} types):",
                            cap_msg.content.types.len()
                        );
                        for (i, typ) in cap_msg.content.types.iter().enumerate() {
                            println!("         {}. {}", i + 1, typ);
                        }
                        println!("  [#{}] Session completed", client_id);
                        break;
                    }

                    AnyMessage::Image(img_msg) => {
                        println!(
                            "       Image: {}x{}x{}, {} bytes",
                            img_msg.content.size[0],
                            img_msg.content.size[1],
                            img_msg.content.size[2],
                            img_msg.content.data.len()
                        );

                        let status = StatusMessage::ok("Image received");
                        let response = IgtlMessage::new(status, "AsyncServer")?;
                        conn.send(&response).await?;
                        println!("  [#{}] Sent STATUS acknowledgment", client_id);
                    }

                    AnyMessage::Position(pos_msg) => {
                        println!(
                            "       Position: ({:.2}, {:.2}, {:.2})",
                            pos_msg.content.position[0],
                            pos_msg.content.position[1],
                            pos_msg.content.position[2]
                        );

                        let status = StatusMessage::ok("Position received");
                        let response = IgtlMessage::new(status, "AsyncServer")?;
                        conn.send(&response).await?;
                        println!("  [#{}] Sent STATUS acknowledgment", client_id);
                    }

                    _ => {
                        println!("       (Generic handler)");
                        let status = StatusMessage::ok(&format!("{} received", msg_type));
                        let response = IgtlMessage::new(status, "AsyncServer")?;
                        conn.send(&response).await?;
                        println!("  [#{}] Sent STATUS acknowledgment", client_id);
                    }
                }
            }
            Err(e) => {
                eprintln!("  [#{}] Connection closed or error: {}", client_id, e);
                break;
            }
        }
    }

    println!(
        "  [#{}] Processed {} messages total",
        client_id, message_count
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_bind() {
        // Test that we can bind to a port
        let result = AsyncIgtlServer::bind("127.0.0.1:0").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_client_counter() {
        let count1 = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let count2 = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
        assert!(count2 > count1);
    }
}
