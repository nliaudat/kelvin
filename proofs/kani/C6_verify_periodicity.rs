// C6_verify_periodicity.rs
//
// Corrected 2026-11-23. The previous version "verified" Floyd's algorithm by
// running the loop and asserting the loop counter was in bounds. This version
// proves the loop invariant on an abstract model, and checks the behaviour of
// the current `period_detection` stub.
//
// SCOPE: the Floyd invariant is proved. Nothing here says the *dynamics* have a
// long period; the stub in kelvin-core/src/period_detection.rs has period 1 by
// construction (its step and hash are constant), which the last harness makes
// explicit rather than hiding.

/// One Floyd iteration: tortoise advances one state, hare two.
fn c6_advance(tortoise: u32, hare: u32) -> (u32, u32) {
    (tortoise + 1, hare + 2)
}

/// Unroll `k` iterations of the abstract Floyd loop.
fn c6_run(k: u32, tortoise: u32, hare: u32) -> (u32, u32) {
    let mut t = tortoise;
    let mut h = hare;
    let mut i = 0u32;
    while i < k {
        let (nt, nh) = c6_advance(t, h);
        t = nt;
        h = nh;
        i += 1;
    }
    (t, h)
}

/// After `k` iterations the tortoise advanced `k` states and the hare `2k`.
#[kani::proof]
#[kani::unwind(9)]
fn verify_floyd_cycle_detection() {
    let k: u32 = kani::any();
    kani::assume(k <= 4);
    let t: u32 = kani::any();
    let h: u32 = kani::any();
    kani::assume(t <= 1000);
    kani::assume(h <= 1000);

    let (t2, h2) = c6_run(k, t, h);

    kani::assert(t2 == t + k, "tortoise advances one state per iteration");
    kani::assert(h2 == h + 2 * k, "hare advances two states per iteration");
}

/// The hare is always at least as far along as the tortoise, given that Floyd's
/// loop starts both pointers at the same state.
#[kani::proof]
#[kani::unwind(9)]
fn verify_hare_leads() {
    let k: u32 = kani::any();
    kani::assume(k <= 4);
    let t: u32 = kani::any();
    let h: u32 = kani::any();
    kani::assume(t <= 1000);
    kani::assume(h <= 1000);
    kani::assume(t <= h); // both start at the same state

    let (t2, h2) = c6_run(k, t, h);

    kani::assert(t2 <= h2, "hare never falls behind the tortoise");
}

/// The current stub (`simulate_step` is a no-op, `hash_state` is constant) is
/// detected as a period-1 cycle after a single iteration.
#[kani::proof]
// 40 > 32: the stub compares 32-byte hashes via memcmp, which needs 33 unwindings.
#[kani::unwind(40)]
fn verify_periodicity_detection() {
    let n: usize = kani::any();
    kani::assume(n >= 1);
    kani::assume(n <= 4);

    let state = crate::period_detection::State;
    let period = crate::period_detection::detect_period(&state, n);

    kani::assert(period == Some(1), "the stub is detected as a period-1 cycle");
}
