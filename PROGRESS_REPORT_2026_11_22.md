# Kelvin Formal Verification — Progress Report (corrected)

> **Correction (2026-11-23).** The original version of this file reported
> "19/23 gaps resolved", three new Kani harnesses "created", 32 Lean theorems,
> and `lean --init mathlib` as the build command. None of that was accurate: the
> Lean files did not parse, the harnesses were never compiled, and the gap counts
> were not derived from any artifact. The numbers below replace them.

## Current status

| Category | Total | Proved | Partial | Empirical | Open |
|----------|-------|--------|---------|-----------|------|
| C1 | 7 | 0 | 2 | 2 | 3 |
| C2 | 6 | 0 | 0 | 2 | 4 |
| C3 | 3 | 0 | 0 | 0 | 3 |
| C4 | 3 | 0 | 1 | 0 | 2 |
| C5 | 4 | 0 | 0 | 1 | 3 |
| **Total** | **23** | **0** | **3** | **5** | **15** |

No security gap is machine-checked. See
[`documentation/formal_verification/PROOF_INDEX.md`](documentation/formal_verification/PROOF_INDEX.md).

## What exists and runs

### Lean (`proofs/lean`)

Self-contained, no Mathlib:

```bash
cd proofs/lean
lake build
```

50 theorems across the primary modules and the legacy adapters. Proved content
is limited to integer arithmetic, the Verlet update schedule, pairwise momentum
cancellation, framing injectivity, conditional key/XOF uniqueness, Kaplan–Yorke
arithmetic, and Floyd loop invariants.

### Kani (`kelvin-core`)

37 harnesses: 8 in `kelvin-core/src/fixed_math.rs` and 29 in `proofs/kani/*.rs`,
included by `kelvin-core/src/kani_harnesses.rs` under `#[cfg(kani)]`. Run
`docker compose -f docker/docker-compose.yml run --rm kani --list` for the list.

```
verify_add_no_overflow
verify_sub_no_overflow
verify_mul_range
verify_c1_preimage_bound
verify_c1_epsilon_bound
verify_c1_division_remainder
verify_c3_step_damping
verify_c3_two_step_dispersion
```

```bash
docker compose -f docker/docker-compose.yml build kani
docker compose -f docker/docker-compose.yml run --rm kani --list
docker compose -f docker/docker-compose.yml run --rm kani
```

The `--proof NAME` flag now selects a single harness; previously it was parsed
and ignored.

## Corrections applied

- Rewrote the Lean tree whose doc blocks never closed and whose theorems were
  false of their own definitions; it now builds with no external dependencies.
- Rewrote eleven Kani files that were never in a Cargo target and asserted
  locally chosen constants (`C1_verify_*`, `C2_verify_*`, `C3_verify_*`,
  `C4_verify_*`, `C5_verify_*`, `C6_verify_*`, `verify_information_loss_total`,
  `verify_lyapunov_spectrum`, `verify_side_channel`, `verify_periodicity`,
  `verify_c3_quantum_reduction`), and included them from the crate so they run.
- Fixed `docker/run-kani.sh` argument handling and added the two C3 harnesses
  that were already integrated but missing from the run list.
- Rewrote `TODO.md`, `KANI_VERIFICATION_RESULTS.md`,
  `FINAL_SUMMARY_2026_11_22.md`, this file, and the whole of
  `documentation/formal_verification/` to match what the artifacts establish.

## Open work

1. **C4 bound.** Choose between `2^-128` (operational SHAKE256) and `2^-960`
   (Grover over `Θ`), and state the assumption behind the chosen number.
2. **C2 harness.** `lyapunov_proofs.rs` is a template. Move it into a Cargo
   target under `#[cfg(kani)]` and run it, or drop the claim.
3. **C1 accounting.** `kelvin-core/src/information_loss.rs` returns
   `steps * 1.0`; it is a placeholder, not a bound.
4. **C5 side-channel.** No implementation, no measurement, no proof.
5. **C6 periodicity.** `kelvin-core/src/period_detection.rs` has a unit `State`
   and a no-op step; the Lean Floyd invariants do not connect to it.

**Last updated:** 2026-11-23
