//! Async client-server communication example
//!
//! Demonstrates the use of asynchronous OpenIGTLink client and server
//! for non-blocking, high-concurrency communication.

use openigtlink_rust::{
    io::{builder::ClientBuilder, AsyncIgtlServer},
    protocol::{message::IgtlMessage, types::StatusMessage},
};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    println!("=== Async OpenIGTLink Communication Demo ===\n");

    // Create server
    let server = AsyncIgtlServer::bind("127.0.0.1:18944").await?;
    let server_addr = server.local_addr()?;
    println!("Server listening on {}\n", server_addr);

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        println!("[Server] Waiting for client connection...");
        let mut conn = server.accept().await.unwrap();
        println!("[Server] Client connected\n");

        // Receive 3 messages from client
        for i in 1..=3 {
            let msg: IgtlMessage<StatusMessage> = conn.receive().await.unwrap();
            println!(
                "[Server] Received message {}: {}",
                i, msg.content.status_string
            );

            // Send acknowledgment
            let ack = StatusMessage::ok(&format!("ACK {}", i));
            let ack_msg = IgtlMessage::new(ack, "Server").unwrap();
            conn.send(&ack_msg).await.unwrap();
            println!("[Server] Sent acknowledgment {}\n", i);

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        println!("[Server] Communication completed");
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Create client task
    let client_handle = tokio::spawn(async move {
        println!("[Client] Connecting to server...");
        let mut client = ClientBuilder::new()
            .tcp(server_addr.to_string())
            .async_mode()
            .build()
            .await
            .unwrap();
        println!("[Client] Connected to server\n");

        // Send 3 messages
        for i in 1..=3 {
            let status = StatusMessage::ok(&format!("Message {}", i));
            let msg = IgtlMessage::new(status, "Client").unwrap();
            client.send(&msg).await.unwrap();
            println!("[Client] Sent message {}", i);

            // Receive acknowledgment
            let ack: IgtlMessage<StatusMessage> = client.receive().await.unwrap();
            println!("[Client] Received: {}\n", ack.content.status_string);

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        println!("[Client] Communication completed");
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(server_handle, client_handle);

    println!("\n=== Demo completed successfully ===");
    Ok(())
}
