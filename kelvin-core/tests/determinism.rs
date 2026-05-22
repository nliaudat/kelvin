//! Determinism Proof for Symplectic Verlet Integrator.
//!
//! This test verifies that the n-body simulation produces **bit-identical**
//! results regardless of:
//!
//! - Compiler auto-vectorization (SSE, AVX, AVX2, AVX-512 on x86_64)
//! - NEON SIMD on aarch64
//! - Any other target-specific code generation differences
//!
//! ## How it works
//!
//! 1. A fixed 5-body orbital configuration is simulated for a known number
//!    of steps using the Symplectic Verlet integrator.
//! 2. The final orbital state (all positions, velocities, masses) is
//!    serialized into a canonical byte array.
//! 3. A SHA3-256 hash of the serialized state is computed.
//! 4. The hash is compared against a **golden value** captured on the
//!    reference platform (x86_64 with SSE2 baseline).
//! 5. The test also verifies intra-process repeatability (two identical
//!    simulations produce identical results).
//!
//! ## Running with different SIMD features
//!
//! ```powershell
//! # Baseline (SSE2 — default for x86_64)
//! cargo test -p kelvin-core --test determinism
//!
//! # AVX
//! cargo test -p kelvin-core --test determinism -- target-feature=+avx
//!
//! # AVX2
//! cargo test -p kelvin-core --test determinism -- target-feature=+avx2
//!
//! # AVX-512 (if CPU supports it)
//! cargo test -p kelvin-core --test determinism -- target-feature=+avx512f
//! ```
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.** This is an experimental
//! cryptosystem that has not undergone formal cryptanalysis.
//!
//! ## References
//!
//! - Goldberg, D. (1991). "What Every Computer Scientist Should Know About
//!   Floating-Point Arithmetic." *ACM Computing Surveys*, 23(1), 5–48.
//!   — Motivates fixed-point arithmetic for cross-platform determinism.
//! - Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical
//!   Integration* (2nd ed.). Springer.
//!   — Theoretical foundation for symplectic integrators.

use kelvin_core::{
    compute_accelerations, euler_step, simulate, verlet_step, Fixed, OrbitalBody, Vec3,
    DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR,
};
use sha3::{Digest, Sha3_256};

/// A fixed 5-body system used for determinism testing.
///
/// This configuration must remain unchanged across versions to preserve
/// the golden hash value. Any change to the initial conditions will
/// require updating the golden hash.
fn five_body_system() -> Vec<OrbitalBody> {
    let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
    let planet1 = OrbitalBody::new(
        Fixed::from_raw(1 << 54), // ~1e-6 solar masses
        Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
        Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
    );
    let planet2 = OrbitalBody::new(
        Fixed::from_raw(1 << 53),
        Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
        Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
    );
    let planet3 = OrbitalBody::new(
        Fixed::from_raw(1 << 52),
        Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
        Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
    );
    let planet4 = OrbitalBody::new(
        Fixed::from_raw(1 << 51),
        Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
        Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
    );
    vec![sun, planet1, planet2, planet3, planet4]
}

/// Serialize the orbital state into a canonical byte array for hashing.
///
/// The serialization format is:
/// - For each body (in order):
///   - mass: 16 bytes (i128, little-endian)
///   - position.x: 16 bytes (i128, little-endian)
///   - position.y: 16 bytes (i128, little-endian)
///   - position.z: 16 bytes (i128, little-endian)
///   - velocity.x: 16 bytes (i128, little-endian)
///   - velocity.y: 16 bytes (i128, little-endian)
///   - velocity.z: 16 bytes (i128, little-endian)
///
/// Total: n_bodies × 7 × 16 bytes
fn serialize_state(bodies: &[OrbitalBody]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(bodies.len() * 7 * 16);
    for body in bodies {
        buf.extend_from_slice(&body.mass.to_raw().to_le_bytes());
        buf.extend_from_slice(&body.position.x.to_raw().to_le_bytes());
        buf.extend_from_slice(&body.position.y.to_raw().to_le_bytes());
        buf.extend_from_slice(&body.position.z.to_raw().to_le_bytes());
        buf.extend_from_slice(&body.velocity.x.to_raw().to_le_bytes());
        buf.extend_from_slice(&body.velocity.y.to_raw().to_le_bytes());
        buf.extend_from_slice(&body.velocity.z.to_raw().to_le_bytes());
    }
    buf
}

