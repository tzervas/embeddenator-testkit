# Embeddenator TestKit

Comprehensive testing utilities and performance benchmarking for embeddenator VSA operations.

**Independent component** extracted from the Embeddenator monolithic repository. Part of the [Embeddenator workspace](https://github.com/tzervas/embeddenator).

**Repository:** [https://github.com/tzervas/embeddenator-testkit](https://github.com/tzervas/embeddenator-testkit)

## Features

- **Test Data Generators**: Create random and deterministic sparse vectors for reproducible testing
- **Performance Metrics**: Granular timing, memory tracking, and throughput measurements
- **Integrity Validation**: Verify VSA operation properties and detect data corruption
- **Chaos Injection**: Test resilience with bitflip injection, erasures, and noise
- **Test Fixtures**: Generate synthetic datasets with various patterns and sizes
- **Test Harness**: Manage temporary directories and coordinate complex test scenarios

## Installation

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
embeddenator-testkit = { path = "../embeddenator-testkit" }
```

## Usage Examples

### Generate Test Vectors

```rust
use embeddenator_testkit::*;
use rand::thread_rng;

// Generate random sparse vector
let mut rng = thread_rng();
let vec = random_sparse_vec(&mut rng, 10000, 200);

// Generate deterministic vector for reproducible tests
let vec = deterministic_sparse_vec(10000, 200, 42);
```

### Performance Measurement

```rust
use embeddenator_testkit::*;

let mut metrics = TestMetrics::new("bind_operation");

// Time an operation
metrics.start_timing();
let result = a.bind(&b);
metrics.stop_timing();

// Or use closure
let result = metrics.time_operation(|| a.bind(&b));

println!("{}", metrics.summary());
```

### Integrity Validation

```rust
use embeddenator_testkit::*;

let validator = IntegrityValidator::new();

// Validate bitsliced vector invariants
let report = validator.validate_bitsliced(&vec);
assert!(report.is_ok());

// Validate bind commutativity
let report = validator.validate_bind_invariants(&a, &b);
println!("{}", report.summary());
```

### Chaos Testing

```rust
use embeddenator_testkit::*;

let injector = ChaosInjector::new(42);

// Inject bitflips for resilience testing
let mut vec = vec.clone();
let flipped = injector.inject_bitflips(&mut vec, 10);

// Create corrupted copy
let corrupted = injector.corrupt_copy(&vec, 0.05); // 5% error rate
```

### Test Dataset Generation

```rust
use embeddenator_testkit::*;

// Create test data with specific pattern
let data = create_test_data(100, TestDataPattern::Random); // 100MB

// Create test harness with automatic cleanup
let harness = TestHarness::new();
let dataset_dir = harness.create_dataset(500); // 500MB dataset

// Create specific files
let file = harness.create_file("test.bin", b"test data");
```

## Module Overview

### `generators`
- `random_sparse_vec()` - Generate random sparse vectors
- `deterministic_sparse_vec()` - Reproducible vector generation
- `sparse_dot()` - Reference dot product implementation
- `generate_noise_pattern()` - Synthetic noise data

### `metrics`
- `TestMetrics` - Performance measurement and statistics
- `TimingStats` - Timing analysis (mean, median, percentiles)

### `integrity`
- `IntegrityValidator` - Verify VSA operation properties
- `IntegrityReport` - Validation results and diagnostics

### `chaos`
- `ChaosInjector` - Inject errors for resilience testing
- Bitflip, erasure, and corruption utilities

### `fixtures`
- `TestDataPattern` - Data pattern types
- `create_test_data()` - Generate test data
- `create_test_dataset()` - Multi-file test datasets

### `harness`
- `TestHarness` - Unified test management
- Temporary directory handling
- Performance metric collection

## Migrated from Monolithic Repo

This testkit extracts and consolidates test utilities from:
- `embeddenator/src/testing/mod.rs` - Performance metrics and integrity validation
- `embeddenator/tests/common/bt_migration.rs` - Vector generators and helpers
- `embeddenator/tests/qa_comprehensive.rs` - Test harness and dataset generation
- Various test modules - Fixture patterns and utilities

## Performance Baselines

Based on v0.20.0-alpha.1 benchmarks (Intel i7-14700K, 46GB RAM):
- Bundle: ~43ns (sparse), ~32µs (dense packed)
- Bind: ~11ns (sparse), ~20µs (dense packed)
- Cosine: ~7ns (sparse), ~14µs (dense packed)
- Ingestion: ~15 MB/s (2GB dataset)
- Extraction: ~41 MB/s (2GB dataset)

## Testing

Run the testkit's own tests:

```bash
cargo test --manifest-path embeddenator-testkit/Cargo.toml
```

Run with specific features:

```bash
cargo test --manifest-path embeddenator-testkit/Cargo.toml --features large-scale
```

## License

MIT
