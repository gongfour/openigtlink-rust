//! Partial Message Transfer Demo
//!
//! Demonstrates chunked transfer of large messages with resume capability.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example partial_transfer_demo
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::partial_transfer::{
    PartialTransferManager, TransferConfig, TransferState,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Partial Message Transfer Demo ===\n");

    // Example 1: Simple file transfer
    println!("📦 Example 1: Simple Transfer");
    {
        let manager = PartialTransferManager::new();

        // Start transfer of 10 MB file
        let file_size = 10 * 1024 * 1024;
        let transfer_id = manager.start_transfer(file_size).await?;

        println!("  Started transfer ID: {}", transfer_id.value());
        println!("  File size: {} MB", file_size / 1024 / 1024);

        // Simulate chunked transfer
        let chunk_size = 65536; // 64 KB
        let mut bytes_transferred = 0;
        let mut chunk_index = 0;

        while bytes_transferred < file_size {
            let chunk_bytes = std::cmp::min(chunk_size, file_size - bytes_transferred);
            bytes_transferred += chunk_bytes;
            chunk_index += 1;

            manager
                .update_progress(transfer_id, bytes_transferred, chunk_index)
                .await?;

            // Simulate transfer time
            tokio::time::sleep(Duration::from_millis(5)).await;

            if chunk_index % 50 == 0 {
                let info = manager.get_transfer(transfer_id).await.unwrap();
                println!(
                    "  Progress: {:.1}% ({}/{} MB)",
                    info.state.progress() * 100.0,
                    bytes_transferred / 1024 / 1024,
                    file_size / 1024 / 1024
                );
            }
        }

        manager.complete_transfer(transfer_id).await?;

        let info = manager.get_transfer(transfer_id).await.unwrap();
        println!("  ✅ Transfer completed!");
        println!("  Total time: {:.2}s", info.elapsed().as_secs_f64());
        println!("  Average speed: {:.2} MB/s", info.speed_bps() / 1_000_000.0);
    }

    println!();

    // Example 2: Transfer with interruption and resume
    println!("📦 Example 2: Interrupt and Resume");
    {
        let manager = PartialTransferManager::new();

        // Start transfer of 5 MB file
        let file_size = 5 * 1024 * 1024;
        let transfer_id = manager.start_transfer(file_size).await?;

        println!("  Started transfer of {} MB", file_size / 1024 / 1024);

        // Transfer first 2 MB
        let chunk_size = 65536;
        let mut bytes_transferred = 0;
        let mut chunk_index = 0;

        while bytes_transferred < 2 * 1024 * 1024 {
            let chunk_bytes = std::cmp::min(chunk_size, file_size - bytes_transferred);
            bytes_transferred += chunk_bytes;
            chunk_index += 1;

            manager
                .update_progress(transfer_id, bytes_transferred, chunk_index)
                .await?;

            tokio::time::sleep(Duration::from_millis(2)).await;
        }

        println!("  Transferred: {} MB", bytes_transferred / 1024 / 1024);

        // Interrupt transfer
        manager.interrupt_transfer(transfer_id).await?;
        println!("  ⚠️  Transfer interrupted!");

        let info = manager.get_transfer(transfer_id).await.unwrap();
        if let TransferState::Interrupted {
            bytes_transferred,
            resumable,
            ..
        } = info.state
        {
            println!("  Interrupted at: {} MB", bytes_transferred / 1024 / 1024);
            println!("  Resumable: {}", resumable);
        }

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Resume transfer
        let resume_from = manager.resume_transfer(transfer_id).await?;
        println!("  🔄 Resuming from: {} MB", resume_from / 1024 / 1024);

        bytes_transferred = resume_from;

        // Continue transfer
        while bytes_transferred < file_size {
            let chunk_bytes = std::cmp::min(chunk_size, file_size - bytes_transferred);
            bytes_transferred += chunk_bytes;
            chunk_index += 1;

            manager
                .update_progress(transfer_id, bytes_transferred, chunk_index)
                .await?;

            tokio::time::sleep(Duration::from_millis(2)).await;
        }

        manager.complete_transfer(transfer_id).await?;
        println!("  ✅ Transfer completed after resume!");

        let info = manager.get_transfer(transfer_id).await.unwrap();
        println!("  Total chunks: {}", chunk_index);
        println!("  Total time: {:.2}s", info.elapsed().as_secs_f64());
    }

    println!();

    // Example 3: Multiple concurrent transfers
    println!("📦 Example 3: Multiple Concurrent Transfers");
    {
        let manager = std::sync::Arc::new(PartialTransferManager::new());

        // Start 5 transfers
        let mut handles = vec![];

        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let file_size = (i + 1) * 1024 * 1024; // 1-5 MB
                let transfer_id = manager_clone.start_transfer(file_size).await.unwrap();

                let chunk_size = 32768; // 32 KB
                let mut bytes_transferred = 0;
                let mut chunk_index = 0;

                while bytes_transferred < file_size {
                    let chunk_bytes = std::cmp::min(chunk_size, file_size - bytes_transferred);
                    bytes_transferred += chunk_bytes;
                    chunk_index += 1;

                    manager_clone
                        .update_progress(transfer_id, bytes_transferred, chunk_index)
                        .await
                        .unwrap();

                    tokio::time::sleep(Duration::from_millis(2)).await;
                }

                manager_clone.complete_transfer(transfer_id).await.unwrap();
                transfer_id
            });

            handles.push(handle);
        }

        // Wait for all transfers
        for (i, handle) in handles.into_iter().enumerate() {
            let transfer_id = handle.await.unwrap();
            let info = manager.get_transfer(transfer_id).await.unwrap();

            println!(
                "  Transfer {}: {} MB in {:.2}s ({:.2} MB/s)",
                i + 1,
                (i + 1),
                info.elapsed().as_secs_f64(),
                info.speed_bps() / 1_000_000.0
            );
        }

        let active = manager.active_transfers().await;
        println!("  Active transfers remaining: {}", active.len());
    }

    println!();

    // Example 4: Transfer monitoring
    println!("📦 Example 4: Real-time Transfer Monitoring");
    {
        let manager = std::sync::Arc::new(PartialTransferManager::new());

        // Start a large transfer
        let file_size = 20 * 1024 * 1024; // 20 MB
        let transfer_id = manager.start_transfer(file_size).await?;

        // Spawn transfer task
        let manager_clone = manager.clone();
        let transfer_task = tokio::spawn(async move {
            let chunk_size = 65536;
            let mut bytes_transferred = 0;
            let mut chunk_index = 0;

            while bytes_transferred < file_size {
                let chunk_bytes = std::cmp::min(chunk_size, file_size - bytes_transferred);
                bytes_transferred += chunk_bytes;
                chunk_index += 1;

                manager_clone
                    .update_progress(transfer_id, bytes_transferred, chunk_index)
                    .await
                    .unwrap();

                tokio::time::sleep(Duration::from_millis(5)).await;
            }

            manager_clone.complete_transfer(transfer_id).await.unwrap();
        });

        // Monitor progress
        let manager_clone = manager.clone();
        let monitor_task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Some(info) = manager_clone.get_transfer(transfer_id).await {
                    match info.state {
                        TransferState::InProgress { .. } => {
                            println!(
                                "  📊 Progress: {:.1}% | Speed: {:.2} MB/s | Elapsed: {:.1}s",
                                info.state.progress() * 100.0,
                                info.speed_bps() / 1_000_000.0,
                                info.elapsed().as_secs_f64()
                            );
                        }
                        TransferState::Completed { .. } => {
                            println!("  ✅ Monitoring: Transfer completed!");
                            break;
                        }
                        _ => break,
                    }
                } else {
                    break;
                }
            }
        });

        // Wait for both tasks
        transfer_task.await.unwrap();
        monitor_task.await.unwrap();

        let info = manager.get_transfer(transfer_id).await.unwrap();
        println!("  Final speed: {:.2} MB/s", info.speed_bps() / 1_000_000.0);
    }

    println!();

    // Example 5: Custom chunk size
    println!("📦 Example 5: Custom Chunk Size Comparison");
    {
        let file_size = 5 * 1024 * 1024; // 5 MB

        for chunk_size in [16384, 65536, 262144] {
            // 16KB, 64KB, 256KB
            let config = TransferConfig {
                chunk_size,
                allow_resume: true,
                timeout_secs: Some(300),
            };

            let manager = PartialTransferManager::with_config(config);
            let transfer_id = manager.start_transfer(file_size).await?;

            let mut bytes_transferred = 0;
            let mut chunk_index = 0;

            let start = std::time::Instant::now();

            while bytes_transferred < file_size {
                let chunk_bytes = std::cmp::min(chunk_size, file_size - bytes_transferred);
                bytes_transferred += chunk_bytes;
                chunk_index += 1;

                manager
                    .update_progress(transfer_id, bytes_transferred, chunk_index)
                    .await?;

                // Simulate small delay per chunk
                tokio::time::sleep(Duration::from_micros(100)).await;
            }

            manager.complete_transfer(transfer_id).await?;

            let elapsed = start.elapsed();
            println!(
                "  Chunk size: {} KB | Chunks: {} | Time: {:.3}s",
                chunk_size / 1024,
                chunk_index,
                elapsed.as_secs_f64()
            );
        }
    }

    println!();

    // Example 6: Cleanup operations
    println!("📦 Example 6: Cleanup and Management");
    {
        let manager = PartialTransferManager::new();

        // Create multiple transfers
        let id1 = manager.start_transfer(1_000_000).await?;
        let id2 = manager.start_transfer(2_000_000).await?;
        let id3 = manager.start_transfer(3_000_000).await?;

        manager.complete_transfer(id1).await?;
        manager
            .fail_transfer(id2, "Network error".to_string())
            .await?;
        // id3 remains in progress

        println!("  Created 3 transfers:");
        println!("    - Transfer 1: Completed");
        println!("    - Transfer 2: Failed");
        println!("    - Transfer 3: In progress");

        let active = manager.active_transfers().await;
        println!("  Active transfers: {}", active.len());

        // Cleanup completed and failed
        manager.cleanup_completed().await;

        println!("  After cleanup:");
        println!("    - Transfer 1: {}", manager.get_transfer(id1).await.is_some());
        println!("    - Transfer 2: {}", manager.get_transfer(id2).await.is_some());
        println!("    - Transfer 3: {}", manager.get_transfer(id3).await.is_some());
    }

    println!("\n✅ All examples completed successfully!");
    println!("\n💡 Key Features:");
    println!("  - Chunked transfer for large messages");
    println!("  - Interrupt and resume capability");
    println!("  - Progress tracking and monitoring");
    println!("  - Concurrent transfer support");
    println!("  - Configurable chunk sizes");
    println!("  - Automatic timeout handling");

    Ok(())
}
