# C5: Configuration Space Cardinality — Proof Sketch

> **Results:** `proofs/kani/results/c5_validation.log`
> **Gap documents:** [`gap{1..4}_*.md`](.)

---

## Motivation

The security of the Kelvin cryptosystem ultimately depends on the inability of an attacker to guess or enumerate valid orbital configurations. C5 quantifies the size of the configuration space $\Theta$.

A large configuration space ensures that even a quantum adversary using Grover's algorithm requires $2^{H/2}$ operations for unstructured search, where $H = \log_2|\Theta|$ is the min-entropy.

## Configuration Space Definition

For $N = 5$ bodies:

1. **Mass**: $m\_i \in (0, 1]$ M☉ → $2^{64} - 1$ values each
2. **Position**: $r\_i \in [-100, 100]^3$ AU → $(201 \times 2^{64})^3$ values
3. **Velocity**: $v\_i \in [-100, 100]^3$ AU/yr → $(201 \times 2^{64})^3$ values
4. **Non-collision**: $r\_i \neq r\_j$ for $i \neq j$
5. **Minimum separation**: $|r\_i - r\_j| \ge \text{min-separation}$
6. **Bound orbit**: Specific energy $< \text{ejection-threshold}$
7. **Chaos horizon**: $\text{total-steps} \ge \text{min-chaos-steps}$

## Analytical Bound Derivation

Each body contributes 7 bounded Q32.64 values. For N=5:

| Component | Per-axis bits | Per-body bits |
|-----------|---------------|---------------|
| Mass $m \in (0, 1]$ | — | 64 |
| Position $p \in [-100, 100]^3$ | $64 + \log_2(201) \approx 71.6$ | $3 \times 71.6 \approx 214.9$ |
| Velocity $v \in [-100, 100]^3$ | $64 + \log_2(201) \approx 71.6$ | $3 \times 71.6 \approx 214.9$ |
| **Per body total** | | **$\approx 493.9$** |

- Unconstrained for N=5: $5 \times 493.9 \approx 2469.5$ bits

Constraint reductions (from gap1):

| Constraint | Reduction (bits) |
|------------|-----------------|
| Non-collision ($r_i \neq r_j$) | $\leq 0.1$ |
| Min separation ($\|r_i - r_j\| \ge$ threshold) | $\leq 5$ |
| Bound orbit ($E <$ ejection-threshold) | $\leq 100$ |

$$H \ge 2469.5 - 0.1 - 5 - 100 \approx 2364.4 \text{ bits} \ge 1920$$

The final bound $H \ge 2364.4$ bits is well above the $2^{1920}$ claim.

## All Gaps Resolved

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Cardinality $\lvert\Theta_5\rvert \ge 2^{1920}$ | [`gap1_cardinality_bound`](gap1_cardinality_bound.md) | ✅ RESOLVED |
| 2 | Min-entropy $H\_{\min} \ge 1800$ bits | [`gap2_min_entropy`](gap2_min_entropy.md) | ✅ RESOLVED |
| 3 | Stability reduction $\le 2^{-100}$ | [`gap3_stability_reduction`](gap3_stability_reduction.md) | ✅ RESOLVED |
| 4 | Symbolic Kani config validation | [`gap4_symbolic_kani_config`](gap4_symbolic_kani_config.md) | ✅ RESOLVED |

## Kani-Verified Harnesses

| Harness | What It Proves |
|---------|---------------|
| `verify_c5_non_positive_mass` | Mass ≤ 0 rejected |
| `verify_c5_identical_positions` | Two bodies at same position rejected |
| `verify_c5_too_few_bodies` | Empty configuration rejected |