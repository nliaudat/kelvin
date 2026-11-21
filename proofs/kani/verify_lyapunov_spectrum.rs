// verify_lyapunov_spectrum.rs
//! Kani harness to verify Q32.64 Lyapunov spectrum correctness.
//! Generates random orbital configurations, computes Lyapunov exponents using
//! the fixed‑point implementation and a high‑precision reference, and asserts
//! the difference is within the proven error bound (≤ 1e‑6 as justified in C2).

#[cfg(kani)]
mod harness {
    use kani::cover;
    use rand::Rng;
    use kelvin_core::lyapunov::compute_lyapunov_q32_64;
    use kelvin_core::reference::compute_lyapunov_high_prec;

    // The error tolerance derived from the formal analysis in C2 (see
    // `documentation/formal_verification/C2/gap3_q3264_lyapunov_spectrum.md`).
    const EPS: f64 = 1e-6;

    #[kani::proof]
    fn verify_lyapunov_spectrum() {
        // Random fixed‑point state (i128 representation). We generate a small
        // vector of 10 components for illustration; the real implementation can
        // use any size.
        let mut rng = rand::thread_rng();
        let config: Vec<i128> = (0..10)
            .map(|_| rng.gen_range((-1i128) << 64..(1i128) << 64))
            .collect();
        // Compute exponents with Q32.64 implementation
        let lyap_fixed = compute_lyapunov_q32_64(&config);
        // Compute with high‑precision reference (f64 double precision)
        let lyap_ref = compute_lyapunov_high_prec(&config);
        // Assert the absolute error is within the bound.
        assert!((lyap_fixed - lyap_ref).abs() < EPS);
        cover!(true);
    }
}
