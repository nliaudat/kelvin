# C3 Gap 1: Ambainis Adversary Method

**Status:** Open. The argument is prose; no artifact establishes a query lower
bound.

**Claim:** any quantum algorithm distinguishing the Kelvin keystream from
uniform requires

$$Q(\Phi^S) \ge \Omega\!\left(\sqrt{|\Theta|}\right) = \Omega(2^{960})$$

queries, where $|\Theta| \ge 2^{1920}$ (C5).

## The argument, with its conditions restored

The reasoning is:

1. The attacker sees only $K = \text{SHAKE256}(\Phi^S(C))$.
2. Finding $C$ means searching $\Theta$.
3. Grover's algorithm is optimal for unstructured search: $\Omega(\sqrt N)$
   queries over $N$ items.
4. With $N = 2^{1920}$ this gives $\Omega(2^{960})$.

Conditions that must be stated, and are not discharged:

| Condition | Status |
|-----------|--------|
| Grover optimality (Zalka 1999) | Literature theorem; applies to unstructured search with a **unique** marked element |
| $|\Theta| \ge 2^{1920}$ | **Open** (C5) |
| The keystream predicate has a unique preimage | **False in general** — several configurations may map to the same keystream; with $t$ matches the bound becomes $\Omega(\sqrt{N/t})$ |
| The code is unknown to the attacker | **False** — Kelvin is open source; Grover's model assumes an otherwise unknown oracle |
| "Constant success probability" ⇒ "advantage bound" | Needs a separate argument for a fixed query budget |

So the bound cannot be claimed "for any algorithm that distinguishes the
keystream". It is a conditional statement: *if* the keystream predicate behaves
like an unstructured oracle with few preimages over a space of size $2^{1920}$,
*then* a constant-success search needs $\Omega(2^{960})$ queries.

## Why the earlier "proof" was removed

An earlier revision quoted Lean theorems `extraction_preimage_resistant` and
`quantum_query_lower_bound`. The first claimed that *no* configuration produces
*any* supplied pool — an easily refuted statement (take the pool to be the
output for any configuration). The second referenced
`c5_gap1_cardinality_bound`, which does not exist. Neither theorem was in the
sources.

## What is proved nearby

`Entropy.domain_separation` (Lean) proves that distinct domain tags produce
distinct framed inputs. That is a framing lemma, not a query bound.

## References

- [C3 overview](readme.md)
- [C3 gap 2](gap2_quantum_query_lower_bound.md)
- [C5 gap 1](../C5/gap1_cardinality_bound.md)
- Zalka, C. (1999). *Phys. Rev. A* 60, 2746.
- [PROOF_INDEX.md](../PROOF_INDEX.md)
