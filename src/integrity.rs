//! Data integrity validation utilities
//!
//! Provides tools for validating:
//! - Sparse vector invariants
//! - VSA operation properties (commutativity, self-inverse, etc.)
//! - Data corruption detection
//! - Algebraic invariants

use embeddenator::SparseVec;
use std::collections::HashSet;

/// Results from integrity validation
#[derive(Clone, Debug, Default)]
pub struct IntegrityReport {
    /// Total checks performed
    pub checks_total: u64,
    /// Checks that passed
    pub checks_passed: u64,
    /// Detected bitflips (single bit errors)
    pub bitflips_detected: u64,
    /// Multi-bit corruption events
    pub corruption_events: u64,
    /// Algebraic invariant violations
    pub invariant_violations: u64,
    /// Specific failure messages
    pub failures: Vec<String>,
}

impl IntegrityReport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if all validations passed
    pub fn is_ok(&self) -> bool {
        self.checks_passed == self.checks_total && self.failures.is_empty()
    }

    /// Pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.checks_total == 0 {
            100.0
        } else {
            (self.checks_passed as f64 / self.checks_total as f64) * 100.0
        }
    }

    /// Record a passed check
    pub fn pass(&mut self) {
        self.checks_total += 1;
        self.checks_passed += 1;
    }

    /// Record a failed check with message
    pub fn fail(&mut self, msg: impl Into<String>) {
        self.checks_total += 1;
        self.failures.push(msg.into());
    }

    /// Record detected bitflip
    pub fn record_bitflip(&mut self) {
        self.bitflips_detected += 1;
    }

    /// Record corruption event
    pub fn record_corruption(&mut self) {
        self.corruption_events += 1;
    }

    /// Record invariant violation
    pub fn record_invariant_violation(&mut self, msg: impl Into<String>) {
        self.invariant_violations += 1;
        self.failures.push(format!("INVARIANT: {}", msg.into()));
    }

    /// Generate summary report
    pub fn summary(&self) -> String {
        format!(
            "Integrity Report:\n\
             - Total checks: {}\n\
             - Passed: {}\n\
             - Failed: {}\n\
             - Pass rate: {:.1}%\n\
             - Bitflips: {}\n\
             - Corruption events: {}\n\
             - Invariant violations: {}",
            self.checks_total,
            self.checks_passed,
            self.checks_total - self.checks_passed,
            self.pass_rate(),
            self.bitflips_detected,
            self.corruption_events,
            self.invariant_violations
        )
    }
}

/// Validates data integrity for VSA operations
pub struct IntegrityValidator {
    /// Enable verbose logging
    pub verbose: bool,
}

impl IntegrityValidator {
    pub fn new() -> Self {
        Self { verbose: false }
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Validate sparse vector invariants
    ///
    /// Checks:
    /// - No overlap between pos and neg indices
    /// - Indices are sorted
    /// - No duplicate indices
    pub fn validate_sparse(&self, v: &SparseVec) -> IntegrityReport {
        let mut report = IntegrityReport::default();

        // Check no overlap between pos and neg
        let pos_set: HashSet<_> = v.pos.iter().collect();
        let neg_set: HashSet<_> = v.neg.iter().collect();
        if pos_set.intersection(&neg_set).count() > 0 {
            report.record_corruption();
            report.fail("Overlap between pos and neg indices");
        } else {
            report.pass();
        }

        // Check sorted
        if !v.pos.windows(2).all(|w| w[0] < w[1]) {
            report.fail("pos indices not sorted");
        } else {
            report.pass();
        }

        if !v.neg.windows(2).all(|w| w[0] < w[1]) {
            report.fail("neg indices not sorted");
        } else {
            report.pass();
        }

        report
    }

    /// Validate algebraic invariants for bind operation
    ///
    /// Checks:
    /// - Commutativity: A ⊙ B = B ⊙ A
    pub fn validate_bind_invariants(&self, a: &SparseVec, b: &SparseVec) -> IntegrityReport {
        let mut report = IntegrityReport::default();

        // Commutativity check
        let ab = a.bind(b);
        let ba = b.bind(a);

        if ab.pos != ba.pos || ab.neg != ba.neg {
            report.record_invariant_violation("Commutativity violation: A⊙B ≠ B⊙A");
        } else {
            report.pass();
        }

        report
    }

    /// Validate bundle operation properties
    pub fn validate_bundle_invariants(&self, a: &SparseVec, b: &SparseVec) -> IntegrityReport {
        let mut report = IntegrityReport::default();

        // Commutativity check
        let ab = a.bundle(b);
        let ba = b.bundle(a);

        if ab.pos != ba.pos || ab.neg != ba.neg {
            report.record_invariant_violation("Bundle commutativity violation: A⊕B ≠ B⊕A");
        } else {
            report.pass();
        }

        report
    }

    /// Detect potential corruption by comparing two vectors
    pub fn detect_differences(&self, expected: &SparseVec, actual: &SparseVec) -> IntegrityReport {
        let mut report = IntegrityReport::default();

        // Compare pos indices
        if expected.pos != actual.pos {
            let diff_count = expected.pos.len().abs_diff(actual.pos.len());
            report.record_corruption();
            report.fail(format!("pos indices differ by {} elements", diff_count));
        } else {
            report.pass();
        }

        // Compare neg indices
        if expected.neg != actual.neg {
            let diff_count = expected.neg.len().abs_diff(actual.neg.len());
            report.record_corruption();
            report.fail(format!("neg indices differ by {} elements", diff_count));
        } else {
            report.pass();
        }

        report
    }
}

impl Default for IntegrityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrity_report() {
        let mut report = IntegrityReport::new();
        assert!(report.is_ok());

        report.pass();
        assert_eq!(report.checks_total, 1);
        assert_eq!(report.checks_passed, 1);

        report.fail("test failure");
        assert_eq!(report.checks_total, 2);
        assert_eq!(report.checks_passed, 1);
        assert!(!report.is_ok());
    }

    #[test]
    fn test_validate_sparse() {
        let validator = IntegrityValidator::new();

        // Create a valid sparse vector
        let sparse = SparseVec {
            pos: vec![0, 10, 20],
            neg: vec![5, 15, 25],
        };

        let report = validator.validate_sparse(&sparse);
        assert!(report.is_ok());
    }

    #[test]
    fn test_bind_invariants() {
        let validator = IntegrityValidator::new();

        let sparse_a = SparseVec {
            pos: vec![0, 10, 20],
            neg: vec![5, 15, 25],
        };
        let sparse_b = SparseVec {
            pos: vec![1, 11, 21],
            neg: vec![6, 16, 26],
        };

        let report = validator.validate_bind_invariants(&sparse_a, &sparse_b);
        // Should pass commutativity
        assert!(report.checks_passed > 0);
    }
}
