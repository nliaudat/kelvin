# Kelvin — Orbital Chaos KDF Cryptosystem
### **Backronym:** **K**ey derivation from n-body **E**lliptic **L**yapunov **V**ortex **IN**stability
### *A chaotic 3D n-body gravitational key derivation system*
### *Three bodies. Infinite chaos...*
================================================================================

**EXPERIMENTAL — NOT FOR PRODUCTION USE.**

Kelvin is an experimental cryptosystem that derives cryptographic keys from the chaotic evolution of an n-body gravitational system. It combines:

- **Q32.64 fixed-point arithmetic** — deterministic across all platforms
- **Symplectic Verlet integrator** — energy-conserving n-body simulation
- **Lyapunov time estimation** — shadow orbit method for chaos quantification
- **SHA3-512 entropy extraction** — domain-separated hashing of orbital state
- **ChaCha20 stream cipher** — XOR-based encryption/decryption
- **Curve25519 Asymmetric Keys** — bias-free ECDH key pairs via 512-bit wide reduction
- **Chaos Quality Test Suite** — Integrated statistical verification (avalanche and uniformity tests)

### What Makes Kelvin Novel

Kelvin is the first cryptosystem to harness the **computational difficulty of n-body orbital integration** as a cryptographic primitive. Unlike traditional KDFs that rely on algebraic hardness (discrete log, factorization) or memory-hard functions (Argon2, scrypt), Kelvin's security derives from the inherent unpredictability of chaotic gravitational dynamics — a fundamentally different source of cryptographic entropy.

The n-body problem is famously non-integrable for N ≥ 3: there is no closed-form solution, and numerical integration is the only path forward. Kelvin exploits this by making the orbital simulation itself part of the key derivation. An attacker cannot shortcut the simulation — they must run the same deterministic Verlet integration step-by-step, with the same fixed-point arithmetic, to reproduce the keystream. This creates a **computational asymmetry**: legitimate parties pay the simulation cost once, while attackers face the same cost for every guess.

Key innovations include:

- **Deterministic chaos as a one-way function** — The exponential divergence of nearby trajectories (quantified by the Lyapunov exponent) ensures that even microscopic differences in initial conditions produce completely different orbital states after sufficient steps. This maps naturally to a cryptographic one-way function: given the final state, recovering the initial configuration is computationally infeasible.

- **Platform-independent fixed-point arithmetic** — Kelvin uses Q32.64 fixed-point math instead of floating-point, guaranteeing bit-identical simulation results across all architectures (x86, ARM, WebAssembly, RISC-V). This is essential for a KDF — the same orbital configuration must produce the same keystream everywhere.

- **Lyapunov time as a security parameter** — The Lyapunov time quantifies the horizon beyond which the system becomes truly unpredictable. Kelvin's shadow orbit method estimates this horizon and rejects configurations that would produce unreliable keystreams, providing a rigorous bound on the security margin.

- **Domain-separated SHA3-512 extraction** — Raw orbital coordinates are not uniformly distributed. Kelvin uses SHA3-512 with domain-specific context strings to extract cryptographically uniform seed material, preventing any leakage of the orbital state into the keystream.

- **Asymmetric Identity (Curve25519)** — Kelvin is the first n-body cryptosystem to bridge the gap between chaotic dynamics and Elliptic Curve Cryptography. By applying **512-bit wide reduction** to the orbital state, it derives perfectly uniform Curve25519 key pairs. This allows a shared chaotic configuration to serve as both a symmetric encryption key and an asymmetric identity.

- **Negotiable physical constants** — Kelvin supports a dynamic gravitational constant ($G$), allowing communicating parties to initialize their chaotic environment with unique physical laws. This increases the configuration space and prevents pre-computation attacks based on fixed gravitational models. Strict validation bounds ($1.0 \le G \le 1000.0$) ensure the system remains within a chaotic yet numerically stable regime.

## How It Works

Kelvin's security rests on the unpredictability of chaotic n-body dynamics. The system follows a deterministic pipeline that transforms a shared orbital configuration into a cryptographic keystream:

1. **Configuration** — The shared secret is an `OrbitalConfig` specifying the number of bodies ($N \ge 3$), their initial positions/velocities, masses, the gravitational constant ($G$), time step, softening factor, and total simulation steps. This configuration is the equivalent of a cryptographic key — anyone with the same config will derive the same keystream.

2. **Lyapunov time estimation** — Before running the full simulation, Kelvin estimates the Lyapunov time of the system using the shadow orbit method. This quantifies the chaotic divergence rate and ensures the simulation runs within the predictable regime. If the requested step count exceeds the safe Lyapunov horizon, the system rejects the configuration.

3. **Orbital simulation** — The n-body system is evolved using a symplectic Verlet integrator that conserves energy and momentum. The deterministic fixed-point arithmetic ensures bit-identical results across all platforms (x86, ARM, WebAssembly, etc.).

