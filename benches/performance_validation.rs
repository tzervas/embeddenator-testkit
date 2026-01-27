use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec};
use std::hint::black_box;

/// Comprehensive performance validation benchmark
///
/// Validates the effectiveness of bt-phase-2 + SIMD optimizations
/// across different vector densities and operation types.
fn bench_vsa_operations_optimized(c: &mut Criterion) {
    let mut group = c.benchmark_group("vsa_operations_optimized");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let config = ReversibleVSAConfig::default();

    // Test data: varied content for realistic performance measurement
    let sparse_text = b"Hello world, this is a test message for sparse vector encoding.".to_vec();
    let medium_text = b"This is a longer piece of text that should create medium-density vectors with some repetition and varied content patterns.".to_vec();
    let dense_binary = vec![b'x'; 1000]; // High repetition = dense vectors

    let test_cases: Vec<(&str, &[u8])> = vec![
        ("sparse_text", &sparse_text),
        ("medium_text", &medium_text),
        ("dense_binary", &dense_binary),
    ];

    for (name, data) in &test_cases {
        let vec = SparseVec::encode_data(data, &config, None);

        // Bundle operations (most critical for hierarchical encoding)
        group.bench_with_input(BenchmarkId::new("bundle", *name), &vec, |bencher, vec| {
            bencher.iter(|| black_box(vec).bundle(black_box(vec)))
        });

        // Bind operations (compositional operations)
        group.bench_with_input(BenchmarkId::new("bind", *name), &vec, |bencher, vec| {
            bencher.iter(|| black_box(vec).bind(black_box(vec)))
        });

        // Cosine similarity (query operations - SIMD accelerated)
        group.bench_with_input(BenchmarkId::new("cosine", *name), &vec, |bencher, vec| {
            bencher.iter(|| black_box(vec).cosine(black_box(vec)))
        });
    }

    group.finish();
}

/// Memory efficiency validation
///
/// Measures memory usage patterns and allocation efficiency
/// of optimized operations.
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let config = ReversibleVSAConfig::default();

    // Test different data sizes
    let sizes: Vec<usize> = vec![100, 1000, 10000, 100000];

    for size in sizes {
        let data = vec![b'a'; size];
        let vec = SparseVec::encode_data(&data, &config, None);

        group.bench_with_input(
            BenchmarkId::new("encode_memory", size),
            &size,
            |bencher, _size| {
                bencher.iter(|| {
                    let v = SparseVec::encode_data(black_box(&data), &config, None);
                    black_box(v)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decode_memory", size),
            &vec,
            |bencher, vec| {
                bencher.iter(|| {
                    let result = black_box(vec).decode_data(&config, None, data.len());
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Scalability validation
///
/// Tests how operations scale with vector complexity
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let config = ReversibleVSAConfig::default();

    // Create vectors of increasing complexity
    let small = b"small".to_vec();
    let medium = b"medium sized data for testing".to_vec();
    let large = b"very large data with lots of content and complexity that should create more intricate vector representations".to_vec();
    let complex = b"extremely complex data with high entropy and varied patterns that will stress the VSA encoding and create dense vector representations requiring optimized operations".to_vec();

    let complexities: Vec<(&str, &[u8])> = vec![
        ("small", &small),
        ("medium", &medium),
        ("large", &large),
        ("complex", &complex),
    ];

    for (name, data) in &complexities {
        let vec = SparseVec::encode_data(data, &config, None);

        group.bench_with_input(
            BenchmarkId::new("complexity_scaling", *name),
            &vec,
            |bencher, vec| {
                // Chain operations to test cumulative performance
                bencher.iter(|| {
                    let mut result = vec.clone();
                    for _ in 0..5 {
                        result = result.bundle(vec);
                    }
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_vsa_operations_optimized,
    bench_memory_efficiency,
    bench_scalability
);
criterion_main!(benches);
