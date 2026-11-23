// verify_lyapunov_spectrum.rs
//
// Corrected 2026-11-23. The previous version called `kelvin_core::lyapunov::*`
// and `kelvin_core::reference::compute_lyapunov_high_prec` with `rand`, and
// asserted an error bound of 1e-6 that no artifact derives.
//
// SCOPE: quantization and budget arithmetic for the estimator. No spectrum,
// exponent or error bound is certified here (C2 is empirical / open).

/// The quantization step (`2^-64`, i.e. one raw unit) divided by a time of at
/// least `2^54` raw units is zero: the estimator cannot resolve it.
#[kani::proof]
fn verify_lyapunov_quantization() {
    let time_raw: i128 = kani::any();
    kani::assume(time_raw >= 1 << 54);
    kani::assume(time_raw <= 1_000_000 * (1 << 54));

    let eps_q: i128 = 1;
    let eps_lambda = eps_q / time_raw;

    kani::assert(eps_lambda == 0, "quantization contributes nothing in raw units");
}

/// Shadow-orbit bias budget: `bias * eps_q <= S * dt` for `bias = 100`,
/// `eps_q = 1`, `S >= 1000` and `dt = 2^54`.
#[kani::proof]
fn verify_shadow_orbit_bias() {
    let s_steps: i128 = kani::any();
    kani::assume(s_steps >= 1000);
    kani::assume(s_steps <= 1_000_000);

    let dt_raw: i128 = 1 << 54;
    let denominator = s_steps * dt_raw;
    let bias_constant: i128 = 100;

    kani::assert(denominator > 0, "shadow-orbit denominator is positive");
    kani::assert(
        bias_constant <= denominator,
        "bias budget is smaller than the accumulated time",
    );
}

/// The perturbation used by the shadow-orbit method fits in Q32.64 and is
/// strictly positive.
#[kani::proof]
fn verify_perturbation_in_range() {
    let delta_raw: i128 = 1 << 40;
    kani::assert(delta_raw > 0, "perturbation is positive");
    kani::assert(delta_raw < (1 << 64), "perturbation is below one raw unit of 1.0");
}
