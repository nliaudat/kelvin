# Formal / analysis tooling images

One image per tool, all built from this directory and defined in
[`../docker-compose.formal.yml`](../docker-compose.formal.yml). They mount the
repository at `/work`, so models under `formal/` and crates are available
without copying.

They are **analysis tools only** — they do not build or ship Kelvin.

| Service | Image | Tool | Purpose |
|---------|-------|------|---------|
| `proverif` | `kelvin-proverif` | ProVerif | protocol secrecy/authentication (applied pi-calculus) |
| `tamarin` | `kelvin-tamarin` | Tamarin prover | protocol equational reasoning |
| `cryptoverif` | `kelvin-cryptoverif` | CryptoVerif | computational-security game proofs |
| `easycrypt` | `kelvin-easycrypt` | EasyCrypt | game-based cryptographic proofs |
| `smt` | `kelvin-smt` | Z3, CVC5 | SMT backends for the above |
| `rust-tools` | `kelvin-rust-tools` | cargo-fuzz, miri, cargo-audit, cargo-deny, cargo-mutants | Rust analysis/fuzzing |

Alt-Ergo (2.6.0, the version EasyCrypt supports) is installed inside the
`easycrypt` image, not the `smt` image.

## Verified in this environment

Built and smoke-tested on 2026-11-23 (Windows, Docker Desktop, Linux containers):

| Image | Version | Smoke result |
|-------|---------|--------------|
| `kelvin-proverif` | ProVerif 2.05 | `RESULT not attacker(key[]) is true` |
| `kelvin-tamarin` | Tamarin 1.12.0, Maude 3.4 | `key_secret (all-traces): verified` |
| `kelvin-cryptoverif` | CryptoVerif 2.13 | usage/help printed |
| `kelvin-easycrypt` | EasyCrypt r2026.10, Why3 1.8.2, Alt-Ergo 2.6.0 | `smoke.ec` checked; 10 provers detected |
| `kelvin-smt` | Z3 4.8.12, CVC5 1.0.3 | versions printed |
| `kelvin-rust-tools` | nightly 1.102.0, cargo-fuzz 0.13.2, cargo-audit 0.23.0, cargo-deny 0.22.0, cargo-mutants 27.1.0, miri 0.1.0 | versions printed |

Beyond the version checks: `cargo miri test -p kelvin-core --lib` passes 61 tests
with no UB, the four proptest targets in `fuzz/` pass, and `cargo audit` reports
the advisories recorded in
[`../../COMPLIANCE.md`](../../COMPLIANCE.md) §6.

## Build

```bash
docker compose -f docker/docker-compose.formal.yml build
```

All images are independent; build only what you need:

```bash
docker compose -f docker/docker-compose.formal.yml build proverif
```

`proverif`, `cryptoverif` and `easycrypt` share `Dockerfile.ocaml` through
Compose `target:`s, so building several reuses the opam base layer. `easycrypt`
is slow (tens of minutes) and pulls Why3 plus Alt-Ergo; it is pinned to the
`r2026.10` release.

## Run

```bash
# ProVerif model
docker compose -f docker/docker-compose.formal.yml run --rm proverif formal/proverif/kelvin_smoke.pv

# Tamarin model
docker compose -f docker/docker-compose.formal.yml run --rm tamarin --prove formal/tamarin/kelvin_smoke.spthy

# Nonce reuse leaks the message (expected: falsified), fresh nonces do not (verified)
docker compose -f docker/docker-compose.formal.yml run --rm tamarin --prove formal/tamarin/nonce_reuse.spthy
docker compose -f docker/docker-compose.formal.yml run --rm tamarin --prove formal/tamarin/nonce_fresh.spthy

# EasyCrypt script
docker compose -f docker/docker-compose.formal.yml run --rm easycrypt formal/easycrypt/smoke.ec

# SMT solvers
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint z3 smt --version
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cvc5 smt --version
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint alt-ergo easycrypt --version

# Rust analysis
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools fuzz --help
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools audit
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools deny check
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools mutants --list
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools miri test -p kelvin-core --lib

# Property tests (proptest targets in fuzz/)
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint cargo rust-tools test --manifest-path fuzz/Cargo.toml
```

## EasyCrypt prover configuration

EasyCrypt needs a Why3 configuration that lists the detected provers. It is
generated per user, so each fresh container starts without one:

```bash
docker compose -f docker/docker-compose.formal.yml run --rm --entrypoint bash easycrypt \
  -c 'easycrypt why3config'
```

That detects Alt-Ergo 2.6.0, CVC5 and Z3 inside the image. To keep the
configuration across runs, mount a volume for `/root/.config/easycrypt`. The
trivial `formal/easycrypt/smoke.ec` lemma does not require provers, so it passes
without this step.

## Windows helper

`scripts/formal-tools.ps1` builds and smoke-tests the images:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/formal-tools.ps1 -Build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/formal-tools.ps1 -Smoke
```

## Scope

The models under `formal/` are **smoke tests and scaffolds**. They show that the
toolchain works and give a place to write the Kelvin-specific models. They are
not proofs about Kelvin; see
[`../../documentation/formal_verification/PROOF_INDEX.md`](../../documentation/formal_verification/PROOF_INDEX.md)
for what is and is not established.

## Not included

- **SAW / Cryptol** (Galois): a heavy GHC build; use the official image if the
  Rust-to-Cryptol path is needed.
- **dudect / ctgrind**: timing harnesses are normally vendored into the project
  under test, not shipped as a standalone image.
- **Aeneas / Charon**: Rust-to-Lean translation for refinement proofs; separate
  project, separate toolchain.
