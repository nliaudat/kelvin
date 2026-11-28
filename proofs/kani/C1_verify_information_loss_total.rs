// C1_verify_information_loss_total.rs
//
// Corrected 2026-11-23. The previous version was never compiled and asserted
// constants chosen inside the harness (`96 * 40 == 3840`). This version proves
// arithmetic properties of the C1 *saturation model*
//
//     L(S) = min(S * k_step, H_max)
//
// for a symbolic step count.
//
// SCOPE: this is a statement about the model, not about the simulation. It does
// NOT show that the simulation loses `k_step` bits per step; that claim has no
// machine-checked artifact (see C1/gap3_saturation_bound.md).

const C1_H_MAX: u64 = 3840; // 6 * 5 * 128 bits for N = 5
const C1_K_STEP: u64 = 40;  // claimed bits lost per step for N = 5

fn c1_saturating_loss(steps: u64, k_step: u64, h_max: u64) -> u64 {
    steps.saturating_mul(k_step).min(h_max)
}

#[kani::proof]
fn verify_information_loss_total() {
    let steps: u64 = kani::any();
    kani::assume(steps <= 1_000_000);

    let loss = c1_saturating_loss(steps, C1_K_STEP, C1_H_MAX);

    kani::assert(loss <= C1_H_MAX, "loss never exceeds H_max");
    kani::assert(
        loss <= steps.saturating_mul(C1_K_STEP),
        "loss never exceeds S * k_step",
    );
}

#[kani::proof]
fn verify_information_loss_monotone() {
    let a: u64 = kani::any();
    let b: u64 = kani::any();
    kani::assume(a <= b);
    kani::assume(b <= 1_000_000);

    let la = c1_saturating_loss(a, C1_K_STEP, C1_H_MAX);
    let lb = c1_saturating_loss(b, C1_K_STEP, C1_H_MAX);

    kani::assert(la <= lb, "loss is monotone in the step count");
}

#[kani::proof]
fn verify_saturation_point() {
    kani::assert(
        c1_saturating_loss(96, C1_K_STEP, C1_H_MAX) == C1_H_MAX,
        "96 steps saturate the model",
    );
    kani::assert(
        c1_saturating_loss(95, C1_K_STEP, C1_H_MAX) < C1_H_MAX,
        "95 steps do not saturate the model",
    );
    kani::assert(
        c1_saturating_loss(1_000_000, C1_K_STEP, C1_H_MAX) == C1_H_MAX,
        "the model stays saturated beyond the crossing point",
    );
}
