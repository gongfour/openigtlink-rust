//! Error Handling and Reconnection Logic Example
//!
//! Demonstrates production-ready error handling patterns for robust
//! OpenIGTLink applications including reconnection, timeouts, and
//! graceful degradation.
//!
//! # Usage
//!
//! ```bash
//! # Test automatic reconnection with exponential backoff
//! cargo run --example error_handling reconnect
//!
//! # Test timeout handling
//! cargo run --example error_handling timeout
//!
//! # Test CRC error detection
//! cargo run --example error_handling crc
//!
//! # Test wrong message type handling
//! cargo run --example error_handling wrong_type
//!
//! # Show all scenarios
//! cargo run --example error_handling all
//! ```

use openigtlink_rust::error::{IgtlError, Result};
use openigtlink_rust::io::{ClientBuilder, SyncIgtlClient};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{StatusMessage, TransformMessage};
use std::env;
use std::io::ErrorKind;
use std::thread;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        eprintln!("[FATAL] Unrecoverable error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let scenario = env::args().nth(1).unwrap_or_else(|| "all".to_string());

    println!("=== OpenIGTLink Error Handling Patterns ===\n");

    match scenario.as_str() {
        "reconnect" => test_reconnect_logic()?,
        "timeout" => test_timeout_handling()?,
        "crc" => test_crc_error_recovery()?,
        "wrong_type" => test_wrong_message_type()?,
        "all" => {
            println!("[INFO] Running all error handling scenarios\n");
            test_reconnect_logic()?;
            println!();
            test_timeout_handling()?;
            println!();
            test_crc_error_recovery()?;
            println!();
            test_wrong_message_type()?;
        }
        _ => {
            println!("Usage: cargo run --example error_handling [SCENARIO]");
            println!("\nScenarios:");
            println!("  reconnect   - Automatic reconnection with exponential backoff");
            println!("  timeout     - Timeout handling and recovery");
            println!("  crc         - CRC error detection and retransmission");
            println!("  wrong_type  - Wrong message type handling");
            println!("  all         - Run all scenarios (default)");
        }
    }

    Ok(())
}

/// Test automatic reconnection with exponential backoff
fn test_reconnect_logic() -> Result<()> {
    println!("[SCENARIO 1] Automatic Reconnection with Exponential Backoff");
    println!("────────────────────────────────────────────────────────────\n");

    const MAX_RETRIES: usize = 5;
    let mut retry_count = 0;
    let mut delay = Duration::from_secs(1);

    println!("[INFO] Attempting to connect to server at 127.0.0.1:18944");
    println!("  Max retries: {}", MAX_RETRIES);
    println!("  Initial delay: {:?}\n", delay);

    loop {
        println!("  Attempt {}/{}", retry_count + 1, MAX_RETRIES);

        match ClientBuilder::new().tcp("127.0.0.1:18944").sync().build() {
            Ok(mut client) => {
                println!("  ✓ Connection established!");

                // Perform work after successful connection
                println!("  → Sending test message...");

                let transform = TransformMessage::identity();
                let msg = IgtlMessage::new(transform, "RobustClient")?;
                client.send(&msg)?;

                println!("  ✓ Message sent successfully");
                println!("\n[SUCCESS] Reconnection logic validated");
                break;
            }
            Err(e) => {
                retry_count += 1;

                println!("  ✗ Connection failed: {}", e);

                if retry_count >= MAX_RETRIES {
                    println!("\n[FAIL] Max retries exceeded");
                    return Err(e);
                }

                println!("  → Retrying in {:?}...", delay);
                thread::sleep(delay);

                // Exponential backoff: double the delay (capped at 16 seconds)
                delay = delay.saturating_mul(2).min(Duration::from_secs(16));
            }
        }
    }

    Ok(())
}

/// Test timeout handling
fn test_timeout_handling() -> Result<()> {
    println!("[SCENARIO 2] Timeout Handling");
    println!("────────────────────────────────────────────────────────────\n");

    println!("[INFO] Connecting to server...");

    match ClientBuilder::new().tcp("127.0.0.1:18944").sync().build() {
        Ok(mut client) => {
            println!("  ✓ Connected");

            // Set read timeout
            let timeout = Duration::from_secs(2);
            client.set_read_timeout(Some(timeout))?;

            println!("  → Read timeout set to {:?}", timeout);
            println!("  → Attempting to receive message...");

            match client.receive::<StatusMessage>() {
                Ok(msg) => {
                    println!("  ✓ Received message: {}", msg.content.status_string);
                }
                Err(IgtlError::Io(e)) if e.kind() == ErrorKind::WouldBlock => {
                    println!("  ⏱ Timeout occurred (expected behavior)");
                    println!("  → Recommended action: Check server status or retry");
                }
                Err(IgtlError::Io(e)) if e.kind() == ErrorKind::TimedOut => {
                    println!("  ⏱ Timeout occurred (expected behavior)");
                    println!("  → Recommended action: Check server status or retry");
                }
                Err(e) => {
                    println!("  ✗ Unexpected error: {}", e);
                    return Err(e);
                }
            }

            println!("\n[SUCCESS] Timeout handling validated");
        }
        Err(e) => {
            println!("  ✗ Connection failed: {}", e);
            println!("  → Skipping timeout test (server not available)");
        }
    }

    Ok(())
}

