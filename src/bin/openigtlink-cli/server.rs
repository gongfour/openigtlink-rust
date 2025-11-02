use crate::cli::ServerArgs;
use crate::msg_loader;
use crate::msg_saver;
use openigtlink_rust::error::Result;
use openigtlink_rust::io::AsyncIgtlServer;
use openigtlink_rust::protocol::AnyMessage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};

/// Run OpenIGTLink server
pub async fn run_server(args: ServerArgs) -> Result<()> {
    // Create and bind server (async)
    let server = AsyncIgtlServer::bind(&args.listen).await?;

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

    // Load send message if enabled (TRANSFORM only)
    let send_msg = if args.send_enable {
        if let Some(ref file_path) = args.send_message_file {
            match msg_loader::load_message_from_file(file_path) {
                Ok(msg) => {
                    // For now, only TRANSFORM is supported for sending via server
                    if msg.message_type() == "TRANSFORM" {
                        info!("✓ Loaded TRANSFORM message from: {}", file_path);
                        println!("✓ Loaded TRANSFORM message from: {}", file_path);
                        Some(msg)
                    } else {
                        error!("✗ Server SEND only supports TRANSFORM messages");
                        eprintln!("✗ Server SEND only supports TRANSFORM messages");
                        None
                    }
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

        match server.accept().await {
            Ok(mut conn) => {
                info!("✓ Client connected");
                println!("✓ Client connected");

                // Send message if enabled
                if let Some(ref msg) = send_msg {
                    // Extract TRANSFORM message if available
                    if let AnyMessage::Transform(transform_msg) = msg {
                        for i in 1..=args.send_repeat_count {
                            match conn.send(transform_msg).await {
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

                // Receive messages if enabled
                if args.receive_enable {
                    let timeout = Duration::from_secs(args.receive_timeout_sec);
                    let max_count = if args.receive_max_count == 0 {
                        u32::MAX
                    } else {
                        args.receive_max_count
                    };
                    let mut received_messages: Vec<AnyMessage> = Vec::new();

                    info!("✓ Waiting to receive messages (timeout: {}s, max: {})",
                        args.receive_timeout_sec,
                        if args.receive_max_count == 0 { "unlimited".to_string() } else { args.receive_max_count.to_string() });
                    println!("✓ Waiting to receive messages (timeout: {}s, max: {})",
                        args.receive_timeout_sec,
                        if args.receive_max_count == 0 { "unlimited".to_string() } else { args.receive_max_count.to_string() });

                    let start_time = std::time::Instant::now();
                    loop {
                        // Check if we should stop based on max_count
                        if received_messages.len() >= max_count as usize {
                            break;
                        }

                        // Check if timeout exceeded
                        if start_time.elapsed() > timeout {
                            info!("✓ Receive timeout reached");
                            println!("✓ Receive timeout reached");
                            break;
                        }

                        // Try to receive a message with a short timeout to allow graceful shutdown
                        match tokio::time::timeout(
                            Duration::from_secs(1),
                            conn.receive_any()
                        ).await {
                            Ok(Ok(msg)) => {
                                // Check if message type matches filter
                                if msg_saver::matches_filter(msg.message_type(), &args.receive_message_types) {
                                    received_messages.push(msg);
                                    let count = received_messages.len();
                                    info!("✓ Received message ({}/{})", count, max_count);
                                    if count % 10 == 0 || count == max_count as usize {
                                        println!("✓ Received message ({}/{})", count, max_count);
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                info!("✓ Receive completed or connection closed: {}", e);
                                break;
                            }
                            Err(_) => {
                                // Timeout on individual receive, check if we should continue
                                if !running.load(Ordering::SeqCst) {
                                    break;
                                }
                                continue;
                            }
                        }
                    }

                    // Save received messages if output file specified
                    if let Some(ref output_file) = args.receive_output_file {
                        if !received_messages.is_empty() {
                            if let Err(e) = msg_saver::save_messages(&received_messages, output_file) {
                                error!("✗ Failed to save received messages: {}", e);
                                eprintln!("✗ Failed to save received messages: {}", e);
                            }
                        }
                    }

                    info!("✓ Received {} message(s)", received_messages.len());
                    println!("✓ Received {} message(s)", received_messages.len());
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
