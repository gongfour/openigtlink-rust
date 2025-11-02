use crate::cli::ServerArgs;
use openigtlink_rust::error::Result;
use openigtlink_rust::io::IgtlServer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, error};

/// Run OpenIGTLink server
pub async fn run_server(args: ServerArgs) -> Result<()> {
    // Create and bind server
    let server = IgtlServer::bind(&args.listen)?;

    info!("✓ Server listening on {}", args.listen);
    println!("✓ Server listening on {}", args.listen);

    // Setup graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        running_clone.store(false, Ordering::SeqCst);
        println!("\n✓ Shutting down gracefully...");
    });

    // Accept connections loop
    loop {
        // Check if we should shutdown
        if !running.load(Ordering::SeqCst) {
            break;
        }

        match server.accept() {
            Ok(_conn) => {
                info!("✓ Client connected");
                println!("✓ Client connected");

                // Basic Phase 1: Just accept and keep connection
                // Actual send/receive functionality will be in Phase 2-3
            }
            Err(e) => {
                error!("✗ Error accepting connection: {}", e);
                eprintln!("✗ Error accepting connection: {}", e);
            }
        }
    }

    info!("✓ Server stopped");
    println!("✓ Server stopped");
    Ok(())
}
