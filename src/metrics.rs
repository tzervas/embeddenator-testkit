//! Performance metrics and timing utilities for testing
//!
//! Provides granular performance measurement tools including:
//! - Operation timing with statistics (mean, median, percentiles)
//! - Memory usage tracking
//! - Throughput calculations
//! - Custom metric recording

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Granular performance metrics for test operations
#[derive(Clone, Debug)]
pub struct TestMetrics {
    /// Operation name for reporting
    pub name: String,
    /// Individual timing samples (nanoseconds)
    pub timings_ns: Vec<u64>,
    /// Start time for current measurement
    start: Option<Instant>,
    /// Operation counts by category
    pub op_counts: HashMap<String, u64>,
    /// Custom numeric metrics
    pub custom_metrics: HashMap<String, f64>,
    /// Memory snapshots (bytes)
    pub memory_samples: Vec<usize>,
    /// Error/warning counts
    pub error_count: u64,
    pub warning_count: u64,
}

impl TestMetrics {
    /// Create new metrics collector for named operation
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            timings_ns: Vec::new(),
            start: None,
            op_counts: HashMap::new(),
            custom_metrics: HashMap::new(),
            memory_samples: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Start timing measurement
    #[inline]
    pub fn start_timing(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Stop timing and record sample
    #[inline]
    pub fn stop_timing(&mut self) {
        if let Some(start) = self.start.take() {
            self.timings_ns.push(start.elapsed().as_nanos() as u64);
        }
    }

    /// Record a timed operation with closure
    #[inline]
    pub fn time_operation<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.start_timing();
        let result = f();
        self.stop_timing();
        result
    }

    /// Increment operation counter
    #[inline]
    pub fn inc_op(&mut self, category: &str) {
        *self.op_counts.entry(category.to_string()).or_insert(0) += 1;
    }

    /// Record custom metric
    #[inline]
    pub fn record_metric(&mut self, name: &str, value: f64) {
        self.custom_metrics.insert(name.to_string(), value);
    }

    /// Record memory usage
    #[inline]
    pub fn record_memory(&mut self, bytes: usize) {
        self.memory_samples.push(bytes);
    }

    /// Record operation count
    #[inline]
    pub fn record_operation(&mut self, count: usize) {
        self.inc_op("operations");
        self.record_metric("last_count", count as f64);
    }

    /// Record an error
    #[inline]
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Record a warning
    #[inline]
    pub fn record_warning(&mut self) {
        self.warning_count += 1;
    }

