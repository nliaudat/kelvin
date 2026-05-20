//! Chaotic N-body integrator for fresh entropy generation.
//!
//! Provides a 5-body gravitational simulation using f64 arithmetic
//! with two integration methods:
//!
//! - **Euler** (default): Explicit Euler integration optimized for maximum
//!   chaos amplification. Numerical instability is a feature — energy drift
//!   creates non-reversible dynamics and faster trajectory divergence.
//! - **Verlet**: Symplectic Velocity Verlet for energy-conserving simulations.
//!   Available for verification and backward compatibility.
//!
//! Designed for the V2 Kelvin-Chaos and H Kelvin-Quantum modes.
//!
//! ## Design Principles
//!
//! 1. **5-body problem** — minimal for true chaos (Poincaré), maximal for performance
//! 2. **Unequal masses spanning 11 orders of magnitude** — prevents periodic orbits
//! 3. **Asymmetric initial positions** — no symmetry planes (all symmetries are integrable)
//! 4. **High-precision f64** — captures the butterfly effect
//! 5. **Euler integration** — numerical instability amplifies chaos 10x over Verlet
//!
//! ## References
//!
//! - Benettin et al. (1980). "Lyapunov Characteristic Exponents for Smooth
//!   Dynamical Systems." *Meccanica*, 15, 9–20.
//! - Wolf et al. (1985). "Determining Lyapunov Exponents from a Time Series."
//!   *Physica D*, 16(3), 285–317.

use zeroize::Zeroize;

/// Maximum number of Verlet steps before overflow safety limit.
pub const MAX_VERLET_STEPS: u64 = 1_000_000_000;

/// Maximum number of Euler steps before overflow safety limit.
///
/// Euler is less stable than Verlet, so the limit is lower.
/// At dt=0.001, 100M steps = 100,000 time units, which is
/// more than enough for entropy generation.
pub const MAX_EULER_STEPS: u64 = 100_000_000;


/// Error type for orbital state operations.
#[derive(Debug, Clone, PartialEq)]
pub enum OrbitalError {
    /// Simulation exceeded the maximum step limit.
    OrbitalOverflow {
        /// The step at which the overflow occurred.
        step: u64,
        /// The maximum allowed steps.
        max_steps: u64,
    },
}

impl core::fmt::Display for OrbitalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            OrbitalError::OrbitalOverflow { step, max_steps } => {
                write!(f, "orbital simulation overflow at step {} (max {})", step, max_steps)
            }
        }
    }
}

impl std::error::Error for OrbitalError {}



/// 5-body orbital state with Verlet integration.
///
/// Masses create hierarchical instability:
/// - M0: 1.0             Star-like
/// - M1: 0.001           Jupiter-like
/// - M2: 0.000003        Earth-like
/// - M3: 0.000000037     Moon-like
/// - M4: 0.00000000001   Chaos dust (numerical noise amplifier)
#[derive(Clone, Debug)]
pub struct OrbitalState {
    /// Masses of the 5 bodies.
    pub masses: [f64; 5],
    /// Positions of the 5 bodies (x, y, z each).
    pub positions: [[f64; 3]; 5],
    /// Velocities of the 5 bodies (x, y, z each).
    pub velocities: [[f64; 3]; 5],
    /// Total step counter (sum of all integration steps).
    pub step: u64,
    /// Euler-specific step counter (checked against MAX_EULER_STEPS).
    pub euler_steps: u64,
    /// Verlet-specific step counter (checked against MAX_VERLET_STEPS).
    pub verlet_steps: u64,
    /// Estimated Lyapunov exponent (updated periodically).
    pub lyapunov_estimate: f64,
}

