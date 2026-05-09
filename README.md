# Kelvin — Orbital Chaos KDF Cryptosystem

**Status:** Planning complete, pre-implementation  
**Language:** Rust (stable 1.75+)  
**License:** TBD  

**kelvin** = **K**DF from **n**-body **L**yapunov **I**nstability **N**aturally

Kelvin is a cryptographic key derivation system that uses the chaotic behavior of 3D gravitational n-body simulations to generate encryption keys.

The core insight: **n-body gravitational systems are computationally irreversible** due to Lyapunov divergence. Running a simulation forward is easy. Running it backward to recover initial conditions is effectively impossible. This makes orbital parameters an excellent shared secret.

---

## How It Works

```
Shared Secret ──▶ Validate ──▶ N-Body Sim ──▶ Hash State ──▶ Master Seed
(orbital params)   config      (1M steps)     (SHA3-512)     (256-bit)
                                                                  │
                                                    ┌─────────────┘
                                                    ▼
                                            ChaCha20 Stream Cipher
                                                    │
                                                    ▼
                                              Keystream (TB-scale)
```

**Two-phase design:**

| Phase | What | Speed | Purpose |
|-------|------|-------|---------|
| **Phase 1: Orbital KDF** | N-body simulation | Slow (seconds–minutes) | Generate master seed from chaotic orbital state |
| **Phase 2: Stream Cipher** | ChaCha20 | Fast (GB/s) | Bulk encryption with standard, audited cipher |

The orbital simulation runs once at initialization and re-extracts a new seed every ~256 GB of output for forward secrecy.

---

## Security Levels

| Level | Bodies | Sim Steps | Setup Time | Max Output |
|-------|--------|-----------|------------|------------|
| **STANDARD** | 3 | 1 million | ~0.5 sec | 2.5 TB |
| **PARANOID** | 5 | 10 million | ~5 sec | 25 TB |
| **MAXIMUM** | 7 | 100 million | ~2 min | 250 TB |

All levels use the same ChaCha20 stream cipher. Higher levels increase the KDF cost, not the bulk encryption cost.

---

## Quick Start

```bash
# Generate a key (Standard security level)
kelvin keygen --level standard > orbital_config.json

# Encrypt a file
kelvin encrypt --config orbital_config.json --input archive.tar --output archive.tar.kelvin

# Decrypt a file
kelvin decrypt --config orbital_config.json --input archive.tar.kelvin --output archive.tar
```

Key exchange is out-of-band. The orbital config (~370–818 bytes) fits in a single QR code for air-gapped exchange, or can be transmitted over an existing encrypted channel.

---

## Security Properties

| Property | Guarantee |
|----------|-----------|
| Brute force cost | Each guess = full simulation (1 sec – 2 min) |
| Parallelization | Inherently sequential (Verlet integrator) |
| Shortcut resistance | No analytical solution to n-body problem |
| Key space | 672–1568 bits (far beyond brute force) |
| Side channels | Constant-time fixed-point operations |
| Forward secrecy | Old seeds unrecoverable from current state |
| Stream cipher | ChaCha20 (well-audited, hardware-accelerated) |
| Determinism | Bit-identical on x86, ARM, WASM |

### Critical constraint: Lyapunov time

The n-body simulation has a hard physical limit — the Lyapunov time. Beyond this horizon, the chaotic system becomes unpredictable. Kelvin estimates this limit before generating any keys and enforces a 10% safety margin. If the limit is reached, a clear error is returned (no silent failure).

---

## When to Use Kelvin

- ✅ Long-term file encryption (archives, backups)
- ✅ High-latency secure messaging (offline, asynchronous)
- ✅ Terabyte-scale data at rest
- ✅ Scenarios where key setup time is acceptable (seconds to minutes)
- ✅ When you want security based on physical chaos, not just math
- ✅ Air-gapped key exchange (orbital config fits in QR code)

### When NOT to Use Kelvin

- ❌ Per-packet or per-message keys (setup cost is seconds)
- ❌ Replacing AES or ChaCha20 — Kelvin feeds them, doesn't replace them
- ❌ General-purpose PRNG — designed for long-running encryption
- ❌ IoT devices without 32-bit+ CPU support (core is compatible, but not ready)

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (stable 1.75+) |
| Core math | Pure integer fixed-point (Q32.64) |
| Integration | Symplectic Verlet (kick-drift-kick) |
| Hash function | SHA3-512 (Keccak) |
| Stream cipher | ChaCha20 (RFC 8439) |
| Serialization | JSON or compact binary |
| FFI | C ABI (iOS, Android, embedded) |
| Web support | WASM (wasm-pack) |

### Dependencies (minimal)

**Runtime:** `sha3`, `chacha20`, `zeroize`, `thiserror`  
**Development:** `serde` + `serde_json`, `clap`  
**Testing:** `proptest`, `criterion`

No dependency on libc, OpenSSL, floating-point math, or SIMD intrinsics.

---

## Comparison with Other KDFs

| | Kelvin | PBKDF2 / Argon2 | Direct ChaCha20 |
|---|---|---|---|
| **Basis** | Physics (n-body chaos) | Hash iterations | None |
| **Parallelization** | Inherently sequential | Theoretically parallelizable | Instant |
| **Attacker cost** | Full simulation per guess | Hash iterations per guess | Brute-force key only |
| **Best for** | Long-term, high-volume | Password hashing | Low-latency encryption |

---

## Project Status

- ✅ Crate structure designed (6 crates)
- ✅ Data formats specified (Q32.64 fixed-point)
- ✅ Algorithms selected (Verlet, SHA3-512, ChaCha20)
- ✅ API surface defined
- ✅ Security levels parameterized
- ✅ Error handling enumerated
- ✅ Test strategy planned
- ✅ Cross-platform support designed
- ⬜ Implementation (estimated 20–22 effort-weeks)

---

## Next Steps

1. Set up repository with crate structure
2. Implement `kelvin-core` (fixed-point math + body types)
3. Implement symplectic integrator
4. Build Lyapunov time estimator
5. Integrate SHA3-512 extraction
6. Build key schedule manager
7. Integrate ChaCha20 stream cipher
8. Build top-level API
9. Create CLI key generator
10. Write comprehensive tests
11. Generate known-answer vectors
12. Document everything
13. Build FFI for mobile
14. Third-party security audit
15. Release v1.0

---

## ⚠️ Important Disclaimer

**Kelvin is experimental cryptography.** The core security claim — that n-body simulation is computationally irreducible and provides a unique KDF property — is plausible but unproven. This project should not be used for production security without formal cryptanalysis and peer-reviewed publication.

---

## License

TBD

## Contributing

This project is in early planning stages. Contributions and discussions are welcome once the repository is set up.
