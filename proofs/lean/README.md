# Kelvin — Lean 4 proofs

Machine-checked Lean 4 formalisation of the parts of Kelvin that can be stated
as plain integer arithmetic.

## Build

```bash
cd proofs/lean
lake build
```

Requirements: `elan`/`lake` with the toolchain pinned in `lean-toolchain`
(`leanprover/lean4:v4.36.0`). There are **no external dependencies** — in
particular Mathlib is not required, so a clean build needs no multi-gigabyte
download.

A successful build ends with:

```
Build completed successfully (31 jobs).
```

Every theorem is closed with `omega`, `simp`, `decide`, or definitional
equality. There are no `sorry`s, `admit`s, `axiom`s, or `native_decide` uses.

## Checking the proofs (the "test" step)

Lean has no separate runtime test layer: the kernel checks every proof while the
module elaborates, so `lake build` *is* the test. Two scripts wrap that check and
add a guard against escape hatches:

```bash
./check.sh          # POSIX shell / CI
```
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\check.ps1   # Windows
```

They run `lake build`, fail if `sorry` / `admit` / `axiom` / `native_decide`
appears anywhere under `src/`, and print the theorem count.

`src/Sanity.lean` adds kernel-checked `example` declarations for the constants
the documentation quotes (`2^64`, `100 * 2^64`, `96 * 40 = 3840`,
`2 * 960 = 1920`, `i128::MAX/MIN`), so a typo in the prose or a change to the
model cannot silently contradict the numbers in the docs. These are checked by
`lake build` like any theorem.

## Layout

The library has two layers. The primary modules are under `KelvinProofs/`; the
legacy paths (`fixed_point/`, `n_body/`, `dynamics/`, `entropy_extraction/`,
`key_schedule/`) import them and restate the facts under the names the
documentation historically used. Both layers build, so the old file paths in the
docs are real.

| File | Contents |
|------|----------|
| `src/KelvinFormalVerif.lean` | Entry point; imports both layers |
| `src/Sanity.lean` | Kernel-checked constants quoted by the docs |
| `src/KelvinProofs.lean` | Imports every primary module |
| `src/KelvinProofs/FixedPoint.lean` | Q32.64 raw-integer arithmetic |
| `src/KelvinProofs/Integrator.lean` | Velocity Verlet scheduling and reversibility |
| `src/KelvinProofs/Gravity.lean` | Pairwise momentum cancellation |
| `src/KelvinProofs/KeySchedule.lean` | Index encoding; conditional key uniqueness |
| `src/KelvinProofs/Entropy.lean` | Domain-separated framing; conditional XOF separation |
| `src/KelvinProofs/PeriodDetection.lean` | Floyd loop invariants |
| `src/fixed_point/FixedPoint.lean` | Legacy adapter: bounds, division, sqrt postcondition |
| `src/n_body/NewtonianGravity.lean` | Legacy adapter: dot product, action–reaction |
| `src/dynamics/VerletIntegrator.lean` | Legacy adapter: schedule, reversibility |
| `src/dynamics/LyapunovExponents.lean` | Legacy adapter: Kaplan–Yorke arithmetic only |
| `src/entropy_extraction/Shake256Extraction.lean` | Legacy adapter: framing, conditional separation |
| `src/key_schedule/HKDFDerivation.lean` | Legacy adapter: info-string injectivity |

50 theorems in total: 23 in the primary modules, 27 in the entry point and the
legacy adapters.

## What is proved

These are facts about the integer model, and each one is checked by the kernel:

- **`FixedPoint`** — bounded `Int` addition/subtraction cannot overflow `i128`
  for physical-range inputs; Euclidean division satisfies
  `a / b * b + a % b = a`; the remainder lies in `[0, b)`; the square-root
  postcondition `q² ≤ n < (q+1)²` follows from a bounded remainder.
- **`Integrator`** — the drift uses the post-kick velocity; both kicks use the
  half timestep; the kick-drift-kick map is exactly reversible; `simulate` with
  zero steps is the identity.
- **`Gravity`** — the two acceleration contributions to a pair's momentum cancel
  exactly for arbitrary masses; a massless body exerts no acceleration; the dot
  product is symmetric and compatible with scaling.
- **`KeySchedule`** — distinct indices produce distinct info strings; if the PRF
  is injective in its index argument then distinct indices produce distinct
  keys.
- **`Entropy`** — distinct domain tags produce distinct framed inputs; if the
  XOF is injective then distinct tags produce distinct outputs.
- **`PeriodDetection`** — after `k` iterations the tortoise advanced `k` steps
  and the hare `2k` steps.
- **`LyapunovExponents` (legacy)** — the Kaplan–Yorke dimension `j + S/g` is at
  least `j` and non-negative when `S ≥ 0` and `g > 0`. Nothing about the
  measured exponent.

## What is *not* proved

This is important, and it is the honest scope of the suite:

- **No cryptographic claim is machine-checked.** SHAKE256 preimage resistance,
  HMAC-SHA512 PRF security, and the keystream indistinguishability bound
  (`Adv ≤ negl(n) + 2^-128`) are **assumptions**, not theorems. The conditional
  lemmas above make the assumptions explicit; they do not discharge them.
- **No entropy accounting is proved.** The C1 "bits lost per step" bounds are
  not derived here. `FixedPoint.div_rem_identity` is a division identity, not an
  entropy statement.
- **No Lyapunov / Kaplan–Yorke statement about Kelvin is proved.** The legacy
  Lyapunov file proves one arithmetic inequality about the Kaplan–Yorke formula;
  it does not show `λ > 0` or `D_KY ∈ [3,4]`.
- **No quantum lower bound is proved.** The Grover/Ambainis argument remains an
  informal argument in `documentation/formal_verification/C3/`.
- **The model is not a refinement of the Rust code.** These files model the
  update *schedules* and integer invariants. They do not extract or refine
  `kelvin-core`; the link to the implementation is by inspection, not by a
  verified translation.

Anything not listed under "What is proved" should be treated as unproven.

## History

An earlier revision of this directory contained seven files that (a) used a bare
slash-dash opener where a comment terminator was required, so the entire file
parsed as a comment, (b) imported Mathlib, which is not vendored, and (c) stated
theorems that were false of their own definitions (for example a `simulate` that
returned its input, a `derive_key` that ignored the key index, and a
Kaplan–Yorke "theorem" discharged by `use 3.5`). Those files have been rewritten
as the legacy adapters listed above; they now compile and state only what holds.
