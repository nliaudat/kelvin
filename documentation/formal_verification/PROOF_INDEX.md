# Proof Index

Gap-by-gap mapping from the 23 documented verification gaps to the artifacts
that exist for them.

Status legend:

| Status | Meaning |
|--------|---------|
| **Proved** | A machine-checked artifact establishes the stated claim. |
| **Partial** | A strictly weaker related fact is machine-checked. |
| **Empirical** | Supported by measurement/literature, not proved. |
| **Open** | No artifact. |

> No cryptographic security claim in this repository is machine-checked. There
> are currently **0 Proved**, **3 Partial**, **5 Empirical**, **15 Open** gaps.

## Artifact legend

| ID | Artifact | Path |
|----|----------|------|
| L-FP | Lean: fixed-point arithmetic | [`proofs/lean/src/KelvinProofs/FixedPoint.lean`](../../proofs/lean/src/KelvinProofs/FixedPoint.lean) |
| L-IN | Lean: Verlet scheduling | [`proofs/lean/src/KelvinProofs/Integrator.lean`](../../proofs/lean/src/KelvinProofs/Integrator.lean) |
| L-GR | Lean: pairwise momentum | [`proofs/lean/src/KelvinProofs/Gravity.lean`](../../proofs/lean/src/KelvinProofs/Gravity.lean) |
| L-KS | Lean: key-schedule structure | [`proofs/lean/src/KelvinProofs/KeySchedule.lean`](../../proofs/lean/src/KelvinProofs/KeySchedule.lean) |
| L-EN | Lean: entropy framing | [`proofs/lean/src/KelvinProofs/Entropy.lean`](../../proofs/lean/src/KelvinProofs/Entropy.lean) |
| L-PD | Lean: Floyd loop invariants | [`proofs/lean/src/KelvinProofs/PeriodDetection.lean`](../../proofs/lean/src/KelvinProofs/PeriodDetection.lean) |
| K-CORE | Kani: harnesses in the crate | `kelvin-core/src/fixed_math.rs` (`#[cfg(kani)]`) |
| K-INC | Kani: harnesses included from `proofs/kani/` | `kelvin-core/src/kani_harnesses.rs` |
| K-KDF | Kani: key schedule and extraction framing | `kelvin-kdf/src/schedule.rs`, `kelvin-kdf/src/extractor.rs` |
| K-AUTH | Kani: authenticated chunk protocol (ideal tag) | `kelvin/src/authenticated.rs` |

**K-CORE (8):** `verify_add_no_overflow`, `verify_sub_no_overflow`,
`verify_mul_range`, `verify_c1_preimage_bound`, `verify_c1_epsilon_bound`,
`verify_c1_division_remainder`, `verify_c3_step_damping`,
`verify_c3_two_step_dispersion`.

**K-INC (29):** `verify_information_loss_total`,
`verify_information_loss_monotone`, `verify_saturation_point`,
`verify_information_loss_saturating`, `verify_information_loss_cap_boundary`,
`verify_lyapunov_spectrum_comprehensive`, `verify_lyapunov_division`,
`verify_lyapunov_error_bounds`, `verify_lyapunov_quantization`,
`verify_shadow_orbit_bias`, `verify_perturbation_in_range`,
`verify_c3_quantum_reduction`, `verify_c3_exponent_arithmetic`,
`verify_c3_exponent_law`, `verify_c3_contraction_arithmetic`,
`verify_domain_separation`, `verify_framing_deterministic`,
`verify_shake256_grover_arithmetic`, `verify_bound_is_min_of_two`,
`verify_ct_select_i128`, `verify_ct_select_sign_bits`, `verify_ct_is_zero_u64`,
`verify_ct_select_u64`, `verify_ct_select_u64_is_exact`,
`verify_floyd_cycle_detection`, `verify_hare_leads`,
`verify_find_cycle_is_complete_and_correct`,
`verify_find_cycle_is_sound_for_any_budget`, `verify_find_cycle_zero_budget`.

**K-KDF (6):** `verify_schedule_info_injective`,
`verify_schedule_key_and_nonce_info_separated`, `verify_schedule_first_indices`,
`verify_schedule_limit_invariant`, `verify_extraction_header_injective`,
`verify_extraction_body_injective` (in `kelvin-kdf/src/schedule.rs` and
`extractor.rs`).

**K-AUTH (3):** `verify_chunk_prefix_injective`,
`verify_chunk_open_all_or_nothing`, `verify_chunk_opens_only_in_place` (in
`kelvin/src/authenticated.rs`).

All 46 are listed by `docker compose -f docker/docker-compose.yml run --rm kani --list`
and run by the same command without arguments. They check arithmetic, scheduling,
framing and loop invariants; **none of them establishes a C1–C5 security gap.**
See [`HARNESS_REVIEW.md`](HARNESS_REVIEW.md) for what each one actually
constrains, which ones exercise the crate, and the mutation results.

