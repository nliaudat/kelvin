# C1 Gap 1: Per-Operation Information Loss `k_op ≥ 1` Bit

**Status:** Partial. Division arithmetic is proved; the entropy claim is open.

**Claim:** each Q32.64 division `q = g / dist_cubed` discards at least 1 bit of
Shannon entropy.

## The argument, stated precisely

For the division

$$q_{\text{raw}} = \left\lfloor \frac{g_{\text{raw}} \cdot 2^{64}}
  {d^3_{\text{raw}}} \right\rfloor,$$

the remainder $r = (g_{\text{raw}} \cdot 2^{64}) \bmod d^3_{\text{raw}}$ is
discarded. The claim is that

$$k_{op} = I(d^3;\, r \mid q) = H(r \mid q) \ge 1 \text{ bit}.$$

## What is actually proved

`FixedPoint.lean` proves two arithmetic facts:

```lean
theorem div_rem_identity (a b : Int) : a / b * b + a % b = a
theorem rem_bounds (a b : Int) (hb : 0 < b) : 0 ≤ a % b ∧ a % b < b
```

These are identities about `Int` division. They do **not** mention entropy, a
distribution, or independence, so they cannot imply $k_{op} \ge 1$.

## What is missing for the entropy claim

1. **A distribution.** $H(r \mid q)$ is defined relative to a distribution on
   $d^3$. C1 gap 2 proposes uniformity but does not prove it.
2. **Conditioning.** $H(r \mid q)$ requires the joint law of $(q, r)$, i.e. the
   map $d^3 \mapsto (q, r)$ and its fibres.
3. **A lower bound on the fibre entropy.** The heuristic "the LSB carries one
   bit" is not a theorem: the fibre of $q$ is an interval of length $d^3$, whose
   size in raw units is what determines the conditional entropy.

The Kani harness `verify_c1_division_remainder` checks the *range* of the
remainder ($0 \le r < \text{den}$). It does not check entropy.

## Per-step extrapolation (unproved)

The sketch multiplies by the operation count:

$$k_{\text{step}} = 2N(N-1)\,k_{op} \ge 40 \text{ bits/step for } N=5.$$

This assumes the per-operation losses are independent and additive. Neither is
proved.

## References

- [C1 overview](readme.md)
- [C1 gap 2](gap2_uniform_distribution.md) — uniformity (empirical)
- [C1 gap 4](gap4_epsilon_bound.md) — ε-version (partial)
- [`../../../proofs/lean/src/KelvinProofs/FixedPoint.lean`](../../../proofs/lean/src/KelvinProofs/FixedPoint.lean)
- [PROOF_INDEX.md](../PROOF_INDEX.md)
