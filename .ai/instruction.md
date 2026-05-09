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

Status: Implementation complete. All 93 tests pass across 6 crates.

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

kelvin-core/     no_std, no alloc. Fixed math + integrator + body types.
kelvin-kdf/      Depends on core. Config, Lyapunov estimator, extraction.
kelvin-stream/   Depends on nothing from kelvin. ChaCha20 (+ optional AES).
kelvin/          Orchestrator. Depends on kdf + stream. Public API.
kelvin-cli/      Binary. Depends on kelvin. Keygen, encrypt/decrypt.
kelvin-ffi/      C ABI exports. For mobile bindings.

Tests live in each crate's tests/ + workspace tests/ for integration.
Known-answer vectors in tests/vectors/ (golden files, checked into git).

===============================================================================

KEY TECHNICAL DECISIONS

Math:    Q32.64 fixed-point (i128). 32 int bits, 64 frac bits.
         NO FLOATING POINT ANYWHERE IN CORE. Determinism depends on this.

Integra- Symplectic Verlet (kick-drift-kick). O(n²) pairwise, n <= 7.
tion:    Softening factor ε prevents singularity. ε = 1e-10 in body.rs.

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
  Kelvin::encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError>
  Kelvin::decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError>
  Kelvin::bytes_processed(&self) -> u64
  Kelvin::remaining_safe_bytes(&self) -> u64

Encrypt and decrypt are the same operation (XOR with keystream).

===============================================================================

FILES IN REPO

implementation_plan.md    Full specification (long, detailed)
project_summary.md        Concise overview (shorter)
ai/instruction.md         This file (session restoration)

===============================================================================

CURRENT STATE

Implementation complete. All 93 tests pass across 6 crates.
- kelvin-core: 51 tests (fixed math, body, integrator, constants)
- kelvin-kdf: 32 tests (config, Lyapunov, extractor, schedule)
- kelvin-stream: 9 tests (ChaCha20 stream cipher)
- kelvin: 1 test (round-trip encrypt/decrypt)
- kelvin-cli: binary (no tests yet)
- kelvin-ffi: C API wrapper (no tests yet)

Next actions: integration tests, known-answer vectors, benchmarks.

===============================================================================

AI BEHAVIOR RULES

1. Always prefer simplicity over cleverness.
2. No floating-point. No exceptions.
3. If a crypto decision is uncertain, choose conservative and flag for review.
4. Remind about the "EXPERIMENTAL" status if production use is suggested.
5. Read implementation_plan.md for details this file doesn't cover.
6. When suggesting code, follow Rust best practices listed above.

===============================================================================