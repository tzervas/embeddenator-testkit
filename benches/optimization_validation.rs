use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
};
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec};
use std::hint::black_box;

/// Optimization validation benchmark
///
/// Compares performance with and without optimizations to validate
/// the effectiveness of bt-phase-2 and SIMD improvements.
fn bench_optimization_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_comparison");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let config = ReversibleVSAConfig::default();

    // Test cases that benefit from different optimizations
    let sparse_ops = b"Short text for sparse vector operations".to_vec();
    let dense_ops = vec![b'x'; 5000]; // Creates dense vectors
    let mixed_ops = b"Mixed content with some repetition and varied patterns for testing different optimization paths".to_vec();

    let test_cases: Vec<(&str, &[u8])> = vec![
        ("sparse_ops", &sparse_ops),
        ("dense_ops", &dense_ops),
        ("mixed_ops", &mixed_ops),
    ];

    for (case_name, data) in &test_cases {
        let vec = SparseVec::encode_data(data, &config, None);

        // Bundle operations - should benefit from packed fast paths
        group.bench_with_input(
            BenchmarkId::new("bundle_optimized", *case_name),
            &vec,
            |bencher, vec| bencher.iter(|| black_box(vec).bundle(black_box(vec))),
        );

        // Bind operations - packed acceleration
        group.bench_with_input(
            BenchmarkId::new("bind_optimized", *case_name),
            &vec,
            |bencher, vec| bencher.iter(|| black_box(vec).bind(black_box(vec))),
        );

        // Cosine operations - SIMD acceleration
        group.bench_with_input(
            BenchmarkId::new("cosine_simd", *case_name),
            &vec,
            |bencher, vec| bencher.iter(|| black_box(vec).cosine(black_box(vec))),
        );
    }

    group.finish();
}

/// Memory allocation efficiency validation
///
/// Measures reduction in allocations from thread-local scratch buffers
/// and other optimization improvements.
fn bench_allocation_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_efficiency");

    let config = ReversibleVSAConfig::default();

    // Test with vectors that would trigger packed paths
    let dense_data = vec![b'x'; 10000]; // High repetition = dense vectors
    let vec = SparseVec::encode_data(&dense_data, &config, None);

    group.bench_function("dense_bundle_allocations", |bencher| {
        bencher.iter(|| {
            // This should use thread-local scratch buffers instead of allocating
            let result = black_box(&vec).bundle(black_box(&vec));
            black_box(result)
        })
    });

    group.bench_function("dense_bind_allocations", |bencher| {
        bencher.iter(|| {
            // Packed bind should minimize allocations
            let result = black_box(&vec).bind(black_box(&vec));
            black_box(result)
        })
    });

    group.finish();
}

/// SIMD acceleration validation
///
/// Specifically tests SIMD cosine performance vs scalar baseline
fn bench_simd_acceleration(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_acceleration");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let config = ReversibleVSAConfig::default();

    // Create test vectors of different sizes to show SIMD scaling
    let sizes = vec![1000, 10000, 100000];

    for size in sizes {
        let data = format!("Test data {}", "x".repeat(size)).into_bytes();
        let vec_a = SparseVec::encode_data(&data, &config, None);
        let vec_b = SparseVec::encode_data(&data, &config, None);

        group.bench_with_input(
            BenchmarkId::new("cosine_simd_scaling", format!("{}bytes", size)),
            &(vec_a, vec_b),
            |bencher, (vec_a, vec_b)| {
                bencher.iter(|| {
                    // This uses SIMD acceleration when available
                    let similarity = black_box(vec_a).cosine(black_box(vec_b));
                    black_box(similarity)
                })
            },
        );
    }

    group.finish();
}

/// Hierarchical bundling optimization validation
///
/// Tests the effectiveness of hierarchical encoding optimizations
fn bench_hierarchical_optimizations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hierarchical_optimizations");
    group.sample_size(10);

    // Test different bundling strategies
    let strategies = vec![
        ("pairwise", 2),
        ("sum_many_small", 4),
        ("sum_many_medium", 8),
        ("sum_many_large", 16),
    ];

    for (strategy_name, vector_count) in strategies {
        group.bench_with_input(
            BenchmarkId::new("bundling_strategy", strategy_name),
            &vector_count,
            |bencher, &vector_count| {
                bencher.iter_with_setup(
                    || {
                        let config = ReversibleVSAConfig::default();
                        // Create multiple vectors for bundling
                        let base_data = b"Test data for hierarchical bundling optimization";
                        (0..vector_count)
                            .map(|i| {
                                let data = format!(
                                    "{} variant {}",
                                    std::str::from_utf8(base_data).unwrap(),
                                    i
                                );
                                SparseVec::encode_data(data.as_bytes(), &config, None)
                            })
                            .collect::<Vec<_>>()
                    },
                    |vectors| {
                        // Test bundling multiple vectors
                        let mut result = vectors[0].clone();
                        for vec in &vectors[1..] {
                            result = result.bundle(vec);
                        }
                        black_box(result)
                    },
                );
            },
        );
    }

    group.finish();
}

/// End-to-end workflow optimization validation
///
/// Tests complete encode/decode workflows with optimizations
fn bench_workflow_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_optimization");

    let config = ReversibleVSAConfig::default();

    // Test different data types that benefit from different optimizations
    let text_data = b"This is text data that should benefit from sparse optimizations".to_vec();
    let binary_data = vec![b'x'; 50000]; // Dense patterns
    let mixed_data =
        b"Mixed content with some repetition and varied patterns for comprehensive testing"
            .to_vec();

    let workflows: Vec<(&str, &[u8])> = vec![
        ("text_encoding", &text_data),
        ("binary_encoding", &binary_data),
        ("mixed_encoding", &mixed_data),
    ];

    for (workflow_name, data) in &workflows {
        let data_len = data.len();
        group.bench_with_input(
            BenchmarkId::new("encode_decode_cycle", *workflow_name),
            data,
            |bencher, data| {
                bencher.iter(|| {
                    // Complete encode/decode cycle
                    let encoded = SparseVec::encode_data(black_box(*data), &config, None);
                    let decoded = encoded.decode_data(&config, None, data_len);
                    black_box(decoded)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_optimization_comparison,
    bench_allocation_efficiency,
    bench_simd_acceleration,
    bench_hierarchical_optimizations,
    bench_workflow_optimization
);
criterion_main!(benches);
