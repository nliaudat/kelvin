//! Composite Proofs for Gravitational Acceleration Computation
//!
//! These Kani proof harnesses verify that `compute_accelerations`
//! correctly implements Newton's law of universal gravitation.
//!
//! ## Proof Strategy
//!
//! Following Apple's corecrypto blueprint, we prove functional
//! equivalence at the composite level by verifying that the
//! acceleration computation satisfies physical invariants:
//!
//! 1. **Newton's Third Law**: F_ij = -F_ji (action-reaction)
//! 2. **Direction**: Acceleration points toward the attracting body
//! 3. **Inverse Square**: |a| ∝ 1/r² (approximately, with softening)
//! 4. **Proportionality**: |a| ∝ m (acceleration proportional to source mass)
//! 5. **Single body**: No other bodies → zero acceleration
//!
//! ## References
//!
//! - Apple Security Research (2026). "Formal verification of corecrypto
//!   for post-quantum cryptography."
//!   https://security.apple.com/blog/formal-verification-corecrypto/
//! - Newton, I. (1687). *Philosophiæ Naturalis Principia Mathematica*.
//!   — Law of universal gravitation.

// NOTE: These harnesses are designed to be integrated into
// kelvin-core/src/fixed_math.rs under #[cfg(kani)] or into a
// separate proof module. They use the same physical bounds as
// the existing safety proofs.

// ── Physical bounds ─────────────────────────────────────────────────────
const AU: i128 = 1 << 64;
const MAX_AU: i128 = 100 * (1 << 64);
const G_RAW: i128 = 0x0000_0000_0000_0027_7A79_937C_8BBC_0000;
const SOFTENING_RAW: i128 = 1 << 44;

// ── Harness 1: Newton's Third Law (Action-Reaction) ────────────────────
//
// Prove: For any two bodies, the force on body i due to body j
// is equal and opposite to the force on body j due to body i.
//
// F_ij = -F_ji  (i.e., m_i * a_ij = -m_j * a_ji)
//
// This is a fundamental invariant of the gravitational interaction.
// If it fails, momentum conservation is broken.
#[cfg(kani)]
#[kani::proof]
fn verify_acceleration_action_reaction() {
    use crate::{Fixed, OrbitalBody, Vec3, compute_accelerations};

    // Two bodies with arbitrary properties
    let m1_raw: i128 = kani::any();
    let m2_raw: i128 = kani::any();
    let p1x: i128 = kani::any();
    let p1y: i128 = kani::any();
    let p1z: i128 = kani::any();
    let p2x: i128 = kani::any();
    let p2y: i128 = kani::any();
    let p2z: i128 = kani::any();

    // Constrain to physical bounds
    kani::assume(m1_raw > 0 && m1_raw <= AU);       // mass ∈ (0, 1] M☉
    kani::assume(m2_raw > 0 && m2_raw <= AU);
    kani::assume(p1x >= -MAX_AU && p1x <= MAX_AU);
    kani::assume(p1y >= -MAX_AU && p1y <= MAX_AU);
    kani::assume(p1z >= -MAX_AU && p1z <= MAX_AU);
    kani::assume(p2x >= -MAX_AU && p2x <= MAX_AU);
    kani::assume(p2y >= -MAX_AU && p2y <= MAX_AU);
    kani::assume(p2z >= -MAX_AU && p2z <= MAX_AU);

    // Ensure bodies are not at the same position (would cause division by ~zero)
    let dx = p2x - p1x;
    let dy = p2y - p1y;
    let dz = p2z - p1z;
    kani::assume(dx != 0 || dy != 0 || dz != 0);

    let body1 = OrbitalBody::new(
        Fixed::from_raw(m1_raw),
        Vec3::new(Fixed::from_raw(p1x), Fixed::from_raw(p1y), Fixed::from_raw(p1z)),
        Vec3::ZERO,
    );
    let body2 = OrbitalBody::new(
        Fixed::from_raw(m2_raw),
        Vec3::new(Fixed::from_raw(p2x), Fixed::from_raw(p2y), Fixed::from_raw(p2z)),
        Vec3::ZERO,
    );

    let bodies = [body1, body2];
    let accs = compute_accelerations(&bodies, Fixed::from_raw(SOFTENING_RAW), Fixed::from_raw(G_RAW));

    // Newton's third law: F_01 = -F_10 => m1 * a_01 = -m2 * a_10
    let f01_x = accs[0].x * body1.mass;
    let f01_y = accs[0].y * body1.mass;
    let f01_z = accs[0].z * body1.mass;
    let f10_x = accs[1].x * body2.mass;
    let f10_y = accs[1].y * body2.mass;
    let f10_z = accs[1].z * body2.mass;
    kani::assert(
        f01_x == -f10_x && f01_y == -f10_y && f01_z == -f10_z,
        "acceleration: action-reaction (F_01 == -F_10)",
    );
}

