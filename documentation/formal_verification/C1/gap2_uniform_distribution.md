# C1 Gap 2: Uniformity of `dist_cubed`

**Status:** Empirical. No proof.

**Claim:** under the physical domain constraints, `dist_cubed` is uniformly
distributed over `[MIN_DIST³, MAX_DIST³]`.

## Why this matters

C1 gap 1 wants to argue that a discarded remainder carries about one bit of
entropy. That requires the input to the division to be distributed in a
particular way. Uniformity of `dist_cubed` is the assumption that makes the
per-operation entropy argument go through.

## Evidence

The claim is supported empirically, not derived:

1. A Kolmogorov–Smirnov test in `tests/information_loss/` reports
   p > 0.05 against a uniform null.
2. Monte-Carlo sampling (10⁶ draws) is reported as consistent across quantiles.
3. A physical plausibility argument: random orbital configurations should
   produce a broad range of separations.

## What would be needed for a proof

- An explicit probability distribution over initial configurations.
- A derivation of the induced distribution of pairwise separations.
- A statement (and proof) that the rounding error is independent of the
  quotient, which is **not** implied by uniformity of the input alone.

None of these exists. In particular, note that `dist_cubed = dist_sq * dist` is
a deterministic function of `dist`, so "uniformity of `dist_cubed`" and
"uniformity of `dist`" cannot both hold on the same interval; the claim needs a
precise statement before it can be proved.

## References

- `tests/information_loss/` (K–S test)
- [C1 overview](readme.md)
- [PROOF_INDEX.md](../PROOF_INDEX.md)
