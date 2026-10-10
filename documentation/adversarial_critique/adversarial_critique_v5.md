# Kelvin — Adversarial Academic Critique v5 — Post-June Delta Review

> **Context:** First critique since v4 (2026-06-09). Covers merged changes PR #80 (session nonces, Lyapunov `ln(10)` fix) and PR #81 (documentation review), plus audit findings from the October `formal_verification/` review.
> **Method:** Delta review — only what changed or was newly discovered since v4, in this chain's format. Carried items reference their original points.
> **Severity:** 🔴 Fatal to the formal security argument · 🟠 Major · 🟡 Moderate · 🟢 Fixed/positive
> **Date:** 2026-10-10

---

## Summary of Status

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | Session-nonce misuse semantics (empty-nonce downgrade, same-nonce reuse) | 🟠 | ACKNOWLEDGED — documented, fail-open on misuse |
| 2 | `ln(10)` fix shifts λ and all horizons calibrated on old values | 🟡 | FIXED in code, erratum in docs; downstream rechecks open |
| 3 | 8-body standard vs N=5 numerics across C1–C5 | 🟠 | ACKNOWLEDGED — scope notes added, re-derivation open |
| 4 | `ml-dsa` 0.1.0-rc.4 → stable 0.1.1 | 🟢 | FIXED — tests green |
| 5 | C1 `H_max` overcount (3840 raw vs ≤2880 usable) propagates to saturation | 🔴 | ACKNOWLEDGED — needs re-derivation, not docs polish |
| 6 | C4 separator table strings don't exist; Newton indices wrong in `code_verification.md` | 🟠 | ACKNOWLEDGED — mechanical fixes open |
| 7 | Finite-precision periodicity still unresolved (carried v1§6) | 🟡 | OPEN — FPPC detector proposed, unimplemented |
| 8 | Full `test_all.bat` green on merged tree | 🟢 | VERIFIED 2026-10-10 |

---

### 🟠 1. [ACKNOWLEDGED] Session Nonces Move the Foot-Gun, They Don't Remove It

v1§20's warning-only resolution has been superseded by a real guardrail (PR #80): the nonce is hashed into every seed extraction, same config + distinct nonces = independent keystreams (verified end-to-end: determinism, separation, round-trip).

The adversarial reading: the failure mode moved, it did not disappear.
- **Empty nonce = legacy keystream.** The default is the dangerous behavior; safety requires an affirmative, unique choice per deployment.
- **Same-nonce reuse = two-time pad**, identical to the old same-config reuse. Nothing in the API prevents it; `new_with_nonce(config, &[])` compiles and runs.
- **The nonce is load-bearing public metadata.** Lose it and data is undecryptable; reuse it and data is breakable. It must be transported alongside the config with uniqueness guarantees — i.e. nonce management, the exact burden nonces were supposed to carry all along.

**Status:** documented (`bad.md` §3, `usage.md` "Session Nonces"), behavior verified. Fail-open on misuse. A future `keygen --nonce` that mints and prints a fresh nonce alongside each config would make the safe path the default path.

**Sources:** `kelvin-kdf/src/extractor.rs`, `kelvin/src/lib.rs`, `kelvin-cli/src/main.rs` (`--nonce`)

---

### 🟡 2. [FIXED, rechecks open] The `ln(10)` Fix Moves Every Calibrated Horizon

The estimator's hardcoded `ln(10)` was 2.152 instead of 2.302585 — λ underestimated ~6.5% per decade, horizons inflated (271 > 200 steps broke `test_vector_kelvin_v1` on pristine `main`). Fixed to `0x4D763776AAA2C000` (ULP-exact); test config bumped 200 → 1000 steps; full suite green.

The adversarial reading: every number ever calibrated against estimator output shifted — `min_chaos_steps` margins, `FAST_STEPS` rationale, the C2 λ ≈ 0.693 empirical claims (all pre-fix, only gap2 carries an erratum). Nothing else has been re-measured. The direction of the old error was fail-closed (reject valid configs), so no unsafe acceptance occurred — but any *tight* margin argument (e.g. a 500-step test config against a ~254 horizon) deserves re-examination under the corrected estimator.

