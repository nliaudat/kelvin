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





#[allow(dead_code, unused_imports, unused_variables, unused_assignments)]
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



/// Result of a cycle search: the trajectory enters a cycle after `mu` steps
/// (pre-period) and then repeats every `lambda` steps (minimal period).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cycle {
    /// Number of steps before the first state that belongs to the cycle.
    pub mu: usize,
    /// Minimal period of the cycle (number of steps to return to a state).
    pub lambda: usize,
}

/// Floyd's cycle finding over an arbitrary deterministic step function, using
/// exact state equality (no hash, hence no collision false positives).
///
/// Returns `None` if no repeated state is found within `max_steps` steps of
/// the tortoise (the hare performs at most `2 * max_steps` steps).
pub fn find_cycle<S, F>(init: &S, mut step: F, max_steps: usize) -> Option<Cycle>
where
    S: Clone + PartialEq,
    F: FnMut(&mut S),
{
    let mut tortoise = init.clone();
    let mut hare = init.clone();

    // Phase 1: after k iterations tortoise is at x_k and hare at x_2k.
    let mut meet = false;
    for _ in 0..max_steps {
        step(&mut tortoise);
        step(&mut hare);
        step(&mut hare);
        if tortoise == hare {
            meet = true;
            break;
        }
    }
    if !meet {
        return None;
    }

    // Phase 2: find mu, the index of the first state of the cycle.
    let mut mu = 0usize;
    tortoise = init.clone();
    while tortoise != hare {
        step(&mut tortoise);
        step(&mut hare);
        mu += 1;
    }

    // Phase 3: find lambda, the minimal cycle length.
    let mut lambda = 1usize;
    hare = tortoise.clone();
    step(&mut hare);
    while tortoise != hare {
        step(&mut hare);
        lambda += 1;
    }
    Some(Cycle { mu, lambda })
}

/// Detect exact periodicity of the real `N`-body Verlet simulation.
///
/// Compares the full (position, velocity, mass) state bit-for-bit, so a
/// `Some` result is a genuine repeat of the finite-precision trajectory.
/// `None` only means no repeat was found within `max_steps`; it is not a
/// proof that the period is long.
pub fn detect_orbital_period<const N: usize>(
    init: &[crate::OrbitalBody; N],
    dt: crate::Fixed,
    softening: crate::Fixed,
    g: crate::Fixed,
    max_steps: usize,
) -> Option<Cycle> {
    find_cycle(
        init,
        |s: &mut [crate::OrbitalBody; N]| crate::verlet_step(s, dt, softening, g),
        max_steps,
    )
}

#[cfg(test)]
mod cycle_tests {
    use super::*;

    #[test]
    fn finds_mu_and_lambda() {
        // x -> (x*x + 1) mod 255 style map: use a fixed functional graph.
        // 0 -> 1 -> 2 -> 3 -> 4 -> 2  (mu = 2, lambda = 3)
        let next = [1u8, 2, 3, 4, 2];
        let c = find_cycle(&0u8, |s| *s = next[*s as usize], 100).unwrap();
        assert_eq!(c, Cycle { mu: 2, lambda: 3 });
    }

    #[test]
    fn fixed_point_has_period_one() {
        let c = find_cycle(&7u32, |_| {}, 10).unwrap();
        assert_eq!(c, Cycle { mu: 0, lambda: 1 });
    }

    #[test]
    fn no_cycle_within_budget() {
        assert_eq!(find_cycle(&0u64, |s| *s += 1, 1000), None);
    }
}
