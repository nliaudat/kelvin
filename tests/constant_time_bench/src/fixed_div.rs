//! Benchmark: Fixed-point division — small vs. large magnitudes.
//!
//! The `Fixed::div` implementation uses a bit-by-bit long division loop that
//! always runs exactly 64 iterations. However, the sign handling and the
//! conditional `if high_bit == 1 || rem >= b_abs` inside the loop could
//! theoretically introduce timing variations depending on the values.
//!
//! This benchmark tests whether the division timing is independent of:
//! - The magnitude of the dividend and divisor
//! - The sign of the operands (positive vs. negative)
//!
//! ## Classes
//!
//! - **Left (small magnitudes):** Dividend and divisor in [1, 100]
//! - **Right (large magnitudes):** Dividend in [10⁶, 10⁹], divisor in [10⁻³, 100]
//!
//! A separate sub-test tests positive vs. negative signs.

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::Fixed;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 100_000;

/// Benchmark division: small vs. large magnitudes.
pub fn bench_fixed_div_magnitude(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut inputs_num = Vec::with_capacity(NUM_SAMPLES);
    let mut inputs_den = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        if rng.random::<bool>() {
            // Left class: small magnitudes
            let num: i64 = rng.random_range(1i64..=100);
            let den: i64 = rng.random_range(1i64..=100);
            inputs_num.push(Fixed::from_int(num));
            inputs_den.push(Fixed::from_int(den));
            classes.push(Class::Left);
        } else {
            // Right class: large magnitudes
            let num: i64 = rng.random_range(1_000_000i64..=1_000_000_000);
            // Use fractional divisors via from_parts
            let den_int: i64 = rng.random_range(1i64..=100);
            let den_frac: u64 = rng.random();
            inputs_num.push(Fixed::from_int(num));
            inputs_den.push(Fixed::from_parts(den_int, den_frac));
            classes.push(Class::Right);
        }
    }

    for ((num, den), class) in inputs_num.into_iter().zip(inputs_den).zip(classes) {
        runner.run_one(class, || {
            let _result = num / den;
        });
    }
}

/// Benchmark division: positive vs. negative signs.
///
/// The sign handling in `Fixed::div` computes `sign = (a < 0) ^ (b < 0)` and
/// negates the result if sign is true. This test verifies that the negation
/// doesn't introduce a measurable timing difference.
pub fn bench_fixed_div_sign(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut inputs_num = Vec::with_capacity(NUM_SAMPLES);
    let mut inputs_den = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        let num: i64 = rng.random_range(1i64..=10_000);
        let den: i64 = rng.random_range(1i64..=100);

        if rng.random::<bool>() {
            // Left class: both positive
            inputs_num.push(Fixed::from_int(num));
            inputs_den.push(Fixed::from_int(den));
            classes.push(Class::Left);
        } else {
            // Right class: one or both negative
            let num_signed = if rng.random::<bool>() { num } else { -num };
            let den_signed = if rng.random::<bool>() { den } else { -den };
            inputs_num.push(Fixed::from_int(num_signed));
            inputs_den.push(Fixed::from_int(den_signed));
            classes.push(Class::Right);
        }
    }

    for ((num, den), class) in inputs_num.into_iter().zip(inputs_den).zip(classes) {
        runner.run_one(class, || {
            let _result = num / den;
        });
    }
}
