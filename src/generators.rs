//! Test data generators for VSA vectors and test datasets
//!
//! Provides utilities to generate:
//! - Random sparse vectors with controlled sparsity
//! - Deterministic vectors for reproducible testing
//! - Noise patterns and synthetic data
//! - Test helper functions for VSA operations

use embeddenator::SparseVec;
use rand::Rng;
use std::collections::HashSet;

/// Generate a random sparse vector with specified dimensions and sparsity
///
/// # Arguments
/// * `rng` - Random number generator
/// * `dims` - Total dimensions of the vector
/// * `sparsity` - Number of non-zero elements (split roughly evenly between pos/neg)
///
/// # Example
/// ```rust,ignore
/// use rand::thread_rng;
/// let mut rng = thread_rng();
/// let vec = random_sparse_vec(&mut rng, 10000, 200);
/// assert_eq!(vec.pos.len() + vec.neg.len(), 200);
/// ```
pub fn random_sparse_vec(rng: &mut impl Rng, dims: usize, sparsity: usize) -> SparseVec {
    let mut used: HashSet<usize> = HashSet::with_capacity(sparsity.saturating_mul(2));
    let mut pos = Vec::with_capacity(sparsity / 2);
    let mut neg = Vec::with_capacity(sparsity / 2);

    // Roughly half pos/half neg.
    let target_each = sparsity / 2;
    while pos.len() < target_each {
        let idx = rng.random_range(0..dims);
        if used.insert(idx) {
            pos.push(idx);
        }
    }
    while neg.len() < target_each {
        let idx = rng.random_range(0..dims);
        if used.insert(idx) {
            neg.push(idx);
        }
    }

    pos.sort_unstable();
    neg.sort_unstable();
    SparseVec { pos, neg }
}

/// Alias for `random_sparse_vec` for backwards compatibility
pub fn mk_random_sparsevec(rng: &mut impl Rng, dims: usize, sparsity: usize) -> SparseVec {
    random_sparse_vec(rng, dims, sparsity)
}

/// Generate a deterministic sparse vector using LCG for reproducibility
///
/// # Arguments
/// * `dim` - Total dimensions of the vector
/// * `nnz` - Number of non-zero elements
/// * `seed` - Random seed for reproducibility
///
/// # Example
/// ```rust,ignore
/// let vec1 = deterministic_sparse_vec(10000, 200, 42);
/// let vec2 = deterministic_sparse_vec(10000, 200, 42);
/// assert_eq!(vec1.pos, vec2.pos);
/// assert_eq!(vec1.neg, vec2.neg);
/// ```
pub fn deterministic_sparse_vec(dim: usize, nnz: usize, seed: u64) -> SparseVec {
    // Split nnz roughly evenly between pos and neg
    let pos_count = nnz / 2;
    let neg_count = nnz - pos_count;

    let mut state = seed;
    let lcg = |s: &mut u64| -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        *s
    };

    let mut pos = Vec::with_capacity(pos_count);
    let mut neg = Vec::with_capacity(neg_count);
    let mut used = HashSet::new();

    for _ in 0..pos_count {
        loop {
            let idx = (lcg(&mut state) as usize) % dim;
            if used.insert(idx) {
                pos.push(idx);
                break;
            }
        }
    }

    for _ in 0..neg_count {
        loop {
            let idx = (lcg(&mut state) as usize) % dim;
            if used.insert(idx) {
                neg.push(idx);
                break;
            }
        }
    }

    pos.sort_unstable();
    neg.sort_unstable();

    SparseVec { pos, neg }
}

/// Count intersections between two sorted slices (used for dot product)
fn intersection_count_sorted(a: &[usize], b: &[usize]) -> usize {
    let mut i = 0;
    let mut j = 0;
    let mut count = 0;
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                i += 1;
                j += 1;
            }
        }
    }
    count
}

