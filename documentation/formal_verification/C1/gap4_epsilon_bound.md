# C1 Gap 4: Epsilon-Bound `k_op ≥ 1 − ε`

**Status:** Partial. A Kani harness checks the *implemented* preimage/division
arithmetic; the entropy bound is not proved.

**Claim:** for the Q32.64 division `q = g / dist_cubed` with softening
$\varepsilon_{\text{soft}} = 10^9$, the per-operation information loss satisfies
$k_{op} \ge 1-\varepsilon \approx 0.98$ bits.

## What the Kani harness actually checks

`verify_c1_epsilon_bound` (in `kelvin-core/src/fixed_math.rs`, `#[cfg(kani)]`)
binds the preimage count for the division implementation. It establishes a
bound of the form "at most $K$ inputs map to the same quotient" for the ranges
encoded in the harness. That is a statement about the map

$$x \mapsto \left\lfloor \frac{g_{\text{raw}} \cdot 2^{64}}{x} \right\rfloor,$$

not about Shannon entropy.

Converting "preimage size ≤ K" into "at least $1-\varepsilon$ bits lost" needs:

1. a distribution over `dist_cubed`;
2. a definition of $k_{op}$ (mutual information? min-entropy loss?);
3. a proof that the quotient and the discarded remainder are conditionally
   independent given the input.

None of the three exists. The number $0.98$ comes from Monte-Carlo
measurement, not from the harness.

## Why the earlier "proof" was removed

An earlier revision quoted a Lean theorem `c1_epsilon_bound` whose final line
was `exact ⟨⟩` — a term that closes no goal — preceded by
`-- Empirical measurement confirms this bound`. It was not in the sources and
could not have compiled.

## What is proved

- `FixedPoint.div_rem_identity`: $a / b \cdot b + a \% b = a$.
- `FixedPoint.rem_bounds`: $0 \le a \% b < b$ for $b > 0$.

## References

- [C1 gap 1](gap1_kop_shannon_bound.md) (ideal case)
- [C1 overview](readme.md)
- `kelvin-core/src/fixed_math.rs` (`verify_c1_epsilon_bound`)
- [PROOF_INDEX.md](../PROOF_INDEX.md)