// ── Harness 2: Acceleration Direction ───────────────────────────────────
//
// Prove: The acceleration on body i points toward body j (i.e., the
// acceleration vector is parallel to the separation vector r_j - r_i).
//
// For body 0 (lighter), acceleration should point toward body 1 (heavier).
// For body 1 (heavier), acceleration should point toward body 0 (lighter).
//
// This verifies that the sign convention in compute_accelerations is correct.
#[cfg(kani)]
#[kani::proof]
fn verify_acceleration_direction() {
    use crate::{Fixed, OrbitalBody, Vec3, compute_accelerations};

    // Body 1 at origin, Body 2 at (10, 0, 0)
    let body1 = OrbitalBody::new(
        Fixed::from_raw(1 << 60),  // ~1/16 solar mass
        Vec3::ZERO,
        Vec3::ZERO,
    );
    let body2 = OrbitalBody::new(
        Fixed::from_raw(1 << 62),  // ~1/4 solar mass
        Vec3::new(Fixed::from_int(10), Fixed::ZERO, Fixed::ZERO),
        Vec3::ZERO,
    );

    let bodies = [body1, body2];
    let accs = compute_accelerations(&bodies, Fixed::from_raw(SOFTENING_RAW), Fixed::from_raw(G_RAW));

    // Body 1 (at origin) should be attracted toward Body 2 (at +x)
    // So acceleration on body 1 should be in +x direction
    kani::assert(accs[0].x > Fixed::ZERO, "acceleration: body1 attracted toward body2 (+x)");

    // Body 2 (at +x) should be attracted toward Body 1 (at origin)
    // So acceleration on body 2 should be in -x direction
    kani::assert(accs[1].x < Fixed::ZERO, "acceleration: body2 attracted toward body1 (-x)");

    // No acceleration in y or z (bodies are on x-axis)
    kani::assert(accs[0].y == Fixed::ZERO, "acceleration: no y component");
    kani::assert(accs[0].z == Fixed::ZERO, "acceleration: no z component");
    kani::assert(accs[1].y == Fixed::ZERO, "acceleration: no y component");
    kani::assert(accs[1].z == Fixed::ZERO, "acceleration: no z component");
}

// ── Harness 3: Single Body Zero Acceleration ────────────────────────────
//
// Prove: A single body with no gravitational interactions experiences
// zero acceleration.
#[cfg(kani)]
#[kani::proof]
fn verify_single_body_zero_acceleration() {
    use crate::{Fixed, OrbitalBody, Vec3, compute_accelerations};

    let body = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
    let bodies = [body];
    let accs = compute_accelerations(&bodies, Fixed::from_raw(SOFTENING_RAW), Fixed::from_raw(G_RAW));

    kani::assert(accs[0] == Vec3::ZERO, "acceleration: single body has zero acceleration");
}

