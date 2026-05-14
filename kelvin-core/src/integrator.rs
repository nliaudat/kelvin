//! Symplectic Verlet integrator for n-body gravitational simulation.
//!
//! Uses the kick-drift-kick (position Verlet) formulation:
//! 1. Kick:   v ← v + a * dt/2
//! 2. Drift:  x ← x + v * dt
//! 3. Kick:   v ← v + a' * dt/2
//!
//! This is symplectic (energy-conserving) and time-reversible.
//!
//! ## References
//!
//! - Verlet, L. (1967). "Computer 'Experiments' on Classical Fluids. I.
//!   Thermodynamical Properties of Lennard-Jones Molecules." *Physical
//!   Review*, 159(1), 98–103. doi:10.1103/PhysRev.159.98
//!   — Original Verlet (leapfrog) integration method.
//! - Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical
//!   Integration: Structure-Preserving Algorithms for Ordinary Differential
//!   Equations* (2nd ed.). Springer.
//!   — Theoretical foundation for symplectic integrators.
//! - Wisdom, J., & Holman, M. (1991). "Symplectic Maps for the N-Body
//!   Problem." *The Astronomical Journal*, 102(4), 1528–1538.
//!   doi:10.1086/115978
//!   — Symplectic integration for n-body gravitational systems.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::body::{OrbitalBody, Vec3};
use crate::Fixed;

/// Compute gravitational accelerations for all bodies.
///
/// For each pair (i, j), computes:
///   a_i += G * m_j * (r_j - r_i) / (|r_ij|² + ε²)^(3/2)
///
/// where ε is the softening factor.
///
/// Complexity: O(n²) where n = number of bodies.
pub fn compute_accelerations(bodies: &[OrbitalBody], softening: Fixed, g: Fixed) -> Vec<Vec3> {
    let n = bodies.len();
    let mut accelerations = vec![Vec3::ZERO; n];

    let softening_sq = softening * softening;

    for i in 0..n {
        for j in (i + 1)..n {
            let diff = bodies[j].position - bodies[i].position;
            let dist_sq = diff.length_squared() + softening_sq;

            // |r_ij|² + ε²
            let dist = dist_sq.sqrt();

            // (|r_ij|² + ε²)^(3/2) = dist³
            let dist_cubed = dist_sq * dist;

            // G / dist³
            let factor = g / dist_cubed;

            // a_i += G * m_j * (r_j - r_i) / dist³
            let acc_i = diff.scale(factor * bodies[j].mass);
            accelerations[i] += acc_i;

            // a_j -= G * m_i * (r_j - r_i) / dist³
            let acc_j = diff.scale(factor * bodies[i].mass);
            accelerations[j] -= acc_j;
        }
    }

    accelerations
}

/// Perform one symplectic Verlet step (kick-drift-kick).
///
/// 1. Kick:   v ← v + a * dt/2
/// 2. Drift:  x ← x + v * dt
/// 3. Compute new accelerations a'
/// 4. Kick:   v ← v + a' * dt/2
pub fn verlet_step(bodies: &mut [OrbitalBody], dt: Fixed, softening: Fixed, g: Fixed) {
    let half_dt = dt / Fixed::from_int(2);

    // Step 1: Kick (half step)
    let accelerations = compute_accelerations(bodies, softening, g);
    for (body, acc) in bodies.iter_mut().zip(accelerations.iter()) {
        body.velocity += acc.scale(half_dt);
    }

    // Step 2: Drift (full step)
    for body in bodies.iter_mut() {
        body.position += body.velocity.scale(dt);
    }

    // Step 3: Compute new accelerations
    let new_accelerations = compute_accelerations(bodies, softening, g);

    // Step 4: Kick (half step)
    for (body, acc) in bodies.iter_mut().zip(new_accelerations.iter()) {
        body.velocity += acc.scale(half_dt);
    }
}

/// Run the simulation for a given number of steps.
///
/// This is the main simulation loop. It modifies the bodies in-place.
pub fn simulate(bodies: &mut [OrbitalBody], steps: u64, dt: Fixed, softening: Fixed, g: Fixed) {
    for _ in 0..steps {
        verlet_step(bodies, dt, softening, g);
    }
}

/// Compute total energy of the system.
///
/// E = KE + PE
/// KE = Σ 0.5 * m_i * v_i²
/// PE = -Σ_{i<j} G * m_i * m_j / |r_ij|
#[allow(dead_code)]
pub fn total_energy(bodies: &[OrbitalBody], g: Fixed) -> Fixed {
    let mut kinetic = Fixed::ZERO;
    let mut potential = Fixed::ZERO;

    for body in bodies {
        kinetic += body.kinetic_energy();
    }

    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let diff = bodies[j].position - bodies[i].position;
            let dist = diff.length();
            if dist > Fixed::ZERO {
                potential -= g * bodies[i].mass * bodies[j].mass / dist;
            }
        }
    }

    kinetic + potential
}

/// Compute total linear momentum of the system.
#[allow(dead_code)]
pub fn total_momentum(bodies: &[OrbitalBody]) -> Vec3 {
    let mut p = Vec3::ZERO;
    for body in bodies {
        p += body.momentum();
    }
    p
}

