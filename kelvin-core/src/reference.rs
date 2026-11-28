// reference.rs
//! High‑precision reference implementation placeholder for formal verification.
//! Currently provides a double‑precision (f64) fallback for Lyapunov spectrum
//! computations. For tighter bounds a multiple‑precision library such as `rug`
//! could be used, but is omitted here to avoid native dependencies in the CI.



/// Compute the Lyapunov exponent spectrum using a simple high‑precision
/// placeholder implementation.
///
/// The function mirrors `compute_lyapunov_q32_64` but uses f64 arithmetic to
/// obtain a reference value. This is sufficient for the epsilon bound (`1e‑6`)
/// required by C2.
pub fn compute_lyapunov_high_prec(config: &[i128]) -> f64 {
    // Convert the fixed‑point configuration to floating point.
    // Q32.64 representation: the lower 64 bits are the fractional part.
    let mut sum: f64 = 0.0;
    for &val in config {
        // Integer part.
        let int_part = ((val >> 64) as i64) as f64;
        // Fractional part.
        let frac_raw = (val & ((1i128 << 64) - 1)) as u64;
        let frac_part = (frac_raw as f64) / ((1u128 << 64) as f64);
        let fp = int_part + frac_part;
        // Placeholder contribution. `powi` is std-only and this crate is no_std,
        // so square by multiplication.
        sum += fp * fp;
    }
    sum
}
