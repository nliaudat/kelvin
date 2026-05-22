//! Benchmark: Full Euler integration step — varying masses.
//!
//! Tests the `euler_step` function which uses explicit Euler integration
//! (position update before velocity update). This is the preferred method
//! for entropy generation due to its numerical instability.
//!
//! ## Classes
//!
//! - **Left (small masses):** Bodies with masses 1–100 M☉
//! - **Right (large masses):** Bodies with masses 10⁶–10⁹ M☉
//!
//! Both classes use the same distance range (1–1000 AU), isolating the mass
//! variation as the only difference between classes.

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::{euler_step, Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 50_000;

/// Benchmark euler_step: small vs. large masses.
pub fn bench_euler_step(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut classes = Vec::with_capacity(NUM_SAMPLES);
    let mut params = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        // Both classes use the same distance range
        let d1 = rng.random_range(1i64..=1000);
        let d2 = rng.random_range(1i64..=1000);
        let d3 = rng.random_range(1i64..=1000);

        if rng.random::<bool>() {
            // Left class: small masses
            let mass1 = rng.random_range(1i64..=100);
            let mass2 = rng.random_range(1i64..=100);
            let mass3 = rng.random_range(1i64..=100);
            params.push((mass1, mass2, mass3, d1, d2, d3));
            classes.push(Class::Left);
        } else {
            // Right class: large masses
            let mass1 = rng.random_range(1_000_000i64..=1_000_000_000);
            let mass2 = rng.random_range(1_000_000i64..=1_000_000_000);
            let mass3 = rng.random_range(1_000_000i64..=1_000_000_000);
            params.push((mass1, mass2, mass3, d1, d2, d3));
            classes.push(Class::Right);
        }
    }

    for ((m1, m2, m3, d1, d2, d3), class) in params.into_iter().zip(classes) {
        runner.run_one(class, || {
            let mut bodies = vec![
                OrbitalBody::new(
                    Fixed::from_int(m1),
                    Vec3::new(Fixed::from_int(d1), Fixed::ZERO, Fixed::ZERO),
                    Vec3::new(Fixed::ZERO, Fixed::from_int(1), Fixed::ZERO),
                ),
                OrbitalBody::new(
                    Fixed::from_int(m2),
                    Vec3::new(Fixed::ZERO, Fixed::from_int(d2), Fixed::ZERO),
                    Vec3::new(Fixed::from_int(-1), Fixed::ZERO, Fixed::ZERO),
                ),
                OrbitalBody::new(
                    Fixed::from_int(m3),
                    Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::from_int(d3)),
                    Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
                ),
            ];
            euler_step(&mut bodies, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G);
        });
    }
}
