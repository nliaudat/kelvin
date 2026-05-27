# Formal Verification — Kelvin Cryptosystem

## Overview

This document describes the formal verification strategy for the Kelvin
cryptosystem, following the blueprint established by Apple's corecrypto
formal verification methodology.

Apple's approach proves **functional equivalence** at every level of the stack:
mathematical specification → C implementation → ARM64 assembly. Kelvin adapts
this blueprint for a pure-Rust codebase using the Kani Rust Verifier.

## Proof Architecture

```
Mathematical Specification (proofs/specs/)
    ↓ equivalence (Kani)
Fixed-point Q32.64 Arithmetic (kelvin-core/src/fixed_math.rs)
    ↓ equivalence (Kani)
Vec3 Vector Operations (kelvin-core/src/body.rs)
    ↓ equivalence (Kani)
compute_accelerations (kelvin-core/src/integrator.rs)
    ↓ equivalence (Kani)
Verlet/Euler Integrator (kelvin-core/src/integrator.rs)
    ↓ equivalence (Kani)
simulate() + extract_seed() Pipeline
    ↓ equivalence (golden hash)
End-to-End Keystream Output
```

## Proof Levels

| Level | What is Proved | Method | Location |
|-------|---------------|--------|----------|
| **L0: Safety** | No panics, no overflows under bounded inputs | Kani model checking | `fixed_math.rs` (existing) |
| **L1: Functional Equivalence** | Arithmetic ops match mathematical spec within 1 ULP | Kani with reference computation | `proofs/kani/` |
| **L2: Composite Correctness** | `compute_accelerations` matches Newtonian gravity formula | Kani with golden reference | `proofs/kani/` |
| **L3: Pipeline Integrity** | Full `simulate_and_extract_seed` produces correct output | Kani + golden hash | `proofs/kani/` |
| **L4: Determinism** | Bit-identical results across platforms | Integration tests | `tests/kelvin_tests/determinism.rs` |

## Proof Strategy (Apple-Inspired)

Apple's corecrypto team proved that their ARM64 assembly is equivalent to their
C code, and their C code is equivalent to the FIPS mathematical specification.
They achieved this by:

1. **Writing reusable lemma libraries** — common proof patterns for operations
   like polynomial addition loops vs. abstract map operations.
2. **Proving equivalence at each level** — not trying to prove assembly directly
   against the FIPS spec, but proving assembly ≅ C ≅ spec.
3. **Composing proofs** — building complex proofs from verified components.

Kelvin adapts this strategy for Rust + Kani:

1. **Prove each Fixed arithmetic op** against its mathematical definition
   (e.g., `Fixed::add(a,b) == a + b` for all bounded inputs).
2. **Prove Vec3 operations** compose correctly from Fixed ops.
3. **Prove `compute_accelerations`** matches Newton's law of gravitation.
4. **Prove Verlet integrator** preserves algebraic invariants (momentum conservation).
5. **Prove end-to-end pipeline** produces correct keystream.

## Running the Proofs

```bash
# Run all Kani proofs (requires Kani installed)
cargo kani -p kelvin-core

# Run determinism integration tests
cargo test -p kelvin --test full_pipeline

# Run constant-time verification
cargo run -p constant_time_bench
```

## References

- Apple Security Research (2026). "Formal verification of corecrypto for
  post-quantum cryptography." [security.apple.com/blog/formal-verification-corecrypto](https://security.apple.com/blog/formal-verification-corecrypto/)
- Apple Inc. (2026). corecrypto open source release.
  [github.com/apple/corecrypto](https://github.com/apple/corecrypto)
- Kani Rust Verifier. [model-checking.github.io/kani/](https://model-checking.github.io/kani/)