    /// Get timing statistics
    pub fn timing_stats(&self) -> TimingStats {
        if self.timings_ns.is_empty() {
            return TimingStats::default();
        }

        let mut sorted = self.timings_ns.clone();
        sorted.sort_unstable();

        let sum: u64 = sorted.iter().sum();
        let count = sorted.len() as f64;
        let mean = sum as f64 / count;

        let variance = sorted
            .iter()
            .map(|&t| {
                let diff = t as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / count;

        TimingStats {
            count: sorted.len(),
            min_ns: sorted[0],
            max_ns: sorted[sorted.len() - 1],
            mean_ns: mean,
            std_dev_ns: variance.sqrt(),
            p50_ns: sorted[sorted.len() / 2],
            p95_ns: sorted[(sorted.len() as f64 * 0.95) as usize],
            p99_ns: sorted[(sorted.len() as f64 * 0.99).min(sorted.len() as f64 - 1.0) as usize],
            total_ns: sum,
        }
    }

    /// Generate summary report
    pub fn summary(&self) -> String {
        let stats = self.timing_stats();
        let mut report = format!("=== {} Metrics ===\n", self.name);

        if stats.count > 0 {
            report.push_str(&format!(
                "Timing: {} ops, mean={:.2}µs, p50={:.2}µs, p95={:.2}µs, p99={:.2}µs\n",
                stats.count,
                stats.mean_ns / 1000.0,
                stats.p50_ns as f64 / 1000.0,
                stats.p95_ns as f64 / 1000.0,
                stats.p99_ns as f64 / 1000.0,
            ));
            report.push_str(&format!(
                "        min={:.2}µs, max={:.2}µs, stddev={:.2}µs\n",
                stats.min_ns as f64 / 1000.0,
                stats.max_ns as f64 / 1000.0,
                stats.std_dev_ns / 1000.0,
            ));
        }

        if !self.op_counts.is_empty() {
            report.push_str("Operations: ");
            let ops: Vec<_> = self
                .op_counts
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            report.push_str(&ops.join(", "));
            report.push('\n');
        }

        if !self.custom_metrics.is_empty() {
            report.push_str("Metrics: ");
            let metrics: Vec<_> = self
                .custom_metrics
                .iter()
                .map(|(k, v)| format!("{}={:.4}", k, v))
                .collect();
            report.push_str(&metrics.join(", "));
            report.push('\n');
        }

        if !self.memory_samples.is_empty() {
            let max_mem = self.memory_samples.iter().max().unwrap_or(&0);
            let avg_mem = self.memory_samples.iter().sum::<usize>() / self.memory_samples.len();
            report.push_str(&format!(
                "Memory: peak={}KB, avg={}KB\n",
                max_mem / 1024,
                avg_mem / 1024,
            ));
        }

        if self.error_count > 0 || self.warning_count > 0 {
            report.push_str(&format!(
                "Issues: errors={}, warnings={}\n",
                self.error_count, self.warning_count
            ));
        }

        report
    }
}

/// Timing statistics
#[derive(Clone, Debug, Default)]
pub struct TimingStats {
    pub count: usize,
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: f64,
    pub std_dev_ns: f64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub total_ns: u64,
}

impl TimingStats {
    /// Total time as Duration
    pub fn total_duration(&self) -> Duration {
        Duration::from_nanos(self.total_ns)
    }

    /// Throughput in operations per second
    pub fn ops_per_sec(&self) -> f64 {
        if self.total_ns == 0 {
            0.0
        } else {
            (self.count as f64) / (self.total_ns as f64 / 1_000_000_000.0)
        }
    }

    /// Mean time as Duration
    pub fn mean_duration(&self) -> Duration {
        Duration::from_nanos(self.mean_ns as u64)
    }

    /// Median time as Duration
    pub fn median_duration(&self) -> Duration {
        Duration::from_nanos(self.p50_ns)
    }
}

/// Accuracy metrics for VSA encoding/decoding fidelity
#[derive(Clone, Debug, Default)]
pub struct AccuracyMetrics {
    /// Total bytes processed
    pub total_bytes: usize,
    /// Bytes requiring correction
    pub correction_bytes: usize,
    /// Number of correction entries
    pub correction_count: usize,
    /// Raw reconstruction accuracy (before corrections)
    pub raw_accuracy: f64,
    /// Final accuracy (after corrections)
    pub final_accuracy: f64,
    /// Signal-to-noise ratio in dB
    pub snr_db: f64,
    /// Compression ratio (original / encoded size)
    pub compression_ratio: f64,
}

impl AccuracyMetrics {
    /// Create new accuracy metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record encoding fidelity from original and reconstructed data
    pub fn record_fidelity(&mut self, original: &[u8], reconstructed: &[u8]) {
        self.total_bytes += original.len();

        let matching = original
            .iter()
            .zip(reconstructed.iter())
            .filter(|(a, b)| a == b)
            .count();

        let errors = original.len().saturating_sub(matching);
        self.correction_bytes += errors;

        self.raw_accuracy = if self.total_bytes > 0 {
            1.0 - (self.correction_bytes as f64 / self.total_bytes as f64)
        } else {
            1.0
        };

        // Calculate SNR: 10 * log10(signal_power / noise_power)
        // For byte data, signal_power ≈ variance of original, noise_power ≈ MSE
        if !original.is_empty() && !reconstructed.is_empty() {
            let signal_power: f64 =
                original.iter().map(|&b| (b as f64).powi(2)).sum::<f64>() / original.len() as f64;
            let noise_power: f64 = original
                .iter()
                .zip(reconstructed.iter())
                .map(|(&a, &b)| ((a as f64) - (b as f64)).powi(2))
                .sum::<f64>()
                / original.len() as f64;

            self.snr_db = if noise_power > 0.0 {
                10.0 * (signal_power / noise_power).log10()
            } else {
                f64::INFINITY // Perfect reconstruction
            };
        }
    }

    /// Record correction store statistics
    pub fn record_corrections(&mut self, correction_count: usize, _correction_size_bytes: usize) {
        self.correction_count += correction_count;
        // Final accuracy assumes corrections fix all remaining errors
        self.final_accuracy = 1.0;
    }

    /// Record compression statistics
    pub fn record_compression(&mut self, original_size: usize, encoded_size: usize) {
        if encoded_size > 0 {
            self.compression_ratio = original_size as f64 / encoded_size as f64;
        }
    }

    /// Get correction ratio (corrections / total bytes)
    pub fn correction_ratio(&self) -> f64 {
        if self.total_bytes > 0 {
            self.correction_bytes as f64 / self.total_bytes as f64
        } else {
            0.0
        }
    }

    /// Generate human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Accuracy: raw={:.2}%, final={:.2}%, SNR={:.1}dB, corrections={} ({:.2}%)",
            self.raw_accuracy * 100.0,
            self.final_accuracy * 100.0,
            self.snr_db,
            self.correction_count,
            self.correction_ratio() * 100.0
        )
    }
}

