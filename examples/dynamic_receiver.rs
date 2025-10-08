//! Dynamic message receiver example
//!
//! This example demonstrates how to receive OpenIGTLink messages without knowing
//! the message type in advance. This is similar to the C++ OpenIGTLink library's
//! MessageFactory pattern.
//!
//! # Usage
//!
//! 1. Start a server that sends various message types:
//!    ```bash
//!    cargo run --example server
//!    ```
//!
//! 2. Run this example:
//!    ```bash
//!    cargo run --example dynamic_receiver
//!    ```
//!
//! # How it works
//!
//! - The client uses `receive_any()` instead of `receive::<T>()`
//! - The message type is determined at runtime from the header
//! - Messages are returned as an `AnyMessage` enum
//! - You can pattern match to handle different message types
//!
//! This is useful for:
//! - Generic receivers that handle multiple message types
//! - Message logging/monitoring tools
//! - Protocol debugging and inspection
//! - Applications that receive unknown or custom message types

use openigtlink_rust::io::builder::ClientBuilder;
use openigtlink_rust::protocol::AnyMessage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Dynamic Message Receiver Example");
    println!("=================================\n");

    // Connect to server
    println!("Connecting to server at 127.0.0.1:18944...");
    let mut client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;

    println!("Connected! Waiting for messages...\n");

    // Receive messages in a loop
    let mut message_count = 0;
    loop {
        match client.receive_any() {
            Ok(msg) => {
                message_count += 1;
                println!("Message #{}: {}", message_count, format_message(&msg)?);
                println!();

                // You can also access common header information
                println!(
                    "  Header info: device='{}', timestamp={}.{}",
                    msg.device_name()?,
                    msg.header().timestamp.seconds,
                    msg.header().timestamp.fraction
                );
                println!();

                // Handle specific message types
                match msg {
                    AnyMessage::Transform(transform_msg) => {
                        println!("  Transform matrix:");
                        let m = &transform_msg.content.matrix;
                        for i in 0..4 {
                            println!(
                                "    [{:8.4}, {:8.4}, {:8.4}, {:8.4}]",
                                m[i][0], m[i][1], m[i][2], m[i][3]
                            );
                        }
                    }
                    AnyMessage::Status(status_msg) => {
                        println!(
                            "  Status: code={}, name='{}'",
                            status_msg.content.code, status_msg.content.error_name
                        );
                        println!("  Message: '{}'", status_msg.content.status_string);
                    }
                    AnyMessage::Image(image_msg) => {
                        println!(
                            "  Image: {}x{}x{}, type={:?}",
                            image_msg.content.size[0],
                            image_msg.content.size[1],
                            image_msg.content.size[2],
                            image_msg.content.scalar_type
                        );
                        println!(
                            "  Data size: {} bytes",
                            image_msg.content.data.len()
                        );
                    }
                    AnyMessage::Position(position_msg) => {
                        println!(
                            "  Position: ({:.2}, {:.2}, {:.2})",
                            position_msg.content.position[0],
                            position_msg.content.position[1],
                            position_msg.content.position[2]
                        );
                        println!(
                            "  Quaternion: ({:.3}, {:.3}, {:.3}, {:.3})",
                            position_msg.content.quaternion[0],
                            position_msg.content.quaternion[1],
                            position_msg.content.quaternion[2],
                            position_msg.content.quaternion[3]
                        );
                    }
                    AnyMessage::String(string_msg) => {
                        println!(
                            "  String encoding: {}",
                            string_msg.content.encoding as u16
                        );
                        println!("  Content: '{}'", string_msg.content.string);
                    }
                    AnyMessage::Capability(capability_msg) => {
                        println!("  Supported message types:");
                        for type_name in &capability_msg.content.types {
                            println!("    - {}", type_name);
                        }
                    }
                    AnyMessage::Sensor(sensor_msg) => {
                        println!("  Sensor data: {} values", sensor_msg.content.data.len());
                        if !sensor_msg.content.data.is_empty() {
                            print!("  Values: ");
                            for (i, value) in sensor_msg.content.data.iter().enumerate() {
                                if i > 0 {
                                    print!(", ");
                                }
                                print!("{:.3}", value);
                                if i >= 9 {
                                    // Limit display to first 10 values
                                    if sensor_msg.content.data.len() > 10 {
                                        print!(", ... ({} more)", sensor_msg.content.data.len() - 10);
                                    }
                                    break;
                                }
                            }
                            println!();
                        }
                    }
                    AnyMessage::Point(point_msg) => {
                        println!("  Point list: {} points", point_msg.content.points.len());
                        for (i, point) in point_msg.content.points.iter().enumerate() {
                            println!(
                                "    Point {}: ({:.2}, {:.2}, {:.2}) - {}",
                                i,
                                point.position[0],
                                point.position[1],
                                point.position[2],
                                point.name
                            );
                        }
                    }
                    AnyMessage::Unknown { header, body } => {
                        println!("  ⚠️  Unknown message type!");
                        println!("  Raw body size: {} bytes", body.len());
                        println!(
                            "  This might be a custom message type not yet supported by this library."
                        );
                    }
                    _ => {
                        println!("  (No detailed handler for this message type)");
                    }
                }
                println!("{}", "=".repeat(60));
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }

    println!("\nReceived {} messages total.", message_count);
    Ok(())
}

/// Format a message for display
fn format_message(msg: &AnyMessage) -> Result<String, Box<dyn std::error::Error>> {
    let msg_type = msg.message_type();
    let device_name = msg.device_name()?;

    Ok(format!("{} from '{}'", msg_type, device_name))
}
