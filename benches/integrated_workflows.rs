//! Integrated workflow benchmarks
//!
//! Criterion benchmarks for cross-component workflows.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use embeddenator_io::{read_bincode_file, write_bincode_file};
use embeddenator_retrieval::{IndexBuilder, QueryEngine};
use embeddenator_testkit::{random_sparse_vec, SparseVec};
use std::hint::black_box;
use std::hint::black_box;
use tempfile::tempdir;

fn bench_ingest_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest_workflow");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut rng = rand::rng();
            let dir = tempdir().unwrap();

            b.iter(|| {
                for i in 0..size {
                    let vec = random_sparse_vec(&mut rng, 10000, 150);
                    let path = dir.path().join(format!("vec{}.bin", i));
                    write_bincode_file(&path, &vec).unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_query_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_workflow");
    let mut rng = rand::rng();

    // Build index
    let mut index = IndexBuilder::new();
    for i in 0..100 {
        let vec = random_sparse_vec(&mut rng, 10000, 150);
        index.add_vector(format!("doc{}", i), vec);
    }
    let index = index.build();
    let engine = QueryEngine::new(index);

    let query = random_sparse_vec(&mut rng, 10000, 150);

    for k in [1, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(k), k, |b, &k| {
            b.iter(|| {
                black_box(engine.top_k(&query, k));
            });
        });
    }

    group.finish();
}

fn bench_e2e_workflow(c: &mut Criterion) {
    c.bench_function("e2e_complete_workflow", |b| {
        let mut rng = rand::rng();
        let dir = tempdir().unwrap();

        b.iter(|| {
            // Ingest
            let mut vectors = Vec::new();
            for i in 0..20 {
                let vec = random_sparse_vec(&mut rng, 10000, 150);
                let path = dir.path().join(format!("doc{}.bin", i));
                write_bincode_file(&path, &vec).unwrap();
                vectors.push((format!("doc{}", i), path));
            }

            // Index
            let mut index = IndexBuilder::new();
            for (id, path) in &vectors {
                let vec: SparseVec = read_bincode_file(path).unwrap();
                index.add_vector(id.clone(), vec);
            }
            let index = index.build();

            // Query
            let engine = QueryEngine::new(index);
            let query = random_sparse_vec(&mut rng, 10000, 150);
            black_box(engine.top_k(&query, 5));
        });
    });
}

fn bench_vsa_io_roundtrip(c: &mut Criterion) {
    c.bench_function("vsa_io_roundtrip", |b| {
        let mut rng = rand::rng();
        let dir = tempdir().unwrap();
        let path = dir.path().join("vec.bin");

        b.iter(|| {
            let vec = random_sparse_vec(&mut rng, 10000, 150);
            write_bincode_file(&path, &vec).unwrap();
            let _: SparseVec = read_bincode_file(&path).unwrap();
        });
    });
}

fn bench_index_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_build");
    let mut rng = rand::rng();

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            // Pre-generate vectors
            let mut vectors = Vec::new();
            for i in 0..size {
                let vec = random_sparse_vec(&mut rng, 10000, 150);
                vectors.push((format!("doc{}", i), vec));
            }

            b.iter(|| {
                let mut index = IndexBuilder::new();
                for (id, vec) in &vectors {
                    index.add_vector(id.clone(), vec.clone());
                }
                black_box(index.build());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_ingest_workflow,
    bench_query_workflow,
    bench_e2e_workflow,
    bench_vsa_io_roundtrip,
    bench_index_build,
);

criterion_main!(benches);