/// Compute center of mass position.
#[allow(dead_code)]
pub fn center_of_mass(bodies: &[OrbitalBody]) -> Vec3 {
    let mut total_mass = Fixed::ZERO;
    let mut weighted_pos = Vec3::ZERO;

    for body in bodies {
        total_mass += body.mass;
        weighted_pos += body.position.scale(body.mass);
    }

    if total_mass > Fixed::ZERO {
        weighted_pos / total_mass
    } else {
        Vec3::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Fixed, DEFAULT_G};

    fn two_body_system() -> Vec<OrbitalBody> {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let planet = OrbitalBody::new(
            Fixed::from_raw(1 << 54), // ~1e-6 solar masses
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        );
        vec![sun, planet]
    }

    #[test]
    fn test_compute_accelerations() {
        let bodies = two_body_system();
        let accs = compute_accelerations(&bodies, Fixed::from_raw(1 << 44), DEFAULT_G);
        assert_eq!(accs.len(), 2);
        // Sun should be attracted toward planet
        assert!(accs[0].x > Fixed::ZERO);
        // Planet should be attracted toward sun
        assert!(accs[1].x < Fixed::ZERO);
    }

    #[test]
    fn test_verlet_step_conserves_momentum() {
        let mut bodies = two_body_system();
        let initial_momentum = total_momentum(&bodies);

        for _ in 0..100 {
            verlet_step(&mut bodies, Fixed::from_raw(1 << 44), Fixed::from_raw(1 << 44), DEFAULT_G);
        }

        let final_momentum = total_momentum(&bodies);
        let diff = (final_momentum - initial_momentum).length();
        assert!(diff < Fixed::from_raw(1 << 40), "Momentum not conserved: {}", diff.to_f64());
    }

    #[test]
    fn test_verlet_step_energy_stability() {
        let mut bodies = two_body_system();
        let initial_energy = total_energy(&bodies, DEFAULT_G);

        for _ in 0..1000 {
            verlet_step(&mut bodies, Fixed::from_raw(1 << 44), Fixed::from_raw(1 << 44), DEFAULT_G);
        }

        let final_energy = total_energy(&bodies, DEFAULT_G);
        let energy_diff = (final_energy - initial_energy).abs();
        let relative_diff = energy_diff / initial_energy.abs();
        assert!(
            relative_diff.to_f64() < 0.01,
            "Energy drift too large: {}",
            relative_diff.to_f64()
        );
    }

    #[test]
    fn test_simulate_runs() {
        let mut bodies = two_body_system();
        simulate(&mut bodies, 100, Fixed::from_raw(1 << 44), Fixed::from_raw(1 << 44), DEFAULT_G);
        // Bodies should have moved
        assert!(
            bodies[0].position.length() > Fixed::ZERO || bodies[1].position.length() > Fixed::ZERO
        );
    }

    #[test]
    fn test_center_of_mass() {
        let bodies = two_body_system();
        let com = center_of_mass(&bodies);
        // COM should be near the sun (since sun is much more massive)
        assert!(com.length() < Fixed::from_int(1));
    }

    #[test]
    fn test_total_energy_negative() {
        let bodies = two_body_system();
        let energy = total_energy(&bodies, DEFAULT_G);
        // Bound orbit should have negative total energy
        assert!(energy < Fixed::ZERO);
    }

    #[test]
    fn test_three_body_symmetry() {
        // Three equal masses at vertices of equilateral triangle
        let mass = Fixed::from_raw(1 << 56); // ~1/256 solar masses
        let bodies = vec![
            OrbitalBody::new(
                mass,
                Vec3::new(Fixed::from_int(1), Fixed::ZERO, Fixed::ZERO),
                Vec3::ZERO,
            ),
            OrbitalBody::new(
                mass,
                Vec3::new(Fixed::from_int(-1), Fixed::ZERO, Fixed::ZERO),
                Vec3::ZERO,
            ),
            OrbitalBody::new(
                mass,
                Vec3::new(Fixed::ZERO, Fixed::from_int(1), Fixed::ZERO),
                Vec3::ZERO,
            ),
        ];

        let accs = compute_accelerations(&bodies, Fixed::from_raw(1 << 44), DEFAULT_G);
        assert_eq!(accs.len(), 3);
        // All accelerations should be non-zero
        for acc in &accs {
            assert!(acc.length() > Fixed::ZERO);
        }
    }

    #[test]
    fn test_single_body_no_acceleration() {
        let body = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let accs = compute_accelerations(&[body], Fixed::from_raw(1 << 44), DEFAULT_G);
        assert_eq!(accs.len(), 1);
        assert_eq!(accs[0], Vec3::ZERO);
    }

    #[test]
    fn test_verlet_step_single_body() {
        let mut bodies = [OrbitalBody::new(
            Fixed::ONE,
            Vec3::new(Fixed::from_int(1), Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::from_int(1), Fixed::ZERO, Fixed::ZERO),
        )];
        let initial_pos = bodies[0].position;
        verlet_step(&mut bodies, Fixed::from_int(1), Fixed::from_raw(1 << 44), DEFAULT_G);
        // Single body should drift at constant velocity
        assert!(bodies[0].position.x > initial_pos.x);
    }
}