4. **Seed extraction** — After simulation, the final orbital state is hashed with SHA3-512 using domain separation. This produces a cryptographically strong seed that is uniformly distributed and independent of the raw orbital coordinates.

5. **Key schedule** — The seed drives a deterministic key schedule that generates a sequence of ChaCha20 keys and nonces. The schedule automatically reseeds at configurable intervals to limit the keystream generated from any single orbital state.

6. **Encryption/Decryption** — Data is encrypted by XORing with the ChaCha20 keystream. Decryption is identical to encryption (XOR is its own inverse). Two independent Kelvin instances with the same configuration produce identical keystreams, enabling round-trip encryption and decryption.

## Architecture

```
kelvin-core/     — Fixed-point math, Vec3, OrbitalBody, Verlet integrator
kelvin-kdf/      — OrbitalConfig, LyapunovEstimator, SHA3-512 extractor, KeySchedule
kelvin-stream/   — ChaCha20 wrapper with StreamCipher trait
kelvin/          — Top-level Kelvin struct (encrypt/decrypt)
kelvin-cli/      — CLI tool (keygen, encrypt, decrypt, benchmark)
kelvin-ffi/      — C FFI bindings for iOS/Android/embedded
```

## Security Levels

| Level    | Bodies | Steps     | G (Dynamic) | Lyapunov Shadow | Setup Time |
|----------|--------|-----------|-------------|-----------------|------------|
| Standard | 3      | 1,000,000 | 39.478...   | 1,000           | ~1s        |
| Paranoid | 5      | 10,000,000| Negotiable  | 10,000          | ~10s       |
| Maximum  | 10     | 100,000,000| Negotiable | 100,000         | ~2min      |

## Documentation

- **[Academic Citations](documentation/citations.md)** — Full academic context for every component of Kelvin, organized by pipeline stage. Each citation includes a summary of its relevance.
- **[Proof of Concept](documentation/proof_of_concept.md)** — Test results, benchmarks, and verification that Kelvin works as a functional cryptosystem.
- **[Threat Model](THREAT_MODEL.md)** — Attacker capabilities, security boundaries, and comparison with existing KDFs.
- **[REFERENCES.bib](REFERENCES.bib)** — Complete BibTeX bibliography for LaTeX integration.

## References

Kelvin builds on foundational work across numerical analysis, chaos theory, and cryptography:

### Fixed-Point Arithmetic & Numerical Methods
- **Goldberg (1991)** — Floating-point non-determinism motivates Kelvin's Q32.64 fixed-point arithmetic [doi:10.1145/103162.103163]
- **Verlet (1967)** — Original leapfrog integration method [doi:10.1103/PhysRev.159.98]
- **Hairer, Lubich & Wanner (2006)** — Symplectic integrator theory (Springer, ISBN 978-3-540-30666-5)

### Chaos Theory & Lyapunov Exponents
- **Benettin et al. (1980)** — Lyapunov exponent computation algorithm [doi:10.1007/BF02128236]
- **Wolf et al. (1985)** — Shadow orbit method for Lyapunov estimation [doi:10.1016/0167-2789(85)90011-9]
- **Sano & Sawada (1985)** — Alternative Lyapunov estimation method [doi:10.1103/PhysRevLett.55.1082]

### N-Body Gravitational Simulation
- **Wisdom & Holman (1991)** — Symplectic maps for the n-body problem [doi:10.1086/115978]
- **Murray & Dermott (1999)** — Solar System Dynamics (Cambridge University Press)

### Cryptographic Hash Functions
- **NIST FIPS PUB 202 (2015)** — SHA3-512 standard [doi:10.6028/NIST.FIPS.202]
- **Bertoni et al. (2013)** — Keccak sponge construction [doi:10.1007/978-3-642-38348-9_19]

### Stream Ciphers
- **Bernstein (2008)** — ChaCha20 specification (SASC 2008)
- **Nir & Langley (2018)** — ChaCha20 IETF standard (RFC 8439) [doi:10.17487/RFC8439]

### Related Work
- **CryptoChaos (Harvard, 2025)** — Hybrid chaos-based cryptographic framework

### Security & Side Channels
- **Koeune & Standaert (2005)** — Side-channel attack methodology [doi:10.1007/11554578_3]

### Reproducibility
- **Gent (2017)** — Recomputation Manifesto [doi:10.1145/3105966]

## License

Licensed under the Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License. See [licence.md](licence.md) for details.

## Citation

If you use Kelvin in academic work, please cite:

```bibtex
@misc{liaudat2025kelvin,
  author    = {Nicolas Liaudat},
  title     = {{Kelvin}: Orbital Chaos {KDF} Cryptosystem},
  year      = {2026},
  howpublished = {\url{https://github.com/nliaudat/kelvin}}
}
```
