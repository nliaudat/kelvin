//! Property-based fuzz test for `OrbitalState` simulation.
//!
//! Generates random orbital states (masses, positions, velocities) and
//! runs the simulation through both integrators:
//! - `verlet_steps(n)` — symplectic Verlet
//! - `euler_steps(n)` — explicit Euler (chaos-amplified)
//! - `estimate_lyapunov(n)` — divergence measurement
//!
//! The test detects panics (crashes), infinite loops, or hangs caused by
//! degenerate orbital configurations that bypass the validation bodyguards.
//!
//! Run with: `cargo test -p kelvin-fuzz --test simulate_state`
//! Heavy run: `PROPTEST_CASES=100000 cargo test -p kelvin-fuzz --test simulate_state`

use proptest::prelude::*;
use kelvin_kdf::OrbitalState;

/// Maximum steps to simulate in a single fuzz iteration.
/// Kept low to avoid long-running tests while still probing edge cases.
const MAX_FUZZ_STEPS: u64 = 10_000;

proptest! {
    #[test]
    fn fuzz_simulate_state(
        masses in proptest::array::uniform5(0.0f64..1.0f64),
        pos_x in proptest::array::uniform5(-100.0f64..100.0f64),
        pos_y in proptest::array::uniform5(-100.0f64..100.0f64),
        pos_z in proptest::array::uniform5(-100.0f64..100.0f64),
        vel_x in proptest::array::uniform5(-10.0f64..10.0f64),
        vel_y in proptest::array::uniform5(-10.0f64..10.0f64),
        vel_z in proptest::array::uniform5(-10.0f64..10.0f64),
        steps in 0u64..MAX_FUZZ_STEPS,
    ) {
        // Build the orbital state from random vectors
        let mut state = OrbitalState {
            masses,
            positions: [
                [pos_x[0], pos_y[0], pos_z[0]],
                [pos_x[1], pos_y[1], pos_z[1]],
                [pos_x[2], pos_y[2], pos_z[2]],
                [pos_x[3], pos_y[3], pos_z[3]],
                [pos_x[4], pos_y[4], pos_z[4]],
            ],
            velocities: [
                [vel_x[0], vel_y[0], vel_z[0]],
                [vel_x[1], vel_y[1], vel_z[1]],
                [vel_x[2], vel_y[2], vel_z[2]],
                [vel_x[3], vel_y[3], vel_z[3]],
                [vel_x[4], vel_y[4], vel_z[4]],
            ],
            step: 0,
            euler_steps: 0,
            verlet_steps: 0,
            lyapunov_estimate: 0.0,
        };

        // ── Verlet integration ──
        // Expected: Ok(()) or Err(OrbitalOverflow). Panic = bug.
        let _ = state.verlet_steps(steps);

        // Reset for Euler test
        let mut state = OrbitalState {
            masses,
            positions: [
                [pos_x[0], pos_y[0], pos_z[0]],
                [pos_x[1], pos_y[1], pos_z[1]],
                [pos_x[2], pos_y[2], pos_z[2]],
                [pos_x[3], pos_y[3], pos_z[3]],
                [pos_x[4], pos_y[4], pos_z[4]],
            ],
            velocities: [
                [vel_x[0], vel_y[0], vel_z[0]],
                [vel_x[1], vel_y[1], vel_z[1]],
                [vel_x[2], vel_y[2], vel_z[2]],
                [vel_x[3], vel_y[3], vel_z[3]],
                [vel_x[4], vel_y[4], vel_z[4]],
            ],
            step: 0,
            euler_steps: 0,
            verlet_steps: 0,
            lyapunov_estimate: 0.0,
        };

        // ── Euler integration ──
        // Expected: Ok(()) or Err(OrbitalOverflow). Panic = bug.
        let _ = state.euler_steps(steps);

        // ── Lyapunov estimation ──
        // Only run if steps > 0 (needs at least some simulation to measure)
        if steps > 0 {
            let mut state = OrbitalState {
                masses,
                positions: [
                    [pos_x[0], pos_y[0], pos_z[0]],
                    [pos_x[1], pos_y[1], pos_z[1]],
                    [pos_x[2], pos_y[2], pos_z[2]],
                    [pos_x[3], pos_y[3], pos_z[3]],
                    [pos_x[4], pos_y[4], pos_z[4]],
                ],
                velocities: [
                    [vel_x[0], vel_y[0], vel_z[0]],
                    [vel_x[1], vel_y[1], vel_z[1]],
                    [vel_x[2], vel_y[2], vel_z[2]],
                    [vel_x[3], vel_y[3], vel_z[3]],
                    [vel_x[4], vel_y[4], vel_z[4]],
                ],
                step: 0,
                euler_steps: 0,
                verlet_steps: 0,
                lyapunov_estimate: 0.0,
            };
            // estimate_lyapunov runs internally and should not panic
            let _ = state.estimate_lyapunov(steps);
        }
    }
}
