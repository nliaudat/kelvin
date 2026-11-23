// verify_periodicity.rs
//
// Corrected 2026-11-23. The previous version imported `rand`, `State::default()`
// and `detect_period`, then asserted that the stub never finds a period within
// 1,000,000 steps. The stub actually has period 1, so that assertion was false;
// the file was also never compiled.
//
// SCOPE: properties of the current `period_detection` stub only. The stub's
// step and hash are constant, so it is period 1 by construction. A real
// simulator would need a different harness.

use crate::period_detection::{detect_period, State};

/// With a zero step budget, no iteration runs and no period is reported.
#[kani::proof]
#[kani::unwind(4)]
fn verify_periodicity_no_steps() {
    let period = detect_period(&State, 0);
    kani::assert(period.is_none(), "a zero step budget reports no period");
}

/// For a positive budget the stub reports a period of 1, and never more than the
/// budget.
#[kani::proof]
#[kani::unwind(8)]
fn verify_periodicity_min_period_bound() {
    let n: usize = kani::any();
    kani::assume(n >= 1);
    kani::assume(n <= 4);

    let period = detect_period(&State, n);

    kani::assert(period.is_some(), "the stub always finds a cycle");
    kani::assert(period == Some(1), "the stub's cycle has length 1");
    kani::assert(period.unwrap() <= n, "reported period does not exceed the budget");
}
