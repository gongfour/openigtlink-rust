//! Throughput benchmarks for OpenIGTLink message types
//!
//! Measures messages per second and bandwidth for different message sizes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use openigtlink_rust::protocol::{
    message::IgtlMessage,
    types::{ImageMessage, ImageScalarType, StatusMessage, TransformMessage},
};

fn bench_transform_encode(c: &mut Criterion) {
    let transform = TransformMessage::identity();
    let msg = IgtlMessage::new(transform, "Benchmark").unwrap();

    c.bench_function("transform_encode", |b| {
        b.iter(|| black_box(msg.encode().unwrap()))
    });
}

fn bench_transform_decode(c: &mut Criterion) {
    let transform = TransformMessage::identity();
    let msg = IgtlMessage::new(transform, "Benchmark").unwrap();
    let data = msg.encode().unwrap();

    c.bench_function("transform_decode", |b| {
        b.iter(|| {
            let _: IgtlMessage<TransformMessage> = black_box(IgtlMessage::decode(&data).unwrap());
        })
    });
}

fn bench_status_encode(c: &mut Criterion) {
    let status = StatusMessage::ok("Test status message");
    let msg = IgtlMessage::new(status, "Benchmark").unwrap();

    c.bench_function("status_encode", |b| {
        b.iter(|| black_box(msg.encode().unwrap()))
    });
}

fn bench_status_decode(c: &mut Criterion) {
    let status = StatusMessage::ok("Test status message");
    let msg = IgtlMessage::new(status, "Benchmark").unwrap();
    let data = msg.encode().unwrap();

    c.bench_function("status_decode", |b| {
        b.iter(|| {
            let _: IgtlMessage<StatusMessage> = black_box(IgtlMessage::decode(&data).unwrap());
        })
    });
}

fn bench_image_encode_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_encode");

    let sizes = vec![
        (64, 64, "64x64"),
        (256, 256, "256x256"),
        (512, 512, "512x512"),
        (1024, 1024, "1024x1024"),
    ];

    for (width, height, name) in sizes {
        let size = width * height;
        group.throughput(Throughput::Bytes((size) as u64));

        let image = ImageMessage::new(
            ImageScalarType::Uint8,
            [width as u16, height as u16, 1],
            vec![128u8; size],
        )
        .unwrap();
        let msg = IgtlMessage::new(image, "Benchmark").unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(name), &msg, |b, msg| {
            b.iter(|| black_box(msg.encode().unwrap()));
        });
    }

    group.finish();
}

fn bench_image_decode_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_decode");

    let sizes = vec![
        (64, 64, "64x64"),
        (256, 256, "256x256"),
        (512, 512, "512x512"),
        (1024, 1024, "1024x1024"),
    ];

    for (width, height, name) in sizes {
        let size = width * height;
        group.throughput(Throughput::Bytes((size) as u64));

        let image = ImageMessage::new(
            ImageScalarType::Uint8,
            [width as u16, height as u16, 1],
            vec![128u8; size],
        )
        .unwrap();
        let msg = IgtlMessage::new(image, "Benchmark").unwrap();
        let data = msg.encode().unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(name), &data, |b, data| {
            b.iter(|| {
                let _: IgtlMessage<ImageMessage> = black_box(IgtlMessage::decode(data).unwrap());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_transform_encode,
    bench_transform_decode,
    bench_status_encode,
    bench_status_decode,
    bench_image_encode_by_size,
    bench_image_decode_by_size
);

criterion_main!(benches);
