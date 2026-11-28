# Harness Specification Review

**Date:** 2026-11-23 (C6, `kelvin-kdf` and `kelvin` updated 2026-11-27)
**Scope:** the 46 Kani harnesses run by `docker/run-kani.sh`.

A harness can be green and still prove nothing about the crate. This document
records, for each harness, what it asserts, what it does **not** assert, and
whether it calls into `kelvin-core` at all.

## Method

Each harness was classified by reading its body:

- **Crate** — the harness calls an exported `kelvin-core` function (`Fixed`
  operators, `verlet_step`, `find_cycle`, …). A regression in the crate can
  make it fail.
- **Model** — the harness computes on values or helper functions declared in the
  harness file itself. It verifies arithmetic, but no change to the crate can
  make it fail.

This distinction is not visible from `VERIFICATION:- SUCCESSFUL`.

## Summary

| | Count |
|---|---|
| Harnesses total | 46 |
| Exercise `kelvin-core` | **11** |
| Exercise `kelvin-kdf` | **6** |
| Exercise `kelvin` | **3** |
| Model-only | **26** |
| Directly address a C1–C5 security gap | 3 (all partial) |
| Cover `kelvin-kdf` (extractor, key schedule, nonce) | **6** (input construction and limits only) |
| Cover `kelvin` (encrypt/decrypt, AEAD) | **0** |
| Cover timing / constant-time | **0** (see C5 note) |

## Core harnesses (`fixed_math.rs`, 8)

| Harness | Asserts | Does not assert | Crate? |
|---|---|---|---|
| `verify_add_no_overflow` | result in `[-200,200]` AU and equals `a+b` for inputs in `[-100,100]` | anything beyond that range | yes |
| `verify_sub_no_overflow` | same for subtraction | — | yes |
| `verify_mul_range` | product in `[-16,16]` for inputs in `[-4,4]` | algebraic correctness, rounding, sign | yes |
| `verify_c1_preimage_bound` | adjacent divisors in a 1026-value window at the softening limit give strictly decreasing quotients | entropy, distribution, global bound | yes |
| `verify_c1_epsilon_bound` | `⌈den/2^64⌉ ≤ 8,000,000` and `≥ 1` | **does not call `Fixed::div`** — pure `i128` shift arithmetic | **no** |
| `verify_c1_division_remainder` | `0 ≤ r < den` for `r = (G<<64) % den` | **does not call `Fixed::div`**; says nothing about the implementation | **no** |
| `verify_c3_step_damping` | for 6 concrete large denominators, a 1-ULP divisor change moves the quotient by ≤ 1 ULP | universal quantification, a Verlet step, a preimage set | yes |
| `verify_c3_two_step_dispersion` | a 1-ULP stage-1 quotient difference does not grow through a second division | the same | yes |

## Included harnesses (`proofs/kani/`, 29)

### C1 — saturation model (5, all model-only)

| Harness | Asserts |
|---|---|
| `verify_information_loss_total` | `min(S·40, 3840) ≤ 3840` and `≤ S·40` |
| `verify_information_loss_monotone` | the model is monotone in `S` |
| `verify_saturation_point` | `96·40 = 3840`, `95·40 < 3840` |
| `verify_information_loss_saturating` | capped loss is the cap or the uncapped value |
| `verify_information_loss_cap_boundary` | a larger `k` saturates no later |

None of these executes a rounding operation. They verify the *formula* in
`C1/gap3`, not the claim that the simulation loses 40 bits per step.

### C2 — estimator arithmetic (6; 2 exercise the crate)

| Harness | Asserts | Crate? |
|---|---|---|
| `verify_lyapunov_spectrum_comprehensive` | Padé ratio `(x−1)/(x+1) ∈ [0,1]` for `x ∈ [1,2]` | yes (`Fixed` div) |
| `verify_lyapunov_division` | `λ ∈ [0, ln r · 2^10]` and finite | yes (`Fixed` div) |
| `verify_lyapunov_error_bounds` | `1/t = 0` for `t ≥ 2^54` (plain `i128`) | no |
| `verify_lyapunov_quantization` | same identity | no |
| `verify_shadow_orbit_bias` | `100 ≤ S·dt` for `S ≥ 1000` | no |
| `verify_perturbation_in_range` | `2^40 > 0` and `< 2^64` | no |

Note: `kelvin-kdf/src/lyapunov.rs` contains a real estimator
(`estimate(...)`), but no harness calls it.

