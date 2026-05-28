# Formal Verification Proofs

This directory contains formal verification artifacts for the Kelvin cryptosystem,
following Apple's corecrypto formal verification blueprint.

See [`documentation/formal_verification.md`](../documentation/formal_verification.md)
for the full proof architecture, strategy, and references.

## Directory Structure

```
proofs/
├── README.md              ← This file (concise overview)
├── specs/
│   ├── fixed_spec.md              ← Q32.64 arithmetic
│   ├── vec3_spec.md               ← Vec3 vector operations
│   ├── orbital_body_spec.md       ← OrbitalBody (mass, position, velocity)
│   ├── orbital_config_spec.md     ← OrbitalConfig (simulation parameters)
│   ├── orbital_state_spec.md      ← OrbitalState (high-level state machine)
│   ├── verlet_spec.md             ← Verlet integrator (symplectic, 2nd order)
│   ├── euler_spec.md              ← Euler integrator (non-symplectic, 1st order)
│   ├── stability_spec.md          ← Stability monitoring (ejection, collapse)
│   ├── lyapunov_spec.md           ← Lyapunov exponent estimation
│   ├── shake256_extraction_spec.md ← SHAKE256 entropy extraction pipeline
│   ├── key_schedule_spec.md       ← Key schedule (HKDF + Blake3 reseed)
│   ├── asymmetric_spec.md         ← Asymmetric key derivation (ML-DSA, Ed25519)
│   ├── cipher_modes_spec.md       ← Cipher modes (Photon, Quantum, Prism, Split, Flare)
│   └── stream_ciphers_spec.md     ← Stream ciphers (AES-256-GCM, ChaCha20-Poly1305)
└── kani/
    ├── fixed_equivalence.rs      ← Q32.64 arithmetic functional equivalence
    ├── acceleration_proofs.rs    ← compute_accelerations composite proofs
    ├── vec3_proofs.rs            ← Vec3 vector operation proofs
    ├── integrator_proofs.rs      ← Verlet + Euler integrator proofs
    ├── stability_proofs.rs       ← Ejection + collapse detection proofs
    ├── extraction_proofs.rs      ← Entropy extraction safety proofs
    └── orbital_state_proofs.rs   ← OrbitalState safety proofs
```

## Quick Start

```bash
# Run all Kani proofs (requires Kani installed)
cargo kani -p kelvin-core

# Run determinism integration tests
cargo test -p kelvin --test full_pipeline
```
