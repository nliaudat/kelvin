# Kelvin Threat Model

**EXPERIMENTAL — NOT FOR PRODUCTION USE.**

## Overview

This document defines the attacker model and security boundaries for the Kelvin cryptosystem. Kelvin is an experimental cryptosystem that derives cryptographic keys from the chaotic evolution of an n-body gravitational system.

## Attacker Capabilities

| Capability | Assumption | Rationale |
|------------|------------|-----------|
| Full ciphertext access | Yes | Standard known-ciphertext attack |
| Some plaintext-ciphertext pairs | Yes | Standard known-plaintext attack |
| Can run Kelvin simulation | Yes | Kerckhoffs's principle — algorithm is public |
| Knowledge of algorithm | Yes | Kerckhoffs's principle |
| Access to OrbitalConfig | **No** | This is the shared secret (the key) |
| Computational resources | Nation-state | GPU clusters, ASICs, cloud computing |

## What Kelvin Defends Against

### 1. Brute-Force Search of Orbital Parameter Space

Each guess of an OrbitalConfig requires running the full n-body simulation (seconds to minutes). The simulation is inherently sequential — step N+1 requires step N. This makes parallel brute-force search fundamentally limited compared to hash-based KDFs like Argon2.

**Reference**: Benettin et al. (1980), Wolf et al. (1985) — Lyapunov exponent computation confirms chaotic divergence, ensuring no short-cuts exist for predicting orbital state.

### 2. Parallelization Attacks

The Verlet integrator is sequential (kick-drift-kick formulation). Unlike memory-hard KDFs that can be parallelized across cores for verification, Kelvin's simulation cannot be sped up with additional hardware beyond a single simulation instance.

**Reference**: Verlet (1967), Hairer, Lubich & Wanner (2006) — symplectic integrators are inherently sequential.

### 3. Side-Channel Recovery of Orbital State

Kelvin uses:
- Fixed-point arithmetic (no floating-point, which has platform-dependent timing)
- Constant-time operations where possible
- No branching on secret-dependent values
- Memory access patterns independent of orbital state

**Reference**: Koeune & Standaert (2005) — side-channel attack methodology.

### 4. Forward Secrecy

- Orbital state is overwritten after each reseed operation
- Old seeds are not recoverable from current orbital state
- All buffers are zeroized on drop (Zeroize trait)

## What Kelvin Does NOT Defend Against

### 1. Mathematical Breakthroughs in N-Body Analysis

The core security claim — that n-body simulation is "computationally irreducible" — is plausible but unproven. An attacker with mathematical sophistication might find shortcuts for specific orbital configurations.

**Mitigation**: Conservative parameter selection (n >= 5 for any real use). Prominent "EXPERIMENTAL" warnings.

### 2. Quantum Attacks on the Stream Cipher

ChaCha20 has 256-bit keys, providing post-quantum 128-bit security (Grover's algorithm). This is considered adequate for the near term.

**Reference**: Bernstein (2008) — ChaCha20 specification.

### 3. Compromise of the OrbitalConfig

The OrbitalConfig is the cryptographic key. If it is compromised during exchange or storage, all security is lost. Kelvin does not define a key exchange protocol — the OrbitalConfig must be established through an out-of-band mechanism.

### 4. Implementation Bugs

Buffer overflows, use-after-free, and other memory safety issues are mitigated by Rust's safety guarantees and `#![forbid(unsafe_code)]` throughout the codebase.

## Comparison with Existing KDFs

| Property | Argon2 | Kelvin |
|----------|--------|--------|
| Memory-hard | Yes | No |
| Parallelizable for verification | Yes (GPU/ASIC) | No (sequential) |
| Side-channel resistant | Partial | Yes (fixed-point) |
| Post-quantum | 128-bit (256-bit hash) | 128-bit (256-bit ChaCha20) |
| Cryptanalysis maturity | High (winner of PHC) | Low (experimental) |
| Setup cost | Configurable | Fixed by orbital parameters |

## Security Levels

| Level | Bodies | Steps | Key Material | Setup Time | Max Keystream |
|-------|--------|-------|-------------|------------|---------------|
| Standard | 3 | 1,000,000 | 672 bits | ~100-500 ms | ~2.5 TB |
| Paranoid | 5 | 10,000,000 | 1120 bits | ~1-5 s | ~25 TB |
| Maximum | 7 | 100,000,000 | 1568 bits | ~30-120 s | ~250 TB |

## Related Work

- **CryptoChaos** (Harvard University, 2025): A hybrid chaos-based cryptographic framework combining deterministic chaos with X25519 Diffie-Hellman key exchange and SHA3-256 hashing. Demonstrates academic interest in chaos-based cryptography for post-quantum applications.

## References

See [REFERENCES.bib](REFERENCES.bib) for the full bibliography.
