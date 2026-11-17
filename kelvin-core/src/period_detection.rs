// period_detection.rs
//! Implements Floyd's cycle‑finding algorithm to detect periodicity in the
//! Q32.64 n‑body simulation. The state is hashed (blake3) on each step to
//! avoid storing the full trajectory.
//!
//! The function returns the number of steps before a repeat is observed, or
//! `None` if no repeat occurs within `max_steps`.
//!
//! This module is deliberately lightweight – it only requires the public API
//! of the simulator: `simulate_step(state: &mut State)` and a way to obtain a
//! serialisable byte representation of the state (`state.as_bytes()`).
//!
//! The implementation is generic enough to be used both in production code
//! and in a Kani harness.

use blake3::Hasher;

// Stub definitions for compilation without the full simulation crate.
#[derive(Clone, Debug, Default)]
pub struct State;

fn simulate_step(_state: &mut State) {}


/// Compute a deterministic hash of a simulation state.
fn hash_state(_state: &State) -> [u8; 32] {
    // Placeholder deterministic hash for the stub state.
    [0u8; 32]
}

/// Detect the period length of the simulation starting from `init`.
/// Returns `Some(period)` if a repeat is found within `max_steps`, otherwise
/// `None`.
pub fn detect_period(init: &State, max_steps: usize) -> Option<usize> {
    // Floyd's tortoise and hare.
    let mut tortoise = init.clone();
    let mut hare = init.clone();

    // Advance hare by two steps, tortoise by one.
    for step in 1..=max_steps {
        // Move tortoise one step.
        simulate_step(&mut tortoise);
        // Move hare two steps.
        simulate_step(&mut hare);
        simulate_step(&mut hare);

        if hash_state(&tortoise) == hash_state(&hare) {
            // Cycle detected – now find the start of the cycle.
            let mut mu = 0usize;
            let mut tortoise2 = init.clone();
            while hash_state(&tortoise) != hash_state(&tortoise2) {
                simulate_step(&mut tortoise);
                simulate_step(&mut tortoise2);
                mu += 1;
            }
            // Find the length of the cycle.
            let mut lam = 1usize;
            simulate_step(&mut hare);
            while hash_state(&tortoise) != hash_state(&hare) {
                simulate_step(&mut hare);
                lam += 1;
                if lam > max_steps { break; }
            }
            return Some(lam);
        }
        // If we exceed max_steps without a match, give up.
        if step == max_steps { break; }
    }
    None
}


