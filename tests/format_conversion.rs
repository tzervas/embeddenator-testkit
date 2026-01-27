//! Format conversion integration tests
//!
//! Tests serialization format conversions work correctly.

#![cfg(feature = "integration")]

use embeddenator_io::{read_bincode_file, read_json_file, write_bincode_file, write_json_file};
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

/// Helper to compare SparseVec instances (since SparseVec doesn't impl PartialEq)
fn sparse_vec_eq(a: &SparseVec, b: &SparseVec) -> bool {
    a.pos == b.pos && a.neg == b.neg
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestRecord {
    id: u64,
    name: String,
    vector: SparseVec,
    tags: Vec<String>,
}

impl PartialEq for TestRecord {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && sparse_vec_eq(&self.vector, &other.vector)
            && self.tags == other.tags
    }
}

/// Test: Bincode round-trip
#[test]
fn test_bincode_roundtrip() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    let record = TestRecord {
        id: 42,
        name: "test record".into(),
        vector: SparseVec::encode_data(b"test content", &config, None),
        tags: vec!["tag1".into(), "tag2".into()],
    };

    let path = dir.path().join("record.bin");
    write_bincode_file(&path, &record).unwrap();
    let loaded: TestRecord = read_bincode_file(&path).unwrap();

    assert_eq!(record, loaded);
}

/// Test: JSON round-trip
#[test]
fn test_json_roundtrip() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    let record = TestRecord {
        id: 42,
        name: "test record".into(),
        vector: SparseVec::encode_data(b"test content", &config, None),
        tags: vec!["tag1".into(), "tag2".into()],
    };

    let path = dir.path().join("record.json");
    write_json_file(&path, &record).unwrap();
    let loaded: TestRecord = read_json_file(&path).unwrap();

    assert_eq!(record, loaded);
}

/// Test: Batch serialization
#[test]
fn test_batch_serialization() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    let records: Vec<TestRecord> = (0..100)
        .map(|i| {
            let content = format!("content {}", i);
            TestRecord {
                id: i,
                name: format!("record {}", i),
                vector: SparseVec::encode_data(content.as_bytes(), &config, None),
                tags: vec![format!("tag{}", i % 5)],
            }
        })
        .collect();

    // Bincode batch
    let bin_path = dir.path().join("batch.bin");
    write_bincode_file(&bin_path, &records).unwrap();
    let loaded_bin: Vec<TestRecord> = read_bincode_file(&bin_path).unwrap();
    assert_eq!(records, loaded_bin);

    // JSON batch
    let json_path = dir.path().join("batch.json");
    write_json_file(&json_path, &records).unwrap();
    let loaded_json: Vec<TestRecord> = read_json_file(&json_path).unwrap();
    assert_eq!(records, loaded_json);
}

/// Test: Empty vector serialization
#[test]
fn test_empty_vector_serialization() {
    let dir = tempdir().unwrap();

    let empty_vec = SparseVec {
        pos: vec![],
        neg: vec![],
    };

    let bin_path = dir.path().join("empty.bin");
    write_bincode_file(&bin_path, &empty_vec).unwrap();
    let loaded_bin: SparseVec = read_bincode_file(&bin_path).unwrap();
    assert!(sparse_vec_eq(&empty_vec, &loaded_bin));

    let json_path = dir.path().join("empty.json");
    write_json_file(&json_path, &empty_vec).unwrap();
    let loaded_json: SparseVec = read_json_file(&json_path).unwrap();
    assert!(sparse_vec_eq(&empty_vec, &loaded_json));
}

/// Test: Large vector serialization
#[test]
fn test_large_vector_serialization() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    // Create a large content to encode
    let large_content: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    let large_vec = SparseVec::encode_data(&large_content, &config, None);

    let bin_path = dir.path().join("large.bin");
    write_bincode_file(&bin_path, &large_vec).unwrap();
    let loaded: SparseVec = read_bincode_file(&bin_path).unwrap();

    assert_eq!(large_vec.pos, loaded.pos);
    assert_eq!(large_vec.neg, loaded.neg);
}

/// Test: Format conversion preserves semantics
#[test]
fn test_format_conversion_preserves_semantics() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    let vec1 = SparseVec::encode_data(b"first vector", &config, None);
    let vec2 = SparseVec::encode_data(b"second vector", &config, None);

    // Compute similarity before serialization
    let cosine_before = vec1.cosine(&vec2);

    // Save and load via bincode
    let path1 = dir.path().join("v1.bin");
    let path2 = dir.path().join("v2.bin");
    write_bincode_file(&path1, &vec1).unwrap();
    write_bincode_file(&path2, &vec2).unwrap();
    let loaded1: SparseVec = read_bincode_file(&path1).unwrap();
    let loaded2: SparseVec = read_bincode_file(&path2).unwrap();

    // Compute similarity after serialization
    let cosine_after = loaded1.cosine(&loaded2);

    assert!(
        (cosine_before - cosine_after).abs() < 1e-10,
        "Serialization should preserve vector semantics"
    );
}
