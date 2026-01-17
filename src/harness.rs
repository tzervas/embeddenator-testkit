//! Test harness for managing temporary directories and test datasets
//!
//! Provides a unified test harness that:
//! - Creates temporary directories automatically cleaned up after tests
//! - Generates test datasets of various sizes and patterns
//! - Tracks performance metrics across test runs
//! - Provides helper methods for common test operations

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;

/// Performance metrics collector shared across tests
#[derive(Clone, Debug, Default)]
pub struct PerformanceMetrics {
    pub operation_times: HashMap<String, Vec<Duration>>,
    pub memory_usage: HashMap<String, Vec<usize>>,
    pub throughput: HashMap<String, Vec<f64>>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a performance metric
    pub fn record(
        &mut self,
        operation: &str,
        duration: Duration,
        memory_kb: usize,
        throughput_mbps: f64,
    ) {
        self.operation_times
            .entry(operation.to_string())
            .or_default()
            .push(duration);
        self.memory_usage
            .entry(operation.to_string())
            .or_default()
            .push(memory_kb);
        self.throughput
            .entry(operation.to_string())
            .or_default()
            .push(throughput_mbps);
    }

    /// Get average time for an operation
    pub fn avg_time(&self, operation: &str) -> Option<Duration> {
        self.operation_times.get(operation).map(|times| {
            let sum: Duration = times.iter().sum();
            sum / times.len() as u32
        })
    }

    /// Get average throughput for an operation
    pub fn avg_throughput(&self, operation: &str) -> Option<f64> {
        self.throughput
            .get(operation)
            .map(|throughputs| throughputs.iter().sum::<f64>() / throughputs.len() as f64)
    }
}

/// Test harness for comprehensive validation
///
/// Manages temporary directories, test datasets, and performance metrics.
/// Automatically cleans up resources when dropped.
pub struct TestHarness {
    temp_dir: TempDir,
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        TestHarness {
            temp_dir: TempDir::new().expect("Failed to create temp directory"),
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        }
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Record a performance metric
    pub fn record_metric(
        &self,
        operation: &str,
        duration: Duration,
        memory_kb: usize,
        throughput_mbps: f64,
    ) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.record(operation, duration, memory_kb, throughput_mbps);
    }

    /// Get a copy of current metrics
    pub fn metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Create a test dataset of specified size in MB
    ///
    /// Creates a directory with various file types and patterns
    pub fn create_dataset(&self, size_mb: usize) -> PathBuf {
        let dataset_dir = self.temp_dir.path().join(format!("dataset_{}mb", size_mb));
        fs::create_dir_all(&dataset_dir).expect("Failed to create dataset directory");

        // Create files of various types and sizes
        let patterns: Vec<(&str, &str, Vec<u8>)> = vec![
            (
                "text",
                "txt",
                b"This is a text file with some content.\n".to_vec(),
            ),
            (
                "json",
                "json",
                br#"{"key": "value", "number": 42}"#.to_vec(),
            ),
            ("binary", "bin", (0..=255).collect::<Vec<u8>>()),
        ];

        let mut total_size = 0;
        let mut file_count = 0;

        while total_size < size_mb * 1024 * 1024 {
            for (content_type, ext, base_content) in &patterns {
                let filename = format!("{}_{:04}.{}", content_type, file_count, ext);
                let filepath = dataset_dir.join(&filename);

                // Vary file size
                let multiplier = (file_count % 10) + 1;
                let content = base_content.repeat(multiplier);

                fs::write(&filepath, &content).expect("Failed to write test file");
                total_size += content.len();
                file_count += 1;

                if total_size >= size_mb * 1024 * 1024 {
                    break;
                }
            }
        }

        dataset_dir
    }

    /// Create a test file with specific content
    pub fn create_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let filepath = self.temp_dir.path().join(name);
        fs::write(&filepath, content).expect("Failed to write test file");
        filepath
    }

    /// Create a directory structure with various files
    pub fn create_directory_structure(&self, name: &str) -> PathBuf {
        let base = self.temp_dir.path().join(name);

        // Create directory structure
        fs::create_dir_all(base.join("dir1")).unwrap();
        fs::create_dir_all(base.join("dir2/nested")).unwrap();
        fs::create_dir_all(base.join("empty_dir")).unwrap();

        // Create test files
        fs::write(base.join("file1.txt"), b"Hello, world!").unwrap();
        fs::write(base.join("file2.log"), b"Log entry 1\nLog entry 2\n").unwrap();
        fs::write(
            base.join("dir1/file3.dat"),
            b"Binary data: \x00\x01\x02\xFF",
        )
        .unwrap();
        fs::write(
            base.join("dir2/file4.md"),
            b"# Markdown\n\n## Section\n\nContent here.",
        )
        .unwrap();
        fs::write(
            base.join("dir2/nested/file5.json"),
            br#"{"key": "value", "number": 42}"#,
        )
        .unwrap();

        base
    }

    /// Create a large file with specified pattern
    pub fn create_large_file(
        &self,
        name: &str,
        size_mb: usize,
        pattern: crate::fixtures::TestDataPattern,
    ) -> PathBuf {
        let filepath = self.temp_dir.path().join(name);
        let data = crate::fixtures::create_test_data(size_mb, pattern);
        fs::write(&filepath, data).expect("Failed to write large file");
        filepath
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::new();
        assert!(harness.temp_dir().exists());
    }

    #[test]
    fn test_create_file() {
        let harness = TestHarness::new();
        let path = harness.create_file("test.txt", b"hello");
        assert!(path.exists());
        assert_eq!(fs::read(&path).unwrap(), b"hello");
    }

    #[test]
    fn test_metrics_recording() {
        let harness = TestHarness::new();
        harness.record_metric("test_op", Duration::from_millis(100), 1024, 10.0);

        let metrics = harness.metrics();
        assert_eq!(metrics.operation_times.get("test_op").unwrap().len(), 1);
    }

    #[test]
    fn test_create_dataset() {
        let harness = TestHarness::new();
        let dataset = harness.create_dataset(1); // 1MB
        assert!(dataset.exists());

        // Check that some files were created
        let entries: Vec<_> = fs::read_dir(&dataset).unwrap().collect();
        assert!(!entries.is_empty());
    }
}
