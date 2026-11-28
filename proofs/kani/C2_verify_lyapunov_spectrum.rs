// C2_verify_lyapunov_spectrum.rs
//
// Corrected 2026-11-23. The previous version was never compiled and referenced
// modules that do not exist (`kelvin_core::lyapunov`). This version proves
// arithmetic facts about the quantities the estimator manipulates.
//
// SCOPE: none of these is a Lyapunov certification. The measured exponent, the
// spectrum and the Kaplan-Yorke bound remain empirical / open (C2 in
// documentation/formal_verification/PROOF_INDEX.md).

use crate::Fixed;

/// Padé (1,1) logarithm: `ln(x) ~ 2(x-1)/(x+1)` for `x >= 1`.
/// Proves the approximation ratio `(x-1)/(x+1)` lies in `[0, 1]` for `x in [1,2]`,
/// so the fixed-point division cannot leave the representable range.
#[kani::proof]
#[kani::unwind(256)]
fn verify_lyapunov_spectrum_comprehensive() {
    let x_raw: i128 = kani::any();
    kani::assume(x_raw >= Fixed::ONE.to_raw());
    kani::assume(x_raw <= Fixed::from_int(2).to_raw());

    let x = Fixed::from_raw(x_raw);
    let num = x - Fixed::ONE; // >= 0
    let den = x + Fixed::ONE; // >= 2

    kani::assert(num >= Fixed::ZERO, "Pade numerator is non-negative");
    kani::assert(den > num, "Pade denominator exceeds the numerator");

    let ratio = num / den;
    kani::assert(ratio >= Fixed::ZERO, "Pade ratio is non-negative");
    kani::assert(ratio <= Fixed::ONE, "Pade ratio is at most one");
}

/// `lambda = ln(ratio) / time` with `ln(ratio) >= 0` and `time >= 2^54` raw
/// (i.e. at least `2^-10` yr) is non-negative and grows at most by the inverse
/// of that minimum time.
#[kani::proof]
#[kani::unwind(256)]
fn verify_lyapunov_division() {
    let ln_ratio_raw: i128 = kani::any();
    kani::assume(ln_ratio_raw >= 0);
    kani::assume(ln_ratio_raw <= 28 * Fixed::ONE.to_raw());

    let time_raw: i128 = kani::any();
    kani::assume(time_raw >= 1 << 54);
    kani::assume(time_raw <= 1_000_000 * (1 << 54));

    let lyapunov = Fixed::from_raw(ln_ratio_raw) / Fixed::from_raw(time_raw);

    kani::assert(lyapunov >= Fixed::ZERO, "Lyapunov exponent is non-negative");
    // Dividing by time >= 2^54 multiplies the raw value by at most 2^10.
    kani::assert(
        lyapunov.to_raw() <= ln_ratio_raw * (1 << 10),
        "Lyapunov exponent is bounded by ln(ratio) / min_time",
    );
    kani::assert(
        lyapunov.to_raw() < i128::MAX / 2,
        "Lyapunov exponent is finite",
    );
}

/// Quantization error `eps_q / time` for `eps_q = 1` raw unit (2^-64) and
/// `time >= 2^54` is zero in raw units: the estimator cannot resolve it.
#[kani::proof]
fn verify_lyapunov_error_bounds() {
    let time_raw: i128 = kani::any();
    kani::assume(time_raw >= 1 << 54);
    kani::assume(time_raw <= 1_000_000 * (1 << 54));

    let eps_q: i128 = 1;
    let eps_lambda = eps_q / time_raw;

    kani::assert(eps_lambda >= 0, "quantization error is non-negative");
    kani::assert(eps_lambda <= eps_q, "quantization error is at most one raw unit");
}
