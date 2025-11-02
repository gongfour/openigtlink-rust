use crate::cli::ClientArgs;
use crate::msg_loader;
use openigtlink_rust::error::Result;
use openigtlink_rust::io::unified_async_client::UnifiedAsyncClient;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};

/// Run OpenIGTLink client
pub async fn run_client(args: ClientArgs) -> Result<()> {
    // Connect to server
    let mut client = UnifiedAsyncClient::connect(&args.connect).await?;

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

    // Load send message if enabled (TRANSFORM only in Phase 2)
    if args.send_enable {
        if let Some(ref file_path) = args.send_message_file {
            match msg_loader::load_transform_from_file(file_path) {
                Ok(msg) => {
                    info!("✓ Loaded TRANSFORM message from: {}", file_path);
                    println!("✓ Loaded TRANSFORM message from: {}", file_path);

                    // Send messages
                    for i in 1..=args.send_repeat_count {
                        match client.send(&msg).await {
                            Ok(_) => {
                                info!("✓ Message sent ({}/{})", i, args.send_repeat_count);
                                if i % 10 == 0 || i == args.send_repeat_count {
                                    println!("✓ Message sent ({}/{})", i, args.send_repeat_count);
                                }
                            }
                            Err(e) => {
                                error!("✗ Failed to send message: {}", e);
                                eprintln!("✗ Failed to send message: {}", e);
                                break;
                            }
                        }

                        if i < args.send_repeat_count && running.load(Ordering::SeqCst) {
                            tokio::time::sleep(Duration::from_millis(args.send_interval_ms)).await;
                        }

                        if !running.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("✗ Failed to load message: {}", e);
                    eprintln!("✗ Failed to load message: {}", e);
                }
            }
        } else {
            warn!("--send-enable specified but no --send-message-file provided");
            eprintln!("⚠ --send-enable specified but no --send-message-file provided");
        }
    }

    // RECEIVE functionality reserved for Phase 3+
    if args.receive_enable {
        warn!("⚠ RECEIVE functionality not yet implemented for client");
        eprintln!("⚠ RECEIVE functionality not yet implemented for client");
    }

    // Keep connection alive while not interrupted
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
