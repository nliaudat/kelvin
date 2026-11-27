// period_detection.rs
//! Floyd's cycle-finding algorithm for deterministic step functions, and its
//! application to the Q32.64 n-body Verlet simulation.
//!
//! States are compared exactly (no hashing), so a reported cycle is a genuine
//! repeat. Memory use is constant: two or three states are kept, never the
//! trajectory.

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
