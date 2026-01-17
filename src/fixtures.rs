//! Test data fixtures and dataset generation
//!
//! Provides utilities for creating test datasets:
//! - Various data patterns (zeros, sequential, random, text, etc.)
//! - File generation with controlled sizes
//! - Realistic test data scenarios

use std::fs;
use std::path::Path;

/// Test data patterns for file generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestDataPattern {
    /// All zeros
    Zeros,
    /// All ones (0xFF)
    Ones,
    /// Sequential bytes (0, 1, 2, ..., 255, 0, 1, ...)
    Sequential,
    /// Pseudo-random pattern (deterministic)
    Random,
    /// Compressible repeating text
    Compressible,
    /// ASCII text pattern
    Text,
}

/// Create test data with specified pattern
///
/// # Arguments
/// * `size_mb` - Size in megabytes
/// * `pattern` - Data pattern to generate
///
/// # Returns
/// Vector of bytes with the specified pattern
pub fn create_test_data(size_mb: usize, pattern: TestDataPattern) -> Vec<u8> {
    let size_bytes = size_mb * 1024 * 1024;

    match pattern {
        TestDataPattern::Zeros => vec![0u8; size_bytes],
        TestDataPattern::Ones => vec![0xFF; size_bytes],
        TestDataPattern::Sequential => (0..size_bytes).map(|i| (i % 256) as u8).collect(),
        TestDataPattern::Random => {
            // Simple deterministic "random" pattern using LCG
            (0..size_bytes)
                .map(|i| ((i.wrapping_mul(2654435761)) % 256) as u8)
                .collect()
        }
        TestDataPattern::Compressible => {
            // Repeating pattern that compresses well
            let pattern = b"The quick brown fox jumps over the lazy dog. ";
            (0..size_bytes)
                .map(|i| pattern[i % pattern.len()])
                .collect()
        }
        TestDataPattern::Text => {
            // ASCII text pattern
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 \n";
            (0..size_bytes).map(|i| chars[i % chars.len()]).collect()
        }
    }
}

/// Verify data matches expected pattern (with sampling for large data)
///
/// # Arguments
/// * `data` - Data to verify
/// * `expected_pattern` - Expected pattern
/// * `sample_points` - Number of points to sample
pub fn verify_data_sampled(data: &[u8], expected_pattern: TestDataPattern, sample_points: usize) {
    let len = data.len();
    let stride = len / sample_points;

    for i in 0..sample_points {
        let pos = i * stride;
        if pos >= len {
            break;
        }
        let expected = match expected_pattern {
            TestDataPattern::Zeros => 0u8,
            TestDataPattern::Ones => 0xFF,
            TestDataPattern::Sequential => (pos % 256) as u8,
            TestDataPattern::Random => ((pos.wrapping_mul(2654435761)) % 256) as u8,
            TestDataPattern::Compressible => {
                let pattern = b"The quick brown fox jumps over the lazy dog. ";
                pattern[pos % pattern.len()]
            }
            TestDataPattern::Text => {
                let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 \n";
                chars[pos % chars.len()]
            }
        };
        assert_eq!(
            data[pos], expected,
            "Mismatch at position {} (sample {}): expected {}, got {}",
            pos, i, expected, data[pos]
        );
    }
}

/// Create a test dataset directory with multiple files
///
/// # Arguments
/// * `base_path` - Base directory for dataset
/// * `size_mb` - Total size in megabytes
/// * `pattern` - Data pattern to use
///
/// # Returns
/// Number of files created
pub fn create_test_dataset(base_path: &Path, size_mb: usize, pattern: TestDataPattern) -> usize {
    fs::create_dir_all(base_path).expect("Failed to create dataset directory");

    let target_bytes = size_mb * 1024 * 1024;
    let mut written = 0;
    let mut file_count = 0;

    // Create files of varying sizes (1KB to 1MB)
    while written < target_bytes {
        let file_size = match file_count % 5 {
            0 => 1024,        // 1KB
            1 => 10 * 1024,   // 10KB
            2 => 100 * 1024,  // 100KB
            3 => 500 * 1024,  // 500KB
            _ => 1024 * 1024, // 1MB
        };

        let actual_size = file_size.min(target_bytes - written);
        let filename = format!("file_{:04}.bin", file_count);
        let filepath = base_path.join(&filename);

        let data = create_test_data_bytes(actual_size, pattern);
        fs::write(&filepath, data).expect("Failed to write test file");

        written += actual_size;
        file_count += 1;
    }

    file_count
}