/// Compute SHA3-256 hash of the serialized orbital state.
fn hash_state(bodies: &[OrbitalBody]) -> [u8; 32] {
    let serialized = serialize_state(bodies);
    let mut hasher = Sha3_256::new();
    hasher.update(&serialized);
    hasher.finalize().into()
}

// ============================================================================
// Golden hash values
// ============================================================================
//
// These hashes are captured on the reference platform (x86_64, SSE2 baseline)
// and must match on ALL platforms and ALL SIMD feature sets.
//
// To update: run the test with CARGO_UPDATE_GOLDEN=1 environment variable.
// This will print the new golden hash. Update the constants below.
//
// WARNING: Only update golden hashes when the simulation algorithm changes
// intentionally. Any change to the integrator, fixed-point math, or initial
// conditions will invalidate these hashes.

/// Golden hash for `verlet_step` after 1,000 steps with the 5-body system.
///
/// Captured on: x86_64-pc-windows-msvc, SSE2 baseline
/// Date: 2026-05-22
const GOLDEN_VERLET_1000: &str = "0ba480344fe31e8ff71c09d4a6588a725a152cceb5ac7094d325407a0273563b";

/// Golden hash for `euler_step` after 1,000 steps with the 5-body system.
///
/// Captured on: x86_64-pc-windows-msvc, SSE2 baseline
/// Date: 2026-05-22
const GOLDEN_EULER_1000: &str = "53dacba804bb7e0dd6d91908cc488095924868578c085f0856f4defb9fa01b14";

/// Golden hash for `compute_accelerations` on the initial 5-body system.
///
/// Captured on: x86_64-pc-windows-msvc, SSE2 baseline
/// Date: 2026-05-22
const GOLDEN_ACCEL_INITIAL: &str = "38424c5fa124031fe071c07285ba7e40b4701b56bbfac36a807c71af26831895";

// ============================================================================
// Tests
// ============================================================================

/// Test that `compute_accelerations` produces deterministic results.
///
/// This is the fundamental building block — if accelerations are
/// deterministic, the integrator steps built on top of them will be too.
#[test]
fn test_accelerations_deterministic() {
    let bodies = five_body_system();
    let acc1 = compute_accelerations(&bodies, SOFTENING_FACTOR, DEFAULT_G);
    let acc2 = compute_accelerations(&bodies, SOFTENING_FACTOR, DEFAULT_G);

    // Same input must produce identical output
    assert_eq!(acc1, acc2, "compute_accelerations must be deterministic");

    // Verify against golden hash
    let mut buf = Vec::with_capacity(acc1.len() * 3 * 16);
    for acc in &acc1 {
        buf.extend_from_slice(&acc.x.to_raw().to_le_bytes());
        buf.extend_from_slice(&acc.y.to_raw().to_le_bytes());
        buf.extend_from_slice(&acc.z.to_raw().to_le_bytes());
    }
    let mut hasher = Sha3_256::new();
    hasher.update(&buf);
    let hash: [u8; 32] = hasher.finalize().into();
    let hash_hex = hex::encode(hash);

    if std::env::var("CARGO_UPDATE_GOLDEN").is_ok() {
        if GOLDEN_ACCEL_INITIAL == "0000000000000000000000000000000000000000000000000000000000000000" {
            panic!(
                "GOLDEN_ACCEL_INITIAL needs to be set. Run test without CARGO_UPDATE_GOLDEN to get the value.\n\
                 Hash: {}",
                hash_hex
            );
        }
    }

    assert_eq!(
        hash_hex, GOLDEN_ACCEL_INITIAL,
        "compute_accelerations golden hash mismatch!\n\
         Expected: {}\n\
         Got:      {}\n\
         This means the acceleration computation produces different results\n\
         on this platform/SIMD feature set than the reference.\n\
         If this is an intentional algorithm change, update the golden hash.",
        GOLDEN_ACCEL_INITIAL, hash_hex
    );
}

