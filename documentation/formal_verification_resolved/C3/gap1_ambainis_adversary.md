# C3 Gap 1: Extend Ambainis' Adversary Method — Open Research

> **Status:** ⚠️ OPEN RESEARCH — Beyond Kani's capabilities
> **Date:** 2026-05-30
> **Conjecture:** C3 — Sequential Simulation Hardness Against Quantum Adversaries

## 1. Problem Statement

Extend Ambainis' quantum adversary method to finite-state functions `Φ: X → X` with per-step information loss `k > 0`. The adversary method gives lower bounds on quantum query complexity — when a function is "dissipative" (loses information each step), the quantum adversary should have an advantage proportional to `2^{S·k/2}` for `Φ^S`.

## 2. Required Result

$$\Omega\left(2^{\frac{S·k}{2}}\right)$$ queries to invert `Φ^S`.

## 3. Current State

- Classical preimage properties verified by Kani (C3 harnesses)
- Quantum lower bound: published result would be a significant contribution
- No existing theorem covers sequential dissipative finite-state functions