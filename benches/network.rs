//! Network I/O benchmarks
//!
//! Measures end-to-end network performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use openigtlink_rust::{
    io::{AsyncIgtlClient, AsyncIgtlServer},
    protocol::{message::IgtlMessage, types::StatusMessage},
};
use tokio::runtime::Runtime;

fn bench_async_roundtrip(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("async_roundtrip_status", |b| {
        b.to_async(&rt).iter(|| async {
            // Create server
            let server = AsyncIgtlServer::bind("127.0.0.1:0").await.unwrap();
            let addr = server.local_addr().unwrap();

            // Spawn server task
            let server_task = tokio::spawn(async move {
                let mut conn = server.accept().await.unwrap();
                let _: IgtlMessage<StatusMessage> = conn.receive().await.unwrap();

                let response = StatusMessage::ok("Response");
                let msg = IgtlMessage::new(response, "Server").unwrap();
                conn.send(&msg).await.unwrap();
            });

            // Client
            let mut client = AsyncIgtlClient::connect(&addr.to_string()).await.unwrap();

            let status = StatusMessage::ok("Request");
            let msg = IgtlMessage::new(status, "Client").unwrap();
            client.send(&msg).await.unwrap();

            let _: IgtlMessage<StatusMessage> = client.receive().await.unwrap();

            server_task.await.unwrap();
            black_box(())
        });
    });
}

fn bench_async_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("async_throughput_10_messages", |b| {
        b.to_async(&rt).iter(|| async {
            let server = AsyncIgtlServer::bind("127.0.0.1:0").await.unwrap();
            let addr = server.local_addr().unwrap();

            let server_task = tokio::spawn(async move {
                let mut conn = server.accept().await.unwrap();
                for _ in 0..10 {
                    let _: IgtlMessage<StatusMessage> = conn.receive().await.unwrap();
                }
            });

            let mut client = AsyncIgtlClient::connect(&addr.to_string()).await.unwrap();

            let status = StatusMessage::ok("Test");
            let msg = IgtlMessage::new(status, "Client").unwrap();

            for _ in 0..10 {
                client.send(&msg).await.unwrap();
            }

            server_task.await.unwrap();
            black_box(())
        });
    });
}

criterion_group!(benches, bench_async_roundtrip, bench_async_throughput);

criterion_main!(benches);
