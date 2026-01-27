//! Cross-component integration tests
//!
//! Tests interactions between pairs of components to verify they work together correctly.

#![cfg(feature = "integration")]

use embeddenator_io::{read_bincode_file, write_bincode_file};
use embeddenator_retrieval::{
    two_stage_search, BruteForceIndex, IndexConfig, RetrievalIndex, SearchConfig,
    TernaryInvertedIndex,
};
use embeddenator_testkit::random_sparse_vec;
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec};
use std::collections::HashMap;
use tempfile::tempdir;

/// Test: VSA + Retrieval integration
/// Verifies similarity search works with VSA vectors
#[test]
fn test_vsa_retrieval_integration() {
    let config = ReversibleVSAConfig::default();
    let index_config = IndexConfig::default();
    let mut index = BruteForceIndex::new(index_config);

    // Create and index vectors
    let mut vectors = HashMap::new();
    for i in 0..20 {
        let data = format!("document-{}", i);
        let vec = SparseVec::encode_data(data.as_bytes(), &config, None);
        index.add(i, &vec);
        vectors.insert(i, vec);
    }
    index.finalize();

    // Query
    let query = SparseVec::encode_data(b"document-10", &config, None);
    let results = index.query_top_k(&query, 5);

    // Should find the exact match
    assert!(!results.is_empty());
    let top_ids: Vec<usize> = results.iter().map(|r| r.id).collect();
    assert!(top_ids.contains(&10), "Should find exact match");
}

/// Test: VSA + IO integration
/// Verifies VSA vectors can be serialized and deserialized correctly
#[test]
fn test_vsa_io_integration() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    // Create vectors
    let vec1 = SparseVec::encode_data(b"test data one", &config, None);
    let vec2 = SparseVec::encode_data(b"test data two", &config, None);

    // Write to disk
    let path1 = dir.path().join("vec1.bin");
    let path2 = dir.path().join("vec2.bin");

    write_bincode_file(&path1, &vec1).unwrap();
    write_bincode_file(&path2, &vec2).unwrap();

    // Read back
    let loaded1: SparseVec = read_bincode_file(&path1).unwrap();
    let loaded2: SparseVec = read_bincode_file(&path2).unwrap();

    // Verify identical
    assert_eq!(vec1.pos, loaded1.pos);
    assert_eq!(vec1.neg, loaded1.neg);
    assert_eq!(vec2.pos, loaded2.pos);
    assert_eq!(vec2.neg, loaded2.neg);

    // Verify operations still work
    let cosine_original = vec1.cosine(&vec2);
    let cosine_loaded = loaded1.cosine(&loaded2);
    assert!((cosine_original - cosine_loaded).abs() < 1e-10);
}

/// Test: Retrieval + IO integration
/// Verifies index can be serialized and operations work after reload
#[test]
fn test_retrieval_io_integration() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    // Build index
    let index_config = IndexConfig::default();
    let mut index = BruteForceIndex::new(index_config.clone());

    let mut vectors: Vec<SparseVec> = Vec::new();
    for i in 0..10 {
        let data = format!("item-{}", i);
        let vec = SparseVec::encode_data(data.as_bytes(), &config, None);
        index.add(i, &vec);
        vectors.push(vec);
    }
    index.finalize();

    // Serialize vectors
    let path = dir.path().join("vectors.bin");
    write_bincode_file(&path, &vectors).unwrap();

    // Load vectors and rebuild index
    let loaded_vectors: Vec<SparseVec> = read_bincode_file(&path).unwrap();
    let mut new_index = BruteForceIndex::new(IndexConfig::default());
    for (i, vec) in loaded_vectors.iter().enumerate() {
        new_index.add(i, vec);
    }
    new_index.finalize();

    // Query both should give same results
    let query = SparseVec::encode_data(b"item-5", &config, None);
    let results1 = index.query_top_k(&query, 3);
    let results2 = new_index.query_top_k(&query, 3);

    assert_eq!(results1.len(), results2.len());
    for (r1, r2) in results1.iter().zip(results2.iter()) {
        assert_eq!(r1.id, r2.id);
    }
}

/// Test: Hierarchical index integration
#[test]
fn test_hierarchical_index_integration() {
    let config = ReversibleVSAConfig::default();
    let mut index = TernaryInvertedIndex::default();

    // Add vectors
    let mut vectors = HashMap::new();
    for i in 0..50 {
        let data = format!("doc-{:03}", i);
        let vec = SparseVec::encode_data(data.as_bytes(), &config, None);
        index.add(i, &vec);
        vectors.insert(i, vec);
    }
    index.finalize();

    // Query
    let query = SparseVec::encode_data(b"doc-025", &config, None);
    let results = index.query_top_k(&query, 10);

    assert!(!results.is_empty());

    // Two-stage search with reranking
    let search_config = SearchConfig {
        candidate_k: 20,
        ..SearchConfig::default()
    };
    let reranked = two_stage_search(&query, &index, &vectors, &search_config, 5);

    assert_eq!(reranked.len(), 5);
    // Results should be sorted by score (descending)
    for i in 1..reranked.len() {
        assert!(reranked[i - 1].score >= reranked[i].score);
    }
}

/// Test: Random vector generation for testing
#[test]
fn test_random_sparse_vec_integration() {
    let mut rng = rand::rng();

    // Generate random vectors
    let vec1 = random_sparse_vec(&mut rng, 10000, 200);
    let vec2 = random_sparse_vec(&mut rng, 10000, 200);

    // Should have correct structure
    assert_eq!(vec1.pos.len() + vec1.neg.len(), 200);
    assert_eq!(vec2.pos.len() + vec2.neg.len(), 200);

    // Should be sorted
    for i in 1..vec1.pos.len() {
        assert!(vec1.pos[i - 1] < vec1.pos[i]);
    }
    for i in 1..vec1.neg.len() {
        assert!(vec1.neg[i - 1] < vec1.neg[i]);
    }

    // Cosine should be valid
    let cosine = vec1.cosine(&vec2);
    assert!((-1.0..=1.0).contains(&cosine));
}
