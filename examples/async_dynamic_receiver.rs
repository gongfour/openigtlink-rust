//! Async dynamic message receiver example
//!
//! This example demonstrates how to receive OpenIGTLink messages asynchronously
//! without knowing the message type in advance.
//!
//! # Usage
//!
//! 1. Start an async server that sends various message types:
//!    ```bash
//!    cargo run --example async_server
//!    ```
//!
//! 2. Run this example:
//!    ```bash
//!    cargo run --example async_dynamic_receiver
//!    ```
//!
//! # Features
//!
//! - Asynchronous message receiving with Tokio
//! - Dynamic message type detection at runtime
//! - Pattern matching for type-specific handling
//! - Graceful shutdown with Ctrl+C

use openigtlink_rust::io::builder::ClientBuilder;
use openigtlink_rust::protocol::AnyMessage;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Async Dynamic Message Receiver Example");
    println!("=======================================\n");

    // Connect to server
    println!("Connecting to server at 127.0.0.1:18944...");
    let mut client = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .async_mode()
        .build()
        .await?;

    println!("Connected! Waiting for messages...");
    println!("Press Ctrl+C to exit.\n");

    // Spawn Ctrl+C handler
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    // Receive messages in a loop
    let mut message_count = 0;
    loop {
        tokio::select! {
            // Handle Ctrl+C
            _ = &mut ctrl_c => {
                println!("\n\nReceived Ctrl+C, shutting down...");
                break;
            }

            // Receive message
            result = client.receive_any() => {
                match result {
                    Ok(msg) => {
                        message_count += 1;
                        handle_message(&msg, message_count).await?;
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
        }
    }

    println!("\nReceived {} messages total.", message_count);
    Ok(())
}

/// Handle a received message
async fn handle_message(
    msg: &AnyMessage,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "📨 Message #{}: {} from '{}'",
        count,
        msg.message_type(),
        msg.device_name()?
    );

    // Pattern match for specific message types
    match msg {
        AnyMessage::Transform(transform_msg) => {
            // Extract position from transform matrix
            let m = &transform_msg.content.matrix;
            let position = (m[0][3], m[1][3], m[2][3]);
            println!(
                "   📍 Position: ({:.2}, {:.2}, {:.2})",
                position.0, position.1, position.2
            );
        }

        AnyMessage::Status(status_msg) => {
            let icon = match status_msg.content.code {
                1 => "✅",
                2 => "⚠️",
                _ => "❌",
            };
            println!(
                "   {} Status: {} - '{}'",
                icon, status_msg.content.error_name, status_msg.content.status_string
            );
        }

        AnyMessage::Image(image_msg) => {
            println!(
                "   🖼️  Image: {}x{}x{}, {} bytes",
                image_msg.content.size[0],
                image_msg.content.size[1],
                image_msg.content.size[2],
                image_msg.content.data.len()
            );
        }

        AnyMessage::Position(position_msg) => {
            println!(
                "   📍 Position: ({:.2}, {:.2}, {:.2})",
                position_msg.content.position[0],
                position_msg.content.position[1],
                position_msg.content.position[2]
            );
        }

        AnyMessage::String(string_msg) => {
            println!("   💬 String: '{}'", string_msg.content.string);
        }

        AnyMessage::Capability(capability_msg) => {
            println!(
                "   🔧 Capabilities: {} types",
                capability_msg.content.types.len()
            );
            for type_name in &capability_msg.content.types {
                println!("      • {}", type_name);
            }
        }

        AnyMessage::Sensor(sensor_msg) => {
            println!("   📊 Sensor: {} values", sensor_msg.content.data.len());
            if !sensor_msg.content.data.is_empty() {
                let avg: f64 =
                    sensor_msg.content.data.iter().sum::<f64>() / sensor_msg.content.data.len() as f64;
                let min = sensor_msg
                    .content
                    .data
                    .iter()
                    .fold(f64::INFINITY, |a, &b| a.min(b));
                let max = sensor_msg
                    .content
                    .data
                    .iter()
                    .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                println!("      Min: {:.3}, Max: {:.3}, Avg: {:.3}", min, max, avg);
            }
        }

        AnyMessage::Point(point_msg) => {
            println!("   📌 Points: {} entries", point_msg.content.points.len());
            for point in &point_msg.content.points {
                println!(
                    "      • {} at ({:.2}, {:.2}, {:.2})",
                    point.name, point.position[0], point.position[1], point.position[2]
                );
            }
        }

        AnyMessage::QtData(qtdata_msg) => {
            println!(
                "   🎯 Tracking: {} instruments",
                qtdata_msg.content.elements.len()
            );
            for elem in &qtdata_msg.content.elements {
                println!("      • {}: type {}", elem.name, elem.instrument_type as u8);
            }
        }

        AnyMessage::TData(tdata_msg) => {
            println!(
                "   🎯 Tracking data: {} elements",
                tdata_msg.content.elements.len()
            );
        }

        AnyMessage::Command(command_msg) => {
            println!(
                "   🎮 Command: {} (ID: {})",
                command_msg.content.command_name, command_msg.content.command_id
            );
            println!("      Content: '{}'", command_msg.content.command);
        }

        // Query messages
        AnyMessage::GetTransform(_) => println!("   ❓ Query: GET_TRANSFORM"),
        AnyMessage::GetStatus(_) => println!("   ❓ Query: GET_STATUS"),
        AnyMessage::GetCapability(_) => println!("   ❓ Query: GET_CAPABILITY"),
        AnyMessage::GetImage(_) => println!("   ❓ Query: GET_IMAGE"),

        // Response messages
        AnyMessage::RtsTransform(_) => println!("   ✅ Response: RTS_TRANSFORM"),
        AnyMessage::RtsStatus(_) => println!("   ✅ Response: RTS_STATUS"),
        AnyMessage::RtsCapability(_) => println!("   ✅ Response: RTS_CAPABILITY"),
        AnyMessage::RtsImage(_) => println!("   ✅ Response: RTS_IMAGE"),

        AnyMessage::Unknown { header, body } => {
            println!("   ⚠️  Unknown message type!");
            println!(
                "      Type: {}, Body: {} bytes",
                header.type_name.as_str().unwrap_or("?"),
                body.len()
            );
        }

        _ => {
            println!("   (No specific handler)");
        }
    }

    println!();
    Ok(())
}
