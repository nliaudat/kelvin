# C2 Gap 3: Full Lyapunov Spectrum in Q32.64 — Open

> **Status:** ⚠️ OPEN — Fixed-point QR decomposition not implemented
> **Date:** 2026-05-30
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

The full Lyapunov spectrum `{λ_i}_{i=1}^{30}` for N=5 can be estimated via QR decomposition of the tangent map. The deviation between f64 (current implementation) and Q32.64 is bounded by `κ(J) · ε_q`, where κ(J) is the Jacobian condition number.

## 2. Implementation Path

1. Implement Q32.64 vector dot product with proven error bounds (leverages L1)
2. Implement Gram-Schmidt QR with per-step rounding error bounds
3. Apply Wedin's theorem to bound spectrum deviation

## 3. Current State

- f64 spectrum computed in `tests/lyapunov_certification/src/main.rs` (empirical only)
- No fixed-point QR decomposition in the production crate
- Kaplan-Yorke dimension estimate ≈ 15.0 (preliminary, from f64 spectrum)