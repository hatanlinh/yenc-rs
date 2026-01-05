use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use yenc;

fn create_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_decode_various_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");

    for size in [1024, 10_240, 102_400, 1_024_000].iter() {
        // Create encoded test data
        let original = create_test_data(*size);
        let mut encoded = Vec::new();
        yenc::encode(&original[..], &mut encoded, "bench.bin").unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut output = Vec::with_capacity(*size);
                yenc::decode(black_box(&encoded[..]), &mut output).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_decode_worst_case(c: &mut Criterion) {
    // Data that requires maximum escaping
    let original = vec![0x00u8; 10_240]; // All NULL bytes need escaping
    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "worst.bin").unwrap();

    c.bench_function("decode_worst_case", |b| {
        b.iter(|| {
            let mut output = Vec::with_capacity(10_240);
            yenc::decode(black_box(&encoded[..]), &mut output).unwrap();
        });
    });
}

fn bench_decode_best_case(c: &mut Criterion) {
    // Data that requires no escaping
    let original = vec![0x41u8; 10_240]; // All 'A' characters
    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "best.bin").unwrap();

    c.bench_function("decode_best_case", |b| {
        b.iter(|| {
            let mut output = Vec::with_capacity(10_240);
            yenc::decode(black_box(&encoded[..]), &mut output).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_decode_various_sizes,
    bench_decode_worst_case,
    bench_decode_best_case
);
criterion_main!(benches);