### C3 — exponent arithmetic (4, all model-only)

`verify_c3_quantum_reduction`, `verify_c3_exponent_arithmetic`,
`verify_c3_exponent_law`, `verify_c3_contraction_arithmetic` assert
`bits/2 ≤ bits`, `2·960 = 1920`, `2^a·2^b = 2^(a+b)`, and that a non-expanding
window stays bounded. These are arithmetic identities; they are not a query
lower bound and do not reference the simulator.

### C4 — framing (4, all model-only)

`verify_domain_separation` and `verify_framing_deterministic` operate on
`c4_frame`, a function declared in the harness, **not** on
`kelvin-kdf/src/extractor.rs`. `verify_shake256_grover_arithmetic` and
`verify_bound_is_min_of_two` compare constants.

### C5 — branch-free primitives (5, all model-only)

`verify_ct_select_i128`, `verify_ct_select_sign_bits`, `verify_ct_is_zero_u64`,
`verify_ct_select_u64`, `verify_ct_select_u64_is_exact` prove functional
correctness of `ct_select`/`ct_ne` helpers declared in the harness. They do not
prove constant-time execution, and no timing harness exists.

### C6 — cycle detection (5; 2 model-only, 3 exercise the crate)

| Harness | Asserts | Crate? |
|---|---|---|
| `verify_floyd_cycle_detection` | after `k` iterations tortoise `+k`, hare `+2k` | no |
| `verify_hare_leads` | hare stays ahead given `t ≤ h` | no |
| `verify_find_cycle_is_complete_and_correct` | for every step map on 6 states and every start, `find_cycle` returns a cycle: it repeats, its period is minimal, `mu` is the first recurring state | yes |
| `verify_find_cycle_is_sound_for_any_budget` | the same correctness for whatever is reported with a budget of 0 to 6 | yes |
| `verify_find_cycle_zero_budget` | a zero budget calls the step function never and reports `None` | yes |

The last three run `kelvin_core::find_cycle`, bounded to 6 states (the zero
budget one is independent of the state space). None runs the Verlet simulation,
so they say nothing about the period of an orbital configuration.

## Key derivation (`kelvin-kdf`, 6)

| Harness | Asserts | Bound |
|---|---|---|
| `verify_schedule_info_injective` | distinct indices give distinct key and nonce HKDF info strings | all `u64` |
| `verify_schedule_key_and_nonce_info_separated` | no key info string is a prefix of a nonce info string | all `u64` |
| `verify_schedule_first_indices` | consecutive indices, no overflow, key limit, permanent exhaustion | all parameters, first 4 calls |
| `verify_schedule_limit_invariant` | one call from any reachable state keeps `step = keys * interval`, the key limit and the step limits | 16-bit parameters |
| `verify_extraction_header_injective` | equal hash inputs imply equal separator, nonce, step, G, softening | 3-byte separator, nonce up to 3 bytes, no bodies |
| `verify_extraction_body_injective` | equal hash inputs imply equal mass, position, velocity | one body |

They call the code used in production (`claim_index`, `key_info`, `nonce_info`,
`feed_orbital_state`). HKDF-SHA512, BLAKE3 and SHAKE256 are not modelled, so
nothing here shows that distinct inputs give distinct keys.

**Framing limit.** The domain separator is not length-prefixed. Hash inputs are
unambiguous only if no separator is a proper prefix of another; a unit test
shows a collision otherwise, and another checks the library's constants.

`kelvin-kdf` also contains older harnesses (extraction, configuration,
Lyapunov) that the run script does not list. One of them kept the crate from
compiling under Kani until 2026-11-27; they have no recorded verdict.

## Chunk protocol (`kelvin`, 3)

| Harness | Asserts | Bound |
|---|---|---|
| `verify_chunk_prefix_injective` | distinct (index, final flag) give distinct MAC input prefixes | all `u64` |
| `verify_chunk_open_all_or_nothing` | a chunk is accepted only with an open stream, the right version and tag; otherwise data and state are unchanged and nothing is decrypted | any 35 received bytes (2-byte chunk), index below `u64::MAX` |
| `verify_chunk_opens_only_in_place` | a sealed chunk opens only at the same index with the same final flag | 2-byte chunk, indices below `u64::MAX` |

They run the production `chunk_prefix`, `seal_chunk` and `open_chunk`. The last
two replace `compute_tag` with an ideal tag that spells out its inputs, and
replace the keystream with a fixed XOR. They also replace error-message
formatting. So they say nothing about KMAC128, forgery resistance, the MAC key,
the customization strings, timing, or the file and Secure-mode formats.

