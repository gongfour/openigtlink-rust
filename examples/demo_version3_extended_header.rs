//! OpenIGTLink Version 3 Extended Header Example
//!
//! Demonstrates Version 3 protocol features including extended headers.
//!
//! # Protocol Version Comparison
//!
//! - Version 2: Header (58) + Content
//! - Version 3: Header (58) + ExtHdrSize (2) + ExtHdr (variable) + Content
//!
//! # Usage
//!
//! ```bash
//! cargo run --example version3_extended_header
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{StatusMessage, TransformMessage};

fn main() -> Result<()> {
    println!("=== OpenIGTLink Version 3 Extended Header Demo ===\n");

    // Example 1: Version 2 message (default)
    println!("📦 Example 1: Version 2 Message (No Extended Header)");
    {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "Device1")?;

        println!("  Version: {}", msg.header.version);
        println!("  Extended Header: {:?}", msg.get_extended_header());

        let encoded = msg.encode()?;
        println!("  Total Size: {} bytes", encoded.len());
        println!("    - Header: 58 bytes");
        println!("    - Content: {} bytes", encoded.len() - 58);
    }

    println!();

    // Example 2: Version 3 with extended header
    println!("📦 Example 2: Version 3 Message (With Extended Header)");
    {
        let status = StatusMessage::ok("Ready");
        let mut msg = IgtlMessage::new(status, "Device2")?;

        // Add custom extended header (e.g., sequence number, priority, etc.)
        let ext_header = vec![
            0x01, 0x02, // Sequence number (example)
            0x00, 0x05, // Priority (example)
            0xAA, 0xBB, // Custom field 1
            0xCC, 0xDD, // Custom field 2
        ];
        msg.set_extended_header(ext_header.clone());

        println!("  Version: {}", msg.header.version);
        println!(
            "  Extended Header: {:02X?}",
            msg.get_extended_header().unwrap()
        );

        let encoded = msg.encode()?;
        println!("  Total Size: {} bytes", encoded.len());
        println!("    - Header: 58 bytes");
        println!("    - ExtHdr Size Field: 2 bytes");
        println!("    - ExtHdr Data: {} bytes", ext_header.len());
        println!(
            "    - Content: {} bytes",
            encoded.len() - 58 - 2 - ext_header.len()
        );
    }

    println!();

    // Example 3: Roundtrip encoding/decoding
    println!("📦 Example 3: Roundtrip Encode/Decode");
    {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "Device3")?;

        // Add extended header with custom metadata
        let ext_header = vec![0xFF, 0xEE, 0xDD, 0xCC];
        msg.set_extended_header(ext_header.clone());

        println!("  Original:");
        println!("    Version: {}", msg.header.version);
        println!("    ExtHdr: {:02X?}", msg.get_extended_header().unwrap());

        // Encode and decode
        let encoded = msg.encode()?;
        let decoded = IgtlMessage::<TransformMessage>::decode(&encoded)?;

        println!("  Decoded:");
        println!("    Version: {}", decoded.header.version);
        println!(
            "    ExtHdr: {:02X?}",
            decoded.get_extended_header().unwrap()
        );
        println!(
            "    Match: {}",
            msg.get_extended_header() == decoded.get_extended_header()
        );
    }

    println!();

    // Example 4: Empty extended header
    println!("📦 Example 4: Empty Extended Header");
    {
        let status = StatusMessage::ok("Empty ExtHdr");
        let mut msg = IgtlMessage::new(status, "Device4")?;

        // Set empty extended header (still upgrades to Version 3)
        msg.set_extended_header(vec![]);

        println!("  Version: {}", msg.header.version);
        println!(
            "  ExtHdr Length: {} bytes",
            msg.get_extended_header().unwrap().len()
        );

        let encoded = msg.encode()?;
        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded)?;

        println!("  Roundtrip Success: {}", decoded.header.version == 3);
        println!("  ExtHdr After Decode: {:?}", decoded.get_extended_header());
    }

    println!();

    // Example 5: Downgrade from Version 3 to Version 2
    println!("📦 Example 5: Version Downgrade");
    {
        let transform = TransformMessage::identity();
        let mut msg = IgtlMessage::new(transform, "Device5")?;

        // Upgrade to Version 3
        msg.set_extended_header(vec![0x01, 0x02]);
        println!(
            "  After set_extended_header: Version {}",
            msg.header.version
        );

        // Clear extended header (downgrades to Version 2)
        msg.clear_extended_header();
        println!(
            "  After clear_extended_header: Version {}",
            msg.header.version
        );
        println!("  ExtHdr: {:?}", msg.get_extended_header());
    }

    println!();

    // Example 6: Large extended header
    println!("📦 Example 6: Large Extended Header");
    {
        let status = StatusMessage::ok("Large ExtHdr");
        let mut msg = IgtlMessage::new(status, "Device6")?;

        // Create 1 KB extended header
        let large_ext_header = vec![0xAB; 1024];
        msg.set_extended_header(large_ext_header.clone());

        println!("  ExtHdr Size: {} bytes", large_ext_header.len());

        let encoded = msg.encode()?;
        println!("  Total Message Size: {} bytes", encoded.len());

        let decoded = IgtlMessage::<StatusMessage>::decode(&encoded)?;
        println!(
            "  Roundtrip Success: {}",
            decoded.get_extended_header().unwrap().len() == 1024
        );
    }

    println!();

    // Example 7: Use case - Sequence numbering
    println!("📦 Example 7: Practical Use Case - Sequence Numbering");
    {
        for seq in 1..=3 {
            let transform = TransformMessage::identity();
            let mut msg = IgtlMessage::new(transform, "Tracker")?;

            // Encode sequence number in extended header
            let seq_bytes = (seq as u32).to_be_bytes();
            msg.set_extended_header(seq_bytes.to_vec());

            let encoded = msg.encode()?;
            let decoded = IgtlMessage::<TransformMessage>::decode(&encoded)?;

            // Extract sequence number
            let decoded_seq = u32::from_be_bytes([
                decoded.get_extended_header().unwrap()[0],
                decoded.get_extended_header().unwrap()[1],
                decoded.get_extended_header().unwrap()[2],
                decoded.get_extended_header().unwrap()[3],
            ]);

            println!(
                "  Message #{}: Encoded {} bytes, Seq={}",
                seq,
                encoded.len(),
                decoded_seq
            );
        }
    }

    println!("\n✅ All examples completed successfully!");

    Ok(())
}
