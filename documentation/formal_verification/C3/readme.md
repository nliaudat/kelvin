# C3: Sequential Quantum Hardness

> **Status:** Open. The argument below is informal. No artifact establishes a
> quantum query lower bound for Kelvin.
> **Gap documents:** `gap1`–`gap3` in this directory
> **Last updated:** 2026-11-23

---

## Motivation

C1 and C2 aim to show that the fixed-point simulation $\Phi$ is irreversible and
chaotic. The quantum question is: given the keystream
$K = \text{SHAKE256}(\Phi^S(C))$, can an adversary recover $C$ or anything
useful about it?

## Attack model

**Rejected model:** "invert $\Phi^S$ given a target state $y$". The attacker
never sees the orbital state, so this is not the right game.

**Adopted model:** the attacker sees only
$K = \text{SHAKE256}(\Phi^S(C))$, and finding $C$ means searching $\Theta$.

## The argument (informal)

$$Q(\Phi^S) \ge \Omega\!\left(\sqrt{|\Theta|}\right) = \Omega(2^{960})
  \quad\text{quantum oracle queries}$$

The chain is:

1. **C5:** $|\Theta| \ge 2^{1920}$ — itself **open** (see C5).
2. $F(C') = [\text{SHAKE256}(\Phi^S(C')) = K]$ marks the preimages of $K$ in
   $\Theta$.
3. Grover optimality (Zalka 1999): $\Omega(\sqrt N)$ queries for unstructured
   search over $N$ items with a unique marked element.
4. $\Omega(\sqrt{2^{1920}}) = \Omega(2^{960})$.

## Qualifications that must not be dropped

- **Grover's bound is for an unstructured oracle with a *unique* marked
  element.** A keystream distinguisher is not literally that game: the code is
  known, and several configurations may map to the same keystream. With $t$
  matching inputs the bound becomes $\Omega(\sqrt{N/t})$, and the reduction from
  "distinguishes the keystream" to "finds a preimage" has to be argued
  separately.
- **$|\Theta| \ge 2^{1920}$ is not established** (C5 is open).
- **SHAKE256 is treated as a random oracle.** That is an assumption, not a
  theorem.
- **A lower bound on queries for constant success is not a bound on an
  adversary's advantage** for a fixed query budget. The two statements have to be
  connected explicitly.

None of the four items above is currently discharged by a checked artifact.

## Gap status

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Extend Ambainis' adversary method | [`gap1_ambainis_adversary.md`](gap1_ambainis_adversary.md) | **Open** |
| 2 | $\Omega(2^{Sk/2})$ lower bound | [`gap2_quantum_query_lower_bound.md`](gap2_quantum_query_lower_bound.md) | **Open** |
| 3 | Relate $k$ to C1's $\varepsilon$-bound | [`gap3_c1_c3_link.md`](gap3_c1_c3_link.md) | **Open** |

## Kani harnesses (integrated)

| Harness | What it checks |
|---------|----------------|
| `verify_c3_step_non_injective` | a 1-ULP perturbation is damped to ≤ 10 ULPs after one Verlet step |
| `verify_c3_two_step_preimage_growth` | four 1-ULP-perturbed states converge within tolerance after two steps |
| `verify_c3_quantum_reduction` | halving an exponent gives a value at most the original |
| `verify_c3_exponent_arithmetic` | $\sqrt{2^{1920}}$ has exponent 960, above 128 |
| `verify_c3_exponent_law` | $2^a \cdot 2^b = 2^{a+b}$ for $a,b \le 32$ |
| `verify_c3_contraction_arithmetic` | non-expanding steps cannot widen a window |

The last four are **exponent and contraction arithmetic only**. Together with the
first two they are one- and two-step numerical statements about the simulator,
not a quantum lower bound. Run them with:

```bash
docker compose -f docker/docker-compose.yml run --rm kani --proof verify_c3_step_non_injective
```

## References

- [`../../../proofs/lean/src/KelvinProofs/`](../../../proofs/lean/src/KelvinProofs/)
- `kelvin-core/src/fixed_math.rs` (`#[cfg(kani)]` module)
- [`../PROOF_INDEX.md`](../PROOF_INDEX.md)
- Zalka, C. (1999). "Grover's quantum searching algorithm is optimal."
  *Phys. Rev. A* 60, 2746.

## See also

- [PROOF_INDEX.md](../PROOF_INDEX.md)
- [C3 gap documents](.)
- [Lean proofs](../../../proofs/lean/README.md)
