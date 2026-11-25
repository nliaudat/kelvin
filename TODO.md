# TODO List

Status vocabulary:

- **Proved** — there is a machine-checked artifact (Lean theorem or integrated
  Kani harness) that establishes the stated claim.
- **Partial** — a strictly weaker, related fact is machine-checked; the security
  claim itself is not.
- **Empirical** — supported by measurement or literature, not formally proved.
- **Open** — no proof artifact.

> **Correction (2026-11-23).** An earlier revision of this file claimed all six
> items were "VERIFIED" by six new Kani harnesses. That was wrong: those files
> were never part of a Cargo target, the `--proof` flag was ignored by
> `docker/run-kani.sh` (so the reported runs actually executed the six
> pre-existing harnesses), and several of them asserted constants chosen inside
> the test. The files have been removed. See
> `documentation/formal_verification/README.md` for the current, accurate status.

## Status

| # | Item | Status | Evidence |
|---|------|--------|----------|
| 1 | C2 — Lyapunov spectrum | **Open** | No formal model of the estimator. `proofs/kani/lyapunov_proofs.rs` is an unintegrated template. |
| 2 | C4 — Keystream bound | **Open** | Informal game-based argument only; `Adv ≤ negl(n) + 2^-128` is not proved. |
| 3 | C5 — Side-channel hardening | **Open** | Not implemented, not measured, not proved. |
| 4 | C1 — Information-loss aggregation | **Partial** | `FixedPoint.div_rem_identity` / `rem_bounds` are proved; no entropy accounting. |
| 5 | C3 — Quantum reduction | **Open** | Informal Grover/Ambainis argument only. |
| 6 | C6 — Periodicity detection | **Partial** | Floyd loop invariants proved in `PeriodDetection.lean`; `kelvin-core::find_cycle` / `detect_orbital_period` now implement exact-equality Floyd on the real Verlet simulator (unit-tested on toy maps; not Kani/Lean-proved). The legacy `detect_period` stub remains for the existing harnesses. |

## Machine-checked artifacts (2026-11-23)

### Lean (`proofs/lean`, `lake build`)

Self-contained, no Mathlib. 50 theorems (23 primary + 27 legacy adapters), no
`sorry`/`axiom`/`native_decide`.

| Module | Proved |
|--------|--------|
| `FixedPoint.lean` | `i128` overflow-freedom for bounded add/sub; `a / b * b + a % b = a`; remainder range |
| `Integrator.lean` | drift uses post-kick velocity; half-step kicks; exact reversibility; `simulate 0 = id` |
| `Gravity.lean` | pairwise momentum cancellation; zero-mass body exerts no force |
| `KeySchedule.lean` | distinct indices ⇒ distinct info strings; conditional key uniqueness |
| `Entropy.lean` | distinct tags ⇒ distinct framed inputs; conditional XOF separation |
| `PeriodDetection.lean` | tortoise advances `k`, hare advances `2k` after `k` iterations |

### Kani (`docker/run-kani.sh`)

37 harnesses, all compiled into `kelvin-core`:

- 8 in `kelvin-core/src/fixed_math.rs` under `#[cfg(kani)]`;
- 29 in `proofs/kani/*.rs`, included by `kelvin-core/src/kani_harnesses.rs`
  under `#[cfg(kani)]`.

```
# fixed_math.rs
verify_add_no_overflow
verify_sub_no_overflow
verify_mul_range
verify_c1_preimage_bound
verify_c1_epsilon_bound
verify_c1_division_remainder
verify_c3_step_non_injective
verify_c3_two_step_preimage_growth

# included from proofs/kani/
verify_information_loss_total          verify_information_loss_monotone
verify_saturation_point                verify_information_loss_saturating
verify_information_loss_cap_boundary   verify_lyapunov_spectrum_comprehensive
verify_lyapunov_division               verify_lyapunov_error_bounds
verify_lyapunov_quantization           verify_shadow_orbit_bias
verify_perturbation_in_range           verify_c3_quantum_reduction
verify_c3_exponent_arithmetic          verify_c3_exponent_law
verify_c3_contraction_arithmetic       verify_domain_separation
verify_framing_deterministic           verify_shake256_grover_arithmetic
verify_bound_is_min_of_two             verify_ct_select_i128
verify_ct_select_sign_bits             verify_ct_is_zero_u64
verify_ct_select_u64                   verify_ct_select_u64_is_exact
verify_floyd_cycle_detection           verify_hare_leads
verify_periodicity_detection           verify_periodicity_no_steps
verify_periodicity_min_period_bound
```

```bash
docker compose -f docker/docker-compose.yml build kani
docker compose -f docker/docker-compose.yml run --rm kani --list
docker compose -f docker/docker-compose.yml run --rm kani --proof <name>
docker compose -f docker/docker-compose.yml run --rm kani          # all 37
```

These check arithmetic, scheduling, framing and loop invariants. They do
**not** establish the C1 entropy bounds, the C2 spectrum, the C3 quantum lower
bound, the C4 advantage bound, constant-time execution, or a long period in the
real simulator.

## Open work

### 1. C2 — Lyapunov spectrum
- Integrate `lyapunov_proofs.rs` into `kelvin-kdf` under `#[cfg(kani)]`.
- Formalise the Padé error bound against a reference `ln`.
- Alternatively, state the discrete-vs-continuous bound as an assumption and
  document it as such.

### 2. C4 — Keystream indistinguishability
- Write the game sequence and the two hybrid bounds explicitly.
- Decide the operational bound (`2^-128` vs `2^-960`) and state the SHAKE256
  assumption it rests on.
- Do not describe it as proved until a checked artifact exists.

### 3. C5 — Side-channel hardening
- Audit `kelvin` encrypt/decrypt for secret-dependent branches.
- Add `subtle`-based constant-time primitives where needed.
- Add dudect/ctgrind measurement before claiming constant-time behaviour.

### 4. C1 — Entropy accounting
- Replace the placeholder `kelvin-core/src/information_loss.rs` (which returns
  `steps * 1.0`) with a derived bound, or delete the public export.
- State the entropy model explicitly (distribution, per-operation rounding).

### 5. C3 — Quantum reduction
- Keep the argument in prose; it depends on Grover optimality and the
  configuration-space size, neither of which is formalised.

### 6. C6 — Periodicity detection
- Done: `find_cycle` / `detect_orbital_period` run Floyd on the real simulator.
- Remaining: repoint the Kani harnesses at `find_cycle` and drop the unit-`State` stub
  (`detect_period`); run `detect_orbital_period` on real configurations and
  document the observed periods.

## Cleanup

- `kelvin-core/src/quantum.rs` and `kelvin-core/src/reference.rs` are
  placeholders exported from `lib.rs`; review before publishing an API.
- The remaining `proofs/kani/*.rs` files are templates, not compiled. The run
  script says so; the documentation should not imply otherwise.
