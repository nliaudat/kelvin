//! Benchmark: Fixed-point multiplication — direct path vs. high/low splitting.
//!
//! The `Fixed::mul` implementation has two code paths:
//! 1. **Direct path:** When `checked_mul` succeeds (no i128 overflow), the
//!    product is computed as `(a * b) >> 64` with rounding.
//! 2. **Fallback path:** When `checked_mul` overflows i128, the operands are
//!    split into high/low 64-bit halves and the result is computed via
//!    multi-term addition.
//!
//! This benchmark tests whether these two paths have measurably different
//! execution times. If they do, an attacker who can control input magnitudes
//! could distinguish which path was taken, leaking information about the
//! values being multiplied.
//!
//! ## Classes
//!
//! - **Left (small values):** Values in [-10⁴, 10⁴] where `checked_mul`
//!   succeeds (no overflow).
//! - **Right (large values):** Values in [10⁹, 10¹²] where `checked_mul`
//!   overflows i128 and the high/low splitting fallback is used.

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
            // Left class: small values where checked_mul succeeds
            // Range: [-10_000, 10_000] in Q32.64
            let a_int: i64 = rng.random_range(-10_000i64..=10_000);
            let b_int: i64 = rng.random_range(-10_000i64..=10_000);
            inputs_a.push(Fixed::from_int(a_int));
            inputs_b.push(Fixed::from_int(b_int));
            classes.push(Class::Left);
        } else {
            // Right class: large values where checked_mul overflows i128
            // Range: [10^9, 10^12] — product ~10^18 to 10^24, exceeds i128 max (~1.7e38)
            // Actually i128 max is ~1.7e38, so we need values where a*b > 2^127
            // 2^127 ≈ 1.7e38. With Q32.64 scaling, a*b in raw i128 must exceed i128::MAX.
            // For values ~10^9 in Q32.64: raw = 10^9 * 2^64 ≈ 1.8e28
            // Product of two such values: ~3.4e56 which overflows i128.
            // So values >= 10^9 will trigger the fallback.
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
