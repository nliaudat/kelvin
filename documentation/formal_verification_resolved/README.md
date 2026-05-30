# Formal Verification — Resolved Proofs

This directory contains formal mathematical proofs that close the gaps identified in `documentation/formal_verification_TODO.md`. Each subdirectory corresponds to a resolved conjecture gap.

## Structure

```
formal_verification_resolved/
├── README.md                               ← This file
├── C1/     (7 gaps — fully resolved)       ← Fixed-point information loss
├── C2/     (6 gaps — 2 resolved, 4 open)   ← Lyapunov exponent certification
├── C3/     (3 gaps — 1 resolved, 2 open)   ← Quantum hardness
├── C4/     (3 gaps — all resolved)         ← Keystream indistinguishability
└── C5/     (4 gaps — 3 resolved, 1 open)  ← Configuration space cardinality
```

---

## C1: Fixed-Point Information Loss — ✅ Fully Resolved (7/7)

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | `k_op ≥ 1` bit Shannon derivation | `C1/gap1_kop_shannon_bound.md` | ✅ RESOLVED |
| 2 | Approximate uniformity of `dist_cubed` | `C1/gap2_uniform_distribution.md` | ✅ JUSTIFIED |
| 3 | Cumulative loss saturation bound | `C1/gap3_saturation_bound.md` | ✅ RESOLVED |
| 4 | ε-bound `k_op ≥ 1 − ε` | `C1/gap4_epsilon_bound.md` | ✅ RESOLVED |
| 5 | Rounding error independence (Lemma A3) | `C1/gap5_error_independence.md` | ✅ JUSTIFIED |
| 6 | Hartley vs min-entropy relationship | `C1/gap6_hartley_min_entropy.md` | ✅ RESOLVED |
| 7 | Verlet double-computation effect | `C1/gap7_verlet_double_effect.md` | ✅ VERIFIED |

## C2: Lyapunov Exponent Certification — ⚠️ 2 Resolved, 4 Open

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | `|λ_disc − λ_cont| ≤ C·ε·S` error bound | `C2/gap1_discrete_lyapunov_bound.md` | ⚠️ Open |
| 2 | Positive λ lower bound `λ ≥ λ_min > 0` | `C2/gap2_positive_lambda_certification.md` | ⚠️ Open |
| 3 | Full Lyapunov spectrum in Q32.64 | `C2/gap3_q3264_lyapunov_spectrum.md` | ⚠️ Open |
| 4 | Kaplan-Yorke as discrete-state entropy bound | `C2/gap4_kaplan_yorke_entropy.md` | ✅ RESOLVED |
| 5 | Shadow orbit bias constant `C_bias` | `C2/gap5_shadow_orbit_bias.md` | ✅ RESOLVED |
| 6 | Lipschitz constant of λ w.r.t. parameters | `C2/gap6_lipschitz_lambda.md` | ⚠️ Open |

## C3: Quantum Hardness — ⚠️ 1 Resolved, 2 Open (Research)

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Extend Ambainis' adversary method | `C3/gap1_ambainis_adversary.md` | ⚠️ Open research |
| 2 | `Ω(2^{S·k/2})` quantum query lower bound | `C3/gap2_quantum_query_lower_bound.md` | ⚠️ Open research |
| 3 | Relate k to C1's ε-bound | `C3/gap3_c1_c3_link.md` | ✅ RESOLVED |

## C4: Keystream Indistinguishability — ✅ All Resolved (3/3)

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Game-based cryptographic reduction proof | `C4/gap1_game_based_reduction.md` | ✅ RESOLVED |
| 2 | Domain separation across all 6 modes | `C4/gap2_domain_separation_all_modes.md` | ✅ RESOLVED |
| 3 | Advantage bound derivation | `C4/gap3_advantage_bound.md` | ✅ RESOLVED |

## C5: Configuration Space — ✅ 3 Resolved, 1 Open

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Exact cardinality bound `|Θ_5| ≥ 2^{1920}` | `C5/gap1_cardinality_bound.md` | ✅ RESOLVED |
| 2 | Min-entropy `H_min ≥ 1800` bits | `C5/gap2_min_entropy.md` | ✅ RESOLVED |
| 3 | Stability reduction factor ≤ 2^{-100} | `C5/gap3_stability_reduction.md` | ⚠️ Open |
| 4 | Symbolic Kani config validation | `C5/gap4_symbolic_kani_config.md` | ✅ RESOLVED |

---

## Summary

| Conjecture | Total Gaps | Resolved | Open | Open Research |
|------------|-----------|----------|------|---------------|
| **C1** | 7 | 7 | 0 | 0 |
| **C2** | 6 | 2 | 4 | 0 |
| **C3** | 3 | 1 | 0 | 2 |
| **C4** | 3 | 3 | 0 | 0 |
| **C5** | 4 | 3 | 1 | 0 |
| **Total** | **23** | **16** | **5** | **2** |