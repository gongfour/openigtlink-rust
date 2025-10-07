//! Compression performance benchmarks
//!
//! Measures compression/decompression speed and ratios.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use openigtlink_rust::compression::{compress, decompress, CompressionLevel, CompressionType};

fn bench_compression_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_deflate");

    let sizes = vec![
        (1024, "1KB"),
        (10 * 1024, "10KB"),
        (100 * 1024, "100KB"),
        (1024 * 1024, "1MB"),
    ];

    for (size, name) in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        // Use gradient data (moderately compressible)
        let data: Vec<u8> = (0..size).map(|i| (i / 100) as u8).collect();

        group.bench_with_input(BenchmarkId::from_parameter(name), &data, |b, data| {
            b.iter(|| {
                black_box(
                    compress(data, CompressionType::Deflate, CompressionLevel::Default).unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_decompression_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompression_deflate");

    let sizes = vec![
        (1024, "1KB"),
        (10 * 1024, "10KB"),
        (100 * 1024, "100KB"),
        (1024 * 1024, "1MB"),
    ];

    for (size, name) in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        let data: Vec<u8> = (0..size).map(|i| (i / 100) as u8).collect();
        let compressed =
            compress(&data, CompressionType::Deflate, CompressionLevel::Default).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &compressed,
            |b, compressed| {
                b.iter(|| black_box(decompress(compressed, CompressionType::Deflate).unwrap()));
            },
        );
    }

    group.finish();
}

fn bench_compression_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_levels");

    let data: Vec<u8> = (0..100_000).map(|i| (i / 100) as u8).collect();
    group.throughput(Throughput::Bytes(data.len() as u64));

    let levels = vec![
        (CompressionLevel::Fast, "fast"),
        (CompressionLevel::Default, "default"),
        (CompressionLevel::Best, "best"),
    ];

    for (level, name) in levels {
        group.bench_with_input(BenchmarkId::from_parameter(name), &data, |b, data| {
            b.iter(|| black_box(compress(data, CompressionType::Deflate, level).unwrap()));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_compression_by_size,
    bench_decompression_by_size,
    bench_compression_levels
);

criterion_main!(benches);
