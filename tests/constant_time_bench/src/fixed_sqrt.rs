//! Benchmark: Fixed-point square root — varying positive magnitudes.
//!
//! The `Fixed::sqrt` implementation uses Newton's method with exactly 20
//! iterations for all positive inputs. However:
//! - Line 110 has a data-dependent branch: `if self < Fixed::from_raw(1 << 8)`
//!   which clamps the initial guess for extremely small values.
//! - The Newton iteration `x = (x + a/x) / 2` involves division, which is
//!   the most timing-sensitive operation.
//!
//! This benchmark tests two scenarios:
//! 1. **All-positive magnitudes:** Values in [1, 10⁹] — the realistic range
//!    for the cryptosystem (squared distances, etc.). Both classes use values
//!    that take the same code path (no early return, no clamping).
//! 2. **Clamping edge case:** Values below vs. above the clamping threshold
//!    to quantify the impact of the branch on line 110.
//!
//! ## Classes (Benchmark 1: Realistic range)
//!
//! - **Left:** Small positive values in [1, 10³]
//! - **Right:** Large positive values in [10⁶, 10⁹]
//!
//! ## Classes (Benchmark 2: Clamping threshold)
//!
//! - **Left:** Values below the clamping threshold (< 2^8 raw)
//! - **Right:** Values just above the clamping threshold

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::Fixed;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 100_000;

/// Benchmark sqrt: small vs. large positive values (realistic range).
///
/// Both classes are positive and take the same code path (no early return).
/// This tests whether the Newton iteration convergence behavior differs
/// based on input magnitude.
pub fn bench_fixed_sqrt(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut inputs = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        if rng.random::<bool>() {
            // Left class: small positive values
            let val: i64 = rng.random_range(1i64..=1_000);
            inputs.push(Fixed::from_int(val));
            classes.push(Class::Left);
        } else {
            // Right class: large positive values
            let val: i64 = rng.random_range(1_000_000i64..=1_000_000_000);
            inputs.push(Fixed::from_int(val));
            classes.push(Class::Right);
        }
    }

    for (input, class) in inputs.into_iter().zip(classes) {
        runner.run_one(class, || {
            let _result = input.sqrt();
        });
    }
}

/// Benchmark sqrt: values below vs. above the clamping threshold.
///
/// This quantifies the timing impact of the branch on line 110 of fixed_math.rs:
/// `let mut x = if self < Fixed::from_raw(1 << 8) { ... } else { ... };`
pub fn bench_fixed_sqrt_clamp(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut inputs = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        if rng.random::<bool>() {
            // Left class: below clamping threshold (< 2^8 raw)
            let frac: u64 = rng.random_range(0u64..=(1u64 << 8) - 1);
            inputs.push(Fixed::from_parts(0, frac));
            classes.push(Class::Left);
        } else {
            // Right class: just above clamping threshold
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