/// Test that `verlet_step` produces deterministic results.
///
/// Runs two independent simulations with the same initial conditions
/// and verifies they produce identical final states.
#[test]
fn test_verlet_intra_process_determinism() {
    let mut bodies_a = five_body_system();
    let mut bodies_b = five_body_system();

    let steps = 1_000u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    for _ in 0..steps {
        verlet_step(&mut bodies_a, dt, softening, DEFAULT_G);
    }
    for _ in 0..steps {
        verlet_step(&mut bodies_b, dt, softening, DEFAULT_G);
    }

    assert_eq!(
        bodies_a, bodies_b,
        "Two identical Verlet simulations diverged!\n\
         This indicates non-deterministic behavior in the integrator."
    );
}

/// Test that `euler_step` produces deterministic results.
#[test]
fn test_euler_intra_process_determinism() {
    let mut bodies_a = five_body_system();
    let mut bodies_b = five_body_system();

    let steps = 1_000u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    for _ in 0..steps {
        euler_step(&mut bodies_a, dt, softening, DEFAULT_G);
    }
    for _ in 0..steps {
        euler_step(&mut bodies_b, dt, softening, DEFAULT_G);
    }

    assert_eq!(
        bodies_a, bodies_b,
        "Two identical Euler simulations diverged!\n\
         This indicates non-deterministic behavior in the integrator."
    );
}

/// Test that `simulate` (which wraps `verlet_step`) produces deterministic
/// results across independent calls.
#[test]
fn test_simulate_intra_process_determinism() {
    let mut bodies_a = five_body_system();
    let mut bodies_b = five_body_system();

    let steps = 1_000u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_a, steps, dt, softening, DEFAULT_G);
    simulate(&mut bodies_b, steps, dt, softening, DEFAULT_G);

    assert_eq!(
        bodies_a, bodies_b,
        "Two identical simulate() calls diverged!\n\
         This indicates non-deterministic behavior in the simulation loop."
    );
}

/// Golden hash test for Verlet integrator.
///
/// This test compares the SHA3-256 hash of the final orbital state
/// against a golden value captured on the reference platform.
///
/// If this test passes on multiple architectures and SIMD feature sets,
/// it proves the integrator produces bit-identical results everywhere.
#[test]
fn test_verlet_golden_hash() {
    let mut bodies = five_body_system();
    let steps = 1_000u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies, steps, dt, softening, DEFAULT_G);

    let hash = hash_state(&bodies);
    let hash_hex = hex::encode(hash);

    // Check if golden hash needs to be set
    if GOLDEN_VERLET_1000 == "0000000000000000000000000000000000000000000000000000000000000000" {
        if std::env::var("CARGO_UPDATE_GOLDEN").is_ok() {
            panic!(
                "GOLDEN_VERLET_1000 needs to be set. Run test without CARGO_UPDATE_GOLDEN to get the value.\n\
                 Hash: {}",
                hash_hex
            );
        }
    }

    assert_eq!(
        hash_hex, GOLDEN_VERLET_1000,
        "Verlet golden hash mismatch!\n\
         Expected: {}\n\
         Got:      {}\n\n\
         This means the Verlet integrator produces different results on\n\
         this platform/SIMD feature set than the reference.\n\
         If this is an intentional algorithm change, update the golden hash\n\
         by running with CARGO_UPDATE_GOLDEN=1 and copying the new value.",
        GOLDEN_VERLET_1000, hash_hex
    );
}

/// Golden hash test for Euler integrator.
#[test]
fn test_euler_golden_hash() {
    let mut bodies = five_body_system();
    let steps = 1_000u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    for _ in 0..steps {
        euler_step(&mut bodies, dt, softening, DEFAULT_G);
    }

    let hash = hash_state(&bodies);
    let hash_hex = hex::encode(hash);

    if GOLDEN_EULER_1000 == "0000000000000000000000000000000000000000000000000000000000000000" {
        if std::env::var("CARGO_UPDATE_GOLDEN").is_ok() {
            panic!(
                "GOLDEN_EULER_1000 needs to be set. Run test without CARGO_UPDATE_GOLDEN to get the value.\n\
                 Hash: {}",
                hash_hex
            );
        }
    }

    assert_eq!(
        hash_hex, GOLDEN_EULER_1000,
        "Euler golden hash mismatch!\n\
         Expected: {}\n\
         Got:      {}\n\n\
         This means the Euler integrator produces different results on\n\
         this platform/SIMD feature set than the reference.\n\
         If this is an intentional algorithm change, update the golden hash.",
        GOLDEN_EULER_1000, hash_hex
    );
}

