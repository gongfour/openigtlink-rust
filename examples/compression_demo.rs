//! Compression demonstration for large medical images
//!
//! Shows how to compress and decompress large message data to reduce
//! network bandwidth usage.

use openigtlink_rust::compression::{
    compress, decompress, CompressionLevel, CompressionStats, CompressionType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OpenIGTLink Compression Demo ===\n");

    // Simulate different types of medical data
    demonstrate_highly_compressible();
    demonstrate_moderately_compressible();
    demonstrate_random_data();
    compare_compression_levels();
    compare_algorithms();

    println!("\n=== Demo completed successfully ===");
    Ok(())
}

fn demonstrate_highly_compressible() {
    println!("1. Highly Compressible Data (zeros - like empty image regions)");
    let data = vec![0u8; 100_000]; // 100KB of zeros

    let compressed = compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();
    let stats = CompressionStats::calculate(
        data.len(),
        compressed.len(),
        CompressionType::Deflate,
        CompressionLevel::Default,
    );

    println!("   Original size: {} bytes", stats.original_size);
    println!("   Compressed size: {} bytes", stats.compressed_size);
    println!("   Compression ratio: {:.1}%", stats.ratio_percent());
    println!("   Space saved: {:.1}%", stats.space_saved_percent());

    // Verify decompression
    let decompressed = decompress(&compressed, CompressionType::Deflate).unwrap();
    assert_eq!(data, decompressed);
    println!("   ✓ Decompression verified\n");
}

fn demonstrate_moderately_compressible() {
    println!("2. Moderately Compressible Data (gradient - like real images)");

    // Simulate a gradient image (more realistic medical imaging data)
    let data: Vec<u8> = (0..100_000).map(|i| (i / 100) as u8).collect();

    let compressed = compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();
    let stats = CompressionStats::calculate(
        data.len(),
        compressed.len(),
        CompressionType::Deflate,
        CompressionLevel::Default,
    );

    println!("   Original size: {} bytes", stats.original_size);
    println!("   Compressed size: {} bytes", stats.compressed_size);
    println!("   Compression ratio: {:.1}%", stats.ratio_percent());
    println!("   Space saved: {:.1}%", stats.space_saved_percent());

    let decompressed = decompress(&compressed, CompressionType::Deflate).unwrap();
    assert_eq!(data, decompressed);
    println!("   ✓ Decompression verified\n");
}

fn demonstrate_random_data() {
    println!("3. Random Data (low compressibility - like encrypted or noisy data)");

    // Random-like data (hard to compress)
    let data: Vec<u8> = (0..100_000).map(|i| (i * 37 % 256) as u8).collect();

    let compressed = compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();
    let stats = CompressionStats::calculate(
        data.len(),
        compressed.len(),
        CompressionType::Deflate,
        CompressionLevel::Default,
    );

    println!("   Original size: {} bytes", stats.original_size);
    println!("   Compressed size: {} bytes", stats.compressed_size);
    println!("   Compression ratio: {:.1}%", stats.ratio_percent());
    println!(
        "   Space saved: {:.1}%",
        stats.space_saved_percent()
    );

    let decompressed = decompress(&compressed, CompressionType::Deflate).unwrap();
    assert_eq!(data, decompressed);
    println!("   ✓ Decompression verified\n");
}

fn compare_compression_levels() {
    println!("4. Comparing Compression Levels (on gradient data)");

    let data: Vec<u8> = (0..100_000).map(|i| (i / 100) as u8).collect();

    let levels = [
        ("Fast", CompressionLevel::Fast),
        ("Default", CompressionLevel::Default),
        ("Best", CompressionLevel::Best),
    ];

    for (name, level) in &levels {
        let compressed = compress(&data, CompressionType::Deflate, *level).unwrap();
        let stats = CompressionStats::calculate(
            data.len(),
            compressed.len(),
            CompressionType::Deflate,
            *level,
        );

        println!(
            "   {:8} - Size: {:6} bytes ({:.1}% ratio, {:.1}% saved)",
            name,
            stats.compressed_size,
            stats.ratio_percent(),
            stats.space_saved_percent()
        );
    }
    println!();
}

fn compare_algorithms() {
    println!("5. Comparing Compression Algorithms (on gradient data)");

    let data: Vec<u8> = (0..100_000).map(|i| (i / 100) as u8).collect();

    let algorithms = [
        ("None", CompressionType::None),
        ("Deflate", CompressionType::Deflate),
        ("Gzip", CompressionType::Gzip),
    ];

    for (name, algorithm) in &algorithms {
        let compressed = compress(&data, *algorithm, CompressionLevel::Default).unwrap();
        let stats = CompressionStats::calculate(
            data.len(),
            compressed.len(),
            *algorithm,
            CompressionLevel::Default,
        );

        println!(
            "   {:8} - Size: {:6} bytes ({:.1}% ratio, {:.1}% saved)",
            name,
            stats.compressed_size,
            stats.ratio_percent(),
            stats.space_saved_percent()
        );
    }
    println!();
}