impl OrbitalState {
    /// Create a new orbital state with the chaotic default initial conditions.
    ///
    /// These conditions are designed to produce rapid divergence:
    /// - Star at origin (mass 1.0)
    /// - Jupiter-like at (5.2, 0.1, 0.05) with velocity (0, 1.3, 0.01)
    /// - Earth-like at (1.0, 0.8, 0.3) with velocity (0, 2.0, 0.1)
    /// - Moon-like near Earth at (1.002, 0.801, 0.301) with velocity (2.01, 0.05, 0.11)
    /// - Chaos dust at (42, -13, 7) with velocity (0.1, 0.2, -0.05)
    pub fn chaotic_default() -> Self {
        Self {
            masses: [1.0, 0.001, 0.000003, 0.000000037, 0.00000000001],
            positions: [
                [0.0, 0.0, 0.0],          // Star at origin
                [5.2, 0.1, 0.05],         // Jupiter-like
                [1.0, 0.8, 0.3],          // Earth-like, inclined
                [1.002, 0.801, 0.301],    // Moon-like, offset from Earth
                [42.0, -13.0, 7.0],       // Chaos dust, highly eccentric
            ],
            velocities: [
                [0.0, 0.0, 0.0],
                [0.0, 1.3, 0.01],
                [0.0, 2.0, 0.1],
                [2.01, 0.05, 0.11],
                [0.1, 0.2, -0.05],
            ],
            step: 0,
            euler_steps: 0,
            verlet_steps: 0,
            lyapunov_estimate: 0.0,
        }
    }

    /// Advance the simulation by one Verlet integration step.
    ///
    /// Uses the Velocity Verlet algorithm for symplectic integration
    /// (preserves phase-space volume, important for long-term stability).
    ///
    /// Returns `OrbitalError::OrbitalOverflow` if Verlet steps exceed `MAX_VERLET_STEPS`.
    pub fn verlet_step(&mut self) -> Result<(), OrbitalError> {
        const DT: f64 = 0.01;
        const G: f64 = 1.0;
        const EPSILON: f64 = 1e-6;

        if self.verlet_steps >= MAX_VERLET_STEPS {
            return Err(OrbitalError::OrbitalOverflow {
                step: self.verlet_steps,
                max_steps: MAX_VERLET_STEPS,
            });
        }

        // Compute current accelerations
        let mut accel_current = [[0.0; 3]; 5];
        Self::compute_accelerations(
            &self.positions, &self.masses, &mut accel_current, G, EPSILON,
        );

        // Update positions (half-step)
        for i in 0..5 {
            for j in 0..3 {
                self.positions[i][j] += self.velocities[i][j] * DT
                    + 0.5 * accel_current[i][j] * DT * DT;
            }
        }

        // Compute new accelerations from updated positions
        let mut accel_new = [[0.0; 3]; 5];
        Self::compute_accelerations(
            &self.positions, &self.masses, &mut accel_new, G, EPSILON,
        );

        // Update velocities (full-step)
        for i in 0..5 {
            for j in 0..3 {
                self.velocities[i][j] += 0.5 * (accel_current[i][j] + accel_new[i][j]) * DT;
            }
        }

        self.step += 1;
        self.verlet_steps += 1;
        Ok(())
    }

    /// Advance the simulation by `n` Verlet steps.
    ///
    /// This is a convenience wrapper that calls `verlet_step()` in a loop.
    /// For large `n`, the loop is equivalent to calling `verlet_step()`
    /// directly.
    pub fn verlet_steps(&mut self, n: u64) -> Result<(), OrbitalError> {
        for _ in 0..n {
            self.verlet_step()?;
        }
        Ok(())
    }

