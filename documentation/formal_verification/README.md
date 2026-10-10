# Formal Verification — Resolved Proofs

This directory contains formal mathematical proofs for the conjectures documented in `documentation/formal_verification.md`. Each subdirectory corresponds to a resolved conjecture gap.

```
formal_verification/
├── README.md
├── C1/     (7 gaps — fully resolved)       ← Fixed-point information loss
├── C2/     (6 gaps — fully resolved)       ← Lyapunov exponent certification
├── C3/     (3 gaps — fully resolved)       ← Quantum hardness
├── C4/     (3 gaps — fully resolved)       ← Keystream indistinguishability
└── C5/     (4 gaps — fully resolved)       ← Configuration space cardinality
```

## Summary

| Conjecture | Gaps | Closed | Proof Sketch |
|------------|------|----------|-------------|
| **C1**: Information Loss | 7 | **7** ✅ (5 proved, 2 justified*) | [`readme.md`](C1/readme.md) |
| **C2**: Lyapunov Certification | 6 | **5** ✅ + 1 ⚠️ justified (gap3) | [`readme.md`](C2/readme.md) |
| **C3**: Quantum Hardness | 3 | **3** ✅ | [`readme.md`](C3/readme.md) |
| **C4**: Keystream Indistinguishability | 3 | **3** ✅ | [`readme.md`](C4/readme.md) |
| **C5**: Configuration Space | 4 | **3** ✅ + 1 ❌ reopened (gap3) | [`readme.md`](C5/readme.md) |
| **Total** | **23** | **22 ✅/⚠️ + 1 ❌ reopened** (19 proved, 3 justified, C5 gap3 open) | |

\* C1 gap2 (uniformity, conditional on unproven ergodicity), C1 gap5
(rounding-error independence), and C2 gap3 (QR perturbation averaging
lemma) are closed by argument + empirical cross-check, not by proof.
See `C1/readme.md:66,69` and `C2/readme.md:42`.

> **Scope note:** all worked numerics in C1–C5 (state entropy 3840 bits,
> k_step ≥ 40 bits/step, T_sat ≈ 96, λ ≈ 0.693/yr, D_KY estimates) are for
> the 5-body analysis configuration. Current `standard`/`paranoid` keygen
> emits 8 bodies, which has a different spectrum and mixing time — the
> figures do not transfer automatically and should be re-derived for N=8
> before quoting.

22 of 23 gaps are closed: 19 with proofs, 3 (C1 gap2, gap5; C2 gap3) by justified
argument with empirical validation. C5 gap3 is re-opened (inverted argument).
