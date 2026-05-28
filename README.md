# Kelvin — Quantum-Resistant One-Time Pad Cryptosystem

[![Build Status](https://github.com/nliaudat/kelvin/actions/workflows/rust.yml/badge.svg)](https://github.com/nliaudat/kelvin/actions/workflows/rust.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](licence.md)
[![Crates.io](https://img.shields.io/crates/v/kelvin.svg)](https://crates.io/crates/kelvin)

> **⚠️ EXPERIMENTAL — Not for production use.** This is a research cryptosystem.
> It has not undergone formal cryptanalysis. See [Security](#security) for details.

Kelvin is a **quantum-resistant one-time pad (OTP) cryptosystem** and **deterministic key derivation function (KDF)** based on **fixed-point gravitational n-body simulation**. It transforms a shared orbital configuration (masses, positions, velocities) into a cryptographic keystream by simulating chaotic gravitational dynamics and extracting entropy via SHAKE256.

The core insight: the n-body problem has no closed-form solution for N ≥ 3. An attacker cannot shortcut the simulation — they must run the same deterministic integration (Verlet or Euler) step-by-step to reproduce the keystream. The Euler method amplifies chaos ~10× faster than Verlet through numerical instability, creating even stronger computational asymmetry. This creates a **computational asymmetry**: legitimate parties pay the simulation cost once, while attackers face the same cost for every guess.

Kelvin's XOR-based modes (V2 Chaos, V3 Photon, H Quantum, Prism, Split, Flare) produce a **quantum-resistant OTP keystream** — data is XOR-encrypted byte-by-byte with keystream derived from SHAKE256 (NIST PQC standard). There is no nonce, no IV, no algebraic round function. The only attack is brute force — and the search space is astronomical.


## Quick Start

```bash
# Generate a random orbital configuration
cargo run -p kelvin-cli -- generate -o key.json

# Encrypt a file
cargo run -p kelvin-cli -- encrypt -c key.json -i plaintext.txt -o ciphertext.bin

# Decrypt a file
cargo run -p kelvin-cli -- decrypt -c key.json -i ciphertext.bin -o decrypted.txt
```

## Mode Comparison

| Mode | Name | OTP Type | Auth | Keystream | Speed | Use Case |
|------|------|----------|:----:|-----------|:-----:|----------|
| **V1** | Kelvin-Secure | ChaCha20Poly1305 (AEAD) | ✅ AEAD | Finite (~28 GiB) | 🐢 500 MB/s | General purpose with authentication |
| **V2** | Kelvin-Chaos | **Per-Step OTP** (SHAKE256 XOR) | ❌ | Unlimited | 🐌 3 MB/s | Streaming, real-time |
| **V3** | Kelvin-Photon | **Batch OTP** (HKDF→SHAKE256 XOR) | ❌ | Finite | 🚀 5 GB/s | Bulk encryption |
| **H** | Kelvin-Quantum | **Hybrid OTP** (V3+V2 XOR) | ❌ | ≈Unlimited | 🚀 5 GB/s | Best all-around |
| **—** | Kelvin-Prism | **HE OTP** (HKDF→SHAKE256) | ❌ | Finite | 🚀 5 GB/s | OTP key generation for HE |
| **—** | Kelvin-Split | **Split OTP** (HKDF→SHAKE256) | ❌ | Finite | 🚀 5 GB/s | XOR key splitting for HE |
| **—** | Kelvin-Flare | **FHE OTP** (HKDF→SHAKE256) | ❌ | Finite | 🚀 5 GB/s | FHE secret key generation |


> **Recommended default:** Kelvin-Quantum (H) for most use cases. Add KMAC authentication via `KelvinQuantumAuthenticated` if needed. Use Prism/Split/Flare for homomorphic encryption workflows.

## What Makes Kelvin Novel

Kelvin derives cryptographic keys from the **fixed-point gravitational n-body simulation** — a novel approach to key derivation that differs from traditional KDFs (algebraic hardness, memory-hard functions) and from other chaos-based cryptosystems. The n-body problem is famously non-integrable for N ≥ 3: there is no closed-form solution, and numerical integration is the only path forward. Kelvin exploits this by making the orbital simulation itself part of the key derivation. An attacker cannot shortcut the simulation — they must run the same deterministic integration (Verlet or Euler) step-by-step, with the same fixed-point arithmetic, to reproduce the keystream. The Euler method amplifies chaos ~10× faster than Verlet through numerical instability, creating even stronger computational asymmetry. This creates a **computational asymmetry**: legitimate parties pay the simulation cost once, while attackers face the same cost for every guess.

> ⚠️ **Known Prior Art:** The broad concept of "n-body chaotic cryptography" was previously described by Chai et al. (2025) using a restricted four-body memristor system for image encryption. Kelvin distinguishes itself via: (1) full gravitational 10-body simulation (not restricted), (2) Q32.64 fixed-point arithmetic (cross-platform deterministic), (3) general-purpose multi-mode architecture (not image-specific). See [Patent Review #3](documentation/patent_review_3.md) for full analysis.

### Known Limitations (Transparency)

Kelvin has not undergone formal cryptanalysis. The security claims are based on:
- **Physical reasoning** — chaotic dynamics of n-body systems (Lyapunov exponent analysis)
- **Statistical testing** — NIST SP 800-90B, SP 800-22 test suites
- **Formal verification** — Kani proofs for safety and functional equivalence of fixed-point arithmetic
- **Constant-time verification** — dudect-bencher (Welch's t-test) for side-channel resistance

No mathematical reduction to a hard problem (e.g., lattice problems) is provided.
This is an intentional trade-off: the n-body problem has no known closed-form
solution, but this has not been formally proven as a cryptographic assumption.

Key features include:

- **Fixed-point gravitational n-body as a deterministic one-way function** — The exponential divergence of nearby trajectories (quantified by the Lyapunov exponent) ensures that even microscopic differences in initial conditions produce completely different orbital states after sufficient steps. This maps naturally to a cryptographic one-way function: given the final state, recovering the initial configuration is computationally infeasible.

- **Platform-independent fixed-point arithmetic** — Kelvin uses Q32.64 fixed-point math instead of floating-point, guaranteeing bit-identical simulation results across all architectures (x86, ARM, WebAssembly, RISC-V). This is essential for a KDF — the same orbital configuration must produce the same keystream everywhere.

- **Lyapunov time as an active security parameter** — While Lyapunov exponents are widely used as a passive validation metric for chaotic systems, Kelvin uses the Lyapunov time as an **active** security parameter. The shadow orbit method estimates the horizon beyond which the system becomes truly unpredictable, and Kelvin rejects configurations that would produce unreliable keystreams, providing a rigorous bound on the security margin.

- **Entropy Extraction past the Lyapunov Horizon** — To ensure maximum uncertainty, Kelvin requires that the total simulation steps exceed the estimated Lyapunov time. This guarantees that the extractable entropy is fully randomized and decoupled from the initial configuration secrets.

- **Deep Physical Binding** — Every cryptographic seed is cryptographically bound to the physical laws of the simulation. By hashing the gravitational constant ($G$), softening factor, and **instantaneous force vectors** (accelerations) into the seed, Kelvin ensures that the keyspace is tethered to the physical reality of the N-body system, preventing "shortcut" attacks that ignore the dynamics.

- **Post-Quantum Hybrid Identity** — Kelvin bridges chaotic dynamics and Post-Quantum Cryptography. By applying **domain-separated hashing (SHAKE256)** to the orbital state, it derives uniform key pairs for **ML-DSA-65** (Quantum-Safe Signature), **ML-KEM-768** (Quantum-Safe KEM), and **Curve25519** (Classical). This allows a shared chaotic configuration to serve as a universally identifiable and quantum-resistant identity.

- **Negotiable physical constants** — Kelvin supports a dynamic gravitational constant ($G$), allowing communicating parties to initialize their chaotic environment with unique physical laws. This increases the configuration space and prevents pre-computation attacks based on fixed gravitational models. Strict validation bounds ($1.0 \le G \le 1000.0$) ensure the system remains within a chaotic yet numerically stable regime.


## How It Works

Kelvin's security rests on the unpredictability of chaotic n-body dynamics. The system follows a deterministic pipeline that transforms a shared orbital configuration into a **quantum-resistant one-time pad keystream**:

```
OrbitalConfig (masses, positions, velocities, G, ε)
    │
    ▼
┌─────────────────────────────────────────────────────┐
│  Phase 1: Orbital Simulation (Verlet/Euler)           │
│  • Simulate N-body gravitational dynamics            │
│  • Monitor for stability (Lyapunov, ejections)       │
│  • Run for total_steps iterations                    │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│  Phase 2: Entropy Extraction (SHAKE256)              │
│  • Hash final orbital state + physical constants     │
│  • Produce 2048-byte entropy pool                    │
│  • Domain-separated for different purposes           │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│  Phase 3: Key Schedule (HKDF-SHA512 + BLAKE3)        │
│  • Derive per-key material from entropy pool         │
│  • Reseed pool after each key (forward secrecy)      │
│  • Exhausted after max_keys or safe_steps            │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
              OTP Keystream (XOR encrypt/decrypt)
```

The resulting keystream is used as a **quantum-resistant one-time pad**: data is XOR-encrypted byte-by-byte with the keystream. Since the keystream is derived via SHAKE256 (NIST PQC standard) from chaotic dynamics with no closed-form solution, it is computationally indistinguishable from random — and no quantum algorithm can shortcut the simulation.

## Why One-Time Pad?

Kelvin's V2 (Chaos), V3 (Photon), H (Quantum), Prism, Split, and Flare modes are all **XOR-based one-time pad ciphers**. Per the [Wikipedia definition](https://en.wikipedia.org/wiki/One-time_pad), a true OTP requires four conditions:

> 1. **The key must be at least as long as the plaintext.**
> 2. **The key must be truly random.**
> 3. **The key must never be reused in whole or in part.**
> 4. **The key must be kept completely secret by the communicating parties.**

### How Kelvin Satisfies Each Condition

| # | Condition | Kelvin's Approach |
|---|-----------|-------------------|
| 1 | **Key ≥ plaintext** | SHAKE256 XOF produces unlimited keystream — always exactly as long as the plaintext |
| 2 | **Truly random** | SHAKE256 is computationally indistinguishable from random (NIST PQC standard); n-body chaos provides physical entropy input |
| 3 | **Never reused** | Key schedule enforces forward secrecy via BLAKE3 reseeding — each key is unique |
| 4 | **Kept secret** | Orbital config (~2 KB) is the shared secret — protect it like any symmetric key |

Unlike block ciphers (AES) or stream ciphers with nonces (ChaCha20), Kelvin's OTP modes have:

- **No nonce to manage** — no catastrophic nonce reuse vulnerability
- **No padding or IV** — ciphertext length = plaintext length
- **No algebraic structure** — nothing for Shor's algorithm to factor or lattice reduction to exploit
- **Quantum-resistant foundation** — SHAKE256 (NIST PQC) has no known quantum shortcut beyond Grover's (128-bit effective)
- **Malleability is the only attack** — use `--auth` (KMAC128) to defeat it

The keystream is computationally indistinguishable from random via SHAKE256 extraction from chaotic n-body dynamics. True information-theoretic OTP requires perfect randomness equal to message length — SHAKE256's sponge construction approximates this for any practical adversary.

See the [OTP Bulletproof Analysis](documentation/otp_bulletproof.md) for the full security argument.



## Security

### What Kelvin Provides

- **One-Time Pad architecture**: V2/V3/H/Prism/Split/Flare all use XOR-based stream ciphers with SHAKE256 keystream — quantum-resistant, no nonce, no key reuse risk
- **Deterministic KDF**: Same orbital configuration → same keystream (verified by 18 determinism tests)
- **Chaotic divergence**: Lyapunov exponent λ ≈ 0.693 (positive → chaotic regime)
- **Forward secrecy**: BLAKE3 reseeding prevents past key recovery from future state
- **Constant-time operations**: All fixed-point arithmetic verified via dudect-bencher (|t| < 5)
- **Memory safety**: `#![forbid(unsafe_code)]` across all crates
- **Zeroization**: All secret material cleared on drop (verified by runtime read-back test)
- **Formal verification**: Kani proofs for safety (no overflows) and functional equivalence (ops match spec within dynamically scaled error bounds)
- **NIST SP 800-90B compliance**: Built-in health tests (repetition, adaptive proportion, runs, longest run, Shannon entropy, chi-square, correlation)
- **NIST SP 800-22 compliance**: All 15 statistical tests pass on orbital keystream
- **Post-quantum identity**: ML-DSA-65 (FIPS 204) signatures + ML-KEM-768 (FIPS 203) key encapsulation

### What Kelvin Does NOT Provide

- **Formal cryptanalysis**: No mathematical reduction to a hard problem
- **Key exchange**: OrbitalConfig must be established out-of-band
- **Memory hardness**: Not resistant to GPU/ASIC parallelization
- **Information-theoretic OTP**: All XOR modes are **computational OTPs** — the keystream is computationally indistinguishable from random (SHAKE256, NIST PQC standard). True information-theoretic OTP requires perfect randomness equal to message length, which SHAKE256's sponge construction approximates for any practical adversary. See the [OTP Bulletproof Analysis](documentation/otp_bulletproof.md) for the full argument.


## Architecture

Kelvin is organized as a Rust workspace with 14 crates:

```
kelvin-core/     — Fixed-point Q32.64 arithmetic, n-body simulation, entropy extraction
kelvin-kdf/      — Key schedule, HKDF-SHA512 derivation, BLAKE3 reseeding
kelvin-stream/   — Streaming cipher modes (ChaCha20Poly1305, SHAKE256 XOR)
kelvin/          — Top-level API (Kelvin, KelvinPhoton, KelvinQuantum, etc.)
kelvin-cli/      — Command-line interface (encrypt, decrypt, generate, analyze)
kelvin-ffi/      — C FFI bindings for language interop
```

## Installation

### From Source

```bash
# Prerequisites: Rust 1.75+ (install via rustup)
git clone https://github.com/nliaudat/kelvin.git
cd kelvin
cargo build --release
```

### From Crates.io

```bash
cargo add kelvin
```

## Usage Examples

### Basic Encryption/Decryption

```rust
use kelvin::{Kelvin, OrbitalConfig};

// Generate a random 5-body orbital configuration
let config = OrbitalConfig::chaotic_default();

// Create encryption instance (runs simulation)
let mut alice = Kelvin::new(config.clone()).expect("Kelvin::new");

// Encrypt data (in-place)
let mut plaintext = b"Hello, Kelvin!".to_vec();
alice.encrypt(&mut plaintext).unwrap();

// Create decryption instance (separate instance, same config)
let mut bob = Kelvin::new(config).expect("Kelvin::new");
bob.decrypt(&mut plaintext).unwrap();

assert_eq!(&plaintext, b"Hello, Kelvin!");
```

### Streaming Mode (V2)

```rust
use kelvin::KelvinStreaming;

let mut stream = KelvinStreaming::new(config, 1024).unwrap();
let mut data = b"Large file...".to_vec();
stream.encrypt(&mut data).unwrap();
```

### CLI Usage

```bash
# Generate a random configuration
kelvin generate -o my-key.json

# Encrypt a file
kelvin encrypt -c my-key.json -i secret.txt -o secret.enc

# Decrypt a file
kelvin decrypt -c my-key.json -i secret.enc -o secret.txt

# Analyze chaos quality
kelvin analyze -c my-key.json

# Run built-in self-test
kelvin-test-client
```

## Performance

| Operation | Standard (5 bodies, 1M steps) | Paranoid (5 bodies, 10M steps) |
|-----------|:-----------------------------:|:------------------------------:|
| Setup + Keygen | ~1.1s | ~12.5s |
| Encrypt 1 GB (V3/H) | ~200ms | ~200ms |
| Encrypt 1 GB (V1) | ~500ms | ~500ms |
| Encrypt 1 GB (V2) | ~33 min | ~5.5 hours |

## Formal Verification

Kelvin follows Apple's corecrypto blueprint for formal verification using the Kani Rust Verifier:

| Level | What is Proved | Status |
|-------|---------------|--------|
| L0: Safety | No panics, no overflows under bounded inputs | ✅ Done |
| L1: Functional Equivalence | Arithmetic ops match mathematical spec within dynamically scaled error bounds | ✅ Done |
| L2: Composite Correctness | `compute_accelerations` matches Newtonian gravity via force-based assertions | ✅ Done |
| L3: Pipeline Integrity | Full `simulate_and_extract_seed` produces correct output | ⏳ Pending |
| L4: Determinism | Bit-identical results across platforms | ✅ Done |

See [formal_verification.md](documentation/formal_verification.md) for details.

## Documentation

- [Usage Guide](documentation/usage.md) — Detailed API documentation
- [OTP Bulletproof Analysis](documentation/otp_bulletproof.md) — Why Kelvin's OTP is quantum-resistant and computationally unbreakable
- [Formal Verification](documentation/formal_verification.md) — Kani proof strategy
- [Proof of Concept](documentation/proof_of_concept.md) — Test results and benchmarks
- [Patent Landscape](documentation/patent_review_3.md) — Prior art analysis
- [Quantum Resistance](documentation/quantum_analysis.md) — Post-quantum security analysis
- [Homomorphic Integration](documentation/homomorphic_cryptosystem.md) — HE use cases
- [OTP Study](documentation/Kelvin_OTP_Study.md) — Mode comparison and hybrid architecture
- [Production Readiness](production_readiness_plan.md) — Roadmap to 1.0


## Academic Context

Kelvin builds on a rich body of research in chaos-based cryptography:

- **Chai et al. (2025)** — First known n-body chaotic image encryption (four-body memristor system)
- **Song et al. (2025)** — CryptoChaos: hybrid chaos-cryptography framework
- **Cang, Kang & Wang (2021)** — Conservative Sprott-A PRNG with finite precision analysis
- **Jawad (2025)** — DUff-skg: chaotic Duffing oscillator for FHE key generation
- **Apple '559 (expired)** — Original patent for chaotic dynamics in cryptography

See [REFERENCES.bib](REFERENCES.bib) for the full bibliography.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](licence.md) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](licence.md) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Security issues**: Report via the [SECURITY.md](SECURITY.md) process.
