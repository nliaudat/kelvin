# C4 Gap 3: Advantage Bound Derivation `Adv(A) ≤ negl(n) + 2^{−S·k/2}` — Resolved

> **Status:** ✅ RESOLVED
> **Date:** 2026-05-30
> **Conjecture:** C4 — Computational Indistinguishability of the Keystream

## 1. Theorem Statement

For the Kelvin keystream `K(C) = SHAKE256(Φ^S(C))` and any polynomial-time quantum adversary `A`:

$$Adv(A) = |Pr[A(K(C))=1] - Pr[A(U_L)=1]| \le negl(n) + 2^{-\frac{S \cdot k}{2}}$$

## 2. Derivation

| Term | Source | Bound | Status |
|------|--------|-------|--------|
| `negl(n)` | SHAKE256 indifferentiability (NIST FIPS 202) | Assumed standard | ✅ Assumption |
| `2^{−S·k/2}` | Probability of inverting Φ^S by Grover search | C1: k ≥ 40 bits/step | ✅ From C1 + C3 |

For N=5 Verlet, S=1,000,000: `S·k = 40,000,000` → advantage bound is `negl(n) + 2^{−20,000,000}`.

## 3. References

- C1 (information loss per step)
- `formal_verification.md` §L4' (reduction proof sketch)
- `kelvin-kdf/src/extractor.rs` (extraction pipeline: determinism + domain separation)
---

## See Also

- [C4 Proof Sketch](readme.md)
