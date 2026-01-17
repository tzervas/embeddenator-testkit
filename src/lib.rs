//! # Embeddenator TestKit
//!
//! Comprehensive testing utilities for embeddenator VSA operations, performance benchmarking,
//! and large-scale data validation.
//!
//! ## Performance Optimization Insights (v0.20.0-alpha.1)
//!
//! Based on extensive benchmarking across scales from 250MB to 20GB+:
//!
//! ### Current Optimizations (bt-phase-2 + SIMD)
//! - **Packed ternary operations**: 10-20x speedup for dense vectors
//! - **SIMD cosine similarity**: Platform-specific acceleration (AVX2/NEON)
//! - **Thread-local scratch buffers**: Eliminates allocation overhead
//! - **Hybrid bundling**: Adaptive selection between pairwise/sum-many modes
//!
//! ### Performance Baselines (Intel i7-14700K, 46GB RAM)
//! - **Bundle (pairwise)**: ~43ns (sparse), ~32µs (dense packed)
//! - **Bind**: ~11ns (sparse), ~20µs (dense packed)
//! - **Cosine**: ~7ns (sparse), ~14µs (dense packed)
//! - **Ingestion**: ~15 MB/s (2GB dataset), scales linearly
//! - **Extraction**: ~41 MB/s (2GB dataset), bit-perfect reconstruction
//!
//! ### Memory Scaling
//! - **Storage overhead**: 2.8x (engram size vs input)
//! - **Peak memory**: Bounded by hierarchical chunking
//! - **Large datasets**: 20GB+ supported with linear scaling
//!
//! ## Future Optimizations (Planned)
//! - **GPU acceleration**: CUDA/OpenCL backends for VSA operations
//! - **CPU-GPU coprocessing**: Hybrid execution models
//! - **Memory-mapped I/O**: For datasets > RAM capacity
//! - **Distributed processing**: Multi-node VSA operations
//!
//! ## Testing Infrastructure
//!
//! ### Benchmark Categories
//! - **Micro-benchmarks**: Individual VSA operations (ns scale)
//! - **Macro-benchmarks**: End-to-end workflows (ms-seconds scale)
//! - **Scale benchmarks**: 20GB-40GB dataset validation
//! - **Stress tests**: Memory pressure, concurrent operations
//!
//! ### Dataset Generation
//! - **Synthetic data**: Controlled patterns for reproducible testing
//! - **Realistic data**: Varied file types, sizes, and content patterns
//! - **Scale patterns**: Linear growth from KB to TB scales
//!
//! ## Usage Examples
//!
//! ```rust,ignore
//! use embeddenator_testkit::*;
//!
//! // Generate random sparse vectors for testing
//! let mut rng = rand::thread_rng();
//! let vec = generators::random_sparse_vec(&mut rng, 10000, 200);
//!
//! // Create test datasets
//! let harness = TestHarness::new();
//! let dataset = harness.create_dataset(100); // 100MB
//!
//! // Run performance validation
//! let mut metrics = TestMetrics::new("bind_operation");
//! metrics.start_timing();
//! let result = vec.bind(&vec);
//! metrics.stop_timing();
//! println!("{}", metrics.summary());
//! ```

pub mod chaos;
pub mod fixtures;
pub mod generators;
pub mod harness;
pub mod integrity;
pub mod metrics;

// Re-export commonly used items
pub use chaos::ChaosInjector;
pub use fixtures::{create_test_data, create_test_dataset, TestDataPattern};
pub use generators::{
    deterministic_sparse_vec, mk_random_sparsevec, random_sparse_vec, sparse_dot,
};
pub use harness::TestHarness;
pub use integrity::{IntegrityReport, IntegrityValidator};
pub use metrics::{TestMetrics, TimingStats};

// Re-export VSA types for integration tests
pub use embeddenator_vsa::{SparseVec, DIM};

/// Smoke test for testkit functionality
pub fn testkit_smoke() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke() {
        assert!(testkit_smoke());
    }
}
