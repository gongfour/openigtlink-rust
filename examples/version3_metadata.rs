//! OpenIGTLink Version 3 Metadata Example
//!
//! Demonstrates Version 3 metadata feature for attaching key-value pairs to messages.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example version3_metadata
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{StatusMessage, TransformMessage};
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=== OpenIGTLink Version 3 Metadata Demo ===\n");

    // Example 1: Adding metadata with set_metadata
    println!("📦 Example 1: Set Metadata");
    {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "Device1")?;

        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("sequence".to_string(), "42".to_string());
        metadata.insert("session_id".to_string(), "abc123".to_string());
        msg.set_metadata(metadata.clone());

        println!("  Version: {}", msg.header.version);
        println!("  Metadata count: {}", metadata.len());
        for (key, value) in metadata.iter() {
            println!("    {}: {}", key, value);
        }
    }

    println!();

    // Example 2: Adding metadata incrementally
    println!("📦 Example 2: Add Metadata Incrementally");
    {
        let status = StatusMessage::ok("Ready");
        let mut msg = IgtlMessage::new(status, "Device2")?;

        msg.add_metadata("timestamp".to_string(), "1234567890".to_string());
        msg.add_metadata("user".to_string(), "surgeon_1".to_string());
        msg.add_metadata("room".to_string(), "OR-3".to_string());

        println!("  Version: {}", msg.header.version);
        if let Some(metadata) = msg.get_metadata() {
            println!("  Metadata count: {}", metadata.len());
            for (key, value) in metadata.iter() {
                println!("    {}: {}", key, value);
            }
        }
    }

    println!();

    // Example 3: Roundtrip with metadata
    println!("📦 Example 3: Roundtrip Encode/Decode with Metadata");
    {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "Tracker")?;

        msg.add_metadata("tool_id".to_string(), "tool_123".to_string());
        msg.add_metadata("tracking_quality".to_string(), "95.5".to_string());

        let encoded = msg.encode()?;
        println!("  Encoded size: {} bytes", encoded.len());

        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded)?;
        println!("  Decoded successfully");

        if let Some(metadata) = decoded.get_metadata() {
            println!("  Metadata:");
            println!("    tool_id: {}", metadata.get("tool_id").unwrap());
            println!(
                "    tracking_quality: {}",
                metadata.get("tracking_quality").unwrap()
            );
        }
    }

    println!();

    // Example 4: Extended header + Metadata
    println!("📦 Example 4: Extended Header + Metadata");
    {
        let status = StatusMessage::ok("All systems nominal");
        let mut msg = IgtlMessage::new(status, "System")?;

        // Add extended header
        msg.set_extended_header(vec![0x01, 0x02, 0x03, 0x04]);

        // Add metadata
        msg.add_metadata("system_state".to_string(), "operational".to_string());
        msg.add_metadata("uptime".to_string(), "86400".to_string());

        let encoded = msg.encode()?;
        println!("  Total size: {} bytes", encoded.len());

        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded)?;
        println!("  Extended header: {:?}", decoded.get_extended_header());

        if let Some(metadata) = decoded.get_metadata() {
            println!("  Metadata:");
            for (key, value) in metadata.iter() {
                println!("    {}: {}", key, value);
            }
        }
    }

    println!();

    // Example 5: UTF-8 metadata values
    println!("📦 Example 5: UTF-8 Metadata");
    {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "Device5")?;

        msg.add_metadata("surgeon".to_string(), "田中　太郎".to_string());
        msg.add_metadata("status".to_string(), "✅ Ready".to_string());
        msg.add_metadata("emoji".to_string(), "🏥🩺💉".to_string());

        let encoded = msg.encode()?;
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded)?;

        if let Some(metadata) = decoded.get_metadata() {
            println!("  Metadata:");
            for (key, value) in metadata.iter() {
                println!("    {}: {}", key, value);
            }
        }
    }

    println!();

    // Example 6: Clear metadata
    println!("📦 Example 6: Clear Metadata");
    {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "Device6")?;

        msg.add_metadata("temp".to_string(), "value".to_string());
        println!("  Version after add: {}", msg.header.version);

        msg.clear_metadata();
        println!("  Version after clear: {}", msg.header.version);
        println!("  Metadata: {:?}", msg.get_metadata());
    }

    println!();

    // Example 7: Practical use case - Surgical tracking
    println!("📦 Example 7: Practical Use Case - Surgical Tracking");
    {
        println!("  Simulating 3 tracking updates with metadata...\n");

        for i in 1..=3 {
            let transform = TransformMessage::identity();
            let mut msg = IgtlMessage::new(transform, "SurgicalTool")?;

            // Add tracking metadata
            msg.add_metadata("frame_id".to_string(), format!("{}", i));
            msg.add_metadata("timestamp_ms".to_string(), format!("{}", i * 1000));
            msg.add_metadata(
                "tracking_error_mm".to_string(),
                format!("{:.2}", 0.5 + i as f32 * 0.1),
            );
            msg.add_metadata("num_markers".to_string(), "4".to_string());
            msg.add_metadata(
                "confidence".to_string(),
                format!("{:.1}", 95.0 + i as f32 * 0.5),
            );

            let encoded = msg.encode()?;
            let decoded = IgtlMessage::<TransformMessage>::decode(&encoded)?;

            println!("  Frame #{}:", i);
            if let Some(metadata) = decoded.get_metadata() {
                println!(
                    "    Timestamp: {} ms",
                    metadata.get("timestamp_ms").unwrap()
                );
                println!(
                    "    Tracking Error: {} mm",
                    metadata.get("tracking_error_mm").unwrap()
                );
                println!("    Confidence: {}%", metadata.get("confidence").unwrap());
                println!("    Message Size: {} bytes", encoded.len());
            }
            println!();
        }
    }

    // Example 8: Size comparison
    println!("📦 Example 8: Size Comparison");
    {
        let transform = TransformMessage::identity();

        // Version 2 (no metadata)
        let msg_v2 = IgtlMessage::new(transform.clone(), "Device")?;
        let size_v2 = msg_v2.encode()?.len();

        // Version 3 with metadata
        let mut msg_v3 = IgtlMessage::new(transform, "Device")?;
        msg_v3.add_metadata("key1".to_string(), "value1".to_string());
        msg_v3.add_metadata("key2".to_string(), "value2".to_string());
        let size_v3 = msg_v3.encode()?.len();

        println!("  Version 2 message: {} bytes", size_v2);
        println!("  Version 3 with metadata: {} bytes", size_v3);
        println!("  Overhead: {} bytes", size_v3 - size_v2);
    }

    println!("\n✅ All examples completed successfully!");

    Ok(())
}
