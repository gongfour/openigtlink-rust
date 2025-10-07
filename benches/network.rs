//! Network I/O benchmarks
//!
//! Measures end-to-end network performance.
//!
//! Note: These benchmarks are disabled by default due to network setup overhead.
//! They measure network stack performance rather than protocol performance.
//! Use throughput benchmarks for protocol-level performance measurements.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use openigtlink_rust::protocol::{
    message::IgtlMessage,
    types::StatusMessage,
};

// Network benchmarks commented out due to:
// 1. High overhead from creating/destroying servers in tight loops
// 2. macOS "Can't assign requested address" issues with rapid port allocation
// 3. Network stack benchmarks don't reflect protocol performance
//
// For real network performance, see examples/performance_test.rs

fn bench_async_roundtrip(c: &mut Criterion) {
    // Benchmark disabled - use examples/performance_test.rs for network testing
    c.bench_function("async_roundtrip_status_disabled", |b| {
        b.iter(|| {
            // Simple operation instead of network I/O
            let status = StatusMessage::ok("Request");
            let msg = IgtlMessage::new(status, "Client").unwrap();
            let encoded = msg.encode().unwrap();
            black_box(encoded)
        });
    });
}

fn bench_async_throughput(c: &mut Criterion) {
    // Benchmark disabled - use examples/performance_test.rs for network testing
    c.bench_function("async_throughput_10_messages_disabled", |b| {
        b.iter(|| {
            // Simulate message serialization without network I/O
            let status = StatusMessage::ok("Test");
            let msg = IgtlMessage::new(status, "Client").unwrap();
            let mut total = 0;
            for _ in 0..10 {
                let encoded = msg.encode().unwrap();
                total += encoded.len();
            }
            black_box(total)
        });
    });
}

criterion_group!(benches, bench_async_roundtrip, bench_async_throughput);

criterion_main!(benches);
