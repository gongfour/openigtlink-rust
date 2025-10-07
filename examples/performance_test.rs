//! Performance testing utility
//!
//! Measures throughput and latency for OpenIGTLink operations.

use openigtlink_rust::{
    compression::{compress, decompress, CompressionLevel, CompressionStats, CompressionType},
    io::{AsyncIgtlClient, AsyncIgtlServer},
    protocol::{
        message::IgtlMessage,
        types::{ImageMessage, ScalarType, StatusMessage, TransformMessage},
    },
};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OpenIGTLink Performance Test ===\n");

    test_transform_throughput();
    test_image_throughput();
    test_compression_performance();
    test_network_latency().await?;

    println!("\n=== Performance test completed ===");
    Ok(())
}

fn test_transform_throughput() {
    println!("1. Transform Message Throughput");

    let transform = TransformMessage::identity();
    let msg = IgtlMessage::new(transform, "PerfTest").unwrap();

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let data = msg.encode().unwrap();
        let _: IgtlMessage<TransformMessage> = IgtlMessage::decode(&data).unwrap();
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!("   Iterations: {}", iterations);
    println!("   Time: {:.2}s", elapsed.as_secs_f64());
    println!("   Throughput: {:.0} msg/s", throughput);
    println!("   Latency: {:.2} µs/msg\n", elapsed.as_micros() as f64 / iterations as f64);
}

fn test_image_throughput() {
    println!("2. Image Message Throughput (512x512 grayscale)");

    let mut image = ImageMessage::new();
    image.set_dimensions([512, 512, 1]);
    image.set_scalar_type(ScalarType::Uint8);
    image.set_image_data(vec![128u8; 512 * 512]);
    let msg = IgtlMessage::new(image, "PerfTest").unwrap();

    let iterations = 1_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let data = msg.encode().unwrap();
        let _: IgtlMessage<ImageMessage> = IgtlMessage::decode(&data).unwrap();
    }

    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();
    let bandwidth = (512 * 512) as f64 * throughput / 1_000_000.0; // MB/s

    println!("   Iterations: {}", iterations);
    println!("   Time: {:.2}s", elapsed.as_secs_f64());
    println!("   Throughput: {:.0} msg/s", throughput);
    println!("   Bandwidth: {:.1} MB/s", bandwidth);
    println!("   Latency: {:.2} ms/msg\n", elapsed.as_millis() as f64 / iterations as f64);
}

fn test_compression_performance() {
    println!("3. Compression Performance (100KB data)");

    let data: Vec<u8> = (0..100_000).map(|i| (i / 100) as u8).collect();

    // Test compression
    let start = Instant::now();
    let compressed = compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();
    let compress_time = start.elapsed();

    // Test decompression
    let start = Instant::now();
    let decompressed = decompress(&compressed, CompressionType::Deflate).unwrap();
    let decompress_time = start.elapsed();

    assert_eq!(data, decompressed);

    let stats = CompressionStats::calculate(
        data.len(),
        compressed.len(),
        CompressionType::Deflate,
        CompressionLevel::Default,
    );

    println!("   Original size: {} KB", data.len() / 1024);
    println!("   Compressed size: {} KB", compressed.len() / 1024);
    println!("   Compression ratio: {:.1}%", stats.ratio_percent());
    println!("   Compression time: {:.2} ms", compress_time.as_micros() as f64 / 1000.0);
    println!("   Decompression time: {:.2} ms", decompress_time.as_micros() as f64 / 1000.0);
    println!("   Compression speed: {:.1} MB/s", data.len() as f64 / compress_time.as_secs_f64() / 1_000_000.0);
    println!("   Decompression speed: {:.1} MB/s\n", data.len() as f64 / decompress_time.as_secs_f64() / 1_000_000.0);
}

async fn test_network_latency() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Network Latency (Async, localhost)");

    let server = AsyncIgtlServer::bind("127.0.0.1:0").await?;
    let addr = server.local_addr()?;

    // Spawn server
    let server_task = tokio::spawn(async move {
        let mut conn = server.accept().await.unwrap();
        for _ in 0..100 {
            let msg: IgtlMessage<StatusMessage> = conn.receive().await.unwrap();
            conn.send(&msg).await.unwrap();
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Client
    let mut client = AsyncIgtlClient::connect(&addr.to_string()).await?;

    let status = StatusMessage::ok("Ping");
    let msg = IgtlMessage::new(status, "PerfTest")?;

    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        client.send(&msg).await?;
        let _: IgtlMessage<StatusMessage> = client.receive().await?;
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed.as_micros() as f64 / iterations as f64 / 1000.0; // ms

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:.2}s", elapsed.as_secs_f64());
    println!("   Average round-trip latency: {:.2} ms", avg_latency);
    println!("   Throughput: {:.0} round-trips/s", iterations as f64 / elapsed.as_secs_f64());

    server_task.await?;

    Ok(())
}
