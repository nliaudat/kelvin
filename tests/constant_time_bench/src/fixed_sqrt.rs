//! Benchmark: Fixed-point square root — varying positive magnitudes.
//!
//! The `Fixed::sqrt` implementation uses a binary digit-by-digit (restoring)
//! algorithm that runs exactly 96 iterations regardless of input magnitude.
//! It uses only comparisons, subtractions, and bit shifts — no division,
//! no multiplication, no data-dependent branching.
//!
//! This benchmark tests whether sqrt timing is independent of input magnitude
//! by comparing small positive values with large positive values.
//!
//! ## Classes
//!
//! - **Left:** Small positive values in [1, 10³]
//! - **Right:** Large positive values in [10⁶, 10⁹]

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::Fixed;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 100_000;

/// Benchmark sqrt: small vs. large positive values (realistic range).
///
/// Both classes are positive and take the same code path (no early return,
/// no clamping). This tests whether the digit-by-digit algorithm's timing
/// is independent of input magnitude.
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
