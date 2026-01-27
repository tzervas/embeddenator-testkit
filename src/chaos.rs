//! Chaos injection and resilience testing utilities
//!
//! Provides tools for testing system resilience:
//! - Random bitflip injection on byte data
//! - Packet loss simulation
//! - Corruption simulation
//! - Noise tolerance testing

/// Chaos injection utilities for resilience testing
pub struct ChaosInjector {
    /// Random seed for reproducibility
    seed: u64,
    /// Injection probability (0.0 - 1.0)
    probability: f64,
}

impl ChaosInjector {
    /// Create new chaos injector with seed
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            probability: 0.01, // 1% default
        }
    }

    /// Set injection probability
    pub fn with_probability(mut self, p: f64) -> Self {
        self.probability = p.clamp(0.0, 1.0);
        self
    }

    /// Inject random noise into byte data
    ///
    /// # Arguments
    /// * `data` - Data to corrupt (modified in place)
    /// * `error_rate` - Fraction of bits to flip (0.0-1.0)
    pub fn corrupt_bytes(&self, data: &mut [u8], error_rate: f64) {
        let mut state = self.seed;
        let num_errors = ((data.len() as f64) * error_rate) as usize;

        for _ in 0..num_errors {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let pos = (state as usize) % data.len();
            let bit = (state >> 8) % 8;
            data[pos] ^= 1u8 << bit;
        }
    }

    /// Create corrupted copy of byte data
    pub fn corrupt_copy(&self, data: &[u8], error_rate: f64) -> Vec<u8> {
        let mut corrupted = data.to_vec();
        self.corrupt_bytes(&mut corrupted, error_rate);
        corrupted
    }

    /// Simulate packet loss by erasing random chunks
    ///
    /// # Arguments
    /// * `data` - Data to corrupt (modified in place)
    /// * `loss_rate` - Fraction of packets to drop (0.0-1.0)
    /// * `packet_size` - Size of each packet in bytes
    pub fn simulate_packet_loss(&self, data: &mut [u8], loss_rate: f64, packet_size: usize) {
        use std::collections::HashSet;

        let num_packets = data.len().div_ceil(packet_size);
        let packets_to_drop = ((num_packets as f64) * loss_rate) as usize;

        let mut state = self.seed;
        let mut dropped = HashSet::new();

        for _ in 0..packets_to_drop {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let packet_idx = (state as usize) % num_packets;
            dropped.insert(packet_idx);
        }

        for packet_idx in dropped {
            let start = packet_idx * packet_size;
            let end = (start + packet_size).min(data.len());
            data[start..end].fill(0);
        }
    }

    /// Inject random erasures (zero out bytes)
    pub fn inject_erasures(&self, data: &mut [u8], count: usize) -> Vec<usize> {
        let mut erased = Vec::new();
        let mut state = self.seed.wrapping_add(12345);

        for _ in 0..count.min(data.len()) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let pos = (state as usize) % data.len();

            if data[pos] != 0 {
                data[pos] = 0;
                erased.push(pos);
            }
        }

        erased
    }
}

impl Default for ChaosInjector {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corrupt_bytes() {
        let mut data = vec![0u8; 100];
        let injector = ChaosInjector::new(42);

        injector.corrupt_bytes(&mut data, 0.1);

        let corrupted_count = data.iter().filter(|&&b| b != 0).count();
        assert!(corrupted_count > 0);
    }

    #[test]
    fn test_corrupt_copy() {
        let data = vec![0xFF; 100];
        let injector = ChaosInjector::new(42);

        let corrupted = injector.corrupt_copy(&data, 0.1);

        // Original unchanged
        assert!(data.iter().all(|&b| b == 0xFF));

        // Corrupted is different
        assert_ne!(data, corrupted);
    }

    #[test]
    fn test_simulate_packet_loss() {
        let mut data = vec![0xFF; 100];
        let injector = ChaosInjector::new(42);

        injector.simulate_packet_loss(&mut data, 0.2, 10); // 20% loss, 10 byte packets

        let zero_count = data.iter().filter(|&&b| b == 0).count();
        assert!(zero_count > 0);
    }

    #[test]
    fn test_inject_erasures() {
        let mut data = vec![0xFF; 100];
        let injector = ChaosInjector::new(42);

        let erased = injector.inject_erasures(&mut data, 10);

        assert!(erased.len() <= 10);

        // Check that erased positions are now zero
        for &pos in &erased {
            assert_eq!(data[pos], 0);
        }
    }

    #[test]
    fn test_determinism() {
        let data = vec![0xFF; 100];

        let injector1 = ChaosInjector::new(42);
        let corrupted1 = injector1.corrupt_copy(&data, 0.1);

        let injector2 = ChaosInjector::new(42);
        let corrupted2 = injector2.corrupt_copy(&data, 0.1);

        assert_eq!(corrupted1, corrupted2);
    }
}
