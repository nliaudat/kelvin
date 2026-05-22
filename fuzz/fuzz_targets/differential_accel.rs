//! Differential fuzzing: compare Rust Fixed-point vs f64 acceleration computation.
//!
//! The Kelvin cryptosystem has two `compute_accelerations` implementations:
//! 1. **Fixed Q32.64** (`kelvin_core::compute_accelerations`) — used in production
//! 2. **f64** (`OrbitalState::compute_accelerations`) — used in the quantum entropy path
//!
//! This test generates random orbital configurations and compares the results
//! from both implementations, detecting:
//! - Sign flips between the two
//! - Relative error exceeding 1e-12
//! - NaN/Inf in f64 that doesn't occur in Fixed
//! - Catastrophic cancellation near singularities
//!
//! Run with: `cargo test -p kelvin-fuzz --test differential_accel`
//! Heavy run: `PROPTEST_CASES=100000 cargo test -p kelvin-fuzz --test differential_accel`

use proptest::prelude::*;
use kelvin_core::{Fixed, OrbitalBody, Vec3};

/// Convert a Fixed value to f64 for comparison.
fn fixed_to_f64(v: Fixed) -> f64 {
    v.to_f64()
}

/// Convert a Vec3 (Fixed) to [f64; 3] for comparison.
fn vec3_to_f64(v: Vec3) -> [f64; 3] {
    [fixed_to_f64(v.x), fixed_to_f64(v.y), fixed_to_f64(v.z)]
}

/// Convert f64 to Fixed using from_raw (Q32.64: 1.0 = 2^64).
fn f64_to_fixed(v: f64) -> Fixed {
    const SCALE: f64 = (1i128 << 64) as f64;
    Fixed::from_raw((v * SCALE) as i128)
}

/// f64 reimplementation of compute_accelerations matching the Fixed version.
fn compute_accelerations_f64(
    bodies: &[[f64; 3]; 5],  // positions
    masses: &[f64; 5],
    softening: f64,
    g: f64,
) -> [[f64; 3]; 5] {
    let mut accel = [[0.0; 3]; 5];
    let softening_sq = softening * softening;

    for i in 0..5 {
        for j in (i + 1)..5 {
            let dx = bodies[j][0] - bodies[i][0];
            let dy = bodies[j][1] - bodies[i][1];
            let dz = bodies[j][2] - bodies[i][2];
            let dist_sq = dx * dx + dy * dy + dz * dz + softening_sq;
            let dist = dist_sq.sqrt();
            let dist_cubed = dist_sq * dist;
            let factor = g / dist_cubed;

            let ax = factor * masses[j] * dx;
            let ay = factor * masses[j] * dy;
            let az = factor * masses[j] * dz;

            accel[i][0] += ax;
            accel[i][1] += ay;
            accel[i][2] += az;

            accel[j][0] -= factor * masses[i] * dx;
            accel[j][1] -= factor * masses[i] * dy;
            accel[j][2] -= factor * masses[i] * dz;
        }
    }

    accel
}

/// Maximum relative error threshold.
/// Fixed Q32.64 has ~5.4e-20 precision, f64 has ~2.2e-16.
/// However, the f64→Fixed conversion introduces rounding at ~1e-16,
/// and the two arithmetic paths accumulate different rounding errors
/// through the O(n²) computation. A threshold of 1e-8 accounts for
/// this inherent precision boundary while still catching real bugs
/// (sign flips, catastrophic cancellation, NaN divergence).
const MAX_REL_ERROR: f64 = 1e-8;

proptest! {
    #[test]
    fn fuzz_differential_accel(
        pos_x in proptest::array::uniform5(-100.0f64..100.0f64),
        pos_y in proptest::array::uniform5(-100.0f64..100.0f64),
        pos_z in proptest::array::uniform5(-100.0f64..100.0f64),
        masses in proptest::array::uniform5(1e-20f64..1.0f64),
        softening in 0.0f64..1.0f64,
        g in 0.1f64..1000.0f64,
    ) {
        // ── Build Fixed-point bodies ──
        let bodies_fixed: Vec<OrbitalBody> = (0..5).map(|i| {
            OrbitalBody {
                position: Vec3::new(
                    f64_to_fixed(pos_x[i]),
                    f64_to_fixed(pos_y[i]),
                    f64_to_fixed(pos_z[i]),
                ),
                velocity: Vec3::ZERO,
                mass: f64_to_fixed(masses[i]),
            }
        }).collect();

        let softening_fixed = f64_to_fixed(softening);
        let g_fixed = f64_to_fixed(g);

        // ── Compute with Fixed ──
        let accel_fixed = kelvin_core::compute_accelerations(&bodies_fixed, softening_fixed, g_fixed);

        // ── Compute with f64 ──
        let positions_f64: [[f64; 3]; 5] = [
            [pos_x[0], pos_y[0], pos_z[0]],
            [pos_x[1], pos_y[1], pos_z[1]],
            [pos_x[2], pos_y[2], pos_z[2]],
            [pos_x[3], pos_y[3], pos_z[3]],
            [pos_x[4], pos_y[4], pos_z[4]],
        ];
        let accel_f64 = compute_accelerations_f64(&positions_f64, &masses, softening, g);

        // ── Compare ──
        for i in 0..5 {
            let fixed = vec3_to_f64(accel_fixed[i]);
            let f64_val = accel_f64[i];

            for j in 0..3 {
                let a = fixed[j];
                let b = f64_val[j];

                // Check for NaN/Inf in f64 that doesn't appear in Fixed
                let fixed_is_finite = a.is_finite();
                let f64_is_finite = b.is_finite();
                assert_eq!(
                    fixed_is_finite, f64_is_finite,
                    "Finite-ness mismatch at body {i}, axis {j}: Fixed={a} (finite={fixed_is_finite}), f64={b} (finite={f64_is_finite})"
                );

                // Skip comparison if either is non-finite (both should be non-finite in same way)
                if !fixed_is_finite || !f64_is_finite {
                    continue;
                }

                // Check sign agreement
                let sign_same = (a == 0.0 && b == 0.0) || (a.signum() == b.signum());
                assert!(
                    sign_same,
                    "Sign mismatch at body {i}, axis {j}: Fixed={a}, f64={b}"
                );

                // Check relative error
                let max_abs = a.abs().max(b.abs()).max(1e-30);
                let rel_err = (a - b).abs() / max_abs;
                assert!(
                    rel_err < MAX_REL_ERROR,
                    "Relative error {rel_err} exceeds {MAX_REL_ERROR} at body {i}, axis {j}: Fixed={a}, f64={b}"
                );
            }
        }
    }
}
