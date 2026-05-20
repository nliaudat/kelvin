//! Benchmark: Fixed-point square root — small vs. large fractional values.
//!
//! The `Fixed::sqrt` implementation uses a binary digit-by-digit (restoring)
//! algorithm that runs exactly 64 iterations regardless of input magnitude.
//! There is no clamping branch — all positive values take the same code path.
//!
//! This benchmark verifies that sqrt timing is independent of the fractional
//! part magnitude by comparing very small fractional values with larger ones.
//!
//! ## Classes
//!
//! - **Left:** Very small fractional values (< 2^8 raw, ~3.9e-19)
//! - **Right:** Larger fractional values
//!
//! ## Expected Result
//!
//! `|t| < 5` — confirming constant-time behavior across the fractional range.
//!
//! ## Note
//!
//! The old Newton-Raphson implementation had a clamping branch at
//! `if self < Fixed::from_raw(1 << 8)` that caused a timing signal of
//! `|t| ≈ 1209`. The digit-by-digit algorithm eliminated this branch.

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::Fixed;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 100_000;

/// Benchmark sqrt: small vs. large fractional values.
pub fn bench_fixed_sqrt_clamp(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut inputs = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        if rng.random::<bool>() {
            // Left class: very small fractional values (< 2^8 raw)
            let frac: u64 = rng.random_range(0u64..=(1u64 << 8) - 1);
            inputs.push(Fixed::from_parts(0, frac));
            classes.push(Class::Left);
        } else {
            // Right class: larger fractional values
            let frac: u64 = rng.random_range((1u64 << 8) + 1..=1u64 << 16);
            inputs.push(Fixed::from_parts(0, frac));
            classes.push(Class::Right);
        }
    }

    for (input, class) in inputs.into_iter().zip(classes) {
        runner.run_one(class, || {
            let _result = input.sqrt();
        });
    }
}