/// Compute sparse ternary dot product: (pp + nn) - (pn + np)
///
/// This is a reference implementation useful for testing optimized dot product implementations.
///
/// # Arguments
/// * `a` - First sparse vector
/// * `b` - Second sparse vector
///
/// # Returns
/// Dot product as i32
pub fn sparse_dot(a: &SparseVec, b: &SparseVec) -> i32 {
    let pp = intersection_count_sorted(&a.pos, &b.pos) as i32;
    let nn = intersection_count_sorted(&a.neg, &b.neg) as i32;
    let pn = intersection_count_sorted(&a.pos, &b.neg) as i32;
    let np = intersection_count_sorted(&a.neg, &b.pos) as i32;
    (pp + nn) - (pn + np)
}

/// Generate synthetic noise pattern using LCG
///
/// Useful for creating reproducible pseudo-random test data.
pub fn generate_noise_pattern(size: usize, seed: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let mut state = seed;
    for _ in 0..size {
        // Simple LCG for reproducible pseudo-random data
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        data.push((state >> 56) as u8);
    }
    data
}

/// Generate synthetic gradient pattern (useful for image-like data)
pub fn generate_gradient_pattern(width: usize, height: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            // Linear gradient from top-left to bottom-right
            let val = ((x + y) * 255) / (width + height);
            data.push(val as u8);
        }
    }
    data
}

/// Generate synthetic binary blob (executable-like pattern)
pub fn generate_binary_blob(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);

    // ELF-like header
    if size >= 16 {
        data.extend_from_slice(&[0x7f, b'E', b'L', b'F']);
        data.extend_from_slice(&[2, 1, 1, 0]); // 64-bit, little endian, v1, SYSV
        data.extend_from_slice(&[0; 8]); // padding
    }

    // Fill with mix of patterns
    let mut offset = data.len();
    while offset < size {
        let pattern_type = (offset / 256) % 4;
        match pattern_type {
            0 => data.push(0x90),                  // NOP slide
            1 => data.push((offset & 0xFF) as u8), // Sequential
            2 => data.push(0x00),                  // Zero fill
            _ => data.push(0xCC),                  // INT3
        }
        offset += 1;
    }

    data.truncate(size);
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_random_sparse_vec() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let vec = random_sparse_vec(&mut rng, 10000, 200);
        let nnz = vec.pos.len() + vec.neg.len();
        assert_eq!(nnz, 200);

        // Check sorted
        assert!(vec.pos.windows(2).all(|w| w[0] < w[1]));
        assert!(vec.neg.windows(2).all(|w| w[0] < w[1]));

        // Check no overlap
        let pos_set: HashSet<_> = vec.pos.iter().collect();
        let neg_set: HashSet<_> = vec.neg.iter().collect();
        assert_eq!(pos_set.intersection(&neg_set).count(), 0);
    }

    #[test]
    fn test_deterministic_sparse_vec() {
        let vec1 = deterministic_sparse_vec(10000, 200, 42);
        let vec2 = deterministic_sparse_vec(10000, 200, 42);
        assert_eq!(vec1.pos, vec2.pos);
        assert_eq!(vec1.neg, vec2.neg);

        // Different seed should give different result
        let vec3 = deterministic_sparse_vec(10000, 200, 43);
        assert_ne!(vec1.pos, vec3.pos);
    }

    #[test]
    fn test_sparse_dot() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let a = random_sparse_vec(&mut rng, 10000, 200);
        let b = random_sparse_vec(&mut rng, 10000, 200);

        let dot = sparse_dot(&a, &b);

        // Dot product should be symmetric
        let dot_rev = sparse_dot(&b, &a);
        assert_eq!(dot, dot_rev);
    }

    #[test]
    fn test_generate_noise_pattern() {
        let data1 = generate_noise_pattern(1000, 42);
        let data2 = generate_noise_pattern(1000, 42);
        assert_eq!(data1, data2);

        let data3 = generate_noise_pattern(1000, 43);
        assert_ne!(data1, data3);
    }
}
