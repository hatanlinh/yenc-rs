use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use yenc;

fn create_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_encode_various_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for size in [1024, 10_240, 102_400, 1_024_000].iter() {
        let data = create_test_data(*size);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut output = Vec::new();
                yenc::encode(black_box(&data[..]), &mut output, "bench.bin").unwrap();
            });
        });
    }

    group.finish();
}

fn bench_encode_worst_case(c: &mut Criterion) {
    // All bytes that need escaping
    let data = vec![0x00u8; 10_240];

    c.bench_function("encode_worst_case", |b| {
        b.iter(|| {
            let mut output = Vec::new();
            yenc::encode(black_box(&data[..]), &mut output, "worst.bin").unwrap();
        });
    });
}

fn bench_encode_best_case(c: &mut Criterion) {
    // No bytes need escaping
    let data = vec![0x41u8; 10_240];

    c.bench_function("encode_best_case", |b| {
        b.iter(|| {
            let mut output = Vec::new();
            yenc::encode(black_box(&data[..]), &mut output, "best.bin").unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_encode_various_sizes,
    bench_encode_worst_case,
    bench_encode_best_case
);
criterion_main!(benches);
