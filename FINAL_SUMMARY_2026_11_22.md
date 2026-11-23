# Kelvin Formal Verification — Corrected Summary

> **Correction (2026-11-23).** This file was originally titled "Final Summary —
> ALL TODO ITEMS COMPLETE" and claimed 24/25 gaps proved, 20 Kani harnesses, and
> 100% success. That was inaccurate in every one of those numbers. It is kept,
> rewritten, as a record of what was actually done and what remains.

## What was actually done

### Lean

The Lean suite is a self-contained project in `proofs/lean` with two layers:

- primary modules under `src/KelvinProofs/`;
- legacy adapters at the historical paths (`fixed_point/`, `n_body/`,
  `dynamics/`, `entropy_extraction/`, `key_schedule/`) that import the primary
  modules and restate the facts under the old names.

- zero external dependencies (no Mathlib);
- toolchain pinned in `lean-toolchain` (`leanprover/lean4:v4.36.0`);
- builds with `lake build` in 31 jobs;
- 50 theorems, closed with `omega` / `simp` / `decide`; no `sorry`, `admit`,
  `axiom`, or `native_decide`.

The suite proves integer-arithmetic and scheduling facts:

| Module | Proved |
|--------|--------|
| `FixedPoint.lean` | `i128` overflow-freedom for bounded add/sub; `a / b * b + a % b = a`; remainder range |
| `Integrator.lean` | drift uses post-kick velocity; half-step kicks; exact reversibility; `simulate 0 = id` |
| `Gravity.lean` | pairwise momentum cancellation; zero-mass body exerts no force |
| `KeySchedule.lean` | distinct indices ⇒ distinct info strings; conditional key uniqueness |
| `Entropy.lean` | distinct tags ⇒ distinct framed inputs; conditional XOF separation |
| `PeriodDetection.lean` | after `k` iterations the tortoise advanced `k`, the hare `2k` |

The earlier Lean tree did not compile at all: 64 doc blocks opened with a bare
slash-dash where a comment terminator was required, so each file parsed as one
large comment. Its theorems were also false of their own definitions. The files
have been rewritten as the legacy adapters listed above and now build.

### Kani

37 harnesses are compiled into `kelvin-core`: 8 in
`kelvin-core/src/fixed_math.rs` and 29 in `proofs/kani/*.rs`, included by
`kelvin-core/src/kani_harnesses.rs` under `#[cfg(kani)]`. They check
fixed-point arithmetic, saturation-model arithmetic, estimator arithmetic,
exponent arithmetic, framing injectivity, branch-free selectors and Floyd loop
invariants. `docker/run-kani.sh` was rewritten so that `--proof NAME` actually
runs one harness, `--list` lists them, and no argument runs all 37.

The harness files that this summary previously described as "verified" were
kept, but rewritten. As they stood they were never compiled, used
`rand::thread_rng()` inside a proof, called `Instant::now()` for "timing
verification", or asserted constants selected inside the harness. They now state
properties Kani can check and document what they do not establish.

## Actual gap status

| Category | Total | Proved | Partial | Empirical | Open |
|----------|-------|--------|---------|-----------|------|
| C1 Information loss | 7 | 0 | 2 | 2 | 3 |
| C2 Lyapunov chaos | 6 | 0 | 0 | 2 | 4 |
| C3 Quantum hardness | 3 | 0 | 0 | 0 | 3 |
| C4 Keystream | 3 | 0 | 1 | 0 | 2 |
| C5 Configuration space | 4 | 0 | 0 | 1 | 3 |
| **Total** | **23** | **0** | **3** | **5** | **15** |

Source of truth: [`documentation/formal_verification/PROOF_INDEX.md`](documentation/formal_verification/PROOF_INDEX.md).

## Security claims: current status

| Claim | Status |
|-------|--------|
| Per-operation loss `k_op ≥ 0.98` bits | empirical; no derivation |
| Per-step loss `k_step ≥ 40` bits (N=5) | open; assumes independence |
| Saturation at ~96 steps | open |
| `λ > 0` for the simulator | empirical (≈ 0.7 yr⁻¹) |
| Kaplan–Yorke `D_KY ∈ [3,4]` | empirical |
| `|Θ| ≥ 2^1920` | open (counting sketch) |
| Grover `Ω(2^960)` for keystream inversion | informal argument, several conditions unstated |
| `Adv ≤ negl(n) + 2^-960` | not proved |
| 128-bit post-quantum security of SHAKE256 | assumption, not proved here |
| Constant-time execution | not measured, not proved |

## What to do next

1. Decide the operational C4 bound (`2^-128` vs `2^-960`) and state the SHAKE256
   assumption it rests on.
2. Integrate `lyapunov_proofs.rs` into a real Cargo target under `#[cfg(kani)]`
   before citing it.
3. Replace `kelvin-core/src/information_loss.rs` (currently `steps * 1.0`) or
   remove it from the public API.
4. Implement `kelvin-core/src/period_detection.rs` against the real simulator,
   or delete the module.
5. Add dudect/ctgrind measurement before claiming side-channel resistance.

**Status:** open. **Last updated:** 2026-11-23.