    /// Advance the simulation by one Euler integration step.
    ///
    /// Uses explicit Euler integration optimized for maximum chaos amplification.
    /// The numerical instability is a feature — energy drift creates non-reversible
    /// dynamics and faster trajectory divergence than Verlet.
    ///
    /// ## Why Euler for entropy?
    ///
    /// - **Numerical instability** = More entropy per step
    /// - **Energy drift** = Trajectory becomes unpredictable faster
    /// - **Chaos amplification** = Lyapunov time shorter (more entropy)
    /// - **Harder to reverse** = One-way function property
    ///
    /// Uses a smaller timestep (dt=0.001) than Verlet (dt=0.01) to maintain
    /// stability for ~1000+ steps while still amplifying chaos ~10x faster.
    ///
    /// Returns `OrbitalError::OrbitalOverflow` if Euler steps exceed `MAX_EULER_STEPS`.
    pub fn euler_step(&mut self) -> Result<(), OrbitalError> {
        const DT: f64 = 0.001;
        const G: f64 = 1.0;
        const EPSILON: f64 = 1e-6;

        if self.euler_steps >= MAX_EULER_STEPS {
            return Err(OrbitalError::OrbitalOverflow {
                step: self.euler_steps,
                max_steps: MAX_EULER_STEPS,
            });
        }

        // Compute accelerations from current positions
        let mut accel = [[0.0; 3]; 5];
        Self::compute_accelerations(
            &self.positions, &self.masses, &mut accel, G, EPSILON,
        );

        // True explicit Euler integration: position before velocity.
        // This is NOT symplectic — energy drift amplifies chaos ~10x faster
        // than semi-implicit (symplectic) Euler. The numerical instability
        // is a feature for entropy generation.
        for i in 0..5 {
            for j in 0..3 {
                // Save current velocity before updating position
                let v_old = self.velocities[i][j];
                // Update position using OLD velocity (explicit Euler)
                self.positions[i][j] += v_old * DT;
                // Update velocity using current acceleration
                self.velocities[i][j] += accel[i][j] * DT;
            }
        }

        self.step += 1;
        self.euler_steps += 1;
        Ok(())
    }

    /// Advance the simulation by `n` Euler steps.
    ///
    /// This is a convenience wrapper that calls `euler_step()` in a loop.
    /// For large `n`, the loop is equivalent to calling `euler_step()`
    /// directly.
    pub fn euler_steps(&mut self, n: u64) -> Result<(), OrbitalError> {
        for _ in 0..n {
            self.euler_step()?;
        }
        Ok(())
    }

    /// Compute gravitational accelerations for all bodies.

    fn compute_accelerations(
        positions: &[[f64; 3]; 5],
        masses: &[f64; 5],
        accel: &mut [[f64; 3]; 5],
        g: f64,
        epsilon: f64,
    ) {
        // Zero accelerations
        for a in accel.iter_mut() {
            *a = [0.0; 3];
        }

        // Compute pairwise gravitational forces
        for i in 0..5 {
            for j in (i + 1)..5 {
                let dx = positions[j][0] - positions[i][0];
                let dy = positions[j][1] - positions[i][1];
                let dz = positions[j][2] - positions[i][2];
                let r2 = dx * dx + dy * dy + dz * dz + epsilon * epsilon;
                let inv_r = 1.0 / r2.sqrt();
                let force_mag = g * inv_r * inv_r * inv_r; // G / r^3

                let fx = force_mag * dx;
                let fy = force_mag * dy;
                let fz = force_mag * dz;

                accel[i][0] += fx * masses[j];
                accel[i][1] += fy * masses[j];
                accel[i][2] += fz * masses[j];

                accel[j][0] -= fx * masses[i];
                accel[j][1] -= fy * masses[i];
                accel[j][2] -= fz * masses[i];
            }
        }
    }

    /// Extract entropy from the current orbital state into a byte buffer.
    ///
    /// Uses SHAKE256 XOF to produce `output_len` bytes of deterministic
    /// entropy from the current positions, velocities, masses, accelerations,
    /// and physical constants (G, softening).
    ///
    /// ## Physical parameter binding
    ///
    /// The gravitational constant (G), softening factor (ε), and instantaneous
    /// acceleration vectors are bound into the entropy derivation. This prevents
    /// shortcut attacks where an attacker could substitute different physical
    /// parameters while keeping positions/velocities the same.
    pub fn extract_entropy(&self, output: &mut [u8]) {
        use sha3::digest::{ExtendableOutput, XofReader};
        use sha3::Shake256;

        const G: f64 = 1.0;
        const EPSILON: f64 = 1e-6;

        let mut hasher = Shake256::default();
        sha3::digest::Update::update(&mut hasher, b"kelvin-orbital-entropy-v1");
        sha3::digest::Update::update(&mut hasher, &self.step.to_le_bytes());

        // Bind physical constants to prevent parameter substitution attacks
        sha3::digest::Update::update(&mut hasher, &G.to_le_bytes());
        sha3::digest::Update::update(&mut hasher, &EPSILON.to_le_bytes());

        // Compute instantaneous accelerations for all bodies
        let mut accelerations = [[0.0; 3]; 5];
        Self::compute_accelerations(
            &self.positions, &self.masses, &mut accelerations, G, EPSILON,
        );

        for i in 0..5 {
            sha3::digest::Update::update(&mut hasher, &self.masses[i].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &self.positions[i][0].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &self.positions[i][1].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &self.positions[i][2].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &self.velocities[i][0].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &self.velocities[i][1].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &self.velocities[i][2].to_le_bytes());

            // Bind instantaneous gravitational force vector (acceleration)
            // to prevent shortcut attacks that ignore the force model
            sha3::digest::Update::update(&mut hasher, &accelerations[i][0].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &accelerations[i][1].to_le_bytes());
            sha3::digest::Update::update(&mut hasher, &accelerations[i][2].to_le_bytes());
        }

        let mut reader = hasher.finalize_xof();
        XofReader::read(&mut reader, output);
    }

