# C4 Gap 1: Game-Based Reduction

**Status:** Open. The reduction is a sketch; no checked artifact establishes an
advantage bound.

**Claim:**

$$|\Pr[\mathcal A(K(C))=1] - \Pr[\mathcal A(U_L)=1]|
  \le \text{negl}(n) + 2^{-960},$$

where $K(C) = \text{SHAKE256}(\Phi^S(C))$ and $U_L$ is uniform.

## The game sequence

| Game | Oracle | Change |
|------|--------|--------|
| 0 | $K(C) = \text{SHAKE256}(\Phi^S(C))$ | real |
| 1 | $R(\Phi^S(C))$, $R$ a random oracle | SHAKE256 → ROM; advantage changes by $\text{negl}(n)$ |
| 2 | $U_L$ | ROM output replaced by uniform |

Triangle inequality:

$$|\mathrm{Adv}_0 - \mathrm{Adv}_2|
  \le |\mathrm{Adv}_0 - \mathrm{Adv}_1| + |\mathrm{Adv}_1 - \mathrm{Adv}_2|.$$

## What is missing

- **Step 1 (ROM switch).** Requires an indifferenceability proof for SHAKE256,
  or an explicit assumption. Neither is formalised.
- **Step 2 (ROM → uniform).** The standard ROM argument says that if $\mathcal A$
  distinguishes $R(x)$ from uniform, it must have queried $R$ at $x$. That
  argument bounds the advantage of a *q-query* adversary by roughly $q/2^n$ — it
  is a function of the **query count**. The quoted bound $2^{-960}$ does not
  contain $q$; it comes from a *query lower bound* (C3), which is a different
  statement.
- **The two steps are asserted.** The earlier text justified each with
  `exact ⟨⟩`, which closes no goal.
- **`game_based_reduction` does not exist** in any Lean file.

## Why the earlier "proof" was removed

An earlier revision quoted Lean theorems `domain_separation_independence` and
`game_based_reduction` from `Shake256Extraction.lean` / `HKDFDerivation.lean`.
Neither file exists in that form any more, and neither theorem was ever present.

## What is proved

[`Entropy.lean`](../../../proofs/lean/src/KelvinProofs/Entropy.lean):

- `domain_separation`: distinct tags ⇒ distinct framed inputs;
- `xof_separates_domains`: **if** the XOF is injective ⇒ distinct tags give
  distinct outputs.

The conditional lemma makes the injectivity assumption explicit. It does not
establish indistinguishability.

## References

- [C4 overview](readme.md)
- [C4 gap 3](gap3_advantage_bound.md)
- [C3 gap 1](../C3/gap1_ambainis_adversary.md)
- [PROOF_INDEX.md](../PROOF_INDEX.md)
