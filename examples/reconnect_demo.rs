//! Automatic reconnection demonstration
//!
//! Shows how ReconnectClient automatically handles connection failures
//! and reconnects with exponential backoff.

use openigtlink_rust::{
    io::{AsyncIgtlServer, ReconnectClient, ReconnectConfig},
    protocol::{message::IgtlMessage, types::StatusMessage},
};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    println!("=== Automatic Reconnection Demo ===\n");

    // Create notify for coordination
    let server_ready = Arc::new(Notify::new());
    let server_ready_clone = server_ready.clone();

    let server_shutdown = Arc::new(Notify::new());
    let server_shutdown_clone = server_shutdown.clone();

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        for round in 1..=2 {
            println!("[Server] Round {}: Starting server...", round);

            let server = AsyncIgtlServer::bind("127.0.0.1:18946").await.unwrap();
            println!("[Server] Round {}: Server listening\n", round);

            // Notify client that server is ready
            server_ready_clone.notify_one();

            // Accept one connection and exchange one message
            let mut conn = server.accept().await.unwrap();
            println!("[Server] Round {}: Client connected", round);

            let msg: IgtlMessage<StatusMessage> = conn.receive().await.unwrap();
            println!(
                "[Server] Round {}: Received: {}",
                round, msg.content.status_string
            );

            let response = StatusMessage::ok(&format!("Response from round {}", round));
            let response_msg = IgtlMessage::new(response, "Server").unwrap();
            conn.send(&response_msg).await.unwrap();
            println!("[Server] Round {}: Sent response\n", round);

            // Drop connection to simulate server restart
            drop(conn);
            drop(server);

            if round < 2 {
                println!("[Server] Round {}: Shutting down (simulating failure)...\n", round);
                tokio::time::sleep(Duration::from_millis(500)).await;
            } else {
                println!("[Server] Round {}: Completed\n", round);
                server_shutdown_clone.notify_one();
                break;
            }
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create reconnecting client
    let config = ReconnectConfig {
        max_attempts: Some(20),
        initial_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 1.5,
        use_jitter: true,
    };

    println!("[Client] Creating reconnecting client...");
    let mut client = ReconnectClient::connect("127.0.0.1:18946", config).await?;
    println!("[Client] Initial connection established\n");

    // Send messages through multiple server restarts
    for i in 1..=2 {
        println!("[Client] Sending message {}...", i);

        let status = StatusMessage::ok(&format!("Message {}", i));
        let msg = IgtlMessage::new(status, "ReconnectClient")?;

        // This will automatically reconnect if needed
        client.send(&msg).await?;
        println!("[Client] Message {} sent", i);

        let response: IgtlMessage<StatusMessage> = client.receive().await?;
        println!("[Client] Received: {}", response.content.status_string);
        println!(
            "[Client] Total reconnections so far: {}\n",
            client.reconnect_count()
        );

        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    println!("=== Demo completed successfully ===");
    println!("\nReconnection statistics:");
    println!("  Total messages sent: 2");
    println!("  Total reconnections: {}", client.reconnect_count());
    println!("  Connection success rate: 100%");
    println!("\n✓ Client automatically recovered from server failure!");

    tokio::time::timeout(Duration::from_secs(2), server_shutdown.notified())
        .await
        .ok();
    let _ = tokio::time::timeout(Duration::from_secs(1), server_handle).await;

    Ok(())
}
