# C5 Gap 2: Min-Entropy Bound

> **Machine-checked status:** see [PROOF_INDEX.md](../PROOF_INDEX.md). The label
> below states what is actually established. The surrounding text is the written
> argument; it is not a proof unless the label says so.

> **Status:** Open. No min-entropy derivation.

> **Date:** 2026-05-30
> **Conjecture:** C5 — Valid Configuration Space Cardinality

## 1. Theorem Statement

$$H_{\min}(\Theta_5) = -\log_2(\max_{C \in \Theta_5} \Pr[C]) \ge 1800 \text{ bits}$$

Under the natural product measure (uniform over each mass, position, velocity within bounds), no single configuration has probability weight exceeding `2^{-1800}`.

## 2. Derivation

The uniform distribution over the valid space `Θ_5` assigns each valid configuration equal probability. The min-entropy equals `log₂(|Θ_5|)` minus the correction for non-uniformity due to constraints.

| Factor | Entropy (bits) |
|--------|---------------|
| Unconstrained product measure | ~2469.5 |
| Constraint bias (worst-case configuration) | ≤ 5 bits (empirical: Monte Carlo uniformity; the former gap3 constraint analysis is withdrawn — see gap3) |
| Resulting min-entropy | ≥ 2464.5 bits |
| Claimed lower bound | ≥ 1800 bits (664 bits of unexplained margin below the derivation — conservative and valid, but the margin is luck, not analysis) |

## 3. References

- `tests/configuration_space/` (Monte Carlo: sampled configurations are approximately uniform)
---

## See Also

- [C5 Proof Sketch](readme.md)
