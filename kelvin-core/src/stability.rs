//! Stability monitoring for n-body simulations.
//!
//! Detects two failure modes that undermine Kelvin's security:
//!
//! 1. **Planet Ejection** — A body acquires enough energy to escape the
//!    system, reducing the effective N from ≥3 to 2. A 2-body system has
//!    a closed-form analytical solution (Kepler's laws), which would allow
//!    an attacker to shortcut the simulation.
//!
//! 2. **Gravitational Collapse** — Two or more bodies approach within a
//!    minimum separation distance, effectively merging into a single body.
//!    This reduces the degrees of freedom and can lead to predictable
//!    post-collapse dynamics.
//!
//! Both conditions are detected at runtime during the simulation. If either
//! occurs, the configuration is rejected (fail-fast) — the user must choose
//! a different initial configuration that remains chaotic and stable for
//! the full simulation duration.
//!
//! ## References
//!
//! - Murray, C. D., & Dermott, S. F. (1999). *Solar System Dynamics*.
//!   Cambridge University Press. — Orbital energy and escape conditions.

extern crate alloc;
use core::fmt;

use crate::body::OrbitalBody;
use crate::constants::{EJECTION_ENERGY_THRESHOLD, MONITOR_INTERVAL};
use crate::Fixed;
use crate::integrator::verlet_step;

/// Errors from stability monitoring.
#[derive(Clone, Debug)]
pub enum StabilityError {
    /// A body has been ejected from the system (unbound orbit).
    ///
    /// The body has total specific energy ≥ 0, meaning it is on a
    /// hyperbolic or parabolic trajectory and will escape to infinity.
    /// This reduces the effective N, making the system predictable.
    BodyEjected {
        /// Index of the ejected body.
        body_index: usize,
        /// Simulation step at which ejection was detected.
        step: u64,
        /// Total specific energy of the body (in AU²/yr²).
        energy: f64,
    },
    /// Two bodies have approached too closely (gravitational collapse).
    ///
    /// The bodies are within `MIN_SEPARATION` distance of each other,
    /// effectively reducing the degrees of freedom of the system.
    BodyCollision {
        /// Index of the first body.
        body_i: usize,
        /// Index of the second body.
        body_j: usize,
        /// Simulation step at which collapse was detected.
        step: u64,
        /// Distance between the bodies (in AU).
        distance: f64,
    },
}

impl fmt::Display for StabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StabilityError::BodyEjected { body_index, step, energy } => {
                write!(
                    f,
                    "body {} ejected at step {} (energy = {:.6e} AU²/yr²)",
                    body_index, step, energy
                )
            }
            StabilityError::BodyCollision { body_i, body_j, step, distance } => {
                write!(
                    f,
                    "bodies {} and {} collided at step {} (distance = {:.6e} AU)",
                    body_i, body_j, step, distance
                )
            }
        }
    }
}

/// Check whether a body is on an unbound (ejected) trajectory.
///
/// Computes the total energy of the body relative to the rest of the system:
///
///   E_i = 0.5 * m_i * v_i² - Σ_{j≠i} G * m_i * m_j / |r_i - r_j|
///
/// If E_i ≥ 0, the body is on a hyperbolic or parabolic trajectory and
/// will escape to infinity. A small threshold (`EJECTION_ENERGY_THRESHOLD`)
/// accounts for numerical precision in fixed-point arithmetic.
///
/// # Arguments
/// * `body_index` — Index of the body to check.
/// * `bodies` — All bodies in the system.
/// * `g` — Gravitational constant.
///
/// # Returns
/// `true` if the body is ejected (unbound), `false` otherwise.
pub fn is_body_ejected(body_index: usize, bodies: &[OrbitalBody], g: Fixed) -> bool {
    let n = bodies.len();
    if body_index >= n {
        return false;
    }

    let body = &bodies[body_index];

    // Kinetic energy: 0.5 * m * v² (use the existing method on OrbitalBody)
    let kinetic = body.kinetic_energy();

    // Potential energy: -Σ_{j≠i} G * m_i * m_j / |r_i - r_j|
    let mut potential = Fixed::ZERO;
    for j in 0..n {
        if j == body_index {
            continue;
        }
        let diff = bodies[j].position - body.position;
        let dist = diff.length();
        if dist > Fixed::ZERO {
            potential -= g * body.mass * bodies[j].mass / dist;
        }
    }

    // Total energy
    let total_energy = kinetic + potential;

    // If total energy >= threshold, the body is unbound (ejected)
    total_energy >= EJECTION_ENERGY_THRESHOLD
}