**Status:** code fixed, erratum recorded (`C2/gap2`), suite green. Downstream re-measurement open.

**Sources:** `kelvin-kdf/src/lyapunov.rs:230`, `tests/kelvin_tests/test_vectors.rs:45`

---

### 🟠 3. [ACKNOWLEDGED] The Standard Is 8-Body; The Proofs Are 5-Body

Current `standard`/`paranoid` keygen emits 8 bodies. Every worked number in C1–C5 (H_max = 3840, k_step ≥ 40, T_sat ≈ 96, λ ≈ 0.693/yr, D_KY estimates, 30-DOF framing) assumes N = 5. An 8-body system has a different Lyapunov spectrum and mixing time; none of the figures transfer automatically. Scope notes have been added (`formal_verification/README.md`, keyspace doc), but the re-derivation for N = 8 does not exist.

**Status:** acknowledged in docs, re-derivation open. Any security argument quoting N = 5 figures for N = 8 deployments is unsupported.

---

### 🟢 4. [FIXED] `ml-dsa` Pinned to a Release Candidate

`kelvin-kdf` pinned `ml-dsa 0.1.0-rc.4` (Jan 2026); stable `0.1.0` (May 2026) and `0.1.1` (June 2026) shipped upstream. Bumped to `"0.1"` → resolves to 0.1.1; `cargo tree` confirms `kelvin-kdf → ml-dsa 0.1.1 + ml-kem 0.3.2`; asymmetric sign/verify tests green in the full suite. (`kelvin-cli` still pins `ml-dsa 0.0` — separate, tracks it.)

---

### 🔴 5. [ACKNOWLEDGED] C1's State Space Overcounts by 960 Bits

`H_max = 6N×128 = 3840` counts raw `i128` storage per component. Q32.64 usable range is 96 bits/component → `H_max ≤ 30×96 = 2880`, which also moves T_sat 96 → ~72 steps. The 960-bit overcount is exactly the figure elsewhere quoted as KY entropy — the foundation (C1) and the ceiling (C2/C3) cannot both be right as stated. This needs re-derivation of the saturation chain, not a wording fix.

**Sources:** `C1/readme.md:14-20`, `C1/gap3_saturation_bound.md`

---

### 🟠 6. [ACKNOWLEDGED] Fictional Separators, Wrong Indices

C4/gap2's domain-separation table lists six separator strings, zero of which occur in `parameters.rs` (real: `kelvin-photon-keystream-v1`, …), claims unconditional `≠` (pigeonhole forbids it), and counts "6 pairs" for 6 modes (C(6,2) = 15). `code_verification.md` states the Newton check as `m1*a_01 == -m2*a_10` (correct: `m0*a_01 == -m1*a_10`) and the mass ratio as `m2/m1` (correct: `m1/m0`). Mechanical fixes, all grep-verifiable — open at the time of writing.

---

### 🟡 7. [OPEN, carried v1§6] Periodicity Has No Detector

Unchanged: no lower bound on Q32.64 cycle length, no runtime detection, reseeding cannot break deterministic cycles. The Brent-checkpoint FPPC proposal (Oct 2026) is the first concrete design offered; it remains unimplemented. This is still the single most cryptographic-fatal open item: everything else in the chain assumes extraction precedes cycling.

---

### 🟢 8. [VERIFIED] Full Suite Green on the Merged Tree

`scripts/test_all.bat` (release builds, clippy, fmt, 215 lib tests, all integration suites incl. 8/8 golden vectors, entropy/NIST/DudeCT/zeroize): **ALL TESTS PASSED** 2026-10-10 on PR #80 + #81. The engineering substrate — determinism, round-trips, constant-time arithmetic, secret zeroization — is in its best-verified state to date.

---

## Synthesis

v4's verdict (see "Synthesis" in `adversarial_critique_v4.md`) stands with one amendment: the chain's *implementation* half keeps getting stronger (nonces, ln10, green suite) while its *proof* half keeps springing arithmetic leaks (H_max, separators, indices). The honest summary from v4VI ("correctly implemented; security bound = SHAKE256's 128-bit") is unaffected by everything in v5 — none of the new findings raise the ceiling, and none lower it either. The work that would actually move the needle remains v1§6: a periodicity detector.
