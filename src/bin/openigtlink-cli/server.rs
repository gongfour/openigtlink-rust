use crate::cli::ServerArgs;
use crate::msg_loader;
use openigtlink_rust::error::Result;
use openigtlink_rust::io::IgtlServer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};

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

    // Load send message if enabled (TRANSFORM only in Phase 2)
    let send_msg = if args.send_enable {
        if let Some(ref file_path) = args.send_message_file {
            match msg_loader::load_transform_from_file(file_path) {
                Ok(msg) => {
                    info!("✓ Loaded TRANSFORM message from: {}", file_path);
                    println!("✓ Loaded TRANSFORM message from: {}", file_path);
                    Some(msg)
                }
                Err(e) => {
                    error!("✗ Failed to load message: {}", e);
                    eprintln!("✗ Failed to load message: {}", e);
                    None
                }
            }
        } else {
            warn!("--send-enable specified but no --send-message-file provided");
            eprintln!("⚠ --send-enable specified but no --send-message-file provided");
            None
        }
    } else {
        None
    };

    // Accept connections loop
    loop {
        // Check if we should shutdown
        if !running.load(Ordering::SeqCst) {
            break;
        }

        match server.accept() {
            Ok(mut conn) => {
                info!("✓ Client connected");
                println!("✓ Client connected");

                // Send message if enabled
                if let Some(ref msg) = send_msg {
                    for i in 1..=args.send_repeat_count {
                        match conn.send(msg) {
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

                        if i < args.send_repeat_count {
                            tokio::time::sleep(Duration::from_millis(args.send_interval_ms)).await;
                        }
                    }
                }
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
