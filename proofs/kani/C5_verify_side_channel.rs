// C5_verify_side_channel.rs
//
// Corrected 2026-11-23. The previous version claimed "constant-time execution
// verified" while only asserting `1024 > 0` and `cover(true)`. Timing cannot be
// checked by a model checker at all.
//
// SCOPE: this file proves the functional correctness of branch-free primitives.
// It does NOT prove that the pipeline runs in constant time; that requires
// measurement (dudect/ctgrind) plus a source audit. See C5 in
// documentation/formal_verification/PROOF_INDEX.md.

/// Branch-free select: returns `a` when `bit & 1 == 0`, `b` when it is 1.
/// No branch and no array index depends on `bit`.
fn c5_ct_select(bit: u8, a: i128, b: i128) -> i128 {
    let mask = 0i128.wrapping_sub((bit & 1) as i128);
    (a & !mask) | (b & mask)
}

/// Branch-free equality test: 0 when equal, 1 otherwise.
fn c5_ct_ne(x: u64, y: u64) -> u64 {
    let diff = x ^ y;
    ((diff | diff.wrapping_neg()) >> 63) as u64
}

#[kani::proof]
fn verify_ct_select_i128() {
    let bit: u8 = kani::any();
    kani::assume(bit <= 1);
    let a: i128 = kani::any();
    let b: i128 = kani::any();

    let out = c5_ct_select(bit, a, b);

    if bit == 0 {
        kani::assert(out == a, "bit = 0 selects the first operand");
    } else {
        kani::assert(out == b, "bit = 1 selects the second operand");
    }
}

#[kani::proof]
fn verify_ct_select_sign_bits() {
    let bit: u8 = kani::any();
    kani::assume(bit <= 1);
    let a: i128 = kani::any();
    let b: i128 = kani::any();

    // The mask is all-zero or all-one, so the result is one of the inputs
    // exactly, never a blend (which would indicate a data-dependent shift).
    let out = c5_ct_select(bit, a, b);
    kani::assert(out == a || out == b, "selection yields one input verbatim");
}

#[kani::proof]
fn verify_ct_is_zero_u64() {
    let x: u64 = kani::any();
    let y: u64 = kani::any();

    let ne = c5_ct_ne(x, y);

    if x == y {
        kani::assert(ne == 0, "equal inputs compare as zero");
    } else {
        kani::assert(ne == 1, "distinct inputs compare as one");
    }
}
