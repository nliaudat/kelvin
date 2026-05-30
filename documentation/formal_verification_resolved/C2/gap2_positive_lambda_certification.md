# C2 Gap 2: Positive λ Lower Bound `λ ≥ λ_min > 0` — Open

> **Status:** ⚠️ OPEN — Requires rigorous interval arithmetic on Benettin algorithm
> **Date:** 2026-05-30
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

For the standard 5-body Verlet configuration, the Lyapunov exponent `λ` satisfies `λ ≥ λ_min > 0` with `λ_min ≥ 0.5`. 

## 2. Required Approach

- Implement rigorous interval arithmetic wrapper around the shadow orbit computation
- Run Benettin algorithm with interval bounds on all arithmetic operations
- If the resulting interval is bounded away from zero, λ > 0 is certified

## 3. Current Evidence

Empirical: λ ≈ 0.693 for standard 5-body Verlet (`cargo run -p lyapunov_certification`)
Kani: `verify_perturbation_linear_regime` confirms O(δ) linear regime