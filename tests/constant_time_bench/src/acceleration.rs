//! Benchmark: N-body gravitational acceleration — varying distances and masses.
//!
//! This benchmark tests the critical path of the KDF entropy extraction:
//! `compute_accelerations()` which computes gravitational forces between
//! bodies. This involves:
//! - `Vec3::length()` (which calls `sqrt()`)
//! - `Fixed::div` for the force computation
//! - `Fixed::mul` for mass scaling
//!
//! The concern is that bodies at very different distances or with very
//! different masses could cause timing variations in the acceleration
//! computation.
//!
//! ## Classes
//!
//! - **Left (small masses):** Bodies with small masses (1–100 solar masses)
//! - **Right (large masses):** Bodies with large masses (10⁶–10⁹ solar masses)
//!
//! Both classes use the same distance range (1–1000 AU), isolating the mass
//! variation as the only difference between classes.

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_core::{compute_accelerations, Fixed, OrbitalBody, Vec3};

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 100_000;

/// Gravitational constant (same as in kelvin-core constants).
const G: Fixed = Fixed::from_raw(1135963431038590976); // 4π² in Q32.64

/// Softening factor (same as in kelvin-core constants).
const SOFTENING: Fixed = Fixed::from_raw(184467440737095516); // 0.01 in Q32.64

/// Benchmark acceleration computation: small vs. large masses.
///
/// Both classes use the same distance range (1–1000 AU), isolating the mass
/// variation as the only difference between classes. This ensures that the
/// magnitude of position values is the same for both classes, eliminating
/// any timing variation from position-dependent operations.
pub fn bench_compute_accelerations(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut body_sets = Vec::with_capacity(NUM_SAMPLES);
    let mut classes = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        // Both classes use the same distance range
        let dist1: i64 = rng.random_range(1i64..=1000);
        let dist2: i64 = rng.random_range(1i64..=1000);
        let dist3: i64 = rng.random_range(1i64..=1000);

        if rng.random::<bool>() {
            // Left class: small masses
            let mass1 = Fixed::from_int(rng.random_range(1i64..=100));
            let mass2 = Fixed::from_int(rng.random_range(1i64..=100));
            let mass3 = Fixed::from_int(rng.random_range(1i64..=100));

            let body1 = OrbitalBody::new(
                mass1,
                Vec3::new(Fixed::from_int(dist1), Fixed::ZERO, Fixed::ZERO),
                Vec3::ZERO,
            );
            let body2 = OrbitalBody::new(
                mass2,
                Vec3::new(Fixed::ZERO, Fixed::from_int(dist2), Fixed::ZERO),
                Vec3::ZERO,
            );
            let body3 = OrbitalBody::new(
                mass3,
                Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::from_int(dist3)),
                Vec3::ZERO,
            );
            body_sets.push(vec![body1, body2, body3]);
            classes.push(Class::Left);
        } else {
            // Right class: large masses
            let mass1 = Fixed::from_int(rng.random_range(1_000_000i64..=1_000_000_000));
            let mass2 = Fixed::from_int(rng.random_range(1_000_000i64..=1_000_000_000));
            let mass3 = Fixed::from_int(rng.random_range(1_000_000i64..=1_000_000_000));

            let body1 = OrbitalBody::new(
                mass1,
                Vec3::new(Fixed::from_int(dist1), Fixed::ZERO, Fixed::ZERO),
                Vec3::ZERO,
            );
            let body2 = OrbitalBody::new(
                mass2,
                Vec3::new(Fixed::ZERO, Fixed::from_int(dist2), Fixed::ZERO),
                Vec3::ZERO,
            );
            let body3 = OrbitalBody::new(
                mass3,
                Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::from_int(dist3)),
                Vec3::ZERO,
            );
            body_sets.push(vec![body1, body2, body3]);
            classes.push(Class::Right);
        }
    }

    for (bodies, class) in body_sets.into_iter().zip(classes) {
        runner.run_one(class, || {
            let _accels = compute_accelerations(&bodies, SOFTENING, G);
        });
    }
}