/// Combined performance and accuracy metrics for VSA operations
#[derive(Clone, Debug)]
pub struct VsaEvaluationMetrics {
    /// Performance timing metrics
    pub performance: TestMetrics,
    /// Accuracy and fidelity metrics
    pub accuracy: AccuracyMetrics,
    /// Configuration used for evaluation
    pub config_dimension: usize,
    pub config_density: f64,
    pub config_scaling: String,
}

impl VsaEvaluationMetrics {
    /// Create new evaluation metrics
    pub fn new(name: &str, dimension: usize, density: f64, scaling: &str) -> Self {
        Self {
            performance: TestMetrics::new(name),
            accuracy: AccuracyMetrics::new(),
            config_dimension: dimension,
            config_density: density,
            config_scaling: scaling.to_string(),
        }
    }

    /// Generate comprehensive summary report
    pub fn full_summary(&self) -> String {
        let mut report = format!("=== VSA Evaluation: {} ===\n", self.performance.name);
        report.push_str(&format!(
            "Config: dim={}, density={:.3}, scaling={}\n",
            self.config_dimension, self.config_density, self.config_scaling
        ));
        report.push_str(&self.performance.summary());
        report.push_str(&self.accuracy.summary());
        report.push('\n');
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_metrics_timing() {
        let mut metrics = TestMetrics::new("test_operation");

        metrics.start_timing();
        thread::sleep(Duration::from_millis(10));
        metrics.stop_timing();

        let stats = metrics.timing_stats();
        assert_eq!(stats.count, 1);
        assert!(stats.mean_ns > 10_000_000.0); // At least 10ms
    }

    #[test]
    fn test_time_operation() {
        let mut metrics = TestMetrics::new("test");

        let result = metrics.time_operation(|| {
            thread::sleep(Duration::from_millis(5));
            42
        });

        assert_eq!(result, 42);
        assert_eq!(metrics.timings_ns.len(), 1);
    }

    #[test]
    fn test_custom_metrics() {
        let mut metrics = TestMetrics::new("test");
        metrics.record_metric("accuracy", 0.95);
        metrics.record_metric("loss", 0.05);

        assert_eq!(metrics.custom_metrics.get("accuracy"), Some(&0.95));
        assert_eq!(metrics.custom_metrics.get("loss"), Some(&0.05));
    }

    #[test]
    fn test_summary() {
        let mut metrics = TestMetrics::new("test_op");
        metrics.start_timing();
        thread::sleep(Duration::from_millis(1));
        metrics.stop_timing();

        let summary = metrics.summary();
        assert!(summary.contains("test_op"));
        assert!(summary.contains("Timing:"));
    }
}
