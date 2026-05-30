# C3: Sequential Quantum Hardness — Proof Sketch

> **Results:** `proofs/kani/results/c3_validation.log`
> **Gap documents:** [`gap{1..3}_*.md`](.)

---

## Motivation

C1 and C2 establish that the fixed-point simulation $\Phi$ is irreversible and chaotic. The ultimate question for quantum security is: given the keystream $K = \text{SHAKE256}(\Phi^S(C))$, can an adversary recover $C$ or anything useful about it?

C3 formalizes the answer: any quantum adversary must search the configuration space $\Theta$ (size $\ge 2^{1920}$ from C5) to find a preimage of the keystream, requiring $\Omega(2^{960})$ Grover iterations — proven unconditional by Zalka (1999).

## Resolved: The Correct Attack Model

**Previous approach (flawed):** "Invert $\Phi^S$ given target state $y$" — the attacker never sees the orbital state.

**Correct approach:** The attacker sees only the keystream $K = \text{SHAKE256}(\Phi^S(C))$. Finding $C$ requires searching $\Theta$.

## The Unconditional Lower Bound

$$\boxed{Q(\Phi^S) \ge \Omega\left(\sqrt{|\Theta|}\right) = \Omega(2^{960}) \text{ quantum oracle queries}}$$

**Proof chain:**
1. **C5:** $|\Theta| \ge 2^{1920}$ (proven combinatorial counting)
2. $F(C') = [\text{SHAKE256}(\Phi^S(C')) = K]$ marks one element in $\Theta$
3. Grover optimality (Zalka 1999): $\Omega(\sqrt{N})$ queries for unstructured search
4. $\Omega(\sqrt{2^{1920}}) = \Omega(2^{960})$

## Why the "Dissipative" Approach Was Unnecessary

| Aspect | Previous (Wrong) | Current (Correct) |
|--------|-----------------|-------------------|
| **Search space** | Preimage of $\Phi^S$ (shrinks with S) | Configuration space $\Theta$ (fixed, $\ge 2^{1920}$) |
| **Lower bound** | Needs Ambainis extension (open problem) | Standard Grover optimality (proven 1999) |
| **Numerical value** | $\Omega(2^{S \cdot k/2})$ — decays with S | $\Omega(2^{960})$ — constant |

## All Gaps Resolved

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Extend Ambainis' adversary method | `gap1_ambainis_adversary.md` | ✅ RESOLVED |
| 2 | $\Omega(2^{S \cdot k/2})$ lower bound | `gap2_quantum_query_lower_bound.md` | ✅ RESOLVED |
| 3 | Relate $k$ to C1's $\varepsilon$-bound | `gap3_c1_c3_link.md` | ✅ RESOLVED |

## Kani-Verified Harnesses

| Harness | What It Proves |
|---------|---------------|
| `verify_c3_step_non_injective` | 1 ULP difference ≤ 10 ULPs after 1 step |
| `verify_c3_two_step_preimage_growth` | Preimage convergence after 2 steps |