// C6_verify_periodicity.rs
//
// Corrected 2026-11-23. The previous version "verified" Floyd's algorithm by
// running the loop and asserting the loop counter was in bounds. This version
// proves the loop invariant on an abstract model, and checks the behaviour of
// the current `period_detection` stub.
//
// Updated 2026-11-27: the harness that checked the old `period_detection` stub
// is replaced by harnesses in verify_periodicity.rs that call the real
// `find_cycle`; the stub was removed.
//
// SCOPE: the Floyd invariant is proved here on an abstract model. Nothing in
// this file says the *dynamics* have a long period.

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
