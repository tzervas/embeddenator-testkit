//! End-to-end workflow integration tests
//!
//! Tests complete user workflows from start to finish.

#![cfg(feature = "integration")]

use embeddenator_fs::EmbrFS;
use embeddenator_io::{read_bincode_file, read_json_file, write_bincode_file, write_json_file};
use embeddenator_retrieval::{BruteForceIndex, IndexConfig, RetrievalIndex};
use embeddenator_vsa::{ReversibleVSAConfig, SparseVec};
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Document {
    id: String,
    content: String,
}

/// Test: Complete ingest → query → extract workflow
#[test]
fn test_ingest_query_extract_workflow() {
    let dir = tempdir().unwrap();
    let source_dir = dir.path().join("source");
    let extract_dir = dir.path().join("extracted");

    // Create source files
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("doc1.txt"), b"This is document one").unwrap();
    fs::write(source_dir.join("doc2.txt"), b"This is document two").unwrap();
    fs::write(source_dir.join("doc3.txt"), b"This is document three").unwrap();

    let config = ReversibleVSAConfig::default();

    // Ingest
    let mut embrfs = EmbrFS::new();
    embrfs
        .ingest_directory(&source_dir, false, &config)
        .unwrap();

    // Extract
    EmbrFS::extract(
        &embrfs.engram,
        &embrfs.manifest,
        &extract_dir,
        false,
        &config,
    )
    .unwrap();

    // Verify extraction
    let extracted1 = fs::read(extract_dir.join("doc1.txt")).unwrap();
    let extracted2 = fs::read(extract_dir.join("doc2.txt")).unwrap();
    let extracted3 = fs::read(extract_dir.join("doc3.txt")).unwrap();

    assert_eq!(extracted1, b"This is document one");
    assert_eq!(extracted2, b"This is document two");
    assert_eq!(extracted3, b"This is document three");
}

/// Test: Vector index persistence workflow
#[test]
fn test_index_persistence_workflow() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    // Create documents
    let documents = vec![
        Document {
            id: "doc1".into(),
            content: "Machine learning basics".into(),
        },
        Document {
            id: "doc2".into(),
            content: "Deep learning neural networks".into(),
        },
        Document {
            id: "doc3".into(),
            content: "Statistical analysis methods".into(),
        },
        Document {
            id: "doc4".into(),
            content: "Data preprocessing techniques".into(),
        },
    ];

    // Encode documents to vectors
    let vectors: Vec<(String, SparseVec)> = documents
        .iter()
        .map(|doc| {
            let vec = SparseVec::encode_data(doc.content.as_bytes(), &config, None);
            (doc.id.clone(), vec)
        })
        .collect();

    // Save vectors and documents
    let vectors_path = dir.path().join("vectors.bin");
    let docs_path = dir.path().join("documents.json");

    write_bincode_file(&vectors_path, &vectors).unwrap();
    write_json_file(&docs_path, &documents).unwrap();

    // Load vectors and documents
    let loaded_vectors: Vec<(String, SparseVec)> = read_bincode_file(&vectors_path).unwrap();
    let loaded_docs: Vec<Document> = read_json_file(&docs_path).unwrap();

    // Verify persistence
    assert_eq!(loaded_vectors.len(), 4);
    assert_eq!(loaded_docs.len(), 4);

    // Build index from loaded vectors
    let mut index = BruteForceIndex::new(IndexConfig::default());
    for (i, (_id, vec)) in loaded_vectors.iter().enumerate() {
        index.add(i, vec);
    }
    index.finalize();

    // Query with exact match - use same content as doc2
    let query = SparseVec::encode_data(b"Deep learning neural networks", &config, None);
    let results = index.query_top_k(&query, 2);

    // Should find the exact match document
    assert!(!results.is_empty());

    // Top result should be doc2 (index 1) since it's an exact match
    assert_eq!(results[0].id, 1, "Exact match should be found");
}

/// Test: Multi-format data workflow  
#[test]
fn test_multi_format_workflow() {
    let dir = tempdir().unwrap();
    let config = ReversibleVSAConfig::default();

    // Create test data
    let vec = SparseVec::encode_data(b"test content", &config, None);

    // Save in multiple formats
    let bin_path = dir.path().join("data.bin");
    let json_path = dir.path().join("data.json");

    write_bincode_file(&bin_path, &vec).unwrap();
    write_json_file(&json_path, &vec).unwrap();

    // Load from both formats
    let from_bin: SparseVec = read_bincode_file(&bin_path).unwrap();
    let from_json: SparseVec = read_json_file(&json_path).unwrap();

    // Should be identical
    assert_eq!(vec.pos, from_bin.pos);
    assert_eq!(vec.neg, from_bin.neg);
    assert_eq!(vec.pos, from_json.pos);
    assert_eq!(vec.neg, from_json.neg);

    // Operations should work identically
    let vec2 = SparseVec::encode_data(b"other content", &config, None);
    let cosine_orig = vec.cosine(&vec2);
    let cosine_bin = from_bin.cosine(&vec2);
    let cosine_json = from_json.cosine(&vec2);

    assert!((cosine_orig - cosine_bin).abs() < 1e-10);
    assert!((cosine_orig - cosine_json).abs() < 1e-10);
}

/// Test: Large batch processing workflow
#[test]
fn test_batch_processing_workflow() {
    let config = ReversibleVSAConfig::default();

    // Generate batch of documents
    let batch_size = 100;
    let mut vectors = Vec::with_capacity(batch_size);

    for i in 0..batch_size {
        let content = format!("Document number {} with unique content {}", i, i * 17);
        let vec = SparseVec::encode_data(content.as_bytes(), &config, None);
        vectors.push(vec);
    }

    // Build index
    let mut index = BruteForceIndex::new(IndexConfig::default());
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i, vec);
    }
    index.finalize();

    // Query for specific document
    let query_content = format!("Document number {} with unique content {}", 42, 42 * 17);
    let query = SparseVec::encode_data(query_content.as_bytes(), &config, None);

    let results = index.query_top_k(&query, 5);

    // Should find the exact match
    assert!(!results.is_empty());
    assert_eq!(results[0].id, 42, "Should find exact match as top result");
}
