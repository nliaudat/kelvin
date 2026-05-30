# Formal Verification — Resolved Proofs

This directory contains formal mathematical proofs that close the gaps identified in `documentation/formal_verification_TODO.md`. Each subdirectory corresponds to a resolved conjecture gap.

## Structure

```
formal_verification_resolved/
├── README.md                       ← This file
└── C1/
    ├── gap1_kop_shannon_bound.md       ← C1 Gap 1: k_op ≥ 1 bit per operation
    ├── gap2_uniform_distribution.md    ← C1 Gap 2: Approximate uniformity of dist_cubed
    ├── gap3_saturation_bound.md        ← C1 Gap 3: Cumulative loss S×k_step saturation
    ├── gap4_epsilon_bound.md           ← C1 Gap 4: ε-bound k_op ≥ 1 − ε
    ├── gap5_error_independence.md      ← C1 Gap 5: Rounding error independence (Lemma A3)
    ├── gap6_hartley_min_entropy.md     ← C1 Gap 6: Hartley vs min-entropy relationship
    └── gap7_verlet_double_effect.md    ← C1 Gap 7: Verlet double-computation effect
```

## Resolved Gaps

| Conjecture | Gap | File | Status |
|------------|-----|------|--------|
| **C1** | Gap 1: `k_op ≥ 1` bit Shannon derivation | [`C1/gap1_kop_shannon_bound.md`](C1/gap1_kop_shannon_bound.md) | ✅ RESOLVED |
| **C1** | Gap 2: Approximate uniformity of `dist_cubed` | [`C1/gap2_uniform_distribution.md`](C1/gap2_uniform_distribution.md) | ✅ JUSTIFIED |
| **C1** | Gap 3: Cumulative loss saturation bound | [`C1/gap3_saturation_bound.md`](C1/gap3_saturation_bound.md) | ✅ RESOLVED |
| **C1** | Gap 4: ε-bound `k_op ≥ 1 − ε` | [`C1/gap4_epsilon_bound.md`](C1/gap4_epsilon_bound.md) | ✅ RESOLVED |
| **C1** | Gap 5: Rounding error independence (Lemma A3) | [`C1/gap5_error_independence.md`](C1/gap5_error_independence.md) | ✅ JUSTIFIED |
| **C1** | Gap 6: Hartley vs min-entropy relationship | [`C1/gap6_hartley_min_entropy.md`](C1/gap6_hartley_min_entropy.md) | ✅ RESOLVED |
| **C1** | Gap 7: Verlet double-computation effect | [`C1/gap7_verlet_double_effect.md`](C1/gap7_verlet_double_effect.md) | ✅ VERIFIED |