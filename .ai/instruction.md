./ai/instruction.md

===============================================================================
KELVIN PROJECT — AI CONTEXT SEED
===============================================================================

WHAT THIS IS:
Drop this file into a new AI session to restore all project context,
decisions, and constraints. Read it, then ask how you can help.

===============================================================================

PROJECT SUMMARY

Kelvin is a Rust cryptosystem. It uses chaotic 3D n-body gravitational
simulation as a KDF (Phase 1), feeding ChaCha20 for bulk keystream
generation (Phase 2).

The shared secret is an OrbitalConfig: masses, 3D positions, 3D velocities
for n bodies. Brute-forcing requires running the full simulation per guess.

Status: Implementation complete. All 181 tests pass across 4 crates.

For operational instructions (build commands, Windows quirks, CLI usage),
see `.clinerules`.

===============================================================================

RUST BEST PRACTICES (MANDATORY)

- no_std in kelvin-core (pure math, no allocator needed)
- #![forbid(unsafe_code)] everywhere unless FFI boundary
- All public types: Debug, Clone, PartialEq, Eq (no PartialOrd on secrets)
- Error handling: thiserror for library, anyhow for CLI only
- Zeroize all secret material on drop
- Constant-time comparisons for sensitive values (subtle crate)
- Serde for config serialization; skip serializing on Drop types
- Workspace structure: one Cargo.toml at root, members per crate
- clap derive for CLI
- criterion for benchmarks, proptest for property tests
- rustfmt + clippy (strict) in CI
- Safety docs on all pub fn that touch secrets
- Dead code forbidden (no `#[allow(dead_code)]`, no unused imports/variables/fields)
- **No `#[allow(...)]` attributes in any Rust source file** — use `[lints.clippy]` in `Cargo.toml` instead. Exception: test/analysis crates in `tests/` may use `[lints.clippy]` in their `Cargo.toml` for legitimate f64 math operations (NIST tests, Lyapunov exponents, entropy analysis, etc.)

===============================================================================

ARCHITECTURE (TWO-PHASE, NON-NEGOTIABLE)

Phase 1 (kelvin-core + kelvin-kdf):
  Fixed-point n-body sim -> SHA3-512 hash -> 256-bit master seed
  Slow, sequential, chaotic. No parallelism. No floating-point.

Phase 2 (kelvin-stream):
  ChaCha20 stream cipher driven by master seed
  Fast, standard, hardware-accelerated where available

Never merge these phases. The design depends on their separation.

===============================================================================

WORKSPACE CRATES

kelvin-core/        no_std, no alloc. Fixed math + integrator + body types.
kelvin-kdf/         Depends on core. Config, Lyapunov estimator, extraction.
kelvin-stream/      Depends on nothing from kelvin. ChaCha20 (+ optional AES).
kelvin/             Orchestrator. Depends on kdf + stream. Public API.
                    Contains 5 modes: Secure (V1), Chaos (V2), Photon (V3),
                    Quantum (H), Prism (HE OTP key generator).
kelvin-cli/         Binary. Depends on kelvin. Keygen, encrypt/decrypt.
kelvin-ffi/         C ABI exports. For mobile bindings.
tests/kelvin-test-server/ Test vector golden file server (serde feature).
tests/kelvin-test-client/ Test vector verification client (serde feature).

Tests live in each crate's tests/ + workspace tests/ for integration.
Known-answer vectors in tests/vectors/ (golden files, checked into git).

===============================================================================

KEY TECHNICAL DECISIONS

Math:    Q32.64 fixed-point (i128). 32 int bits, 64 frac bits.
         NO FLOATING POINT ANYWHERE IN CORE. Determinism depends on this.

Integra- Symplectic Verlet (default) or explicit Euler (`--euler` flag).
tion:    Euler's numerical instability amplifies chaos ~10x faster.
         O(n²) pairwise, n <= 7. Softening factor ε prevents singularity.

Hash:    SHA3-512 (Keccak). Domain separator b"kelvin-orbital-state-v1".
         Step counter in hash input. First 256 bits become master key.

Cipher:  ChaCha20 (chacha20 crate). 256 GB/seed then reseed.
         96-bit nonce: 64-bit counter + 32 zero pad.

Lyapunov: Shadow orbit method. Conservative 10% margin.
          Simulation MUST NOT exceed safe_steps. Hard error if exhausted.

