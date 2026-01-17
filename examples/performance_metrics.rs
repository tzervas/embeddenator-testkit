//! Example: Performance measurement with TestMetrics
//!
//! Run with: cargo run --example performance_metrics

use embeddenator_testkit::*;
use rand::SeedableRng;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Embeddenator TestKit - Performance Metrics ===\n");

    // Create test metrics
    let mut metrics = TestMetrics::new("example_operations");

    // Time a simple operation
    println!("1. Timing vector generation...");
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    for _ in 0..10 {
        metrics.time_operation(|| {
            let _vec = random_sparse_vec(&mut rng, 10000, 200);
        });
    }

    println!("   Completed 10 operations");
    let stats = metrics.timing_stats();
    println!("   Mean: {:.2}µs", stats.mean_ns / 1000.0);
    println!("   Median: {:.2}µs", stats.p50_ns as f64 / 1000.0);
    println!("   P95: {:.2}µs", stats.p95_ns as f64 / 1000.0);

    // Record custom metrics
    println!("\n2. Recording custom metrics...");
    metrics.record_metric("accuracy", 0.95);
    metrics.record_metric("precision", 0.92);
    metrics.record_metric("recall", 0.89);
    metrics.inc_op("validation_checks");

    // Record memory usage
    println!("\n3. Recording memory snapshots...");
    for i in 1..=5 {
        metrics.record_memory(i * 1024 * 1024); // Simulate growing memory usage
    }

    // Display full summary
    println!("\n4. Full metrics summary:");
    println!("{}", metrics.summary());

    // Test timing with actual work
    println!("\n5. Timing with simulated work...");
    let mut work_metrics = TestMetrics::new("simulated_work");

    for sleep_ms in [1, 2, 5, 10, 20] {
        work_metrics.time_operation(|| {
            thread::sleep(Duration::from_millis(sleep_ms));
        });
    }

    let work_stats = work_metrics.timing_stats();
    println!("   Operations: {}", work_stats.count);
    println!(
        "   Total time: {:.2}ms",
        work_stats.total_ns as f64 / 1_000_000.0
    );
    println!("   Throughput: {:.2} ops/sec", work_stats.ops_per_sec());

    println!("\n✅ Performance metrics example complete!");
}
