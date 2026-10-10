# C3 Gap 3: Relating k to C1's ε-Bound — Resolved

> **Status:** ✅ RESOLVED
> **Date:** 2026-05-30
> **Conjecture:** C3 — Sequential Simulation Hardness Against Quantum Adversaries
> **Dependencies:** C1 Gaps 1, 4, 5

## 1. Theorem Statement

The per-step information loss `k` in the quantum query lower bound `Ω(2^{S·k/2})` is:

$$k = k_{step} = 2N(N-1) \times k_{op} \ge 2N(N-1) \times (1 - \varepsilon)$$

where `ε ≤ 1/log₂(MAX/MIN) ≈ 0.0197` (from C1 Gap 4).

## 2. Derivation

| C1 Result | Value | Source |
|-----------|-------|--------|
| k_op (per operation) | ≥ 0.980 bits | Gap 4: ε-bound |
| k_ops/step (Verlet, N=5) | 40 operations | Gap 7: Verlet counting |
| k_step (per step) | ≥ 39.2 bits | k_step = 40 × 0.980 |
| Saturation cap | 3840 bits (raw storage; ≤ 2880 usable — see C1 scope notes) | Gap 3: state space limit |

> **Scope note (2026-10-10):** the Grover exponent `S·k/2` must be capped
> by the actual search space — `min(S·k, H_max)` — otherwise S = 10⁶ steps
> implies a 2^40M search space, ~10⁴× the code's state. The cap, not the
> per-step rate, governs operational bounds (see C4/readme scope note).

## 3. References

- C1 Gap 1 (k_op ≥ 1 bit), Gap 4 (ε-bound), Gap 5 (independence), Gap 7 (Verlet counting)
---

## See Also

- [C3 Proof Sketch](readme.md)
