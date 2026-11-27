// verify_periodicity.rs
//
// Rewritten 2026-11-27. These harnesses call the real
// `kelvin_core::find_cycle`, the function behind `detect_orbital_period`. The
// earlier versions checked a stub whose step function did nothing.
//
// Method: the step function is an arbitrary map on `C6_STATES` states, chosen
// symbolically together with the start state. Every such map and start state is
// therefore covered.
//
// SCOPE: bounded to `C6_STATES` states. These harnesses do not run the Verlet
// simulation, and say nothing about the period of any orbital configuration.

use crate::period_detection::{find_cycle, Cycle};

const C6_STATES: usize = 6;

/// An arbitrary deterministic step function and start state.
fn c6_any_map() -> ([u8; C6_STATES], u8) {
    let next: [u8; C6_STATES] = kani::any();
    let mut i = 0;
    while i < C6_STATES {
        kani::assume((next[i] as usize) < C6_STATES);
        i += 1;
    }
    let start: u8 = kani::any();
    kani::assume((start as usize) < C6_STATES);
    (next, start)
}

/// The state reached from `start` after `steps` steps.
fn c6_state_at(next: &[u8; C6_STATES], start: u8, steps: usize) -> u8 {
    let mut state = start;
    let mut i = 0;
    while i < steps {
        state = next[state as usize];
        i += 1;
    }
    state
}

/// `cycle` is the true cycle: it repeats, its length is minimal and it begins
/// at the first state that recurs.
fn c6_check_cycle(next: &[u8; C6_STATES], start: u8, cycle: Cycle) {
    let Cycle { mu, lambda } = cycle;
    kani::assert(lambda >= 1, "the period is positive");
    kani::assert(mu + lambda <= C6_STATES, "the cycle fits in the state space");

    let entry = c6_state_at(next, start, mu);
    kani::assert(c6_state_at(next, start, mu + lambda) == entry, "the state repeats");

    let shorter: usize = kani::any();
    kani::assume(shorter >= 1 && shorter < lambda);
    kani::assert(c6_state_at(next, start, mu + shorter) != entry, "the period is minimal");

    let earlier: usize = kani::any();
    kani::assume(earlier < mu);
    kani::assert(
        c6_state_at(next, start, earlier + lambda) != c6_state_at(next, start, earlier),
        "no earlier state belongs to the cycle",
    );
}

/// With a budget of at least the number of states, `find_cycle` always finds a
/// cycle, and it is the correct one.
#[kani::proof]
#[kani::unwind(14)]
fn verify_find_cycle_is_complete_and_correct() {
    let (next, start) = c6_any_map();
    let result = find_cycle(&start, |s| *s = next[*s as usize], C6_STATES);

    kani::assert(result.is_some(), "a cycle is found within the budget");
    if let Some(cycle) = result {
        c6_check_cycle(&next, start, cycle);
        kani::cover!(cycle.mu > 0 && cycle.lambda > 1, "tail and long cycle reachable");
    }
}

/// With any smaller budget, whatever is reported is still the correct cycle.
#[kani::proof]
#[kani::unwind(14)]
fn verify_find_cycle_is_sound_for_any_budget() {
    let (next, start) = c6_any_map();
    let budget: usize = kani::any();
    kani::assume(budget <= C6_STATES);
    let result = find_cycle(&start, |s| *s = next[*s as usize], budget);

    if let Some(cycle) = result {
        c6_check_cycle(&next, start, cycle);
    }
    kani::cover!(result.is_none() && budget > 0, "a short budget can find nothing");
}

/// With a zero budget the step function is never called and nothing is reported.
#[kani::proof]
#[kani::unwind(4)]
fn verify_find_cycle_zero_budget() {
    let start: u8 = kani::any();
    let mut calls = 0u32;
    let result = find_cycle(&start, |_| calls += 1, 0);

    kani::assert(result.is_none(), "a zero step budget reports no cycle");
    kani::assert(calls == 0, "no step is taken");
}