## Findings

1. **26 of 46 harnesses never touch the crate.** They are executable
   documentation for model arithmetic. That is not worthless, but it cannot
   catch a regression in `kelvin-core`.
2. **Two "core" harnesses are model-only.** `verify_c1_epsilon_bound` and
   `verify_c1_division_remainder` recompute with `i128` shifts and `%`; they do
   not call `Fixed::div`. Documentation that describes them as checking the
   implemented division is too strong.
3. **The protocol layer is barely covered.** `kelvin-kdf` is covered only for
   input construction and key limits (not `lyapunov.rs`), and `kelvin` only for
   the chunk state machine in `authenticated` (not `encrypt` or `decrypt`).
   Those are exactly the layers where nonce reuse and domain-separation bugs
   live.
4. **The C3 harnesses check division rounding on concrete denominators.** They
   show that a one-ULP divisor change moves the quotient by at most one ULP,
   the arithmetic reason rounding is many-to-one. They do not run a Verlet step
   (the symbolic version was intractable for CBMC) and they are not a quantum
   lower bound.
5. **C4 framing is tested against the harness's own model**, so it duplicates
   the Lean lemma rather than validating the Rust framing that actually runs.
6. **C6 checks the algorithm, not the dynamics.** `find_cycle` is verified on
   small state spaces. A green result is not evidence about periodicity of the
   orbital simulation.

## Not covered in this pass

- **Protocol-level proofs are not written yet.** The tooling now exists:
  `docker/docker-compose.formal.yml` builds ProVerif, Tamarin, CryptoVerif and
  EasyCrypt images, and `formal/` holds smoke tests that exercise them. The
  Kelvin-specific models (handshake, nonce uniqueness, domain separation,
  orbital-state compromise) remain to be written. This is the layer where
  nonce-reuse and domain-separation bugs live.
- **Fuzzing was not run.** The repository has four cargo-fuzz targets
  (`fuzz/fuzz_targets/api_encrypt.rs`, `simulate_state.rs`,
  `differential_accel.rs`, `deserialize_config.rs`) and the `kelvin-rust-tools`
  image provides `cargo-fuzz`, but no fuzzing campaign was executed here.
- **Timing was not measured.** The `kelvin-rust-tools` image provides `miri` and
  the Rust analyzers; `dudect`/`ctgrind` are not packaged. The existing DudeCT
  benches cover 12 arithmetic/simulation cases, not the full pipeline.
- **Most of `kelvin` has no Kani harness.** Only the authenticated chunk state
  machine is covered, against an ideal tag.

## Recommended next harnesses

1. `kelvin-cli::file_crypto` — the chunking that decides which chunk is final,
   once separated from file I/O.
2. `verlet_step` semantic pinning: compare against a reference implementation of
   kick-drift-kick in the harness, so the drift ordering is checked in Rust (the
   Lean model currently has no refinement link).
3. `kelvin-kdf::lyapunov::estimate` — range/monotonicity of the estimator, which
   is the code the C2 documentation describes.
4. Timing: keep dudect for measurement; do not create a Kani harness that claims
   to prove constant time.

## Mutation check

`scripts/mutation-check.ps1` copies the tree, injects one defect, and runs the
target harness (the `find_cycle` row was re-checked in Docker on 2026-11-27):

| Mutation | Harness | Expected | Observed |
|----------|---------|----------|----------|
| `Fixed::add` returns `a + b + 1` | `verify_add_no_overflow` | fail (caught) | **FAIL — caught** |
| `Fixed::mul` returns product `+ 1` | `verify_mul_range` | pass (survives — range-only) | **FAIL — caught**, but only at the boundary `a = b = 4`, where `16 << 64 + 1` leaves the asserted interval |
| `find_cycle` returns `lambda + 1` | `verify_find_cycle_is_complete_and_correct` | fail (caught) | **FAIL — caught** |

Interpretation: the harnesses do constrain the code. The `mul` result is a
caution rather than a win — the mutation was detected by the range check, not by
a semantic comparison, so a one-ULP error strictly inside `[-16, 16]` would
survive. That is exactly why `verify_mul_range` should not be described as
functional equivalence.

A row that observes **PASS** would be a weak harness: the defect survives.

Update this table after running the script; the full run takes several minutes
because each mutation recompiles `kelvin-core`.
