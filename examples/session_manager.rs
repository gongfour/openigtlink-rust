//! SessionManager demonstration
//!
//! Demonstrates multi-client session management with broadcasting and
//! per-client messaging capabilities.
//!
//! # Usage
//!
//! Terminal 1 - Start server:
//! ```bash
//! cargo run --example session_manager_demo
//! ```
//!
//! Terminal 2, 3, 4 - Connect multiple clients:
//! ```bash
//! cargo run --example client
//! ```

use openigtlink_rust::io::{ClientId, MessageHandler, SessionManager};
use openigtlink_rust::protocol::types::StatusMessage;
use std::sync::Arc;
use tokio::time::{interval, sleep, Duration};

/// Custom message handler that logs all received messages
struct LoggingHandler;

impl MessageHandler for LoggingHandler {
    fn handle_message(&self, client_id: ClientId, type_name: &str, _data: &[u8]) {
        println!("[Handler] Client #{} sent {} message", client_id, type_name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OpenIGTLink SessionManager Demo ===\n");

    // Create session manager
    let manager = Arc::new(SessionManager::new("127.0.0.1:18944").await?);
    println!(
        "[INFO] Server listening on {}",
        manager.local_addr().unwrap()
    );

    // Register message handler
    manager.add_handler(Box::new(LoggingHandler)).await;
    println!("[INFO] Registered message handler\n");

    // Spawn client acceptor task
    let mgr_acceptor = manager.clone();
    tokio::spawn(async move {
        mgr_acceptor.accept_clients().await;
    });

    // Spawn status broadcaster (every 5 seconds)
    let mgr_broadcaster = manager.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;

            let client_count = mgr_broadcaster.client_count().await;
            if client_count > 0 {
                let status =
                    StatusMessage::ok(&format!("Heartbeat: {} clients connected", client_count));

                if let Err(e) = mgr_broadcaster.broadcast(&status).await {
                    eprintln!("[ERROR] Broadcast failed: {}", e);
                } else {
                    println!("[Broadcast] Sent heartbeat to {} clients", client_count);
                }
            }
        }
    });

    // Spawn statistics reporter
    let mgr_stats = manager.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;

            let count = mgr_stats.client_count().await;
            let ids = mgr_stats.client_ids().await;

            println!("\n[Stats] Active clients: {}", count);
            for client_id in ids {
                if let Some(info) = mgr_stats.client_info(client_id).await {
                    println!(
                        "  - Client #{}: {} (uptime: {:?})",
                        info.id, info.addr, info.uptime
                    );
                }
            }
        }
    });

    // Main loop: wait for a bit, then send individual messages
    sleep(Duration::from_secs(3)).await;

    loop {
        sleep(Duration::from_secs(15)).await;

        // Send personalized message to each client
        let client_ids = manager.client_ids().await;
        for client_id in client_ids {
            let personal_msg =
                StatusMessage::ok(&format!("Personal message for client #{}", client_id));

            if let Err(e) = manager.send_to(client_id, &personal_msg).await {
                eprintln!("[ERROR] Failed to send to client #{}: {}", client_id, e);
            } else {
                println!("[Personal] Sent message to client #{}", client_id);
            }
        }
    }
}
