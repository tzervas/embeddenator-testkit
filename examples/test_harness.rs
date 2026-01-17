//! Example: Test harness usage with datasets
//!
//! Run with: cargo run --example test_harness

use embeddenator_testkit::*;

fn main() {
    println!("=== Embeddenator TestKit - Test Harness ===\n");

    // Create test harness (automatically manages temp directory)
    println!("1. Creating test harness...");
    let harness = TestHarness::new();
    println!("   Temp directory: {:?}", harness.temp_dir());

    // Create a simple test file
    println!("\n2. Creating test file...");
    let file_path = harness.create_file("test.txt", b"Hello, World!");
    println!("   Created: {:?}", file_path);
    println!("   Exists: {}", file_path.exists());

    // Create a test dataset
    println!("\n3. Creating test dataset (5MB)...");
    let dataset_path = harness.create_dataset(5);
    println!("   Dataset directory: {:?}", dataset_path);

    // List files in dataset
    let files: Vec<_> = std::fs::read_dir(&dataset_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    println!("   Files created: {}", files.len());

    if let Some(first_file) = files.first() {
        let metadata = first_file.metadata().unwrap();
        println!("   First file size: {} bytes", metadata.len());
    }

    // Create large file with specific pattern
    println!("\n4. Creating large file with pattern...");
    let large_file = harness.create_large_file(
        "sequential.bin",
        10, // 10MB
        TestDataPattern::Sequential,
    );
    let metadata = std::fs::metadata(&large_file).unwrap();
    println!("   Created: {:?}", large_file);
    println!(
        "   Size: {} bytes ({} MB)",
        metadata.len(),
        metadata.len() / 1024 / 1024
    );

    // Create directory structure
    println!("\n5. Creating directory structure...");
    let dir_structure = harness.create_directory_structure("project");
    println!("   Base: {:?}", dir_structure);
    println!("   Contains dir1: {}", dir_structure.join("dir1").exists());
    println!(
        "   Contains dir2/nested: {}",
        dir_structure.join("dir2/nested").exists()
    );

    // Record metrics
    println!("\n6. Recording performance metrics...");
    harness.record_metric(
        "dataset_creation",
        std::time::Duration::from_secs(1),
        5120, // 5MB in KB
        5.0,  // 5 MB/s
    );

    let metrics = harness.metrics();
    if let Some(avg_time) = metrics.avg_time("dataset_creation") {
        println!("   Average time: {:?}", avg_time);
    }
    if let Some(avg_throughput) = metrics.avg_throughput("dataset_creation") {
        println!("   Average throughput: {:.2} MB/s", avg_throughput);
    }

    println!("\n✅ Test harness example complete!");
    println!("   (Temp directory will be automatically cleaned up)");
}
