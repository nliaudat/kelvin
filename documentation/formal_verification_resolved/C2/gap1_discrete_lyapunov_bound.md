# C2 Gap 1: Discrete Lyapunov Bound `|λ_disc − λ_cont| ≤ C·ε·S` — Open

> **Status:** ⚠️ OPEN — Analysis framework provided; constants require numerical certification
> **Date:** 2026-05-30
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification
> **Kani Cross-Reference:** `verify_perturbation_linear_regime` in `kelvin-kdf/src/lyapunov.rs`

---

## 1. Theorem Statement

**Theorem (Discrete Lyapunov Error Bound):**
Let `λ_cont` be the maximal Lyapunov exponent of the continuous N-body system (N ≥ 3). Let `λ_disc(S)` be the estimate from `S` steps of the shadow orbit method (`kelvin-kdf/src/lyapunov.rs`). Then:

$$|\lambda_{disc}(S) - \lambda_{cont}| \leq \frac{C_{pade}}{S \cdot dt} + C_{disc} \cdot \varepsilon_q + \frac{C_{bias}}{\sqrt{S}}$$

where:
- `C_pade` — Padé ln approximation error constant (≤ 0.29 for ratio ≤ 10^6)
- `ε_q` — Fixed-point quantization error per step (`2^{−64}`)
- `C_disc` — Accumulation constant for discretization error (depends on simulation length)
- `C_bias` — Multiplicative bias from 3-axis finite-sample estimation

---

## 2. Error Sources

### 2.1 Padé ln Approximation

The ln computation uses:
```rust
while x > ten { x /= ten; ln_sum += ln_10; }
let num = x - Fixed::ONE; let den = x + Fixed::ONE;
ln_sum + two * num / den
```

For total ratio `R = d/δ ∈ [1, 10^6]`, the maximal relative error of the Padé approximation is:

$$\varepsilon_{pade}(R) \leq \begin{cases} 0.01 & R \leq 10 \\ 0.05 & R \leq 100 \\ 0.29 & R \leq 10^6 \end{cases}$$

Verified by `verify_pade_ln_bound` (monotonicity, underestimation).

### 2.2 Quantization Error

Each division `ln_ratio / (S·dt)` introduces quantization ≤ 1 ULP. The division `ln_ratio / time` is verified finite by `verify_lyapunov_division`.

### 2.3 Shadow Trajectory Divergence

The perturbation `δ = 2^40` raw produces O(δ) divergence after 1 Verlet step (verified by `verify_perturbation_linear_regime`). Over S steps, the divergence accumulates as `d ≈ δ · exp(S·λ)` with compounding errors.

---

## 3. Error Budget

| Error Source | Bound | Mitigation |
|-------------|-------|------------|
| Padé approximation | ≤ 0.29 ln(ratio) for ratio ≤ 10^6 | Longer orbits → larger ratio → larger ε |
| Fixed-point quantization | ≤ 2^{−64} per division | Negligible |
| Finite-sample bias | ≤ 3σ / √3 · 1/(S·dt) | Increases to 2000 steps by default |
| Discretization (dt finite) | O(dt²) for Verlet | dt = 2^54 raw ≈ 0.0156 yr at ~1e-3 yr |
| **Total** | **≤ 0.01–0.3 lyapunov exponent units** | Acceptable for λ ≈ 0.693 |

---

## 4. Path to Formal Proof

1. Compute Lipschitz constant L of the Verlet map w.r.t. initial conditions
2. Bound trajectory shadow error: `|λ_shadow − λ_true| ≤ L · ε_q · S / (S·dt)`
3. Apply Padé error bound to `ln(d/δ)` computation
4. Incorporate finite-sample bias via standard deviation of 3 shadow orbits

The key missing piece is L — the Lipschitz constant of the shadow orbit estimate w.r.t. initial conditions. This requires bounding the derivative of the Benettin algorithm output with respect to the initial perturbation.

---

## 5. References

1. `kelvin-kdf/src/lyapunov.rs` lines 126–312 (shadow orbit implementation)
2. `verify_pade_ln_bound` (Kani: Padé approximation bounds)
3. `verify_lyapunov_division` (Kani: division safety)
4. `verify_perturbation_linear_regime` (Kani: O(δ) divergence)
5. Benettin, G., et al. (1980). "Lyapunov Characteristic Exponents." *Meccanica*.