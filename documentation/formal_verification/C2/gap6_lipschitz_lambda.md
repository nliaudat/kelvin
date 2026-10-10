# C2 Gap 6: Lipschitz Constant of λ w.r.t. Parameters — RE-OPENED (2026-10-10)

> **Status:** ❌ RE-OPENED — the numerical bound drops the theorem's own
> `1/δ` factor and the conclusion is false as stated (derivation below).
> **Date:** 2026-05-31 (re-opened 2026-10-10)
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

The Lyapunov exponent estimate `λ_disc` is Lipschitz with respect to simulation parameters `(G, ε, dt, masses)` with constant:

$$L_{\lambda} \leq \frac{1}{S \cdot dt} \cdot \left( L_{\ln} + \frac{1}{\delta} \cdot L_{div} \right)$$

where:

| Constant | Bound | Source |
|----------|-------|--------|
| `L_ln` (Lipschitz of ln computation) | ≤ 2 (bounded by slope of Padé on [1,10]) | verify_pade_ln_bound |
| `L_div` (Lipschitz of divergence measurement) | ≤ 2·|∇a|·S·dt | verify_perturbation_linear_regime |
| `δ` (initial perturbation) | 2^40 raw ≈ 6e-8 AU | System constant |

## 2. Derivation

λ = ln(d/δ) / (S·dt)

Differentiating with respect to any parameter p:
∂λ/∂p = (1/(S·dt)) · (1/(d/δ)) · (1/δ) · ∂d/∂p

The divergence d is Lipschitz with respect to all parameters because:
- The Verlet step is smoothly dependent on G, ε, dt, and masses
- A small change in any parameter produces an O(dt²·|∇a|) change in d per step

## 3. Numerical Bound — WITHDRAWN (2026-10-10)

The previous revision claimed `L_λ ≤ (1/31.2)·(2 + 0.1) ≈ 0.067`, i.e. "a
1% parameter change moves λ by at most 0.067%". Three independent errors:

1. **Dropped `1/δ`.** The theorem's own formula contains `(1/δ)·L_div`
   with `δ = 2⁻²⁴ ≈ 6e-8`, i.e. a ≈ 1.7×10⁷ multiplier the numerics replace
   with `0.1`. Restored, the bound is astronomically large (vacuous), not
   0.067.
2. **Wrong `S·dt`.** Uses 31.2 (= 2000 × 0.0156, the false dt); true
   `S·dt ≈ 2000 × 2⁻¹⁰ ≈ 1.95`, so the prefactor is `1/1.95 ≈ 0.51`.
3. **Absolute/relative confusion.** A Lipschitz constant of 0.067
   (λ-units per parameter-unit) × 0.01 parameter change gives ≤ 6.7e-4
   absolute (≈0.1% of λ ≈ 0.693), not "0.067%".

Beyond arithmetic, a single Lipschitz constant across heterogeneous
parameters (G, ε, dt, masses — different units) is dimensionally
incoherent. The *empirical* observation (λ stable across random configs)
stands but is unproved; re-closure requires either per-parameter bounds or
a measurement-backed justification. Until then this gap is open, and no
robustness figure should be quoted from it.

## 4. References

- verify_pade_ln_bound (Kani: Padé monotonicity)
- verify_perturbation_linear_regime (Kani: O(δ) divergence)
- kelvin-kdf/src/lyapunov.rs (λ computation)
---

## See Also

- [C2 Proof Sketch](readme.md)
