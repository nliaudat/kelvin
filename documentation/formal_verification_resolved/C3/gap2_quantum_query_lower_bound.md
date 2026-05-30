# C3 Gap 2: `Ω(2^{S·k/2})` Quantum Query Lower Bound — Open Research

> **Status:** ⚠️ OPEN RESEARCH — Depends on Gap 1
> **Date:** 2026-05-30
> **Conjecture:** C3 — Sequential Simulation Hardness Against Quantum Adversaries

## 1. Theorem Statement

Any quantum algorithm inverting `Φ^S` requires `Ω(2^{min(S·k/2, H_max−log₂(A))/2})` queries.

## 2. Reduction Strategy

| Step | Argument | Status |
|------|----------|--------|
| 1 | Φ is dissipative (k bits lost per step) | ✅ C1 proven |
| 2 | Ambainis adversary lower bound for dissipative functions | ⚠️ Gap 1 |
| 3 | Compose over S steps | ⚠️ Depends on Gap 1 + 2 |

## 3. Current Best Known

Classical brute force: requires `O(2^{S·k})` operations. Grover gives `O(2^{S·k/2})`. The claim is that no quantum algorithm can do better than Grover.