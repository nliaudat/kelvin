//! Kani proof harnesses for integrator operations.
//!
//! Proves functional equivalence: Verlet and Euler integrators
//! satisfy their physical invariants within bounded inputs.

use kelvin_core::body::{OrbitalBody, Vec3};
use kelvin_core::fixed_math::Fixed;
use kelvin_core::integrator;

// ── Helpers ──────────────────────────────────────────────────────────────

fn small_fixed() -> Fixed {
    let int: i64 = kani::any();
    kani::assume(int >= -10 && int <= 10);
    Fixed::from_int(int)
}

fn small_vec3() -> Vec3 {
    Vec3::new(small_fixed(), small_fixed(), small_fixed())
}

fn small_body() -> OrbitalBody {
    OrbitalBody::new(
        Fixed::from_int(1),  // unit mass
        small_vec3(),
        small_vec3(),
    )
}

fn two_bodies() -> Vec<OrbitalBody> {
    vec![
        OrbitalBody::new(
            Fixed::from_int(1),
            Vec3::new(Fixed::from_int(-1), Fixed::from_int(0), Fixed::from_int(0)),
            Vec3::new(Fixed::from_int(0), Fixed::from_int(1), Fixed::from_int(0)),
        ),
        OrbitalBody::new(
            Fixed::from_int(1),
            Vec3::new(Fixed::from_int(1), Fixed::from_int(0), Fixed::from_int(0)),
            Vec3::new(Fixed::from_int(0), Fixed::from_int(-1), Fixed::from_int(0)),
        ),
    ]
}

// ── compute_accelerations ────────────────────────────────────────────────

#[kani::proof]
fn verify_compute_accelerations_single_body_zero() {
    let body = small_body();
    let bodies = vec![body];
    let accels = integrator::compute_accelerations(
        &bodies,
        Fixed::from_parts(0, 1000),  // softening = 0.001
        Fixed::from_int(1),           // g = 1
    );
    assert_eq!(accels.len(), 1);
    // Single body should have zero acceleration
    let zero = Vec3::new(Fixed::from_int(0), Fixed::from_int(0), Fixed::from_int(0));
    assert_eq!(accels[0], zero);
}

#[kani::proof]
fn verify_compute_accelerations_two_body_equal_mass() {
    let bodies = two_bodies();
    let accels = integrator::compute_accelerations(
        &bodies,
        Fixed::from_parts(0, 1000),
        Fixed::from_int(1),
    );
    assert_eq!(accels.len(), 2);
    // Equal masses → equal magnitude accelerations toward each other
    // a0 should point toward body1, a1 should point toward body0
    // a0.x > 0 (body1 is at +1, body0 at -1)
    // a1.x < 0 (body0 is at -1, body1 at +1)
    assert!(accels[0].x > Fixed::from_int(0));
    assert!(accels[1].x < Fixed::from_int(0));
    // y-components should be zero (bodies are on x-axis)
    assert_eq!(accels[0].y, Fixed::from_int(0));
    assert_eq!(accels[1].y, Fixed::from_int(0));
    assert_eq!(accels[0].z, Fixed::from_int(0));
    assert_eq!(accels[1].z, Fixed::from_int(0));
}

#[kani::proof]
fn verify_compute_accelerations_newton_third_law() {
    let bodies = two_bodies();
    let accels = integrator::compute_accelerations(
        &bodies,
        Fixed::from_parts(0, 1000),
        Fixed::from_int(1),
    );
    // Newton's third law: F_12 = -F_21
    // With equal masses: a0 = -a1
    assert_eq!(accels[0].x, -accels[1].x);
    assert_eq!(accels[0].y, -accels[1].y);
    assert_eq!(accels[0].z, -accels[1].z);
}

// ── Verlet Step ──────────────────────────────────────────────────────────

#[kani::proof]
fn verify_verlet_step_conserves_momentum() {
    let mut bodies = two_bodies();
    let initial_momentum = integrator::total_momentum(&bodies);

    integrator::verlet_step(
        &mut bodies,
        Fixed::from_parts(0, 100),  // dt = 0.01
        Fixed::from_parts(0, 1000), // softening = 0.001
        Fixed::from_int(1),          // g = 1
    );

    let final_momentum = integrator::total_momentum(&bodies);
    // Momentum should be conserved (within numerical precision)
    assert_eq!(initial_momentum, final_momentum);
}

#[kani::proof]
fn verify_verlet_step_single_body_no_change() {
    let mut bodies = vec![small_body()];
    let initial_pos = bodies[0].position;
    let initial_vel = bodies[0].velocity;

    integrator::verlet_step(
        &mut bodies,
        Fixed::from_parts(0, 100),
        Fixed::from_parts(0, 1000),
        Fixed::from_int(1),
    );

    // Single body with no external forces should not change
    assert_eq!(bodies[0].position, initial_pos);
    assert_eq!(bodies[0].velocity, initial_vel);
}

// ── Euler Step ───────────────────────────────────────────────────────────

#[kani::proof]
fn verify_euler_step_conserves_momentum() {
    let mut bodies = two_bodies();
    let initial_momentum = integrator::total_momentum(&bodies);

    integrator::euler_step(
        &mut bodies,
        Fixed::from_parts(0, 100),
        Fixed::from_parts(0, 1000),
        Fixed::from_int(1),
    );

    let final_momentum = integrator::total_momentum(&bodies);
    // Euler should also conserve momentum (internal forces only)
    assert_eq!(initial_momentum, final_momentum);
}

#[kani::proof]
fn verify_euler_step_single_body_no_change() {
    let mut bodies = vec![small_body()];
    let initial_pos = bodies[0].position;
    let initial_vel = bodies[0].velocity;

    integrator::euler_step(
        &mut bodies,
        Fixed::from_parts(0, 100),
        Fixed::from_parts(0, 1000),
        Fixed::from_int(1),
    );

    // Single body with no external forces should not change
    assert_eq!(bodies[0].position, initial_pos);
    assert_eq!(bodies[0].velocity, initial_vel);
}

// ── Total Energy ─────────────────────────────────────────────────────────

#[kani::proof]
fn verify_total_energy_negative_for_bound_system() {
    let bodies = two_bodies();
    let energy = integrator::total_energy(&bodies, Fixed::from_int(1));
    // Two-body system with zero initial velocity should have negative energy
    assert!(energy < Fixed::from_int(0));
}

// ── Center of Mass ───────────────────────────────────────────────────────

#[kani::proof]
fn verify_center_of_mass_two_equal_bodies() {
    let bodies = two_bodies();
    let com = integrator::center_of_mass(&bodies);
    // Two equal masses at ±1 on x-axis → COM at origin
    assert_eq!(com.x, Fixed::from_int(0));
    assert_eq!(com.y, Fixed::from_int(0));
    assert_eq!(com.z, Fixed::from_int(0));
}