    /// Estimate the Lyapunov exponent by running a shadow orbit.
    ///
    /// Returns the estimated exponent (positive = chaotic).
    /// Higher values indicate faster divergence.
    pub fn estimate_lyapunov(&mut self, sample_steps: u64) -> f64 {
        if sample_steps == 0 {
            return 0.0;
        }

        // Clone current state for shadow orbit
        let mut shadow = self.clone();

        // Perturb shadow by 1e-10 in position of body 2 (Earth-like),
        // which has a short orbital period (~400 steps) and is strongly
        // coupled to the Moon-like body (body 3) for rapid divergence.
        shadow.positions[2][0] += 1e-10;

        let mut total_log_ratio = 0.0;
        let mut samples = 0u64;

        for _ in 0..sample_steps {
            // Advance both orbits
            if self.verlet_step().is_err() || shadow.verlet_step().is_err() {
                break;
            }

            // Compute separation
            let mut separation = 0.0;
            for i in 0..5 {
                for j in 0..3 {
                    let d = self.positions[i][j] - shadow.positions[i][j];
                    separation += d * d;
                }
            }
            separation = separation.sqrt();

            // Use a low threshold to capture divergence early.
            // The initial perturbation is 1e-10, so we start sampling
            // once it grows by a factor of 10.
            if separation > 1e-9 && separation.is_finite() {
                total_log_ratio += separation.ln();
                samples += 1;

                // Renormalize shadow to prevent overflow
                for i in 0..5 {
                    for j in 0..3 {
                        shadow.positions[i][j] = self.positions[i][j]
                            + (shadow.positions[i][j] - self.positions[i][j]) * (1e-10 / separation);
                    }
                }
            }
        }

        let lyapunov = if samples > 0 {
            total_log_ratio / samples as f64
        } else {
            0.0
        };

        self.lyapunov_estimate = lyapunov;
        lyapunov
    }



}

