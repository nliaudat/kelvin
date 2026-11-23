// verify_side_channel.rs
//
// Corrected 2026-11-23. The previous version wrote temp files, called
// `Instant::now()` twice and asserted the two durations were within 10
// microseconds. Kani cannot measure time, and the file was never compiled.
//
// SCOPE: functional correctness of 64-bit branch-free selection. Timing is not
// verified; that needs dudect/ctgrind plus a source audit.

/// Branch-free select over `u64`: `a` when `bit & 1 == 0`, `b` otherwise.
fn vsct_select_u64(bit: u64, a: u64, b: u64) -> u64 {
    let mask = 0u64.wrapping_sub(bit & 1);
    (a & !mask) | (b & mask)
}

#[kani::proof]
fn verify_ct_select_u64() {
    let bit: u64 = kani::any();
    kani::assume(bit <= 1);
    let a: u64 = kani::any();
    let b: u64 = kani::any();

    let out = vsct_select_u64(bit, a, b);

    if bit == 0 {
        kani::assert(out == a, "bit = 0 selects a");
    } else {
        kani::assert(out == b, "bit = 1 selects b");
    }
}

/// The result is always one of the two inputs, so the selection introduces no
/// arithmetic on secret bits.
#[kani::proof]
fn verify_ct_select_u64_is_exact() {
    let bit: u64 = kani::any();
    kani::assume(bit <= 1);
    let a: u64 = kani::any();
    let b: u64 = kani::any();

    let out = vsct_select_u64(bit, a, b);

    kani::assert(out == a || out == b, "selection returns an input verbatim");
    kani::assert(
        out <= a.max(b) && out >= a.min(b),
        "selection stays within the operand interval",
    );
}
