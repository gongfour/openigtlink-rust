use crate::cli::ClientArgs;
use openigtlink_rust::error::Result;
use openigtlink_rust::io::ClientBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Run OpenIGTLink client
pub async fn run_client(args: ClientArgs) -> Result<()> {
    // Parse address
    let _client = ClientBuilder::new()
        .tcp(&args.connect)
        .async_mode()
        .build()
        .await?;

    info!("✓ Connected to {}", args.connect);
    println!("✓ Connected to {}", args.connect);

    // Setup graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        running_clone.store(false, Ordering::SeqCst);
        println!("\n✓ Disconnecting...");
    });

    // Keep connection alive
    // Basic Phase 1: Just maintain connection
    // Actual send/receive functionality will be in Phase 2-3
    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!("✓ Client disconnected");
    println!("✓ Client disconnected");
    Ok(())
}
