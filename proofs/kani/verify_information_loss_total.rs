// verify_information_loss_total.rs
//
// Corrected 2026-11-23. The previous version pulled `rand::thread_rng()` into a
// Kani proof and compared the placeholder implementation against itself, which
// verifies nothing and does not compile under Kani.
//
// SCOPE: arithmetic of the saturation model `L = min(S * k_step, H_max)` with
// symbolic `S` and `k_step`. Not an entropy statement.

const VILT_H_MAX: u64 = 3840;
const VILT_MAX_STEPS: u64 = 100_000;
const VILT_MAX_K: u64 = 128;

#[kani::proof]
fn verify_information_loss_saturating() {
    let steps: u64 = kani::any();
    let k: u64 = kani::any();
    kani::assume(steps <= VILT_MAX_STEPS);
    kani::assume(k <= VILT_MAX_K);

    let uncapped = steps.saturating_mul(k);
    let capped = uncapped.min(VILT_H_MAX);

    kani::assert(capped <= VILT_H_MAX, "capped loss never exceeds H_max");
    kani::assert(capped <= uncapped, "capping never increases the loss");
    kani::assert(
        capped == VILT_H_MAX || capped == uncapped,
        "capped loss is either the cap or the uncapped value",
    );
}

#[kani::proof]
fn verify_information_loss_cap_boundary() {
    let steps: u64 = kani::any();
    kani::assume(steps <= VILT_MAX_STEPS);

    // With k = 128 the cap is reached at S = 30; with k = 1 only at S = 3840.
    let at_30 = steps.saturating_mul(128).min(VILT_H_MAX);
    let at_1 = steps.saturating_mul(1).min(VILT_H_MAX);

    kani::assert(at_30 <= VILT_H_MAX, "cap holds for k = 128");
    kani::assert(at_1 <= VILT_H_MAX, "cap holds for k = 1");
    kani::assert(at_30 >= at_1, "a larger per-step loss saturates no later");
}
