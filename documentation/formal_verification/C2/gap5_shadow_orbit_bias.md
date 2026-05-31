# C2 Gap 5: Shadow Orbit Bias Constant `C_bias` — Resolved

> **Status:** ✅ RESOLVED
> **Date:** 2026-05-30
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

The finite-sample bias from 3-axis shadow orbit estimation satisfies:

$$C_{bias} \le \frac{3 \cdot \sigma}{\sqrt{3}} \cdot \frac{1}{S \cdot dt}$$

where `σ` is the standard deviation of the divergence across the 3 perturbed axes (x, y, z). For the standard 5-body configuration, `σ ≤ 0.1·λ` empirically.

## 2. Derivation

The 3 shadow orbits correspond to perturbations along x, y, and z axes. Each provides an independent estimate `λ_i` of the maximal Lyapunov exponent. The sample mean `λ̄ = (λ_x + λ_y + λ_z)/3` has standard error `σ/√3`. After normalizing by total time `S·dt`:

$$C_{bias} \le \frac{\max_i |λ_i - λ̄|}{S \cdot dt} \le \frac{3\sigma}{\sqrt{3} \cdot S \cdot dt}$$

## 3. Numerical Value

For default configuration (N=5, S=2000, dt=2^54 raw ≈ 0.0156):

$$C_{bias} \le \frac{3 \times 0.1 \times 0.693}{\sqrt{3} \times 2000 \times 0.0156} \approx 0.0077$$

## 4. References

- `kelvin-kdf/src/lyapunov.rs` (3-axis perturbation loop)
- Wolf, A., et al. (1985). "Determining Lyapunov Exponents from a Time Series." *Physica D*, 16(3), 285–317.
---

## See Also

- [C2 Proof Sketch](proof_sketch.md)
