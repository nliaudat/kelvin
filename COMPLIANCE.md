# Kelvin — Compliance Register

> Last verified against code and docs: **2026-10-10** (tree incl. PR #80/#81;
> re-verified with this correction pass).
>
> This document replaces `bad.md` (removed 2026-10-10). Every limb of the old
> production-cipher critique is tracked below as **FIXED**, **BOUNDED**, or
> **ACCEPTED LIMITATION** with evidence. Nothing here raises the effective
> security above the stated bound: **128-bit post-quantum via SHAKE256**.

## 1. Security ceiling — BOUNDED

The keystream is SHAKE256 output; no simulation depth raises effective
security past 128-bit PQ (Grover). This is now the *stated* claim everywhere
(`README` mode table, `THREAT_MODEL.md` §3.3, `formal_verification.md` §2,
`stream_cipher_security.md` §8). The old "astronomical keyspace / 2⁹⁶⁰"
framing is removed from all normative statements; remaining 2^1920 figures
are explicitly scoped to the representable (not generated) space.

## 2. KDF cost — ACCEPTED LIMITATION

Setup costs seconds (1M steps) to minutes and is compute-bound with no
memory hardness — weaker per-dollar than Argon2id against ASIC/GPU brute
force. Mitigations in place: `keygen --fast` for non-production use,
Lyapunov-gated minimum steps (no wasted oversimulation), measured costs
published (`README` performance table). Positional statement: the cost is a
sequential work factor, not a memory-hard KDF; do not present it as one.

## 3. Nonce handling — FIXED

Session nonces are bound into every seed extraction
(`extract_*_with_nonce`, `Kelvin::new_with_nonce`,
`KelvinStreaming::new_with_nonce`, CLI `--nonce`):
- CLI encrypt generates a fresh 16-byte nonce by default and prints it.
- CLI decrypt requires `--nonce` (no silent default); `--nonce=`
  explicitly selects pre-nonce legacy output.
- Rule: one (config, nonce) pair per instance. Same-nonce reuse still
  reuses keystream (two-time pad) — documented in `usage.md`.

## 4. Known-plaintext / malleability — FIXED (default-auth)

XOR stream ciphers are malleable and known plaintext reveals keystream —
inherent, shared with all stream ciphers. Guardrail: KMAC128 authentication
is **on by default** for chaos/photon/quantum (`--no-auth` opts out);
V1 Secure is AEAD. Decrypt verifies in constant time and rejects tampering
instead of returning malleated plaintext.

## 5. Production hallmarks — status per item

| Hallmark | Status |
|----------|--------|
| Formal security reduction | ACCEPTED LIMITATION - none; C1-C5 are plausibility arguments (0 proved, 3 partial, 5 empirical, 15 open), not reductions |
| Independent cryptanalysis | ACCEPTED LIMITATION — none to date; adversarial chain (v1–v5) is self-review, not independent audit |
| Standardized primitives | MET where used — SHAKE256, ChaCha20Poly1305, HKDF-SHA512, BLAKE3, ML-KEM-768, ML-DSA-65 |
| Side-channel hardening | PARTIAL — DudeCT \|\|t\| < 5 on arithmetic primitives + extract/simulate/schedule (12 benches, gated in `test_all.bat`); full encrypt/decrypt pipeline not covered |
| Nonce management | FIXED — §3 above |
| Authentication | FIXED — default-on (§4 above) |
| Key setup cost | DOCUMENTED — measured table in `README`; `SECURITY.md` states experimental status |

## Residual risks (must-read before any deployment)

1. Same-(config, nonce) reuse → identical keystream (two-time pad).
2. No memory hardness — budget-constrained defenders lose to ASIC/GPU attackers.
3. No reduction, no independent audit — research-grade, not FIPS-grade.
4. Finite-precision periodicity has no runtime detector (see v1§6 / v5§7).
5. `verlet_step` timing variation (|t| ≈ 75 in one DudeCT configuration) is
   under investigation, not exonerated.

## 6. Dependency advisories — 2026-11-23

Run with `cargo-audit 0.23.0` and `cargo-deny 0.22.0` from the
`kelvin-rust-tools` image:

```bash
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools audit
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools deny check
```

