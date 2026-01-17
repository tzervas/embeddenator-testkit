//! Example: Basic usage of test data generators
//!
//! Run with: cargo run --example basic_generators

use embeddenator_testkit::*;
use rand::SeedableRng;

fn main() {
    println!("=== Embeddenator TestKit - Basic Generators ===\n");

    // Create a deterministic RNG for reproducibility
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    // Generate random sparse vector
    println!("1. Generating random sparse vector...");
    let vec = random_sparse_vec(&mut rng, 10000, 200);
    println!(
        "   Created vector: dim={}, nnz={} (pos={}, neg={})",
        10000,
        vec.pos.len() + vec.neg.len(),
        vec.pos.len(),
        vec.neg.len()
    );

    // Generate deterministic sparse vector
    println!("\n2. Generating deterministic sparse vector...");
    let vec1 = deterministic_sparse_vec(10000, 200, 42);
    let vec2 = deterministic_sparse_vec(10000, 200, 42);
    println!(
        "   Vector 1: pos.len={}, neg.len={}",
        vec1.pos.len(),
        vec1.neg.len()
    );
    println!(
        "   Vector 2: pos.len={}, neg.len={}",
        vec2.pos.len(),
        vec2.neg.len()
    );
    println!(
        "   Determinism check: {}",
        vec1.pos == vec2.pos && vec1.neg == vec2.neg
    );

    // Test sparse dot product
    println!("\n3. Computing sparse dot product...");
    let a = deterministic_sparse_vec(10000, 100, 123);
    let b = deterministic_sparse_vec(10000, 100, 456);
    let dot_ab = sparse_dot(&a, &b);
    let dot_ba = sparse_dot(&b, &a);
    println!("   dot(a, b) = {}", dot_ab);
    println!("   dot(b, a) = {}", dot_ba);
    println!("   Symmetric: {}", dot_ab == dot_ba);

    // Generate noise pattern
    println!("\n4. Generating noise patterns...");
    let noise1 = generators::generate_noise_pattern(1024, 42);
    let noise2 = generators::generate_noise_pattern(1024, 42);
    println!("   Noise 1 length: {}", noise1.len());
    println!("   Noise 2 length: {}", noise2.len());
    println!("   Deterministic: {}", noise1 == noise2);

    // Generate gradient pattern
    println!("\n5. Generating gradient pattern...");
    let gradient = generators::generate_gradient_pattern(256, 256);
    println!("   Gradient size: {} bytes", gradient.len());
    println!("   First pixel: {}", gradient[0]);
    println!("   Center pixel: {}", gradient[gradient.len() / 2]);
    println!("   Last pixel: {}", gradient[gradient.len() - 1]);

    println!("\n✅ All generators working correctly!");
}
