//! Observability integration tests
//!
//! Tests metrics and tracing work correctly across component boundaries.

#![cfg(feature = "integration")]

use embeddenator_retrieval::{BruteForceIndex, IndexConfig, RetrievalIndex};
use embeddenator_testkit::{random_sparse_vec, TestMetrics};
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec};
use std::time::Instant;
use tempfile::tempdir;

/// Test: TestMetrics utility works
#[test]
fn test_test_metrics_utility() {
    let mut metrics = TestMetrics::new("test_operation");

    metrics.start_timing();

    // Do some work
    let mut sum = 0u64;
    for i in 0..1000 {
        sum = sum.wrapping_add(i);
    }
    let _ = sum; // use the value

    metrics.stop_timing();

    // Should have recorded non-zero duration
    let stats = metrics.timing_stats();
    assert!(stats.total_ns > 0);
}

/// Test: TestMetrics multiple samples
#[test]
fn test_test_metrics_multiple_samples() {
    let mut metrics = TestMetrics::new("multi_sample");

    for _ in 0..10 {
        metrics.start_timing();
        let mut sum = 0u64;
        for i in 0..100 {
            sum = sum.wrapping_add(i);
        }
        let _ = sum;
        metrics.stop_timing();
    }

    let stats = metrics.timing_stats();
    assert_eq!(stats.count, 10);
    assert!(stats.mean_ns > 0.0);
}

/// Test: VSA operations work with metrics
#[test]
fn test_vsa_operations_tracked() {
    let mut rng = rand::rng();

    for _ in 0..10 {
        let vec1 = random_sparse_vec(&mut rng, 10000, 200);
        let vec2 = random_sparse_vec(&mut rng, 10000, 200);
        let _ = vec1.cosine(&vec2);
        let _ = vec1.bind(&vec2);
        let _ = vec1.bundle(&vec2);
    }

    // Just verify operations complete without panicking
}

/// Test: Timing operations don't interfere with functionality
#[test]
fn test_timing_no_interference() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    let start = Instant::now();

    // Create and save vector
    let vec = SparseVec::encode_data(b"test data", &config, None);
    let path = dir.path().join("test.bin");
    embeddenator_io::write_bincode_file(&path, &vec).unwrap();

    // Load and verify
    let loaded: SparseVec = embeddenator_io::read_bincode_file(&path).unwrap();

    let elapsed = start.elapsed();

    // Functionality should work correctly
    assert_eq!(vec.pos, loaded.pos);
    assert_eq!(vec.neg, loaded.neg);

    // Timing should be recorded
    assert!(elapsed.as_nanos() > 0);
}

/// Test: Index operations complete correctly
#[test]
fn test_index_operations() {
    let config = ReversibleVSAConfig::default();

    let mut index = BruteForceIndex::new(IndexConfig::default());
    for i in 0..10 {
        let data = format!("doc-{}", i);
        let vec = SparseVec::encode_data(data.as_bytes(), &config, None);
        index.add(i, &vec);
    }
    index.finalize();

    // Perform queries
    let query = SparseVec::encode_data(b"doc-5", &config, None);
    let results = index.query_top_k(&query, 3);

    // Should return results
    assert!(!results.is_empty());
}