Config:  Serde JSON or compact binary. Contains all orbital params.
         estimate_setup_time() runs benchmark on current hardware.

===============================================================================

NON-NEGOTIABLE SECURITY PROPERTIES

1. Deterministic output on x86_64, aarch64, wasm32 (tested in CI)
2. No simulation shortcut (step n+1 requires step n)
3. Constant-time on all secret-dependent paths
4. Forward secrecy via reseeding (old state overwritten)
5. Zeroize on drop (all buffers, all state)
6. "EXPERIMENTAL" warning on all pub entry points

===============================================================================

PUBLIC API (kelvin crate)

  Kelvin::new(config: OrbitalConfig) -> Result<Self, KelvinError>
  Kelvin::new_with_method(config, IntegrationMethod) -> Result<Self, KelvinError>
  Kelvin::encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError>
  Kelvin::decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError>
  Kelvin::bytes_processed(&self) -> u64
  Kelvin::remaining_safe_bytes(&self) -> u64

  KelvinStreaming::new(config, bytes_per_step) -> Result<Self, KelvinError>
  KelvinStreaming::new_with_method(config, bytes_per_step, IntegrationMethod) -> Result<Self, KelvinError>

  simulate_and_extract_seed_with_method(config, IntegrationMethod) -> ([u8; 2048], Vec<OrbitalBody>)

  IntegrationMethod enum: Verlet (default), Euler

CLI: --euler flag on encrypt/decrypt selects Euler integration.
     Default (no flag) uses Verlet. Must match between encrypt/decrypt.

Encrypt and decrypt are the same operation (XOR with keystream).

===============================================================================

FILES IN REPO

documentation/implementation_plan.md    Full specification (long, detailed)
project_summary.md        Concise overview (shorter)
ai/instruction.md         This file (session restoration)
.clinerules               AI tool operational instructions (build, CLI, Windows)

===============================================================================

CURRENT STATE

Implementation complete. All 181+ tests pass across 4 crates.
- kelvin-core: 61 tests (fixed math, body, integrator, constants, stability)
- kelvin-kdf: 64 tests (config, Lyapunov, extractor, schedule, asymmetric)
- kelvin-stream: 10 tests (ChaCha20 stream cipher, AEAD, rekey)
- kelvin: 46 tests (photon/quantum stream, authenticated encryption, streaming)

Additional test infrastructure:
- fuzz/ — cargo-fuzz targets for config deserialization, simulation state, API encrypt, differential acceleration
- tests/constant_time_bench/ — dudect-bencher suite for constant-time verification of euler_step, verlet_step, simulate, extract_seed
- tests/differential_fuzzing/ — Python reference model for differential fuzzing
- tests/entropy_analysis/ — entropy quality analysis scripts
- tests/nist_tests/ — NIST statistical test suite integration

Formal Verification (Apple corecrypto-inspired):
- proofs/README.md — Multi-level proof architecture (L0-L4)
- proofs/specs/fixed_spec.md — Q32.64 arithmetic mathematical specification
- proofs/specs/verlet_spec.md — Verlet integrator specification with invariants
- proofs/kani/fixed_equivalence.rs — Functional equivalence proof harnesses (add, sub, mul, div, sqrt, Vec3)
- proofs/kani/acceleration_proofs.rs — Composite proof harnesses (Newton's laws, symmetry, mass proportionality)
- kelvin-core/src/fixed_math.rs — Enhanced Kani harnesses with functional equivalence assertions

Key references:
- Apple Security Research (2026). "Formal verification of corecrypto for post-quantum cryptography."
  https://security.apple.com/blog/formal-verification-corecrypto/
- Apple Inc. (2026). corecrypto open source release. https://github.com/apple/corecrypto

Next actions: integration tests, known-answer vectors, benchmarks, acceleration composite proofs, end-to-end keystream proof.

===============================================================================

AI BEHAVIOR RULES

1. Always prefer simplicity over cleverness.
2. No floating-point. No exceptions.
3. If a crypto decision is uncertain, choose conservative and flag for review.
4. Remind about the "EXPERIMENTAL" status if production use is suggested.
5. When suggesting code, follow Rust best practices listed above.
6. **Documentation-only changes** (README.md, documentation/*.md, proofs/specs/*.md, etc.) do NOT require compilation or test execution. Skip build/test steps and proceed directly to `attempt_completion`.

===============================================================================
