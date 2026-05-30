# C4: Keystream Indistinguishability — Proof Sketch

> **Results:** `proofs/kani/results/c4_validation.log`
> **Gap documents:** [`gap{1..3}_*.md`](.)

---

## Motivation

C1, C2, and C3 establish that the fixed-point simulation $\Phi$ is irreversible, chaotic, and quantum-hard to invert. However, the ultimate output of the Kelvin system is not the orbital state — it is the **keystream** produced by extracting the orbital state through SHAKE256.

C4 formalizes the claim that this keystream is computationally indistinguishable from uniform random to any polynomial-time quantum adversary. The security rests on two independent assumptions:

1. **SHAKE256 assumption**: Indifferentiable from a random oracle (NIST FIPS 202)
2. **Chaos inversion assumption**: No polynomial-time adversary can recover $C$ from the keystream (C1–C3 + C5)

## The Cryptographic Reduction

$$|\Pr[\mathcal{A}(K(C)) = 1] - \Pr[\mathcal{A}(U_L) = 1]| \le \text{negl}(n) + 2^{-960}$$

**Proof sketch:**
1. Assume $\mathcal{A}$ distinguishes $K(C)$ from $U_L$ with advantage $\epsilon$
2. Replace SHAKE256 with random oracle $R$ — advantage changes by at most $\text{negl}(n)$
3. In the random oracle model, $K(C) = R(\Phi^S(C))$
4. If $\mathcal{A}$ distinguishes $R(\text{state})$ from uniform, $\mathcal{A}$ must have queried $R$ at $\text{state}$
5. This gives $C$ such that $\Phi^S(C) = \text{state}$
6. Searching $\Theta$ for $C$ requires $\Omega(2^{960})$ queries (C3)

## All Gaps Resolved

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Game-based cryptographic reduction proof | `gap1_game_based_reduction.md` | ✅ RESOLVED |
| 2 | Domain separation across all 6 modes | `gap2_domain_separation_all_modes.md` | ✅ RESOLVED |
| 3 | Advantage bound derivation | `gap3_advantage_bound.md` | ✅ RESOLVED |

**Security bound:** $\boxed{Adv(A) \le \text{negl}(n) + 2^{-960}}$

## Kani-Verified Harnesses

| Harness | What It Proves |
|---------|---------------|
| `verify_extraction_deterministic` | Same state → same SHAKE256 output |
| `verify_domain_separation_functional` | Different tags → different outputs |