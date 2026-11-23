# Kani Verification Results

> **Correction (2026-11-23).** This file previously claimed that three new
> harnesses (`C1_verify_information_loss_total`, `C2_verify_lyapunov_spectrum`,
> `C3_verify_quantum_reduction`) had been "VERIFIED SUCCESSFULLY", with "6/6
> harnesses" passing each. That was false.
>
> - The files were not part of a Cargo target, so `cargo kani` never compiled
>   them.
> - `docker/run-kani.sh` ignored `--proof`, so each reported "run" executed the
>   same pre-existing harnesses regardless of the name passed.
> - Several files asserted constants chosen inside the harness (for example
>   `assert!(QUANTUM_QUERY_LOWER_BOUND > 10)`), which verifies nothing about
>   Kelvin.
>
> The files have since been restored, rewritten to state properties Kani can
> actually check, and wired into the crate through
> `kelvin-core/src/kani_harnesses.rs`. Nothing was deleted permanently.

## What runs

37 harnesses are compiled into `kelvin-core`:

- **8** in `kelvin-core/src/fixed_math.rs` under `#[cfg(kani)]`;
- **29** in `proofs/kani/*.rs`, included by `kelvin-core/src/kani_harnesses.rs`
  under `#[cfg(kani)]`.

```bash
docker compose -f docker/docker-compose.yml build kani
docker compose -f docker/docker-compose.yml run --rm kani --list
docker compose -f docker/docker-compose.yml run --rm kani --proof <name>
docker compose -f docker/docker-compose.yml run --rm kani          # all 37
```

Per-harness logs land in `proofs/kani/results/`.

## What they check

### Arithmetic and range (`fixed_math.rs`)

| Harness | Claim checked |
|---------|---------------|
| `verify_add_no_overflow` | `i128` addition is in range for physical-domain inputs |
| `verify_sub_no_overflow` | `i128` subtraction is in range for physical-domain inputs |
| `verify_mul_range` | the fixed-point multiply stays within its documented range |
| `verify_c1_preimage_bound` | preimage size of the division near the softening limit |
| `verify_c1_epsilon_bound` | preimage bound for the implemented division |
| `verify_c1_division_remainder` | `0 ≤ remainder < denominator` |
| `verify_c3_step_damping` | for 6 concrete large denominators, a 1-ULP divisor change moves the quotient by at most 1 ULP |
| `verify_c3_two_step_dispersion` | a 1-ULP stage-1 quotient difference does not grow through a second division |

### Saturation model (C1)

`verify_information_loss_total`, `verify_information_loss_monotone`,
`verify_saturation_point`, `verify_information_loss_saturating`,
`verify_information_loss_cap_boundary`: the model `L = min(S·k, H_max)` never
exceeds `H_max`, is monotone in `S`, and saturates at 96 steps for `k = 40`.

### Estimator arithmetic (C2)

`verify_lyapunov_spectrum_comprehensive`, `verify_lyapunov_division`,
`verify_lyapunov_error_bounds`, `verify_lyapunov_quantization`,
`verify_shadow_orbit_bias`, `verify_perturbation_in_range`: the Padé ratio lies
in `[0,1]`, `λ = ln(ratio)/time` is in `[0, ln(ratio)]`, the quantization term
is below one raw unit, and the shadow-orbit budget is positive.

### Exponent arithmetic (C3, C4)

`verify_c3_quantum_reduction`, `verify_c3_exponent_arithmetic`,
`verify_c3_exponent_law`, `verify_c3_contraction_arithmetic`,
`verify_shake256_grover_arithmetic`, `verify_bound_is_min_of_two`:
`2^a·2^b = 2^(a+b)`, `sqrt(2^1920)` has exponent 960, Grover on 256 bits leaves
128, and the operational bound is the smaller exponent.

### Framing and branch-free primitives (C4, C5)

`verify_domain_separation`, `verify_framing_deterministic`,
`verify_ct_select_i128`, `verify_ct_select_sign_bits`, `verify_ct_is_zero_u64`,
`verify_ct_select_u64`, `verify_ct_select_u64_is_exact`: distinct tags give
distinct framed inputs, and branch-free select/inequality return the expected
value for every input.

### Floyd invariants (C6)

`verify_floyd_cycle_detection`, `verify_hare_leads`,
`verify_periodicity_detection`, `verify_periodicity_no_steps`,
`verify_periodicity_min_period_bound`: after `k` iterations the tortoise
advanced `k` and the hare `2k`; the current `period_detection` stub is detected
as a period-1 cycle.

## What is *not* verified by Kani

- C1 per-operation or per-step entropy loss.
- C2 Lyapunov exponent, spectrum, or Kaplan–Yorke bound.
- C3 Grover/Ambainis query lower bound (only its exponent arithmetic).
- C4 keystream indistinguishability / advantage bound.
- C5 configuration-space cardinality, and constant-time execution (only
  branch-free primitives are checked; timing needs dudect/ctgrind).
- C6 period length of the real simulator (the stub has period 1).

The remaining `proofs/kani/*.rs` files that are not in the list above
(`acceleration_proofs.rs`, `lyapunov_proofs.rs`, `configuration_proofs.rs`,
`extraction_proofs.rs`, `fixed_equivalence.rs`, `fixed_safety.rs`,
`information_loss.rs`, `integrator_proofs.rs`, `keystream_proofs.rs`,
`orbital_state_proofs.rs`, `pipeline_proofs.rs`, `quantum_hardness.rs`,
`stability_proofs.rs`, `vec3_proofs.rs`) are **templates**: not included by
`kani_harnesses.rs`, never compiled, and not to be cited as verified.

## Historical note

An earlier revision of this file listed "20 Kani harnesses", "18 verified today",
"24/25 gaps proved". None of those numbers came from a tool. The honest totals
are: **37 runnable harnesses**, **0 proved security gaps**, 3 partial, 5
empirical, 15 open. See
[`documentation/formal_verification/PROOF_INDEX.md`](documentation/formal_verification/PROOF_INDEX.md).

**Last updated:** 2026-11-23