## C1 — Information loss

| Gap | Claim | Artifact | Status |
|-----|-------|----------|--------|
| [C1/gap1](C1/gap1_kop_shannon_bound.md) | $k_{op} \ge 1$ bit per division | L-FP (`div_rem_identity`, `rem_bounds`) | Partial |
| [C1/gap2](C1/gap2_uniform_distribution.md) | Uniformity of $d^3$ | — | Empirical |
| [C1/gap3](C1/gap3_saturation_bound.md) | Cumulative loss saturates | — | Open |
| [C1/gap4](C1/gap4_epsilon_bound.md) | $k_{op} \ge 1-\varepsilon$ | K-CORE (`verify_c1_epsilon_bound`) | Partial |
| [C1/gap5](C1/gap5_error_independence.md) | Error independence | — | Empirical |
| [C1/gap6](C1/gap6_hartley_min_entropy.md) | Hartley vs min-entropy | — | Open |
| [C1/gap7](C1/gap7_verlet_double_effect.md) | Verlet double computation | L-IN | Partial |

## C2 — Lyapunov chaos

| Gap | Claim | Artifact | Status |
|-----|-------|----------|--------|
| [C2/gap1](C2/gap1_discrete_lyapunov_bound.md) | Discrete Lyapunov error bound | — | Open |
| [C2/gap2](C2/gap2_positive_lambda_certification.md) | Positive $\lambda$ | — | Empirical |
| [C2/gap3](C2/gap3_q3264_lyapunov_spectrum.md) | Spectrum in Q32.64 | — | Empirical |
| [C2/gap4](C2/gap4_kaplan_yorke_entropy.md) | Kaplan–Yorke bound | — | Open |
| [C2/gap5](C2/gap5_shadow_orbit_bias.md) | Shadow-orbit bias | — | Open |
| [C2/gap6](C2/gap6_lipschitz_lambda.md) | Lipschitz constant | — | Open |

## C3 — Quantum hardness

| Gap | Claim | Artifact | Status |
|-----|-------|----------|--------|
| [C3/gap1](C3/gap1_ambainis_adversary.md) | Ambainis adversary method | — | Open |
| [C3/gap2](C3/gap2_quantum_query_lower_bound.md) | Query lower bound | — | Open |
| [C3/gap3](C3/gap3_c1_c3_link.md) | Link to C1 $\varepsilon$ | — | Open |

K-CORE contains `verify_c3_step_damping` and
`verify_c3_two_step_dispersion`. On concrete large denominators they check that
a 1-ULP divisor change moves the quotient by at most 1 ULP, and that this does
not grow through a second division. These are statements about `Fixed::div`
rounding, **not** about a Verlet step and **not** a quantum lower bound. The
K-INC exponent and contraction harnesses are arithmetic only.

## C4 — Keystream indistinguishability

| Gap | Claim | Artifact | Status |
|-----|-------|----------|--------|
| [C4/gap1](C4/gap1_game_based_reduction.md) | Game-based reduction | — | Open |
| [C4/gap2](C4/gap2_domain_separation_all_modes.md) | Domain separation | L-EN (`domain_separation`, conditional) | Partial |
| [C4/gap3](C4/gap3_advantage_bound.md) | Advantage bound | — | Open |

## C5 — Configuration space

| Gap | Claim | Artifact | Status |
|-----|-------|----------|--------|
| [C5/gap1](C5/gap1_cardinality_bound.md) | $\lvert\Theta_5\rvert \ge 2^{1920}$ | — | Open |
| [C5/gap2](C5/gap2_min_entropy.md) | $H_{\min} \ge 1800$ bits | — | Open |
| [C5/gap3](C5/gap3_stability_reduction.md) | Stability reduction ≈ 1 | — | Empirical |
| [C5/gap4](C5/gap4_symbolic_kani_config.md) | Symbolic Kani config | — | Open |

## Totals

| Category | Proved | Partial | Empirical | Open |
|----------|--------|---------|-----------|------|
| C1 | 0 | 2 | 2 | 3 |
| C2 | 0 | 0 | 2 | 4 |
| C3 | 0 | 0 | 0 | 3 |
| C4 | 0 | 1 | 0 | 2 |
| C5 | 0 | 0 | 1 | 3 |
| **Total** | **0** | **3** | **5** | **15** |

## What remains unproved

Passing Kani harnesses and a clean `lake build` do **not** mean the security
gaps are closed. They establish internal arithmetic and implementation facts.
This section records, gap by gap, what a real proof would still have to supply.

### C1 — information loss

