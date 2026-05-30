# C2 Gap 6: Lipschitz Constant of λ w.r.t. Parameters — Open

> **Status:** ⚠️ OPEN — Requires perturbation analysis of Benettin algorithm
> **Date:** 2026-05-30
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

The constants `C_pade` and `C_div` in the error budget `|λ_disc − λ_cont|` are bounded by the Lipschitz constant `L_λ` of the Lyapunov exponent estimate:

$$C_{pade} \le L_λ \cdot \frac{\partial \lambda}{\partial (\ln \text{ ratio})} \quad C_{div} \le L_λ \cdot \frac{\partial \lambda}{\partial (\text{time})}$$

## 2. Known Bounds

| Parameter | Influence on λ | Bound |
|-----------|---------------|-------|
| Perturbation δ | Weak if in linear regime | Verified by `verify_perturbation_linear_regime` |
| Softening ε | Moderate | Not bounded formally |
| Mass ratio | Strong (near-integrable regimes) | Bounded by avoiding extreme ratios |

## 3. Required Approach

Formally the most challenging C2 gap. Requires differentiating the Benettin algorithm output with respect to input parameters and bounding the resulting expression's norm.