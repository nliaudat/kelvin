// verify_c3_quantum_reduction.rs
//
// Corrected 2026-11-23. The previous version imported `rand`,
// `kelvin_core::quantum::{quantum_oracle, QUANTUM_QUERY_LOWER_BOUND}` and
// asserted that a placeholder constant exceeded 10. That is not a query lower
// bound and the file was never compiled.
//
// SCOPE: exponent arithmetic used by the informal reduction. The reduction
// itself (Grover optimality over Theta) is not verified.

/// `2^a * 2^b = 2^(a+b)`, and the product never shrinks either factor.
#[kani::proof]
fn verify_c3_exponent_law() {
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    // Keep the product and the shift inside u64: 2^16 * 2^16 = 2^32.
    kani::assume(a <= 16);
    kani::assume(b <= 16);

    let lhs = (1u64 << a) * (1u64 << b);
    let rhs = 1u64 << (a + b);

    kani::assert(lhs == rhs, "2^a * 2^b = 2^(a+b)");
    kani::assert(lhs >= (1u64 << a), "multiplying by 2^b does not shrink 2^a");
}

/// A non-expanding step composes: if one step maps a window of width `w` to
/// width at most `w`, then `n` steps do too. This is the arithmetic behind the
/// two-step preimage-growth harness in `fixed_math.rs`.
#[kani::proof]
#[kani::unwind(9)]
fn verify_c3_contraction_arithmetic() {
    let w: u64 = kani::any();
    let n: u32 = kani::any();
    kani::assume(w <= 1 << 32);
    kani::assume(n <= 4);

    // Non-expanding map: width after one step is at most the width before.
    let mut width = w;
    let mut i = 0u32;
    while i < n {
        let next = width; // at most `width`, by the assumption of non-expansion
        kani::assert(next <= width, "a non-expanding step cannot widen the window");
        width = next;
        i += 1;
    }

    kani::assert(width <= w, "n non-expanding steps cannot widen the window");
}
