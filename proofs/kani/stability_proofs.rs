//! Kani proof harnesses for stability monitoring.
//!
//! Proves functional equivalence: ejection detection and collapse
//! detection satisfy their physical invariants.

use kelvin_core::body::{OrbitalBody, Vec3};
use kelvin_core::fixed_math::Fixed;
use kelvin_core::stability;

// ── Helpers ──────────────────────────────────────────────────────────────

fn bounded_fixed() -> Fixed {
    let int: i64 = kani::any();
    kani::assume(int >= -100 && int <= 100);
    Fixed::from_int(int)
}

fn bounded_vec3() -> Vec3 {
    Vec3::new(bounded_fixed(), bounded_fixed(), bounded_fixed())
}

fn bounded_body() -> OrbitalBody {
    OrbitalBody::new(
        Fixed::from_int(1),
        bounded_vec3(),
        bounded_vec3(),
    )
}

// ── Gravitational Potential ──────────────────────────────────────────────

#[kani::proof]
fn verify_gravitational_potential_negative() {
    let body = bounded_body();
    let bodies = vec![bounded_body(), bounded_body()];
    let pot = stability::gravitational_potential(
        &body,
        &bodies,
        Fixed::from_int(1),
    );
    // Gravitational potential should be negative (attractive force)
    assert!(pot <= Fixed::from_int(0) || pot.is_zero());
}

#[kani::proof]
fn verify_gravitational_potential_zero_for_single_body() {
    let body = bounded_body();
    let bodies = vec![body];  // only the body itself
    let pot = stability::gravitational_potential(
        &body,
        &bodies,
        Fixed::from_int(1),
    );
    // A body alone has no gravitational potential from others
    assert_eq!(pot, Fixed::from_int(0));
}

// ── Ejection Detection ───────────────────────────────────────────────────

#[kani::proof]
fn verify_is_body_ejected_no_false_positive_for_bound() {
    // A body at rest at the origin with another body nearby
    // should NOT be detected as ejected
    let body = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let other = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(1), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let bodies = vec![body, other];
    let ejected = stability::is_body_ejected(
        &bodies[0],
        &bodies,
        Fixed::from_int(1),
    );
    assert!(!ejected);
}

#[kani::proof]
fn verify_is_body_ejected_detects_escape_velocity() {
    // A body with very high velocity should be detected as ejected
    let body = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(100), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let other = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(1), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let bodies = vec![body, other];
    let ejected = stability::is_body_ejected(
        &bodies[0],
        &bodies,
        Fixed::from_int(1),
    );
    assert!(ejected);
}

// ── Collapse Detection ───────────────────────────────────────────────────

#[kani::proof]
fn verify_detect_collapse_no_false_positive() {
    // Two bodies far apart should not trigger collapse
    let body1 = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(-100), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let body2 = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(100), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let bodies = vec![body1, body2];
    let collapsed = stability::detect_collapse(
        &bodies,
        Fixed::from_parts(0, 1000),  // threshold = ~0.001
    );
    assert!(!collapsed);
}

#[kani::proof]
fn verify_detect_collapse_detects_close_bodies() {
    // Two bodies at the same position should trigger collapse
    let body1 = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let body2 = OrbitalBody::new(
        Fixed::from_int(1),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
        Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0)),
    );
    let bodies = vec![body1, body2];
    let collapsed = stability::detect_collapse(
        &bodies,
        Fixed::from_parts(0, 1),  // threshold = ~0.0000000000000000001
    );
    assert!(collapsed);
}