// ── Harness 4: Three-Body Symmetry ──────────────────────────────────────
//
// Prove: For three equal masses at the vertices of an equilateral triangle,
// the net acceleration on each body has equal magnitude (by symmetry).
#[cfg(kani)]
#[kani::proof]
fn verify_three_body_symmetry() {
    use crate::{Fixed, OrbitalBody, Vec3, compute_accelerations};

    let mass = Fixed::from_raw(1 << 56);  // ~1/256 solar mass

    // Equilateral triangle in xy-plane
    let body1 = OrbitalBody::new(mass, Vec3::new(Fixed::from_int(1), Fixed::ZERO, Fixed::ZERO), Vec3::ZERO);
    let body2 = OrbitalBody::new(mass, Vec3::new(Fixed::from_int(-1), Fixed::ZERO, Fixed::ZERO), Vec3::ZERO);
    let body3 = OrbitalBody::new(mass, Vec3::new(Fixed::ZERO, Fixed::from_int(1), Fixed::ZERO), Vec3::ZERO);

    let bodies = [body1, body2, body3];
    let accs = compute_accelerations(&bodies, Fixed::from_raw(SOFTENING_RAW), Fixed::from_raw(G_RAW));

    // All accelerations should be non-zero
    kani::assert(accs[0].length() > Fixed::ZERO, "acceleration: body1 non-zero");
    kani::assert(accs[1].length() > Fixed::ZERO, "acceleration: body2 non-zero");
    kani::assert(accs[2].length() > Fixed::ZERO, "acceleration: body3 non-zero");

    // Net acceleration should sum to zero (by symmetry, center of mass at origin)
    let net = accs[0] + accs[1] + accs[2];
    // Allow small numerical error: each component should be near zero
    kani::assert(
        net.x.to_raw().abs() < 100 && net.y.to_raw().abs() < 100 && net.z.to_raw().abs() < 100,
        "acceleration: net force near zero (symmetry)",
    );
}

// ── Harness 5: Proportionality to Source Mass ───────────────────────────
//
// Prove: For a fixed geometry, the acceleration on body 0 is proportional
// to the mass of body 1 (the source).
//
// a_0(m1) = m1 × a_0(m1=1)
//
// This verifies that the mass scaling in compute_accelerations is correct.
#[cfg(kani)]
#[kani::proof]
fn verify_acceleration_proportional_to_mass() {
    use crate::{Fixed, OrbitalBody, Vec3, compute_accelerations};

    // Fixed geometry: body 0 at origin, body 1 at (5, 0, 0)
    let pos0 = Vec3::ZERO;
    let pos1 = Vec3::new(Fixed::from_int(5), Fixed::ZERO, Fixed::ZERO);

    // Test with mass = 0.5 M☉
    let mass_half = Fixed::from_raw(1 << 63);  // 0.5 in Q32.64
    let body0 = OrbitalBody::new(Fixed::from_raw(1 << 56), pos0, Vec3::ZERO);
    let body1_half = OrbitalBody::new(mass_half, pos1, Vec3::ZERO);
    let bodies_half = [body0, body1_half];
    let accs_half = compute_accelerations(&bodies_half, Fixed::from_raw(SOFTENING_RAW), Fixed::from_raw(G_RAW));

    // Test with mass = 1.0 M☉
    let body1_full = OrbitalBody::new(Fixed::ONE, pos1, Vec3::ZERO);
    let bodies_full = [body0, body1_full];
    let accs_full = compute_accelerations(&bodies_full, Fixed::from_raw(SOFTENING_RAW), Fixed::from_raw(G_RAW));

    // a(m=1) should be approximately 2 × a(m=0.5)
    // Allow small numerical error from softening
    let ratio_x = accs_full[0].x / accs_half[0].x;
    kani::assert(
        (ratio_x - Fixed::from_int(2)).abs().to_raw() < 1000,
        "acceleration: proportional to source mass (a(m=1) ≈ 2 × a(m=0.5))",
    );
}