/// Create test data with exact byte count (helper)
fn create_test_data_bytes(size_bytes: usize, pattern: TestDataPattern) -> Vec<u8> {
    match pattern {
        TestDataPattern::Zeros => vec![0u8; size_bytes],
        TestDataPattern::Ones => vec![0xFF; size_bytes],
        TestDataPattern::Sequential => (0..size_bytes).map(|i| (i % 256) as u8).collect(),
        TestDataPattern::Random => (0..size_bytes)
            .map(|i| ((i.wrapping_mul(2654435761)) % 256) as u8)
            .collect(),
        TestDataPattern::Compressible => {
            let pattern = b"The quick brown fox jumps over the lazy dog. ";
            (0..size_bytes)
                .map(|i| pattern[i % pattern.len()])
                .collect()
        }
        TestDataPattern::Text => {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 \n";
            (0..size_bytes).map(|i| chars[i % chars.len()]).collect()
        }
    }
}

/// Write a file of specified size with pattern
pub fn write_file_of_size(
    path: &Path,
    size_bytes: usize,
    pattern: TestDataPattern,
) -> std::io::Result<()> {
    let data = create_test_data_bytes(size_bytes, pattern);
    fs::write(path, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_test_data() {
        let data = create_test_data(1, TestDataPattern::Zeros);
        assert_eq!(data.len(), 1024 * 1024);
        assert!(data.iter().all(|&b| b == 0));

        let data = create_test_data(1, TestDataPattern::Ones);
        assert!(data.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_sequential_pattern() {
        let data = create_test_data_bytes(512, TestDataPattern::Sequential);
        assert_eq!(data.len(), 512);
        for i in 0..256 {
            assert_eq!(data[i], i as u8);
        }
        // Should wrap around
        for i in 256..512 {
            assert_eq!(data[i], (i % 256) as u8);
        }
    }

    #[test]
    fn test_compressible_pattern() {
        let data = create_test_data_bytes(100, TestDataPattern::Compressible);
        let pattern = b"The quick brown fox jumps over the lazy dog. ";

        // Check first occurrence
        assert_eq!(&data[0..pattern.len()], pattern);
    }

    #[test]
    fn test_verify_data_sampled() {
        let data = create_test_data_bytes(10000, TestDataPattern::Sequential);
        // Should not panic
        verify_data_sampled(&data, TestDataPattern::Sequential, 100);
    }

    #[test]
    #[should_panic(expected = "Mismatch at position")]
    fn test_verify_data_sampled_mismatch() {
        let mut data = create_test_data_bytes(1000, TestDataPattern::Sequential);
        data[500] = 0xFF; // Corrupt data
        verify_data_sampled(&data, TestDataPattern::Sequential, 100);
    }

    #[test]
    fn test_create_test_dataset() {
        let temp_dir = TempDir::new().unwrap();
        let dataset_path = temp_dir.path().join("dataset");

        let file_count = create_test_dataset(&dataset_path, 5, TestDataPattern::Random);

        assert!(file_count > 0);
        assert!(dataset_path.exists());

        // Verify total size is approximately correct
        let mut total_size = 0;
        for entry in fs::read_dir(&dataset_path).unwrap() {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            total_size += metadata.len();
        }

        let expected_size = 5 * 1024 * 1024;
        assert!(total_size >= expected_size - 1024 * 1024); // Within 1MB
        assert!(total_size <= expected_size + 1024 * 1024);
    }

    #[test]
    fn test_write_file_of_size() {
        let temp_dir = TempDir::new().unwrap();
        let filepath = temp_dir.path().join("test.bin");

        write_file_of_size(&filepath, 4096, TestDataPattern::Random).unwrap();

        assert!(filepath.exists());
        let metadata = fs::metadata(&filepath).unwrap();
        assert_eq!(metadata.len(), 4096);
    }
}