/// Detect gravitational collapse between any pair of bodies.
///
/// Checks if any two bodies are closer than `min_separation` distance.
/// If so, the system has undergone effective degree-of-freedom reduction.
///
/// # Arguments
/// * `bodies` — All bodies in the system.
/// * `min_separation` — Minimum allowed distance between any two bodies.
///
/// # Returns
/// `Some((i, j, distance))` if collapse detected, `None` otherwise.
pub fn detect_collapse(
    bodies: &[OrbitalBody],
    min_separation: Fixed,
) -> Option<(usize, usize, Fixed)> {
    let n = bodies.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let diff = bodies[j].position - bodies[i].position;
            let dist = diff.length();
            if dist < min_separation {
                return Some((i, j, dist));
            }
        }
    }
    None
}

/// Run the simulation with periodic stability monitoring.
///
/// Wraps the standard `simulate()` loop with periodic checks for:
/// - Body ejection (unbound orbit energy)
/// - Gravitational collapse (bodies too close)
///
/// Checks occur every `monitor_interval` steps. If a stability violation
/// is detected, the simulation stops and returns a `StabilityError`.
///
/// # Arguments
/// * `bodies` — Mutable slice of orbital bodies (modified in-place).
/// * `steps` — Total number of simulation steps.
/// * `dt` — Time step.
/// * `softening` — Softening factor.
/// * `g` — Gravitational constant.
/// * `min_separation` — Minimum allowed distance between bodies.
/// * `monitor_interval` — Steps between stability checks.
///
/// # Returns
/// `Ok(())` if the simulation completed without stability violations.
/// `Err(StabilityError)` if ejection or collapse was detected.
pub fn simulate_with_monitoring(
    bodies: &mut [OrbitalBody],
    steps: u64,
    dt: Fixed,
    softening: Fixed,
    g: Fixed,
    min_separation: Fixed,
    monitor_interval: u64,
) -> Result<(), StabilityError> {
    let effective_interval = if monitor_interval == 0 {
        MONITOR_INTERVAL
    } else {
        monitor_interval
    };

    // Ensure we check at least once at the end
    let check_interval = effective_interval.min(steps);

    // Check initial configuration before any steps
    if let Some((i, j, dist)) = detect_collapse(bodies, min_separation) {
        return Err(StabilityError::BodyCollision {
            body_i: i,
            body_j: j,
            step: 0,
            distance: dist.to_f64(),
        });
    }
    for i in 0..bodies.len() {
        if is_body_ejected(i, bodies, g) {
            let energy = (bodies[i].kinetic_energy()
                + gravitational_potential(i, bodies, g))
                .to_f64();
            return Err(StabilityError::BodyEjected {
                body_index: i,
                step: 0,
                energy,
            });
        }
    }

    for step in 0..steps {
        verlet_step(bodies, dt, softening, g);

        // Check stability at intervals and at the final step
        if (step + 1) % check_interval == 0 || step + 1 == steps {
            // Check for collapse first (geometric check, cheaper)
            if let Some((i, j, dist)) = detect_collapse(bodies, min_separation) {
                return Err(StabilityError::BodyCollision {
                    body_i: i,
                    body_j: j,
                    step: step + 1,
                    distance: dist.to_f64(),
                });
            }

            // Check for ejection (energy-based check)
            for i in 0..bodies.len() {
                if is_body_ejected(i, bodies, g) {
                    let energy = (bodies[i].kinetic_energy()
                        + gravitational_potential(i, bodies, g))
                        .to_f64();
                    return Err(StabilityError::BodyEjected {
                        body_index: i,
                        step: step + 1,
                        energy,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Compute the gravitational potential energy of a single body with respect
/// to all other bodies in the system.
fn gravitational_potential(body_index: usize, bodies: &[OrbitalBody], g: Fixed) -> Fixed {
    let n = bodies.len();
    let body = &bodies[body_index];
    let mut potential = Fixed::ZERO;

    for j in 0..n {
        if j == body_index {
            continue;
        }
        let diff = bodies[j].position - body.position;
        let dist = diff.length();
        if dist > Fixed::ZERO {
            potential -= g * body.mass * bodies[j].mass / dist;
        }
    }

    potential
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;
    use crate::body::Vec3;
    use crate::constants::DEFAULT_G;
    use crate::Fixed;

    /// Create a stable 3-body system (bound orbits).
    fn stable_three_body_system() -> Vec<OrbitalBody> {
        let sun = OrbitalBody::new(
            Fixed::ONE,
            Vec3::ZERO,
            Vec3::ZERO,
        );
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
        vec![sun, planet1, planet2]
    }

    /// Create a system where one body is on an escape trajectory.
    fn ejection_system() -> Vec<OrbitalBody> {
        let sun = OrbitalBody::new(
            Fixed::ONE,
            Vec3::ZERO,
            Vec3::ZERO,
        );
        // Planet on hyperbolic trajectory (escape velocity)
        let planet = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::from_int(100), Fixed::ZERO, Fixed::ZERO), // Very high velocity
        );
        let planet2 = OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
        );
        vec![sun, planet, planet2]
    }

    /// Create a system with two bodies very close together (collapse).
    ///
    /// Two planets are placed at nearly the same position with the same
    /// velocity, so they remain close without immediately ejecting each
    /// other. The close proximity should trigger the collapse detector.
    fn collapse_system() -> Vec<OrbitalBody> {
        let sun = OrbitalBody::new(
            Fixed::ONE,
            Vec3::ZERO,
            Vec3::ZERO,
        );
        // Two planets at nearly the same position, same velocity
        // so they stay close without a huge relative acceleration
        let planet1 = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        );
        let planet2 = OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ONE + Fixed::from_raw(1 << 30), Fixed::ZERO, Fixed::ZERO), // ~9.3e-10 AU apart
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO), // Same velocity as planet1
        );
        vec![sun, planet1, planet2]
    }

    #[test]
    fn test_is_body_ejected_detects_escape() {
        let bodies = ejection_system();
        // The high-velocity planet should be detected as ejected
        assert!(is_body_ejected(1, &bodies, DEFAULT_G));
    }

    #[test]
    fn test_is_body_ejected_confirms_bound() {
        let bodies = stable_three_body_system();
        // All bodies should be bound (not ejected)
        for i in 0..bodies.len() {
            assert!(!is_body_ejected(i, &bodies, DEFAULT_G),
                "body {} should be bound but was detected as ejected", i);
        }
    }

    #[test]
    fn test_detect_collapse_detects_close_bodies() {
        let bodies = collapse_system();
        let result = detect_collapse(&bodies, Fixed::from_raw(1 << 35));
        assert!(result.is_some(), "collapse should be detected");
        let (i, j, dist) = result.unwrap();
        assert!(i < j, "i should be less than j");
        assert!(dist < Fixed::from_raw(1 << 35), "distance should be below threshold");
    }

    #[test]
    fn test_detect_collapse_no_false_positive() {
        let bodies = stable_three_body_system();
        let result = detect_collapse(&bodies, Fixed::from_raw(1 << 35));
        assert!(result.is_none(), "stable system should not trigger collapse");
    }

    #[test]
    fn test_simulate_with_monitoring_stable_system() {
        let mut bodies = stable_three_body_system();
        let result = simulate_with_monitoring(
            &mut bodies,
            100, // Short run
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            DEFAULT_G,
            Fixed::from_raw(1 << 35),
            50,
        );
        assert!(result.is_ok(), "stable system should complete: {:?}", result);
    }

    #[test]
    fn test_simulate_with_monitoring_detects_ejection() {
        let mut bodies = ejection_system();
        let result = simulate_with_monitoring(
            &mut bodies,
            100,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            DEFAULT_G,
            Fixed::from_raw(1 << 35),
            10,
        );
        assert!(result.is_err(), "ejection system should be rejected");
        match result.unwrap_err() {
            StabilityError::BodyEjected { .. } => {} // Expected
            other => panic!("expected BodyEjected, got: {:?}", other),
        }
    }

    #[test]
    fn test_simulate_with_monitoring_detects_collapse() {
        let mut bodies = collapse_system();
        let result = simulate_with_monitoring(
            &mut bodies,
            100,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            DEFAULT_G,
            Fixed::from_raw(1 << 35), // Sensitive threshold
            10,
        );
        assert!(result.is_err(), "collapse system should be rejected");
        match result.unwrap_err() {
            StabilityError::BodyCollision { .. } => {} // Expected
            other => panic!("expected BodyCollision, got: {:?}", other),
        }
    }

    #[test]
    fn test_simulate_with_monitoring_zero_interval_defaults() {
        let mut bodies = stable_three_body_system();
        let result = simulate_with_monitoring(
            &mut bodies,
            50,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            DEFAULT_G,
            Fixed::from_raw(1 << 35),
            0, // Should use default MONITOR_INTERVAL
        );
        assert!(result.is_ok(), "should handle zero interval gracefully");
    }

    #[test]
    fn test_gravitational_potential_negative() {
        let bodies = stable_three_body_system();
        let pot = gravitational_potential(0, &bodies, DEFAULT_G);
        assert!(pot < Fixed::ZERO, "gravitational potential should be negative");
    }

    #[test]
    fn test_gravitational_potential_zero_for_single_body() {
        let body = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let pot = gravitational_potential(0, &[body], DEFAULT_G);
        assert_eq!(pot, Fixed::ZERO, "single body should have zero potential");
    }
}