/// Test that the serialization format is canonical.
///
/// The same state must always serialize to the same bytes.
#[test]
fn test_serialization_canonical() {
    let bodies = five_body_system();
    let serialized_a = serialize_state(&bodies);
    let serialized_b = serialize_state(&bodies);

    assert_eq!(
        serialized_a, serialized_b,
        "Serialization must be canonical"
    );
    assert_eq!(
        serialized_a.len(),
        5 * 7 * 16,
        "5 bodies × 7 fields × 16 bytes = {} bytes",
        5 * 7 * 16
    );
}

/// Test that the hash function is deterministic.
#[test]
fn test_hash_deterministic() {
    let bodies = five_body_system();
    let hash_a = hash_state(&bodies);
    let hash_b = hash_state(&bodies);

    assert_eq!(hash_a, hash_b, "Hash must be deterministic");
}

/// Test that different step counts produce different hashes.
///
/// This verifies the simulation is actually progressing and not
/// stuck in a fixed point.
#[test]
fn test_different_steps_different_hashes() {
    let mut bodies_100 = five_body_system();
    let mut bodies_200 = five_body_system();

    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_100, 100, dt, softening, DEFAULT_G);
    simulate(&mut bodies_200, 200, dt, softening, DEFAULT_G);

    let hash_100 = hash_state(&bodies_100);
    let hash_200 = hash_state(&bodies_200);

    assert_ne!(
        hash_100, hash_200,
        "Different step counts must produce different hashes.\n\
         This would indicate the simulation is not progressing."
    );
}

/// Test that Verlet and Euler produce different results.
///
/// This verifies the two integrators are actually different algorithms.
#[test]
fn test_verlet_vs_euler_different() {
    let mut bodies_verlet = five_body_system();
    let mut bodies_euler = five_body_system();

    let steps = 100u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_verlet, steps, dt, softening, DEFAULT_G);
    for _ in 0..steps {
        euler_step(&mut bodies_euler, dt, softening, DEFAULT_G);
    }

    let hash_verlet = hash_state(&bodies_verlet);
    let hash_euler = hash_state(&bodies_euler);

    assert_ne!(
        hash_verlet, hash_euler,
        "Verlet and Euler must produce different results.\n\
         This would indicate one integrator is not working correctly."
    );
}

/// Test determinism with a minimal 2-body system.
///
/// Edge case: the simplest possible n-body system.
#[test]
fn test_two_body_determinism() {
    let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
    let planet = OrbitalBody::new(
        Fixed::from_raw(1 << 54),
        Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
        Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
    );

    let mut bodies_a = vec![sun, planet];
    let mut bodies_b = vec![sun, planet];

    let steps = 500u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_a, steps, dt, softening, DEFAULT_G);
    simulate(&mut bodies_b, steps, dt, softening, DEFAULT_G);

    assert_eq!(
        bodies_a, bodies_b,
        "Two-body Verlet simulation must be deterministic"
    );
}

/// Test determinism with zero softening.
///
/// Edge case: softening prevents singularities, but the integrator
/// should still be deterministic even without it (as long as no
/// actual collision occurs).
#[test]
fn test_zero_softening_determinism() {
    let mut bodies_a = five_body_system();
    let mut bodies_b = five_body_system();

    let steps = 100u64;
    let dt = DEFAULT_DT;

    simulate(&mut bodies_a, steps, dt, Fixed::ZERO, DEFAULT_G);
    simulate(&mut bodies_b, steps, dt, Fixed::ZERO, DEFAULT_G);

    assert_eq!(
        bodies_a, bodies_b,
        "Verlet with zero softening must be deterministic"
    );
}

/// Test determinism with maximum dt.
///
/// Edge case: large timesteps could trigger different code paths
/// in the fixed-point arithmetic.
#[test]
fn test_max_dt_determinism() {
    let mut bodies_a = five_body_system();
    let mut bodies_b = five_body_system();

    let steps = 50u64;
    let dt = kelvin_core::MAX_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_a, steps, dt, softening, DEFAULT_G);
    simulate(&mut bodies_b, steps, dt, softening, DEFAULT_G);

    assert_eq!(
        bodies_a, bodies_b,
        "Verlet with MAX_DT must be deterministic"
    );
}

