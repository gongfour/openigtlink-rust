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

    // Load send message if enabled (all message types supported)
    let send_msg = if args.send_enable {
        if let Some(ref file_path) = args.send_message_file {
            match msg_loader::load_message_from_file(file_path) {
                Ok(msg) => {
                    info!("✓ Loaded {} message from: {}", msg.message_type(), file_path);
                    println!("✓ Loaded {} message from: {}", msg.message_type(), file_path);
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

        match server.accept().await {
            Ok(mut conn) => {
                info!("✓ Client connected");
                println!("✓ Client connected");

                // Send message if enabled (all message types supported)
                if let Some(ref msg) = send_msg {
                    for i in 1..=args.send_repeat_count {
                        let send_result = match msg {
                            // Core message types
                            AnyMessage::Transform(transform_msg) => conn.send(transform_msg).await,
                            AnyMessage::Status(status_msg) => conn.send(status_msg).await,
                            AnyMessage::Capability(capability_msg) => conn.send(capability_msg).await,
                            AnyMessage::String(string_msg) => conn.send(string_msg).await,
                            AnyMessage::Position(position_msg) => conn.send(position_msg).await,
                            AnyMessage::Sensor(sensor_msg) => conn.send(sensor_msg).await,
                            // Complex message types
                            AnyMessage::Image(image_msg) => conn.send(image_msg).await,
                            AnyMessage::QtData(qtdata_msg) => conn.send(qtdata_msg).await,
                            AnyMessage::TData(tdata_msg) => conn.send(tdata_msg).await,
                            AnyMessage::Point(point_msg) => conn.send(point_msg).await,
                            AnyMessage::Trajectory(traj_msg) => conn.send(traj_msg).await,
                            AnyMessage::NdArray(ndarray_msg) => conn.send(ndarray_msg).await,
                            AnyMessage::Bind(bind_msg) => conn.send(bind_msg).await,
                            AnyMessage::ColorTable(ct_msg) => conn.send(ct_msg).await,
                            AnyMessage::ImgMeta(imgmeta_msg) => conn.send(imgmeta_msg).await,
                            AnyMessage::LbMeta(lbmeta_msg) => conn.send(lbmeta_msg).await,
                            AnyMessage::PolyData(polydata_msg) => conn.send(polydata_msg).await,
                            AnyMessage::Video(video_msg) => conn.send(video_msg).await,
                            AnyMessage::VideoMeta(videometa_msg) => conn.send(videometa_msg).await,
                            AnyMessage::Command(cmd_msg) => conn.send(cmd_msg).await,
                            // Query messages
                            AnyMessage::GetTransform(get_msg) => conn.send(get_msg).await,
                            AnyMessage::GetStatus(get_msg) => conn.send(get_msg).await,
                            AnyMessage::GetCapability(get_msg) => conn.send(get_msg).await,
                            AnyMessage::GetImage(get_msg) => conn.send(get_msg).await,
                            AnyMessage::GetImgMeta(get_msg) => conn.send(get_msg).await,
                            AnyMessage::GetLbMeta(get_msg) => conn.send(get_msg).await,
                            AnyMessage::GetPoint(get_msg) => conn.send(get_msg).await,
                            AnyMessage::GetTData(get_msg) => conn.send(get_msg).await,
                            // Response messages
                            AnyMessage::RtsTransform(rts_msg) => conn.send(rts_msg).await,
                            AnyMessage::RtsStatus(rts_msg) => conn.send(rts_msg).await,
                            AnyMessage::RtsCapability(rts_msg) => conn.send(rts_msg).await,
                            AnyMessage::RtsImage(rts_msg) => conn.send(rts_msg).await,
                            AnyMessage::RtsTData(rts_msg) => conn.send(rts_msg).await,
                            // Streaming control messages
                            AnyMessage::StartTData(stt_msg) => conn.send(stt_msg).await,
                            AnyMessage::StopTransform(stp_msg) => conn.send(stp_msg).await,
                            AnyMessage::StopPosition(stp_msg) => conn.send(stp_msg).await,
                            AnyMessage::StopQtData(stp_msg) => conn.send(stp_msg).await,
                            AnyMessage::StopTData(stp_msg) => conn.send(stp_msg).await,
                            AnyMessage::StopImage(stp_msg) => conn.send(stp_msg).await,
                            AnyMessage::StopNdArray(stp_msg) => conn.send(stp_msg).await,
                            // Unknown message type
                            AnyMessage::Unknown { .. } => {
                                error!("✗ Cannot send unknown message type");
                                eprintln!("✗ Cannot send unknown message type");
                                break;
                            }
                        };

                        match send_result {
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