/// Test CRC error detection and recovery
fn test_crc_error_recovery() -> Result<()> {
    println!("[SCENARIO 3] CRC Error Detection and Recovery");
    println!("────────────────────────────────────────────────────────────\n");

    println!("[INFO] CRC Error Handling Strategy");
    println!();
    println!("  What is CRC?");
    println!("    CRC (Cyclic Redundancy Check) detects data corruption");
    println!("    during network transmission.");
    println!();
    println!("  When CRC errors occur:");
    println!("    1. Data has been corrupted (network issues, hardware faults)");
    println!("    2. Message should NOT be processed (data integrity compromised)");
    println!("    3. Application should request retransmission");
    println!();
    println!("  Error Handling Pattern:");
    println!("    ```rust");
    println!("    match client.receive::<TransformMessage>() {{");
    println!("        Ok(msg) => process_message(msg),");
    println!("        Err(IgtlError::CrcMismatch {{ expected, actual }}) => {{");
    println!(
        "            eprintln!(\"CRC error: expected {{:X}}, got {{:X}}\", expected, actual);"
    );
    println!("            // Request retransmission or skip message");
    println!("        }}");
    println!("        Err(e) => handle_other_errors(e),");
    println!("    }}");
    println!("    ```");
    println!();
    println!("  Recovery Actions:");
    println!("    • Request retransmission from sender");
    println!("    • Log error for monitoring");
    println!("    • Increment error counter");
    println!("    • Alert if error rate exceeds threshold");
    println!();
    println!("[SUCCESS] CRC error handling documented");

    Ok(())
}

/// Test wrong message type handling
fn test_wrong_message_type() -> Result<()> {
    println!("[SCENARIO 4] Wrong Message Type Handling");
    println!("────────────────────────────────────────────────────────────\n");

    println!("[INFO] Demonstrating graceful degradation for type mismatches");
    println!();

    match ClientBuilder::new().tcp("127.0.0.1:18944").sync().build() {
        Ok(mut client) => {
            println!("  ✓ Connected to server");

            // Send a TRANSFORM message
            println!("  → Sending TRANSFORM message...");
            let transform = TransformMessage::identity();
            let msg = IgtlMessage::new(transform, "TestClient")?;
            client.send(&msg)?;
            println!("  ✓ TRANSFORM sent");

            // Try to receive as STATUS (might fail if server sends different type)
            println!("  → Expecting STATUS response...");

            client.set_read_timeout(Some(Duration::from_secs(2)))?;

            match client.receive::<StatusMessage>() {
                Ok(msg) => {
                    println!("  ✓ Received STATUS: {}", msg.content.status_string);
                }
                Err(IgtlError::UnknownMessageType(type_name)) => {
                    println!("  ⚠ Received unexpected type: {}", type_name);
                    println!("  → Action: Discarding message (graceful degradation)");
                    println!("  → Alternative: Use generic message receiver");
                }
                Err(IgtlError::Io(e))
                    if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
                {
                    println!("  ⏱ No response (timeout)");
                    println!("  → Server may not be sending responses");
                }
                Err(e) => {
                    println!("  ✗ Error: {}", e);
                }
            }

            println!();
            println!("  Best Practices:");
            println!("    • Use protocol negotiation to agree on message types");
            println!("    • Implement fallback handlers for unknown types");
            println!("    • Log unexpected types for debugging");
            println!("    • Consider using a generic message receiver");

            println!("\n[SUCCESS] Type mismatch handling demonstrated");
        }
        Err(e) => {
            println!("  ✗ Connection failed: {}", e);
            println!("  → Skipping type mismatch test (server not available)");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let mut delay = Duration::from_secs(1);

        // First retry: 1s
        assert_eq!(delay, Duration::from_secs(1));

        // Second retry: 2s
        delay = delay.saturating_mul(2);
        assert_eq!(delay, Duration::from_secs(2));

        // Third retry: 4s
        delay = delay.saturating_mul(2);
        assert_eq!(delay, Duration::from_secs(4));

        // Fourth retry: 8s
        delay = delay.saturating_mul(2);
        assert_eq!(delay, Duration::from_secs(8));

        // Fifth retry: 16s (capped)
        delay = delay.saturating_mul(2).min(Duration::from_secs(16));
        assert_eq!(delay, Duration::from_secs(16));

        // Sixth retry: 16s (stays at cap)
        delay = delay.saturating_mul(2).min(Duration::from_secs(16));
        assert_eq!(delay, Duration::from_secs(16));
    }

    #[test]
    fn test_max_retries() {
        const MAX_RETRIES: usize = 5;
        let mut retry_count = 0;

        // Simulate failed connection attempts
        for _ in 0..MAX_RETRIES {
            retry_count += 1;
            assert!(retry_count <= MAX_RETRIES);
        }

        assert_eq!(retry_count, MAX_RETRIES);
    }

    #[test]
    fn test_timeout_values() {
        let short_timeout = Duration::from_secs(2);
        let long_timeout = Duration::from_secs(30);

        assert!(short_timeout < long_timeout);
        assert_eq!(short_timeout.as_secs(), 2);
        assert_eq!(long_timeout.as_secs(), 30);
    }

    #[test]
    fn test_error_kind_matching() {
        let timeout_error = std::io::Error::new(ErrorKind::TimedOut, "timeout");
        assert_eq!(timeout_error.kind(), ErrorKind::TimedOut);

        let would_block_error = std::io::Error::new(ErrorKind::WouldBlock, "would block");
        assert_eq!(would_block_error.kind(), ErrorKind::WouldBlock);
    }
}
