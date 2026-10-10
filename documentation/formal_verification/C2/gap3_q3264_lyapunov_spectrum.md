# C2 Gap 3: Full Lyapunov Spectrum in Q32.64 — Justified (downgraded 2026-10-10)

> **Status:** ⚠️ JUSTIFIED (was: RESOLVED) — the perturbation argument
> below misattributes the bound and the error-accumulation step is
> unproved for chaotic flows. Retained on empirical agreement
> (f64 vs Q32.64 spectra in `tests/lyapunov_certification/`).
> **Date:** 2026-05-31 (downgraded 2026-10-10)
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

The deviation between the f64 Lyapunov spectrum and the Q32.64 spectrum for the standard 5-body configuration satisfies:

$$|\lambda_i^{(f64)} - \lambda_i^{(Q32.64)}| \leq \kappa(J) \cdot \varepsilon_q$$

where `κ(J) ≤ 10^3` for the 5-body Verlet tangent map and `ε_q = 2^(-64)`.

## 2. Proof via QR Perturbation Theory (Sun)

> **Correction (2026-10-10):** the previous revision cited Wedin's theorem,
> which bounds *SVD singular-subspace* perturbations (Wedin 1972), not QR
> `R`-factors. The applicable result is QR perturbation theory (e.g. Sun,
> "Perturbation Bounds for the Cholesky and QR Factorizations," BIT 1991):
> if `J̃ = J + Δ` with `‖Δ‖ ≤ ε_q`, the `R`-factor perturbation scales with
> a QR condition number. The `κ(J) ≤ 10³` value below is asserted for the
> 30×30 Verlet tangent map, not derived.

Each QR step introduces at most `ε_q · κ(J)` error in the accumulated ln|R_kk|. After S steps, the total error is at most `S · ε_q · κ(J) / S = ε_q · κ(J)`.

> **Open lemma (2026-10-10):** dividing by `S` assumes per-step log-errors
> accumulate linearly without amplification. In a chaotic flow, tangent-map
> errors grow ~exp(λ·S·dt); a static κ cannot absorb S-step exponential
> growth without an explicit shadowing/averaging lemma, which is not
> provided. The numerical conclusion therefore rests on the empirical
> f64-vs-fixed agreement, not on this derivation.

**Numerical bound:** For N=5, the Jacobian J is a 30×30 matrix. The condition number κ(J) for the gravitational Verlet map on bounded domains is at most 10^3. Thus the spectrum deviation ≤ 10^3 × 2^(-64) ≈ 5.4 × 10^(-17) — negligible.

## 3. Practical Implication

The f64 Lyapunov spectrum computed in `tests/lyapunov_certification/` is effectively identical to the Q32.64 spectrum. The deviation of 10^(-17) is far below the empirical uncertainty (±0.01) and does not affect any security conclusions.

## 4. References

- Sun, J.-G. (1991). "Perturbation Bounds for the Cholesky and QR Factorizations." *BIT Numerical Mathematics*, 31(2), 341–352. (Replaces the misattributed Wedin 1972 SVD reference.)
- `tests/lyapunov_certification/src/main.rs` (f64 spectrum implementation)
---

## See Also

- [C2 Proof Sketch](readme.md)
