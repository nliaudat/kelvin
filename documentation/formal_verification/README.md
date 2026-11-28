# Formal Verification Documentation Index

This directory documents what is and is not formally verified about the Kelvin
cryptosystem.

> **Read this first.** No cryptographic security claim in this repository is
> currently machine-checked. What *is* machine-checked is a set of arithmetic and
> scheduling lemmas (Lean) plus 46 Kani harnesses over fixed-point arithmetic,
> model arithmetic, key derivation inputs, framing and loop invariants. Everything else is empirical, an explicit assumption, or open.

**Last updated:** 2026-11-23

## Status summary

| Category | Gaps | Proved | Partial | Empirical | Open |
|----------|------|--------|---------|-----------|------|
| C1 Information loss | 7 | 0 | 2 | 2 | 3 |
| C2 Lyapunov chaos | 6 | 0 | 0 | 2 | 4 |
| C3 Quantum hardness | 3 | 0 | 0 | 0 | 3 |
| C4 Keystream indistinguishability | 3 | 0 | 1 | 0 | 2 |
| C5 Configuration space | 4 | 0 | 0 | 1 | 3 |
| **Total** | **23** | **0** | **3** | **5** | **15** |

- **Proved** — machine-checked artifact establishes the stated claim.
- **Partial** — a strictly weaker related fact is machine-checked.
- **Empirical** — measured or taken from literature, not proved.
- **Open** — no artifact.

## Where the artifacts are

| Artifact | Location | How to run |
|----------|----------|-----------|
| Lean 4 proofs | [`proofs/lean/`](../../proofs/lean/README.md) | `cd proofs/lean && lake build` |
| Kani harnesses (in-crate) | `kelvin-core/src/fixed_math.rs` (`#[cfg(kani)]`) | `docker compose -f docker/docker-compose.yml run --rm kani` |
| Kani harnesses (included from `proofs/kani/`) | `kelvin-core/src/kani_harnesses.rs` | same; `--list` names all 46 |
| Kani harnesses (`kelvin-kdf`) | `kelvin-kdf/src/schedule.rs`, `extractor.rs` | same |
| Kani harnesses (`kelvin`) | `kelvin/src/authenticated.rs` | same |
| Kani harness templates (not compiled) | other `proofs/kani/*.rs` files | n/a — templates only |
| Protocol proof tooling (no Kelvin models yet) | [`docker/formal/`](../../docker/formal/README.md), [`formal/`](../../formal/README.md) | `docker compose -f docker/docker-compose.formal.yml run --rm proverif ...` |

## Quick links

| Document | Description |
|----------|-------------|
| [`PROOF_INDEX.md`](PROOF_INDEX.md) | Gap-by-gap mapping to artifacts |
| [`HARNESS_REVIEW.md`](HARNESS_REVIEW.md) | What each of the 46 harnesses actually constrains |
| [`../formal_verification.md`](../formal_verification.md) | Architecture and threat model |
| [`code_verification.md`](code_verification.md) | Dynamic error-bound strategy |

---

## C1: Information Loss (7 gaps)

**Location:** [`C1/`](C1/readme.md)

Irreversibility of fixed-point rounding.

| Gap | Claim | Status |
|-----|-------|--------|
| [`C1/gap1_kop_shannon_bound.md`](C1/gap1_kop_shannon_bound.md) | $k_{op} \ge 1$ bit per division | **Partial** — `div_rem_identity` proved; entropy claim open |
| [`C1/gap2_uniform_distribution.md`](C1/gap2_uniform_distribution.md) | Uniformity of $d^3$ | **Empirical** |
| [`C1/gap3_saturation_bound.md`](C1/gap3_saturation_bound.md) | Cumulative loss saturates | **Open** |
| [`C1/gap4_epsilon_bound.md`](C1/gap4_epsilon_bound.md) | $k_{op} \ge 1-\varepsilon$ | **Partial** — Kani `verify_c1_epsilon_bound` checks the implemented arithmetic |
| [`C1/gap5_error_independence.md`](C1/gap5_error_independence.md) | Error independence (Lemma A3) | **Empirical / assumption** |
| [`C1/gap6_hartley_min_entropy.md`](C1/gap6_hartley_min_entropy.md) | Hartley vs min-entropy | **Open** |
| [`C1/gap7_verlet_double_effect.md`](C1/gap7_verlet_double_effect.md) | Verlet double computation | **Partial** — `Integrator.lean` proves the two-kick schedule |

**Lean modules:** [`FixedPoint.lean`](../../proofs/lean/src/KelvinProofs/FixedPoint.lean),
[`Integrator.lean`](../../proofs/lean/src/KelvinProofs/Integrator.lean)