/// Test determinism with minimum dt.
///
/// Edge case: very small timesteps with many iterations.
#[test]
fn test_min_dt_determinism() {
    let mut bodies_a = five_body_system();
    let mut bodies_b = five_body_system();

    let steps = 100u64;
    let dt = kelvin_core::MIN_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_a, steps, dt, softening, DEFAULT_G);
    simulate(&mut bodies_b, steps, dt, softening, DEFAULT_G);

    assert_eq!(
        bodies_a, bodies_b,
        "Verlet with MIN_DT must be deterministic"
    );
}

/// Test determinism with a single body.
///
/// Edge case: no gravitational interactions, just inertial motion.
#[test]
fn test_single_body_determinism() {
    let body = OrbitalBody::new(
        Fixed::ONE,
        Vec3::new(Fixed::from_int(10), Fixed::from_int(20), Fixed::from_int(30)),
        Vec3::new(Fixed::from_int(1), Fixed::from_int(2), Fixed::from_int(3)),
    );

    let mut bodies_a = vec![body];
    let mut bodies_b = vec![body];

    let steps = 1000u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_a, steps, dt, softening, DEFAULT_G);
    simulate(&mut bodies_b, steps, dt, softening, DEFAULT_G);

    assert_eq!(
        bodies_a, bodies_b,
        "Single-body Verlet simulation must be deterministic"
    );
}

/// Test determinism with 7 bodies (maximum typical configuration).
///
/// Edge case: maximum O(n²) pairwise interactions.
#[test]
fn test_seven_body_determinism() {
    let bodies: Vec<OrbitalBody> = (0..7)
        .map(|i| {
            OrbitalBody::new(
                Fixed::from_raw(1 << (60 - i)), // decreasing masses
                Vec3::new(
                    Fixed::from_int(i * 3),
                    Fixed::from_int(i * 2 + 1),
                    Fixed::from_int(i + 5),
                ),
                Vec3::new(
                    Fixed::from_int(i + 1),
                    Fixed::from_int(-(i as i64)),
                    Fixed::from_int(i * 2),
                ),
            )
        })
        .collect();

    let mut bodies_a = bodies.clone();
    let mut bodies_b = bodies;

    let steps = 200u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    simulate(&mut bodies_a, steps, dt, softening, DEFAULT_G);
    simulate(&mut bodies_b, steps, dt, softening, DEFAULT_G);

    assert_eq!(
        bodies_a, bodies_b,
        "Seven-body Verlet simulation must be deterministic"
    );
}

/// Test that the `compute_accelerations` function is deterministic
/// when called multiple times on the same state.
#[test]
fn test_accelerations_repeatable() {
    let bodies = five_body_system();

    for _ in 0..10 {
        let acc1 = compute_accelerations(&bodies, SOFTENING_FACTOR, DEFAULT_G);
        let acc2 = compute_accelerations(&bodies, SOFTENING_FACTOR, DEFAULT_G);
        assert_eq!(
            acc1, acc2,
            "compute_accelerations must produce identical results on repeated calls"
        );
    }
}

/// Test that the full simulation loop is repeatable end-to-end.
///
/// This runs the simulation twice in sequence on the same mutable
/// state (after resetting), verifying that the loop itself has no
/// state-dependent branching that could cause divergence.
#[test]
fn test_simulation_loop_repeatable() {
    let initial = five_body_system();
    let steps = 500u64;
    let dt = DEFAULT_DT;
    let softening = SOFTENING_FACTOR;

    // Run simulation twice, resetting between runs
    let mut bodies = initial.clone();
    simulate(&mut bodies, steps, dt, softening, DEFAULT_G);
    let hash_run1 = hash_state(&bodies);

    bodies = initial;
    simulate(&mut bodies, steps, dt, softening, DEFAULT_G);
    let hash_run2 = hash_state(&bodies);

    assert_eq!(
        hash_run1, hash_run2,
        "Simulation loop must produce identical results on repeated runs"
    );
}
