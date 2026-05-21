//! Benchmark: Fixed-point multiplication — small vs. large values.
//!
//! The `Fixed::mul` implementation always uses high/low 64-bit splitting
//! (the `checked_mul` branch was removed in the constant-time rewrite).
//! This benchmark verifies that the single code path runs in constant time
//! regardless of operand magnitude.
//!
//! ## Classes
//!
//! - **Left (small values):** Values in [-10⁴, 10⁴]
//! - **Right (large values):** Values in [10⁹, 10¹²]

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::Fixed;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 100_000;

/// Benchmark multiplication: direct path vs. high/low splitting fallback.
pub fn bench_fixed_mul(runner: &mut CtRunner, rng: &mut BenchRng) {
    // Pre-allocate vectors
    let mut inputs_a = Vec::with_capacity(NUM_SAMPLES);
    let mut inputs_b = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        // Randomly assign to Left or Right distribution
        if rng.random::<bool>() {
            // Left class: small values
            // Range: [-10_000, 10_000] in Q32.64
            let a_int: i64 = rng.random_range(-10_000i64..=10_000);
            let b_int: i64 = rng.random_range(-10_000i64..=10_000);
            inputs_a.push(Fixed::from_int(a_int));
            inputs_b.push(Fixed::from_int(b_int));
            classes.push(Class::Left);
        } else {
            // Right class: large values
            // Range: [10^9, 10^12] in Q32.64
            let a_int: i64 = rng.random_range(1_000_000_000i64..=10_000_000_000i64);
            let b_int: i64 = rng.random_range(1_000_000_000i64..=10_000_000_000i64);
            inputs_a.push(Fixed::from_int(a_int));
            inputs_b.push(Fixed::from_int(b_int));
            classes.push(Class::Right);
        }
    }

    // Time each multiplication
    for ((a, b), class) in inputs_a.into_iter().zip(inputs_b).zip(classes) {
        runner.run_one(class, || {
            let _result = a * b;
        });
    }
}