---

## C2: Lyapunov Chaos Certification (6 gaps)

**Location:** [`C2/`](C2/readme.md)

| Gap | Claim | Status |
|-----|-------|--------|
| [`C2/gap1_discrete_lyapunov_bound.md`](C2/gap1_discrete_lyapunov_bound.md) | Discrete Lyapunov error bound | **Open** |
| [`C2/gap2_positive_lambda_certification.md`](C2/gap2_positive_lambda_certification.md) | Positive $\lambda$ lower bound | **Empirical** ($\lambda \approx 0.7\,\text{yr}^{-1}$) |
| [`C2/gap3_q3264_lyapunov_spectrum.md`](C2/gap3_q3264_lyapunov_spectrum.md) | Full spectrum in Q32.64 | **Empirical** |
| [`C2/gap4_kaplan_yorke_entropy.md`](C2/gap4_kaplan_yorke_entropy.md) | Kaplan–Yorke entropy bound | **Open** |
| [`C2/gap5_shadow_orbit_bias.md`](C2/gap5_shadow_orbit_bias.md) | Shadow-orbit bias constant | **Open** |
| [`C2/gap6_lipschitz_lambda.md`](C2/gap6_lipschitz_lambda.md) | Lipschitz constant of $\lambda$ | **Open** |

`proofs/kani/lyapunov_proofs.rs` is a template that is **not** part of any Cargo
target and has never been run.

---

## C3: Quantum Hardness (3 gaps)

**Location:** [`C3/`](C3/readme.md)

| Gap | Claim | Status |
|-----|-------|--------|
| [`C3/gap1_ambainis_adversary.md`](C3/gap1_ambainis_adversary.md) | Ambainis adversary method | **Open** |
| [`C3/gap2_quantum_query_lower_bound.md`](C3/gap2_quantum_query_lower_bound.md) | $\Omega(2^{S k/2})$ lower bound | **Open** |
| [`C3/gap3_c1_c3_link.md`](C3/gap3_c1_c3_link.md) | Link $k$ to the C1 $\varepsilon$-bound | **Open** |

The Grover/Ambainis argument is prose. It depends on Grover optimality (Zalka
1999) and the configuration-space size, neither of which is formalised.

---

## C4: Keystream Indistinguishability (3 gaps)

**Location:** [`C4/`](C4/readme.md)

| Gap | Claim | Status |
|-----|-------|--------|
| [`C4/gap1_game_based_reduction.md`](C4/gap1_game_based_reduction.md) | Game-based reduction | **Open** |
| [`C4/gap2_domain_separation_all_modes.md`](C4/gap2_domain_separation_all_modes.md) | Domain separation across 6 modes | **Partial** — framing injectivity proved; XOF separation conditional |
| [`C4/gap3_advantage_bound.md`](C4/gap3_advantage_bound.md) | Advantage bound | **Open** |

**Lean module:** [`Entropy.lean`](../../proofs/lean/src/KelvinProofs/Entropy.lean)

---

## C5: Configuration Space Cardinality (4 gaps)

**Location:** [`C5/`](C5/readme.md)

| Gap | Claim | Status |
|-----|-------|--------|
| [`C5/gap1_cardinality_bound.md`](C5/gap1_cardinality_bound.md) | $|\Theta_5| \ge 2^{1920}$ | **Open** |
| [`C5/gap2_min_entropy.md`](C5/gap2_min_entropy.md) | $H_{\min} \ge 1800$ bits | **Open** |
| [`C5/gap3_stability_reduction.md`](C5/gap3_stability_reduction.md) | Stability reduction ≈ 1 | **Empirical** |
| [`C5/gap4_symbolic_kani_config.md`](C5/gap4_symbolic_kani_config.md) | Symbolic Kani config | **Open** |

---

## Building

### Lean

```bash
cd proofs/lean
lake build
```

No Mathlib required. See [`proofs/lean/README.md`](../../proofs/lean/README.md).

To run the CI-style check (build + reject `sorry`/`admit`/`axiom` +
`native_decide` + count theorems):

```bash
cd proofs/lean
./check.sh                 # or: powershell -ExecutionPolicy Bypass -File .\check.ps1
```

### Kani

```bash
docker compose -f docker/docker-compose.yml build kani
docker compose -f docker/docker-compose.yml run --rm kani --list
docker compose -f docker/docker-compose.yml run --rm kani --proof verify_add_no_overflow
docker compose -f docker/docker-compose.yml run --rm kani
```

---

## License

Apache License, Version 2.0 or MIT at your option.
