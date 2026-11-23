# C4: Keystream Indistinguishability

> **Status:** Open. The reduction below is a sketch; no checked artifact
> establishes an advantage bound.
> **Gap documents:** `gap1`–`gap3` in this directory
> **Last updated:** 2026-11-23

---

## Motivation

C1–C3 aim to show that $\Phi$ is irreversible, chaotic and quantum-hard to
invert. The actual output of Kelvin is not the orbital state but the **keystream**
obtained by extracting that state through SHAKE256.

C4 is the claim that the keystream is computationally indistinguishable from
uniform. It rests on two assumptions:

1. **SHAKE256 assumption:** indifferentiable from a random oracle (FIPS 202).
2. **Chaos-inversion assumption:** no polynomial-time adversary recovers $C$ from
   the keystream (C1–C3, C5).

Both are assumptions. Neither is proved in this repository.

## The reduction (sketch)

$$|\Pr[\mathcal A(K(C))=1] - \Pr[\mathcal A(U_L)=1]|
  \le \text{negl}(n) + 2^{-960}$$

Sketch:

1. Suppose $\mathcal A$ distinguishes $K(C)$ from $U_L$ with advantage $\epsilon$.
2. Replace SHAKE256 by a random oracle $R$; the advantage changes by at most
   $\text{negl}(n)$.
3. In the ROM, $K(C) = R(\Phi^S(C))$.
4. If $\mathcal A$ distinguishes $R(\text{state})$ from uniform, it must have
   queried $R$ at $\text{state}$ (ROM bookkeeping).
5. That yields $C$ with $\Phi^S(C) = \text{state}$.
6. Searching $\Theta$ for $C$ needs $\Omega(2^{960})$ queries (C3).

Steps 2, 4, 5 and 6 each need work that is not done here. Step 6 inherits the
qualifications listed in [C3](../C3/readme.md), and the C5 cardinality bound it
depends on is itself open.

## Gap status

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | Game-based reduction | [`gap1_game_based_reduction.md`](gap1_game_based_reduction.md) | **Open** |
| 2 | Domain separation across 6 modes | [`gap2_domain_separation_all_modes.md`](gap2_domain_separation_all_modes.md) | **Partial** — framing injectivity proved; XOF separation conditional |
| 3 | Advantage bound | [`gap3_advantage_bound.md`](gap3_advantage_bound.md) | **Open** |

**Claimed bound:** $\text{Adv}(A) \le \text{negl}(n) + 2^{-960}$ — *not proved*.

> **Scope note.** The three exponents in circulation answer different questions
> and must not be quoted interchangeably:
>
> | Exponent | Reading |
> |----------|---------|
> | $2^{-960}$ | Grover search over the representable configuration space ($\lvert\Theta\rvert = 2^{1920}$) |
> | $2^{-128}$ | operational post-quantum security of SHAKE256 (Grover on a 256-bit preimage space) |
> | $2^{-20{,}000{,}000}$ | a per-step-loss reading, $S\cdot k/2$ with $S=10^6$, $k=40$ |
>
> The operational claim in `COMPLIANCE.md` and
> [`../formal_verification.md`](../../formal_verification.md) is "128-bit
> post-quantum via SHAKE256". If that is the intended headline, the bound should
> be restated as $\text{negl}(n) + 2^{-128}$ and the $2^{-960}$ derivation kept
> as a separate, conditional remark. This choice is still open (see `TODO.md`).

## What is proved

[`Entropy.lean`](../../../proofs/lean/src/KelvinProofs/Entropy.lean) proves:

- `domain_separation`: distinct domain tags produce distinct framed inputs;
- `xof_separates_domains`: **if** the XOF is injective, distinct tags produce
  distinct outputs.

The second is conditional on an injectivity assumption that SHAKE256 does not
literally satisfy (it is a fixed-length-to-variable-length function, not
injective). It is a structural lemma, not a security proof.

## Kani harnesses

`proofs/kani/keystream_proofs.rs` (`verify_extraction_deterministic`,
`verify_domain_separation_functional`) is a **template**: not included by
`kani_harnesses.rs` and never compiled.

What does run, from `C4_verify_security_bound.rs` and
`C3_verify_quantum_reduction.rs`:

| Harness | What it checks |
|---------|----------------|
| `verify_domain_separation` | distinct tags give distinct framed inputs, for every payload |
| `verify_framing_deterministic` | framing is deterministic |
| `verify_shake256_grover_arithmetic` | Grover on 256 bits leaves 128 |
| `verify_bound_is_min_of_two` | the operational bound is the smaller exponent |

These are structural and arithmetic facts. They do not establish an advantage
bound.

## References

- [`../../../proofs/lean/src/KelvinProofs/Entropy.lean`](../../../proofs/lean/src/KelvinProofs/Entropy.lean)
- [`../../../proofs/lean/src/KelvinProofs/KeySchedule.lean`](../../../proofs/lean/src/KelvinProofs/KeySchedule.lean)
- `kelvin-kdf/src/extractor.rs`
- [`../PROOF_INDEX.md`](../PROOF_INDEX.md)

## See also

- [PROOF_INDEX.md](../PROOF_INDEX.md)
- [C4 gap documents](.)
- [Lean proofs](../../../proofs/lean/README.md)
