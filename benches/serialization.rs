//! Serialization benchmarks
//!
//! Measures message serialization performance (encoding without network I/O).
//!
//! Note: Originally network benchmarks, but network I/O is unsuitable for
//! Criterion's tight iteration loops due to port allocation overhead.
//! Use examples/performance_test.rs for real network performance testing.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use openigtlink_rust::protocol::{
    message::IgtlMessage,
    types::StatusMessage,
};

fn bench_status_serialization(c: &mut Criterion) {
    c.bench_function("status_message_serialize", |b| {
        b.iter(|| {
            let status = StatusMessage::ok("Request");
            let msg = IgtlMessage::new(status, "Client").unwrap();
            let encoded = msg.encode().unwrap();
            black_box(encoded)
        });
    });
}

fn bench_batch_serialization(c: &mut Criterion) {
    c.bench_function("batch_10_messages_serialize", |b| {
        b.iter(|| {
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

criterion_group!(benches, bench_status_serialization, bench_batch_serialization);

criterion_main!(benches);
