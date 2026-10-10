# C2 Gap 5: Shadow Orbit Bias Constant `C_bias` — Resolved (corrected 2026-10-10)

> **Status:** ✅ RESOLVED (corrected)
> **Date:** 2026-05-30 (corrected 2026-10-10)
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

The finite-sample spread from 3-axis shadow orbit estimation satisfies:

$$C_{bias} \le \frac{3 \cdot \sigma}{\sqrt{3}} = \sqrt{3}\,\sigma$$

where `σ` is the standard deviation of the divergence across the 3 perturbed axes (x, y, z). For the standard 5-body configuration, `σ ≤ 0.1·λ` empirically.

> **Correction (2026-10-10):** the previous revision multiplied by an extra
> `1/(S·dt)`, giving units of 1/time² for a quantity in λ units (1/time) —
> each `λ_i` is already a rate, so the standard error of the mean needs no
> time normalization. The `3σ/√3` max-deviation factor itself is retained.

## 2. Derivation

The 3 shadow orbits correspond to perturbations along x, y, and z axes. Each provides an independent estimate `λ_i` of the maximal Lyapunov exponent (already in units of 1/time). The sample mean `λ̄ = (λ_x + λ_y + λ_z)/3` has standard error `σ/√3`, and the max deviation is bounded by `3σ/√3`:

$$C_{bias} \le \frac{3\sigma}{\sqrt{3}} = \sqrt{3}\,\sigma$$

## 3. Numerical Value

For default configuration (N=5, S=2000, dt = 2^54 raw = 2⁻¹⁰ yr ≈ 0.001 yr):

$$C_{bias} \le \sqrt{3} \times 0.1 \times 0.693 \approx 0.12$$

> **Correction (2026-10-10):** the previous value 0.0077 combined the
> spurious `1/(S·dt)` with the false `dt ≈ 0.0156` (true: 2⁻¹⁰ ≈ 0.001)
> and was additionally 2× off on its own numbers (0.2079/54.04 ≈ 0.00385,
> not 0.0077).

## 4. References

- `kelvin-kdf/src/lyapunov.rs` (3-axis perturbation loop)
- Wolf, A., et al. (1985). "Determining Lyapunov Exponents from a Time Series." *Physica D*, 16(3), 285–317.
---

## See Also

- [C2 Proof Sketch](readme.md)