| Gap | What is missing |
|-----|-----------------|
| `k_op ≥ 1` bit | An explicit distribution over `dist³` and a fibre-entropy bound. `div_rem_identity` is a division identity, not an entropy statement. |
| Uniformity of `dist³` | A derivation from a stated distribution over configurations; currently only a K–S test. |
| Cumulative saturation | The `min(S·k_step, H_max − log₂A)` formula is a modelling assumption; no process-level proof. |
| `k_op ≥ 1−ε` | The same fibre argument as gap 1. Kani only bounds preimage size for the implemented division. |
| Error independence | The joint law of rounding errors across pairs and steps; currently an assumption (Lemma A3). |
| Hartley vs min-entropy | The sampling distribution needed to define `H_min` at all. |
| Verlet double computation | The schedule is proved; converting operation counts into bits of loss is not. |

### C2 — Lyapunov chaos

| Gap | What is missing |
|-----|-----------------|
| Discrete error bound | A real-valued model of the Padé `ln`, the Q32.64 rounding, and a *deterministic* shadowing argument (the `σ/√S` term presumes i.i.d. samples). |
| `λ > 0` | A certified lower bound; `λ ≈ 0.7 yr⁻¹` is measured, not derived. |
| Q32.64 spectrum | A conditioning bound for this estimator, not a generic heuristic. |
| Kaplan–Yorke | `D_KY` defined from a spectrum and tied to the simulator; the Lean lemma is only `j ≤ j + S/g`. |
| Shadow-orbit bias | A derivation of the bias constant. |
| Lipschitz constant | A proved bound on the estimator map. |

### C3 — quantum hardness

| Gap | What is missing |
|-----|-----------------|
| Ambainis adversary method | A formal quantum query model for the keystream predicate. |
| `Ω(2^{Sk/2})` | Depends on C5 **and** on the predicate having few preimages; with `t` matches the bound is `Ω(√(N/t))`. |
| Link to C1 `ε` | Depends on C1, which is itself open. |

### C4 — keystream indistinguishability

| Gap | What is missing |
|-----|-----------------|
| Game-based reduction | Formal probabilistic games and the SHAKE256 → random-oracle switch. |
| Domain separation | The framing injection is proved; XOF injectivity is conditional and SHAKE256 is not literally injective as a variable-length function. |
| Advantage bound | The reduction itself: `Adv ≤ negl(n) + 2^{-128}` is asserted, never derived. |

### C5 — configuration space

| Gap | What is missing |
|-----|-----------------|
| `\lvert\Theta_5\rvert ≥ 2^{1920}` | An explicit measure over the admissible set; "constraint reduction ≤ b bits" is asserted. |
| `H_min ≥ 1800` | The sampling distribution (min-entropy is not `log₂` of the set size). |
| Stability reduction | Currently a 0/200 rejection measurement. |
| Symbolic Kani config | The three harnesses are still templates, not integrated. |

### Assumptions that cannot be discharged here

These are not gaps to close but premises the argument rests on; they have to be
stated and accepted:

1. **SHAKE256 is indifferentiable from a random oracle.**
2. **HMAC-SHA512 is injective in its index argument** (makes
   `key_unique_of_injective` usable).
3. **The reachable dynamics are hard to invert** given the keystream.
4. **Grover optimality applies** to a predicate with few preimages over `Θ`.
5. **The pipeline executes in constant time.** Only branch-free primitives are
   checked; timing requires dudect/ctgrind plus a source audit.

### Cross-cutting gaps

- **No refinement proof.** There is no verified translation from the Lean model
  to `kelvin-core`; the correspondence is by inspection.
- **Placeholder modules.** `information_loss.rs` returns `steps * 1.0`;
  `reference.rs` and `quantum.rs` are stubs.

### What is realistically provable next

1. **C5 cardinality** — define the admissible set and its measure explicitly,
   then do a real counting proof. Most tractable of the five.
2. **C6** — `find_cycle` is verified on small state spaces; what remains is a
   statement about the orbital simulation itself, which is out of reach for
   bounded model checking.
3. **C1 as a conditional theorem** — fix a distribution, then prove the entropy
   bound under stated independence assumptions.
4. **C2 Padé error** — formalizable against a reference `ln`, but needs a
   real-analysis library (Mathlib or hand-rolled).
5. **C3/C4** — require formalized probability and quantum query games; outside
   what Kani can express.

Closing every item above still leaves the five assumptions listed earlier, so
the end state is "reduced to standard cryptographic assumptions", never
"unconditionally proved".

## Maintenance

When an artifact changes, update this file and the matching category
`readme.md` in the same commit. Do not mark a gap **Proved** unless the artifact
is (a) built by CI or by the documented command and (b) actually states the
claim in the gap document.
