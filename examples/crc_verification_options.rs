//! CRC Verification Options Example
//!
//! Demonstrates selective CRC verification for performance optimization
//! in trusted environments.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example crc_verification_options
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{TransformMessage, StatusMessage};
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== CRC Verification Options Demo ===\n");

    // Example 1: Default behavior (CRC enabled)
    println!("📦 Example 1: Default CRC Verification (Enabled)");
    {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform.clone(), "Device1")?;

        let encoded = msg.encode()?;

        // Default decode() uses CRC verification
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded)?;

        println!("  Message decoded successfully with CRC verification");
        println!("  Content matches: {}", decoded.content == transform);
    }

    println!();

    // Example 2: Explicit CRC verification control
    println!("📦 Example 2: Explicit CRC Control");
    {
        let status = StatusMessage::ok("Test message");
        let msg = IgtlMessage::new(status.clone(), "Device2")?;

        let encoded = msg.encode()?;

        // Decode with CRC verification enabled
        let decoded_with_crc = IgtlMessage::<StatusMessage>::decode_with_options(&encoded, true)?;
        println!("  With CRC: Decoded successfully");

        // Decode with CRC verification disabled
        let decoded_without_crc = IgtlMessage::<StatusMessage>::decode_with_options(&encoded, false)?;
        println!("  Without CRC: Decoded successfully");

        println!("  Both results match: {}",
                 decoded_with_crc.content == decoded_without_crc.content);
    }

    println!();

    // Example 3: Corrupted data detection
    println!("📦 Example 3: Corrupted Data Detection");
    {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "Device3")?;

        let mut encoded = msg.encode()?;

        // Corrupt one byte in the content
        encoded[58] ^= 0xFF;

        // Try to decode with CRC verification (should fail)
        match IgtlMessage::<TransformMessage>::decode_with_options(&encoded, true) {
            Ok(_) => println!("  ⚠️  Unexpected: Corrupted data passed CRC check"),
            Err(e) => println!("  ✅ CRC check detected corruption: {:?}", e),
        }

        // Try to decode without CRC verification (should succeed but with corrupted data)
        match IgtlMessage::<TransformMessage>::decode_with_options(&encoded, false) {
            Ok(_) => println!("  ⚠️  No CRC: Corrupted data decoded (silent corruption)"),
            Err(e) => println!("  Error: {:?}", e),
        }
    }

    println!();

    // Example 4: Performance comparison
    println!("📦 Example 4: Performance Comparison");
    {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "Device4")?;
        let encoded = msg.encode()?;

        let iterations = 10000;

        // Benchmark with CRC verification
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = IgtlMessage::<TransformMessage>::decode_with_options(&encoded, true)?;
        }
        let duration_with_crc = start.elapsed();

        // Benchmark without CRC verification
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = IgtlMessage::<TransformMessage>::decode_with_options(&encoded, false)?;
        }
        let duration_without_crc = start.elapsed();

        println!("  {} iterations:", iterations);
        println!("    With CRC:    {:?} ({:.2} µs/msg)",
                 duration_with_crc,
                 duration_with_crc.as_micros() as f64 / iterations as f64);
        println!("    Without CRC: {:?} ({:.2} µs/msg)",
                 duration_without_crc,
                 duration_without_crc.as_micros() as f64 / iterations as f64);

        let speedup = duration_with_crc.as_micros() as f64 / duration_without_crc.as_micros() as f64;
        println!("    Speedup: {:.2}x faster without CRC", speedup);
    }

    println!();

    // Example 5: Client with CRC disabled
    println!("📦 Example 5: Use Cases for Disabling CRC");
    println!("  ✅ Recommended scenarios:");
    println!("    - Loopback communication (127.0.0.1)");
    println!("    - Local network with reliable hardware");
    println!("    - High-frequency data (>1000 Hz) where latency matters");
    println!("    - Testing/development environments");
    println!();
    println!("  ⚠️  NOT recommended:");
    println!("    - Internet communication");
    println!("    - Unreliable networks (WiFi, cellular)");
    println!("    - Medical/safety-critical applications");
    println!("    - Long-distance communication");

    println!();

    // Example 6: Version 3 with CRC options
    println!("📦 Example 6: Version 3 Features with CRC Options");
    {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform.clone(), "Device6")?;

        // Add Version 3 features
        msg.set_extended_header(vec![0xAA, 0xBB, 0xCC, 0xDD]);
        msg.add_metadata("seq".to_string(), "123".to_string());
        msg.add_metadata("priority".to_string(), "high".to_string());

        let encoded = msg.encode()?;
        println!("  Message size: {} bytes", encoded.len());

        // Decode with CRC disabled (Version 3 features still work)
        let decoded = IgtlMessage::<TransformMessage>::decode_with_options(&encoded, false)?;

        println!("  Version: {}", decoded.header.version);
        println!("  Extended header: {:?}", decoded.get_extended_header());
        println!("  Metadata count: {}", decoded.get_metadata().map(|m| m.len()).unwrap_or(0));
        println!("  Content matches: {}", decoded.content == transform);
    }

    println!();

    // Example 7: Real-world scenario
    println!("📦 Example 7: Real-World Scenario - High-Frequency Tracking");
    {
        println!("  Simulating 1000 Hz tracking system...");

        let iterations = 100;
        let mut total_time_with_crc = std::time::Duration::ZERO;
        let mut total_time_without_crc = std::time::Duration::ZERO;

        for i in 0..iterations {
            let transform = TransformMessage::identity();
            let mut msg = IgtlMessage::new(transform, "Tracker")?;

            // Add metadata for tracking
            msg.add_metadata("frame_id".to_string(), format!("{}", i));
            msg.add_metadata("timestamp_us".to_string(), format!("{}", i * 1000));

            let encoded = msg.encode()?;

            // Measure with CRC
            let start = Instant::now();
            let _ = IgtlMessage::<TransformMessage>::decode_with_options(&encoded, true)?;
            total_time_with_crc += start.elapsed();

            // Measure without CRC
            let start = Instant::now();
            let _ = IgtlMessage::<TransformMessage>::decode_with_options(&encoded, false)?;
            total_time_without_crc += start.elapsed();
        }

        let avg_with_crc = total_time_with_crc.as_micros() / iterations;
        let avg_without_crc = total_time_without_crc.as_micros() / iterations;

        println!("  Average decode time:");
        println!("    With CRC:    {} µs", avg_with_crc);
        println!("    Without CRC: {} µs", avg_without_crc);
        println!("    Saved: {} µs per message", avg_with_crc - avg_without_crc);

        if avg_with_crc > 0 {
            let max_rate_with = 1_000_000 / avg_with_crc;
            let max_rate_without = 1_000_000 / avg_without_crc;
            println!("  Theoretical max rate:");
            println!("    With CRC:    ~{} Hz", max_rate_with);
            println!("    Without CRC: ~{} Hz", max_rate_without);
        }
    }

    println!("\n✅ All examples completed successfully!");
    println!("\n💡 Remember: Only disable CRC in trusted, low-latency environments!");

    Ok(())
}