### Vulnerabilities — resolved (was 3)

| Crate | Was | Action | Now |
|-------|-----|--------|-----|
| `pyo3` | 0.24.2 (RUSTSEC-2026-0176, -0177) | bumped to `0.29`; the binding compiled unchanged | 0.29.3 |
| `ml-dsa` | 0.0.4 (RUSTSEC-2025-0144) | removed an unused direct dependency of `kelvin-cli`; `kelvin-kdf` already used `ml-dsa 0.1.2` | 0.1.2 only |

`cargo audit` now reports **0 vulnerabilities** (272 dependencies scanned).

### Warnings — 3 reachable, no fixed upgrade

| Crate | Advisory | Reached through | Status |
|-------|----------|-----------------|--------|
| `ansi_term` | RUSTSEC-2021-0139 | `dudect-bencher` → `clap 2` (`tests/constant_time_bench`) | unmaintained |
| `atty` | RUSTSEC-2024-0375 | same | unmaintained |
| `paste` | RUSTSEC-2024-0436 | `nistrs` → `nalgebra` (`tests/nist_tests`) | unmaintained |

`paste` was removed from first-party code: `kelvin-ffi` now uses the maintained
successor `pastey 0.2`. The remaining `paste` edge is test-only.

All three are benchmark/test dependencies; none is linked by `kelvin-core` or
`kelvin` at runtime. There is no fixed upgrade, so they are listed in
`deny.toml` under `[advisories] ignore` with this justification.

### Licences

`cargo deny check licenses` failed because 11 internal crates under `tests/` had
no `license` field. They now declare `MIT OR Apache-2.0`, matching the project.
`deny.toml` was updated to the cargo-deny >= 0.16 schema (`unmaintained = "all"`;
the obsolete `vulnerability`, `notice` and licenses `deny` keys were removed).

Result: **`advisories ok`, `bans ok`, `licenses ok`, `sources ok`**.

### Property tests (`fuzz/`)

`cargo test --manifest-path fuzz/Cargo.toml` now passes all four proptest
targets: `api_encrypt`, `deserialize_config`, `differential_accel` and
`simulate_state`.

Two defects in the suite itself were fixed on 2026-11-23:

- `api_encrypt` called `KelvinQuantumAuthenticated::with_config`, which no longer
  exists; it now uses `new(seed, max_reseeds)` guarded by `max_reseeds > 0`.
- The authenticated round-trip assertions compared the whole buffer, including
  the 33-byte version + tag area that `encrypt` deliberately overwrites. They now
  compare the plaintext prefix, which is what `decrypt` restores.

The suite had evidently not been run since those APIs changed.

### Miri

`cargo miri test -p kelvin-core --lib` reports **61 passed, 0 failed** (254 s)
with no undefined behaviour.

## Proof-to-Requirement Mapping

None of the entries below is a security proof. "Runnable" artifacts are built by
`lake build` or executed by `docker/run-kani.sh`; the two files at the end are
templates and are **not** compiled.

| Requirement | Runnable artifact | Status |
|-------------|-------------------|--------|
| Security ceiling - BOUNDED | Lean `FixedPoint` (`div_rem_identity`, `rem_bounds`); Kani `verify_c1_preimage_bound`, `verify_c1_epsilon_bound` | arithmetic and preimage bounds only |
| KDF cost - ACCEPTED LIMITATION | none | no artifact; setup cost measured in `README` |
| Nonce handling - FIXED | `tests/kelvin_tests/determinism.rs` | tested, not proved |
| Known-plaintext - FIXED (default-auth) | `kelvin` crate tests | tested, not proved |
| Side-channel hardening - PARTIAL | DudeCT benches (12 cases) | measurement only; full pipeline untested |
| Verlet schedule | Lean `Integrator` (`step_fst`, `step_snd`, `step_reversible`) | proved |
| Pairwise momentum | Lean `Gravity` (`pair_momentum_cancel`) | proved |
| Domain framing | Lean `Entropy` (`domain_separation`) + Kani `verify_domain_separation` | proved (framing only) |

`proofs/kani/fixed_equivalence.rs` and `proofs/kani/pipeline_proofs.rs` are
templates outside every Cargo target. They are not cited as evidence.
