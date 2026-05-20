//! Benchmark: Fixed-point square root — zero/negative vs. positive.
//!
//! The `Fixed::sqrt` implementation uses a binary digit-by-digit (restoring)
//! algorithm that runs exactly 64 iterations for **all** inputs, including
//! zero and negative values. There is no early return branch.
//!
//! For non-positive inputs, `unsigned_abs()` converts them to their absolute
//! value, and the algorithm naturally produces zero since all input bits are
//! zero (for zero) or the absolute value is used (for negative inputs, the
//! sqrt of the absolute value is computed — though in practice, callers
//! should use `abs()` first).
//!
//! ## Classes
//!
//! - **Left (zero/negative):** Values ≤ 0
//! - **Right (positive):** Small positive values
//!
//! ## Expected Result
//!
//! `|t| < 5` — confirming that the digit-by-digit sqrt is truly constant-time
//! across all input classes, including non-positive values.
//!
//! ## Note
//!
//! The old Newton-Raphson implementation had an early return for
//! `if self.0 <= 0 { return Fixed::ZERO }` that caused a timing signal of
//! `|t| ≈ 358`. The digit-by-digit algorithm eliminated this branch.

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::Fixed;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 100_000;

/// Benchmark sqrt: zero/negative vs. positive.
pub fn bench_fixed_sqrt_edge(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut inputs = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        if rng.random::<bool>() {
            // Left class: zero or negative
            if rng.random::<bool>() {
                inputs.push(Fixed::ZERO);
            } else {
                let neg: i64 = rng.random_range(-10_000i64..=-1);
                inputs.push(Fixed::from_int(neg));
            }
            classes.push(Class::Left);
        } else {
            // Right class: small positive
            let val: i64 = rng.random_range(1i64..=10_000);
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
