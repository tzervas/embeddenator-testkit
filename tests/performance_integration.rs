//! Performance integration tests
//!
//! Tests performance characteristics of integrated workflows.

#![cfg(feature = "integration")]

use embeddenator_io::{read_bincode_file, write_bincode_file};
use embeddenator_retrieval::{BruteForceIndex, HierarchicalIndex, IndexConfig, RetrievalIndex};
use embeddenator_testkit::random_sparse_vec;
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec};
use std::time::Instant;
use tempfile::tempdir;

/// Test: Ingest throughput
/// Measures data processing rate for ingestion workflow
#[test]
fn test_ingest_throughput() {
    let mut rng = rand::rng();
    let dir = tempdir().unwrap();

    let num_vectors = 100;
    let vector_size = 10000;
    let sparsity = 200;

    let start = Instant::now();

    // Generate vectors
    let vectors: Vec<SparseVec> = (0..num_vectors)
        .map(|_| random_sparse_vec(&mut rng, vector_size, sparsity))
        .collect();

    // Save vectors
    let path = dir.path().join("vectors.bin");
    write_bincode_file(&path, &vectors).unwrap();

    let elapsed = start.elapsed();
    let total_us = elapsed.as_micros() as f64;
    let us_per_op = total_us / num_vectors as f64;
    let ops_per_sec = 1_000_000.0 / us_per_op;

    eprintln!(
        "Ingest: {} vectors in {:.3}ms ({:.1}µs/vec, {:.0} vec/sec)",
        num_vectors,
        total_us / 1000.0,
        us_per_op,
        ops_per_sec
    );

    // Should be reasonably fast (at least 10 vectors/sec on any reasonable hardware)
    assert!(
        ops_per_sec > 10.0,
        "Ingest too slow: {} ops/sec",
        ops_per_sec
    );
}

/// Test: Query throughput
/// Measures query processing rate
#[test]
fn test_query_throughput() {
    let config = ReversibleVSAConfig::default();

    // Build index
    let mut index = BruteForceIndex::new(IndexConfig::default());
    for i in 0..100 {
        let data = format!("document-{}", i);
        let vec = SparseVec::encode_data(data.as_bytes(), &config, None);
        index.add(i, &vec);
    }
    index.finalize();

    // Generate queries
    let queries: Vec<SparseVec> = (0..50)
        .map(|i| {
            let data = format!("query-{}", i);
            SparseVec::encode_data(data.as_bytes(), &config, None)
        })
        .collect();

    // Measure query throughput
    let start = Instant::now();
    for query in &queries {
        let _ = index.query_top_k(query, 10);
    }
    let elapsed = start.elapsed();

    let total_us = elapsed.as_micros() as f64;
    let us_per_query = total_us / queries.len() as f64;
    let qps = 1_000_000.0 / us_per_query;

    eprintln!(
        "Query: {} queries in {:.3}ms ({:.1}µs/query, {:.0} qps)",
        queries.len(),
        total_us / 1000.0,
        us_per_query,
        qps
    );

    // Should be reasonably fast
    assert!(qps > 100.0, "Query too slow: {} qps", qps);
}

/// Test: Index build scalability
#[test]
fn test_index_build_scalability() {
    let config = ReversibleVSAConfig::default();

    for size in [10, 100, 500] {
        let mut index = HierarchicalIndex::new(IndexConfig {
            hierarchical: true,
            ..IndexConfig::default()
        });

        let start = Instant::now();
        for i in 0..size {
            let data = format!("doc-{}", i);
            let vec = SparseVec::encode_data(data.as_bytes(), &config, None);
            index.add(i, &vec);
        }
        index.finalize();
        let elapsed = start.elapsed();

        let total_us = elapsed.as_micros() as f64;
        let us_per_vec = total_us / size as f64;
        let rate = 1_000_000.0 / us_per_vec;

        eprintln!(
            "Index build n={}: {:.3}ms ({:.1}µs/vec, {:.0} vec/sec)",
            size,
            total_us / 1000.0,
            us_per_vec,
            rate
        );
    }
}

/// Test: Serialization performance
#[test]
fn test_serialization_performance() {
    let config = ReversibleVSAConfig::default();
    let dir = tempdir().unwrap();

    // Generate test data
    let vectors: Vec<SparseVec> = (0..100)
        .map(|i| {
            let data = format!("test-document-{}", i);
            SparseVec::encode_data(data.as_bytes(), &config, None)
        })
        .collect();

    let path = dir.path().join("perf_test.bin");

    // Measure write
    let start = Instant::now();
    write_bincode_file(&path, &vectors).unwrap();
    let write_elapsed = start.elapsed();

    // Measure read
    let start = Instant::now();
    let _: Vec<SparseVec> = read_bincode_file(&path).unwrap();
    let read_elapsed = start.elapsed();

    let write_us = write_elapsed.as_micros() as f64;
    let read_us = read_elapsed.as_micros() as f64;

    eprintln!(
        "Serialization 100 vectors: Write {:.3}ms ({:.0}µs), Read {:.3}ms ({:.0}µs)",
        write_us / 1000.0,
        write_us,
        read_us / 1000.0,
        read_us
    );

    // Both should be reasonably fast
    assert!(write_elapsed.as_millis() < 1000, "Write too slow");
    assert!(read_elapsed.as_millis() < 1000, "Read too slow");
}

/// Test: Memory efficiency with large batches
#[test]
fn test_memory_efficiency() {
    let mut rng = rand::rng();

    // Process in batches to avoid memory spikes
    let batch_size = 100;
    let num_batches = 5;

    let mut total_vectors = 0;

    for _ in 0..num_batches {
        let batch: Vec<SparseVec> = (0..batch_size)
            .map(|_| random_sparse_vec(&mut rng, 10000, 200))
            .collect();

        // Build index for batch
        let mut index = BruteForceIndex::new(IndexConfig::default());
        for (i, vec) in batch.iter().enumerate() {
            index.add(total_vectors + i, vec);
        }
        index.finalize();

        // Query batch
        let query = &batch[0];
        let _ = index.query_top_k(query, 10);

        total_vectors += batch_size;
    }

    assert_eq!(total_vectors, batch_size * num_batches);
}

/// Test: Concurrent-like workload
#[test]
fn test_interleaved_operations() {
    let config = ReversibleVSAConfig::default();
    let dir = tempdir().unwrap();

    // Interleave different operations
    for i in 0..20 {
        // Create vector
        let data = format!("item-{}", i);
        let vec = SparseVec::encode_data(data.as_bytes(), &config, None);

        // Save
        let path = dir.path().join(format!("vec_{}.bin", i));
        write_bincode_file(&path, &vec).unwrap();

        // Load
        let loaded: SparseVec = read_bincode_file(&path).unwrap();

        // Verify
        assert_eq!(vec.pos, loaded.pos);
        assert_eq!(vec.neg, loaded.neg);
    }
}
