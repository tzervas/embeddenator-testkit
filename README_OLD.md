# embeddenator-testkit

Comprehensive testing utilities and performance benchmarking for embeddenator VSA operations.

## Overview

This crate provides a complete testing framework for validating embeddenator performance, correctness, and scalability. It includes benchmarks for micro-operations, macro workflows, and large-scale dataset validation.

## Performance Insights (Migrated from Main Project)

Based on extensive benchmarking across scales from 250MB to 20GB+:

### Current Optimizations (bt-phase-2 + SIMD)
- **Packed ternary operations**: 10-20x speedup for dense vectors
- **SIMD cosine similarity**: Platform-specific acceleration (AVX2/NEON)
- **Thread-local scratch buffers**: Eliminates allocation overhead
- **Hybrid bundling**: Adaptive selection between pairwise/sum-many modes

### Performance Baselines (Intel i7-14700K, 46GB RAM)
- **Bundle (pairwise)**: ~43ns (sparse), ~32µs (dense packed)
- **Bind**: ~11ns (sparse), ~20µs (dense packed) 
- **Cosine**: ~7ns (sparse), ~14µs (dense packed)
- **Ingestion**: ~15 MB/s (2GB dataset), scales linearly
- **Extraction**: ~41 MB/s (2GB dataset), bit-perfect reconstruction

### Memory Scaling
- **Storage overhead**: 2.8x (engram size vs input)
- **Peak memory**: Bounded by hierarchical chunking
- **Large datasets**: 20GB+ supported with linear scaling

## Testing Infrastructure

### Benchmark Categories
- **Micro-benchmarks**: Individual VSA operations (ns scale)
- **Macro-benchmarks**: End-to-end workflows (ms-seconds scale)
- **Scale benchmarks**: 20GB-40GB dataset validation
- **Stress tests**: Memory pressure, concurrent operations

### Dataset Generation
- **Synthetic data**: Controlled patterns for reproducible testing
- **Realistic data**: Varied file types, sizes, and content patterns
- **Scale patterns**: Linear growth from KB to TB scales

## Usage

### Basic Testing

```rust
use embeddenator_testkit::*;

// Run smoke tests
assert!(testkit_smoke());

// Run performance validation
run_performance_validation();

// Test large-scale operations
test_large_scale_operations();

// Validate optimizations
validate_optimizations();
```

### Benchmarking

#### Performance Validation
```bash
cargo bench --bench performance_validation
```

#### Large-Scale Operations (20GB+ datasets)
```bash
cargo bench --bench large_scale_operations --features large-scale
```

#### Optimization Validation
```bash
cargo bench --bench optimization_validation
```

### Feature Flags

- `gpu`: Future GPU testing support
- `distributed`: Future distributed testing
- `large-scale`: Enable 20GB+ dataset tests

## Future Optimizations (Planned)

### GPU Acceleration
- **CUDA/OpenCL backends**: For VSA operations on GPU
- **Memory transfer optimization**: Minimize CPU↔GPU data movement
- **Kernel fusion**: Combine operations to reduce kernel launches

### CPU-GPU Coprocessing
- **Hybrid execution**: CPU for sparse, GPU for dense operations
- **Adaptive scheduling**: Runtime selection of optimal compute target
- **Unified memory**: Transparent CPU/GPU memory management

### Advanced Features
- **Memory-mapped I/O**: For datasets > RAM capacity
- **Distributed processing**: Multi-node VSA operations
- **Progressive loading**: On-demand data loading for massive datasets

## Benchmark Results Interpretation

### Micro-Benchmarks
- Measure individual operation performance
- Validate optimization effectiveness
- Identify algorithmic bottlenecks

### Macro-Benchmarks
- End-to-end workflow performance
- Memory usage patterns
- Scalability validation

### Scale Benchmarks
- Large dataset throughput
- Memory efficiency at scale
- System resource utilization

## Development

### Adding New Benchmarks

1. Create benchmark file in `benches/`
2. Use criterion for measurement
3. Include comprehensive documentation
4. Add feature gates for resource-intensive tests

### Testing Large Datasets

For 20GB+ testing:
```bash
# Enable large-scale feature
cargo bench --bench large_scale_operations --features large-scale

# With GPU support (future)
cargo bench --bench gpu_acceleration --features gpu
```

## Integration with Main Project

This testkit is designed to be used as a dev dependency:

```toml
[dev-dependencies]
embeddenator-testkit = { path = "../embeddenator-testkit", features = ["large-scale"] }
```

## Contributing

1. Add comprehensive documentation for new benchmarks
2. Include performance baselines for different hardware
3. Validate correctness before performance optimization
4. Document any new feature flags or dependencies

## License

MIT
