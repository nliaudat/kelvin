//! Kani proof harnesses for Vec3 vector operations.
//!
//! Proves functional equivalence: Vec3 operations match their
//! mathematical specifications within bounded inputs.

use kelvin_core::body::Vec3;
use kelvin_core::fixed_math::Fixed;

// ── Helpers ──────────────────────────────────────────────────────────────

fn bounded_fixed() -> Fixed {
    // Values in [-1000, 1000] with reasonable precision
    let int: i64 = kani::any();
    kani::assume(int >= -1000 && int <= 1000);
    Fixed::from_int(int)
}

fn bounded_vec3() -> Vec3 {
    Vec3::new(bounded_fixed(), bounded_fixed(), bounded_fixed())
}

// ── Addition ─────────────────────────────────────────────────────────────

#[kani::proof]
fn verify_vec3_add_commutative() {
    let a = bounded_vec3();
    let b = bounded_vec3();
    assert_eq!(a + b, b + a);
}

#[kani::proof]
fn verify_vec3_add_associative() {
    let a = bounded_vec3();
    let b = bounded_vec3();
    let c = bounded_vec3();
    assert_eq!((a + b) + c, a + (b + c));
}

#[kani::proof]
fn verify_vec3_add_identity() {
    let a = bounded_vec3();
    let zero = Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0));
    assert_eq!(a + zero, a);
}

// ── Subtraction ──────────────────────────────────────────────────────────

#[kani::proof]
fn verify_vec3_sub_self_zero() {
    let a = bounded_vec3();
    let zero = Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0));
    assert_eq!(a - a, zero);
}

// ── Negation ─────────────────────────────────────────────────────────────

#[kani::proof]
fn verify_vec3_neg_add_self_zero() {
    let a = bounded_vec3();
    let zero = Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0));
    assert_eq!(a + (-a), zero);
}

// ── Dot Product ──────────────────────────────────────────────────────────

#[kani::proof]
fn verify_vec3_dot_commutative() {
    let a = bounded_vec3();
    let b = bounded_vec3();
    assert_eq!(a.dot(b), b.dot(a));
}

#[kani::proof]
fn verify_vec3_dot_zero() {
    let a = bounded_vec3();
    let zero = Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0));
    assert_eq!(a.dot(zero), Fixed::from_int(0));
}

// ── Cross Product ────────────────────────────────────────────────────────

#[kani::proof]
fn verify_vec3_cross_anticommutative() {
    let a = bounded_vec3();
    let b = bounded_vec3();
    assert_eq!(a.cross(b), -(b.cross(a)));
}

#[kani::proof]
fn verify_vec3_cross_self_zero() {
    let a = bounded_vec3();
    let zero = Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0));
    assert_eq!(a.cross(a), zero);
}

#[kani::proof]
fn verify_vec3_cross_orthogonal_dot() {
    let a = bounded_vec3();
    let b = bounded_vec3();
    let cross = a.cross(b);
    // (a × b) · a ≈ 0 (within numerical precision)
    let dot_a = cross.dot(a);
    let dot_b = cross.dot(b);
    // Due to fixed-point precision, check near-zero
    assert!(dot_a.abs() <= Fixed::from_parts(0, 10) || dot_a == Fixed::from_int(0));
    assert!(dot_b.abs() <= Fixed::from_parts(0, 10) || dot_b == Fixed::from_int(0));
}

// ── Length ───────────────────────────────────────────────────────────────

#[kani::proof]
fn verify_vec3_length_nonnegative() {
    let a = bounded_vec3();
    let len = a.length();
    assert!(len >= Fixed::from_int(0) || len.is_zero());
}

#[kani::proof]
fn verify_vec3_length_squared_nonnegative() {
    let a = bounded_vec3();
    let len_sq = a.length_squared();
    assert!(len_sq >= Fixed::from_int(0) || len_sq.is_zero());
}

#[kani::proof]
fn verify_vec3_length_squared_eq_dot() {
    let a = bounded_vec3();
    assert_eq!(a.length_squared(), a.dot(a));
}

// ── Scalar Multiplication ────────────────────────────────────────────────

#[kani::proof]
fn verify_vec3_scale_zero() {
    let a = bounded_vec3();
    let zero = Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0));
    assert_eq!(a * Fixed::from_int(0), zero);
}

#[kani::proof]
fn verify_vec3_scale_one() {
    let a = bounded_vec3();
    assert_eq!(a * Fixed::from_int(1), a);
}

// ── OrbitalBody ──────────────────────────────────────────────────────────

use kelvin_core::body::OrbitalBody;

fn bounded_body() -> OrbitalBody {
    OrbitalBody::new(
        bounded_fixed(),   // mass
        bounded_vec3(),    // position
        bounded_vec3(),    // velocity
    )
}

#[kani::proof]
fn verify_orbital_body_kinetic_energy_nonnegative() {
    let body = bounded_body();
    let ke = body.kinetic_energy();
    assert!(ke >= Fixed::from_int(0) || ke.is_zero());
}

#[kani::proof]
fn verify_orbital_body_zero_velocity_zero_ke() {
    let body = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    assert_eq!(body.kinetic_energy(), Fixed::from_int(0));
}

#[kani::proof]
fn verify_orbital_body_momentum_proportional_to_velocity() {
    let body = bounded_body();
    let p = body.momentum();
    // Momentum should be non-zero when velocity is non-zero
    // (mass is always positive)
    if body.velocity.length_squared() > Fixed::from_int(0) {
        assert!(p.length_squared() > Fixed::from_int(0));
    }
}
