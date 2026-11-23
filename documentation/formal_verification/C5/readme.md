# C5: Configuration Space Cardinality

> **Status:** Open. The counting argument below is a sketch; no checked artifact
> establishes $|\Theta| \ge 2^{1920}$.
> **Gap documents:** `gap1`–`gap4` in this directory
> **Last updated:** 2026-11-23

---

## Motivation

Kelvin's security is meant to rest on the difficulty of guessing or enumerating
valid orbital configurations. C5 tries to quantify the size of the configuration
space $\Theta$: a large space means Grover search needs $2^{H/2}$ operations,
where $H = \log_2|\Theta|$.

## Configuration space (as defined)

For $N=5$ bodies:

1. **Mass:** $m_i \in (0,1]$ M☉ → $2^{64}-1$ values each
2. **Position:** $r_i \in [-100,100]^3$ AU → $(201 \cdot 2^{64})^3$ values
3. **Velocity:** $v_i \in [-100,100]^3$ AU/yr → $(201 \cdot 2^{64})^3$ values
4. **Non-collision:** $r_i \ne r_j$ for $i \ne j$
5. **Minimum separation:** $|r_i - r_j| \ge$ min-separation
6. **Bound orbit:** specific energy below the ejection threshold
7. **Chaos horizon:** total steps ≥ min chaos steps

## Counting sketch

| Component | Per-axis bits | Per-body bits |
|-----------|---------------|---------------|
| Mass $m \in (0,1]$ | — | 64 |
| Position | $64 + \log_2 201 \approx 71.6$ | $\approx 214.9$ |
| Velocity | $64 + \log_2 201 \approx 71.6$ | $\approx 214.9$ |
| **Per body** | | **$\approx 493.9$** |

- Unconstrained, $N=5$: $5 \times 493.9 \approx 2469.5$ bits.
- Constraint reductions: non-collision ≤ 0.1 bits, min separation ≤ 5 bits,
  bound orbit ≤ 100 bits.

$$H \ge 2469.5 - 0.1 - 5 - 100 \approx 2364.4 \text{ bits} \ge 1920.$$

**Why this is not a proof:**

- "Constraint reduction ≤ $b$ bits" is asserted, not derived. A constraint
  reduction is a statement about the *measure* of the admissible set, which
  requires a distribution over configurations.
- $\log_2|\Theta|$ is not min-entropy. Min-entropy is
  $-\log_2 \max_x \Pr[X = x]$, which needs the sampling distribution.
- The admissible set is defined by a dynamical condition ("bound orbit",
  "chaos horizon"), so its size depends on the dynamics, not only on the box
  bounds.
- Reachability is not shown: the simulator may never visit most of $\Theta$.

## Gap status

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Cardinality $\lvert\Theta_5\rvert \ge 2^{1920}$ | [`gap1_cardinality_bound.md`](gap1_cardinality_bound.md) | **Open** |
| 2 | Min-entropy $H_{\min} \ge 1800$ bits | [`gap2_min_entropy.md`](gap2_min_entropy.md) | **Open** |
| 3 | Stability reduction ≈ 1 | [`gap3_stability_reduction.md`](gap3_stability_reduction.md) | **Empirical** (0/200 rejections, 2026-10-10) |
| 4 | Symbolic Kani config validation | [`gap4_symbolic_kani_config.md`](gap4_symbolic_kani_config.md) | **Open** |

## Kani harnesses

`proofs/kani/configuration_proofs.rs` contains sketches named
`verify_c5_non_positive_mass`, `verify_c5_identical_positions`, and
`verify_c5_too_few_bodies`. These are **templates**: not included by
`kani_harnesses.rs` and never compiled.

Even if integrated, they would check input validation, not cardinality. No Kani
harness says anything about $|\Theta|$ or $H_{\min}$, and none is planned until
the counting argument is made precise.

The branch-free `ct_select` / `ct_ne` harnesses in `C5_verify_side_channel.rs`
and `verify_side_channel.rs` belong to the separate side-channel item in
`TODO.md`; they prove functional correctness of the selectors, not constant-time
execution, and not anything about this configuration space.

## References

- [`../../../proofs/lean/src/KelvinProofs/FixedPoint.lean`](../../../proofs/lean/src/KelvinProofs/FixedPoint.lean)
- `kelvin-core/src/constants.rs` (physical bounds)
- [`../PROOF_INDEX.md`](../PROOF_INDEX.md)

## See also

- [PROOF_INDEX.md](../PROOF_INDEX.md)
- [C5 gap documents](.)
- [Lean proofs](../../../proofs/lean/README.md)
