//! Benchmark: KDF entropy extraction — varying masses.
//!
//! Tests the `extract_seed` function which hashes the orbital state
//! (positions, velocities, masses, accelerations) through SHA3-512.
//! SHA3-512 should be constant-time by design, but this benchmark
//! verifies that the composition with `feed_orbital_state` and
//! `compute_accelerations` doesn't introduce timing variations.
//!
//! ## Classes
//!
//! - **Left (small masses):** Bodies with masses 1–100 M☉
//! - **Right (large masses):** Bodies with masses 10⁶–10⁹ M☉
//!
//! Both classes use the same distance range (1–1000 AU), isolating the mass
//! variation as the only difference between classes.

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_G, SOFTENING_FACTOR};
use kelvin_kdf::extract_seed;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 50_000;

/// Benchmark extract_seed: small vs. large masses.
pub fn bench_extract_seed(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut classes = Vec::with_capacity(NUM_SAMPLES);
    let mut params = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        // Both classes use the same distance range
        let d1 = rng.random_range(1i64..=1000);
        let d2 = rng.random_range(1i64..=1000);
        let step = rng.random_range(0u64..=10_000_000);

        if rng.random::<bool>() {
            // Left class: small masses
            let mass1 = rng.random_range(1i64..=100);
            let mass2 = rng.random_range(1i64..=100);
            params.push((mass1, mass2, d1, d2, step));
            classes.push(Class::Left);
        } else {
            // Right class: large masses
            let mass1 = rng.random_range(1_000_000i64..=1_000_000_000);
            let mass2 = rng.random_range(1_000_000i64..=1_000_000_000);
            params.push((mass1, mass2, d1, d2, step));
            classes.push(Class::Right);
        }
    }

    for ((m1, m2, d1, d2, step), class) in params.into_iter().zip(classes) {
        runner.run_one(class, || {
            let bodies = [
                OrbitalBody::new(
                    Fixed::from_int(m1),
                    Vec3::new(Fixed::from_int(d1), Fixed::ZERO, Fixed::ZERO),
                    Vec3::ZERO,
                ),
                OrbitalBody::new(
                    Fixed::from_int(m2),
                    Vec3::new(Fixed::ZERO, Fixed::from_int(d2), Fixed::ZERO),
                    Vec3::ZERO,
                ),
            ];
            let _seed =
                extract_seed(&bodies, step, DEFAULT_G, SOFTENING_FACTOR, b"kelvin-ct-bench");
        });
    }
}