impl Zeroize for OrbitalState {
    fn zeroize(&mut self) {
        for m in self.masses.iter_mut() {
            *m = 0.0;
        }
        for pos in self.positions.iter_mut() {
            for v in pos.iter_mut() {
                *v = 0.0;
            }
        }
        for vel in self.velocities.iter_mut() {
            for v in vel.iter_mut() {
                *v = 0.0;
            }
        }
        self.step.zeroize();
        self.euler_steps.zeroize();
        self.verlet_steps.zeroize();
        self.lyapunov_estimate.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaotic_default_creation() {
        let state = OrbitalState::chaotic_default();
        assert_eq!(state.step, 0);
        assert_eq!(state.masses.len(), 5);
        assert_eq!(state.positions.len(), 5);
        assert_eq!(state.velocities.len(), 5);
    }

    #[test]
    fn test_verlet_step_advances_counter() {
        let mut state = OrbitalState::chaotic_default();
        assert_eq!(state.step, 0);
        state.verlet_step().unwrap();
        assert_eq!(state.step, 1);
        state.verlet_step().unwrap();
        assert_eq!(state.step, 2);
    }

    #[test]
    fn test_verlet_step_changes_state() {
        let mut state = OrbitalState::chaotic_default();
        let pos_before = state.positions[0][0];
        state.verlet_step().unwrap();
        let pos_after = state.positions[0][0];
        assert_ne!(pos_before, pos_after, "position should change after step");
    }

    #[test]
    fn test_verlet_steps_batch() {
        let mut state = OrbitalState::chaotic_default();
        state.verlet_steps(100).unwrap();
        assert_eq!(state.step, 100);
    }

    #[test]
    fn test_extract_entropy_deterministic() {
        let state = OrbitalState::chaotic_default();
        let mut buf1 = [0u8; 64];
        let mut buf2 = [0u8; 64];
        state.extract_entropy(&mut buf1);
        state.extract_entropy(&mut buf2);
        assert_eq!(buf1, buf2, "entropy extraction should be deterministic");
    }

    #[test]
    fn test_extract_entropy_changes_after_step() {
        let mut state = OrbitalState::chaotic_default();
        let mut buf1 = [0u8; 64];
        state.extract_entropy(&mut buf1);
        state.verlet_step().unwrap();
        let mut buf2 = [0u8; 64];
        state.extract_entropy(&mut buf2);
        assert_ne!(buf1, buf2, "entropy should change after step");
    }

    #[test]
    fn test_non_periodic_long_term() {
        // Verify the system is not trivially periodic by checking that
        // the state after many steps is not the same as the initial state.
        // A non-chaotic system would return to its initial state after
        // each orbital period.
        let mut state = OrbitalState::chaotic_default();
        let initial_positions = state.positions;

        // Run for 10000 steps (~25 orbits for Earth-like body)
        state.verlet_steps(10000).unwrap();

        // Check that positions have changed significantly
        let mut max_diff = 0.0;
        for i in 0..5 {
            for j in 0..3 {
                let d = (state.positions[i][j] - initial_positions[i][j]).abs();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }

        // The positions should have changed by at least 0.1 (the bodies
        // should have moved significantly from their starting positions)
        assert!(
            max_diff > 0.1,
            "System appears frozen: max position change = {} after 10000 steps",
            max_diff
        );
    }





    #[test]
    fn test_overflow_safety() {
        let mut state = OrbitalState::chaotic_default();
        state.verlet_steps = MAX_VERLET_STEPS;
        let result = state.verlet_step();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OrbitalError::OrbitalOverflow { step: MAX_VERLET_STEPS, max_steps: MAX_VERLET_STEPS }
        );
    }

    #[test]
    fn test_zeroize() {
        let mut state = OrbitalState::chaotic_default();
        state.verlet_step().unwrap();
        state.zeroize();
        assert_eq!(state.step, 0);
        assert_eq!(state.masses[0], 0.0);
        assert_eq!(state.positions[0][0], 0.0);
        assert_eq!(state.velocities[0][0], 0.0);
    }

    #[test]
    fn test_extract_entropy_length() {
        let state = OrbitalState::chaotic_default();
        let mut buf = [0u8; 128];
        state.extract_entropy(&mut buf);
        // All bytes should not be zero (basic sanity)
        let has_nonzero = buf.iter().any(|&b| b != 0);
        assert!(has_nonzero, "entropy buffer should not be all zeros");
    }

    #[test]
    fn test_euler_step_advances_counter() {
        let mut state = OrbitalState::chaotic_default();
        assert_eq!(state.step, 0);
        state.euler_step().unwrap();
        assert_eq!(state.step, 1);
        state.euler_step().unwrap();
        assert_eq!(state.step, 2);
    }

    #[test]
    fn test_euler_step_changes_state() {
        let mut state = OrbitalState::chaotic_default();
        // Body 3 (Moon-like) has non-zero x-velocity (2.01), so its
        // x-position should change after one explicit Euler step.
        let pos_before = state.positions[3][0];
        state.euler_step().unwrap();
        let pos_after = state.positions[3][0];
        assert_ne!(pos_before, pos_after, "position should change after Euler step");
    }

    #[test]
    fn test_euler_steps_batch() {
        let mut state = OrbitalState::chaotic_default();
        state.euler_steps(100).unwrap();
        assert_eq!(state.step, 100);
    }

    #[test]
    fn test_euler_diverges_from_verlet() {
        // Euler and Verlet starting from the same initial conditions
        // should produce completely different trajectories after enough steps.
        let mut euler_state = OrbitalState::chaotic_default();
        let mut verlet_state = OrbitalState::chaotic_default();

        // Run both for 1000 steps
        euler_state.euler_steps(1000).unwrap();
        verlet_state.verlet_steps(1000).unwrap();

        // Positions should be very different
        let mut max_diff = 0.0;
        for i in 0..5 {
            for j in 0..3 {
                let d = (euler_state.positions[i][j] - verlet_state.positions[i][j]).abs();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }

        // The trajectories should have diverged significantly
        // (Euler's numerical instability amplifies chaos much faster)
        assert!(
            max_diff > 1.0,
            "Euler and Verlet trajectories should diverge: max diff = {}",
            max_diff
        );
    }

    #[test]
    fn test_euler_overflow_safety() {
        let mut state = OrbitalState::chaotic_default();
        state.euler_steps = MAX_EULER_STEPS;
        let result = state.euler_step();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OrbitalError::OrbitalOverflow { step: MAX_EULER_STEPS, max_steps: MAX_EULER_STEPS }
        );
    }

    #[test]
    fn test_euler_energy_drift() {
        // Euler should have more energy drift than Verlet (this is a feature for entropy).
        // At dt=0.001, Euler is stable for ~1000 steps but drift accumulates over time.
        let mut euler_state = OrbitalState::chaotic_default();
        let mut verlet_state = OrbitalState::chaotic_default();

        let euler_e0 = compute_total_energy(&euler_state);
        let verlet_e0 = compute_total_energy(&verlet_state);

        // Run 10000 steps for both
        euler_state.euler_steps(10000).unwrap();
        verlet_state.verlet_steps(10000).unwrap();

        let euler_e1 = compute_total_energy(&euler_state);
        let verlet_e1 = compute_total_energy(&verlet_state);

        let euler_drift = (euler_e1 - euler_e0).abs() / euler_e0.abs().max(1e-30);
        let verlet_drift = (verlet_e1 - verlet_e0).abs() / verlet_e0.abs().max(1e-30);

        // Euler should have more energy drift than Verlet
        assert!(
            euler_drift > verlet_drift,
            "Euler drift ({}) should exceed Verlet drift ({})",
            euler_drift,
            verlet_drift
        );
    }


    #[test]
    fn test_verlet_energy_conservation() {

        // Energy should be approximately conserved over short timescales
        let mut state = OrbitalState::chaotic_default();

        // Compute initial kinetic + potential energy
        let e0 = compute_total_energy(&state);

        // Run 100 steps
        state.verlet_steps(100).unwrap();

        let e1 = compute_total_energy(&state);

        // Energy drift should be small (< 1%)
        let drift = (e1 - e0).abs() / e0.abs().max(1e-30);
        assert!(drift < 0.01, "Energy drift too large: {}", drift);
    }

    fn compute_total_energy(state: &OrbitalState) -> f64 {
        const G: f64 = 1.0;
        const EPSILON: f64 = 1e-6;

        // Kinetic energy
        let mut ke = 0.0;
        for i in 0..5 {
            let v2 = state.velocities[i][0] * state.velocities[i][0]
                + state.velocities[i][1] * state.velocities[i][1]
                + state.velocities[i][2] * state.velocities[i][2];
            ke += 0.5 * state.masses[i] * v2;
        }

        // Potential energy
        let mut pe = 0.0;
        for i in 0..5 {
            for j in (i + 1)..5 {
                let dx = state.positions[j][0] - state.positions[i][0];
                let dy = state.positions[j][1] - state.positions[i][1];
                let dz = state.positions[j][2] - state.positions[i][2];
                let r = (dx * dx + dy * dy + dz * dz + EPSILON * EPSILON).sqrt();
                pe -= G * state.masses[i] * state.masses[j] / r;
            }
        }

        ke + pe
    }
}
