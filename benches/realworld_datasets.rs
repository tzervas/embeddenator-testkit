//! Benchmarks for real-world dataset operations
//!
//! Tests VSA operations against various data types and sizes.
//! Enable with: cargo bench --features realworld-datasets

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec, VsaConfig};
use std::hint::black_box;

/// Benchmark VSA operations with different vector dimensions
fn bench_dimension_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("dimension_scaling");

    let configs = [
        ("small_1k", VsaConfig::new(1_000)),
        ("medium_10k", VsaConfig::medium()),
        ("large_100k", VsaConfig::large()),
    ];

    for (name, config) in configs.iter() {
        group.throughput(Throughput::Elements(config.dimension as u64));

        // Benchmark random vector generation
        group.bench_with_input(BenchmarkId::new("random_gen", name), config, |b, cfg| {
            b.iter(|| black_box(SparseVec::random_with_config(cfg)));
        });

        // Benchmark bundle operation
        let vec_a = SparseVec::random_with_config(config);
        let vec_b = SparseVec::random_with_config(config);

        group.bench_with_input(
            BenchmarkId::new("bundle", name),
            &(&vec_a, &vec_b),
            |bencher, (va, vb)| {
                bencher.iter(|| black_box(va.bundle(vb)));
            },
        );

        // Benchmark bind operation
        group.bench_with_input(
            BenchmarkId::new("bind", name),
            &(&vec_a, &vec_b),
            |bencher, (va, vb)| {
                bencher.iter(|| black_box(va.bind(vb)));
            },
        );

        // Benchmark cosine similarity
        group.bench_with_input(
            BenchmarkId::new("cosine", name),
            &(&vec_a, &vec_b),
            |bencher, (va, vb)| {
                bencher.iter(|| black_box(va.cosine(vb)));
            },
        );
    }

    group.finish();
}

/// Benchmark encoding different data sizes
fn bench_data_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_encoding");
    group.sample_size(20); // Reduce for longer operations

    let sizes = [
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    let config = ReversibleVSAConfig::default();

    for (name, size) in sizes.iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        // Generate test data
        let data: Vec<u8> = (0..*size).map(|i| (i % 256) as u8).collect();

        group.bench_with_input(BenchmarkId::new("encode", name), &data, |bencher, d| {
            bencher.iter(|| black_box(SparseVec::encode_data(d, &config, None)));
        });
    }

    group.finish();
}

/// Benchmark with different sparsity levels
fn bench_sparsity_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparsity_impact");

    let densities = [("0.5%", 0.005), ("1%", 0.01), ("2%", 0.02), ("5%", 0.05)];

    for (name, density) in densities.iter() {
        let config = VsaConfig::new(10_000).with_density(*density);
        let sparsity = config.sparsity();

        group.throughput(Throughput::Elements(sparsity as u64 * 2));

        let vec_a = SparseVec::random_with_config(&config);
        let vec_b = SparseVec::random_with_config(&config);

        group.bench_with_input(
            BenchmarkId::new("bundle", name),
            &(&vec_a, &vec_b),
            |bencher, (va, vb)| {
                bencher.iter(|| black_box(va.bundle(vb)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("bind", name),
            &(&vec_a, &vec_b),
            |bencher, (va, vb)| {
                bencher.iter(|| black_box(va.bind(vb)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cosine", name),
            &(&vec_a, &vec_b),
            |bencher, (va, vb)| {
                bencher.iter(|| black_box(va.cosine(vb)));
            },
        );
    }

    group.finish();
}

/// Benchmark batch operations (multi-vector bundling)
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.sample_size(20);

    let batch_sizes = [10, 100, 1000];
    let config = VsaConfig::medium();

    for batch_size in batch_sizes.iter() {
        let vectors: Vec<SparseVec> = (0..*batch_size)
            .map(|_| SparseVec::random_with_config(&config))
            .collect();

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("bundle_many", batch_size),
            &vectors,
            |bencher, vecs| {
                bencher.iter(|| {
                    // Bundle all vectors together
                    black_box(SparseVec::bundle_sum_many(vecs.iter()))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_dimension_scaling,
    bench_data_encoding,
    bench_sparsity_impact,
    bench_batch_operations,
);

criterion_main!(benches);
