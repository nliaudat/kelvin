# Kelvin — Orbital Chaos KDF Cryptosystem
### **Backronym:** **K**ey derivation from n-body **E**lliptic **L**yapunov **V**ortex **IN**stability
### *An n-body simulation based Key Derivation Function*

[Proof of Concept](documentation/proof_of_concept.md) | [Usage Guide](documentation/usage.md) | [Quantum Analysis](documentation/quantum_analysis.md)

---
### *Three bodies. Infinite chaos...*
================================================================================

**EXPERIMENTAL — NOT FOR PRODUCTION USE.**

Kelvin is an experimental cryptosystem that derives cryptographic keys from the chaotic evolution of an n-body gravitational system. It combines:

- **Q32.64 fixed-point arithmetic** — deterministic across all platforms
- **Symplectic Verlet integrator** — energy-conserving n-body simulation
- **Lyapunov time estimation** — shadow orbit method for chaos quantification
- **SHA3-512 entropy extraction** — domain-separated hashing of orbital state
- **ChaCha20 stream cipher** — XOR-based encryption/decryption
- **Post-Quantum Hybrid Identity** — ML-DSA-65 (Primary), ML-KEM-768, and Curve25519 identities
- **Chaos Quality Test Suite** — Integrated statistical verification (avalanche and uniformity tests)

### What Makes Kelvin Novel

Kelvin is the first cryptosystem to harness the **computational difficulty of n-body orbital integration** as a cryptographic primitive. Unlike traditional KDFs that rely on algebraic hardness (discrete log, factorization) or memory-hard functions (Argon2, scrypt), Kelvin's security derives from the inherent unpredictability of chaotic gravitational dynamics — a fundamentally different source of cryptographic entropy.

The n-body problem is famously non-integrable for N ≥ 3: there is no closed-form solution, and numerical integration is the only path forward. Kelvin exploits this by making the orbital simulation itself part of the key derivation. An attacker cannot shortcut the simulation — they must run the same deterministic Verlet integration step-by-step, with the same fixed-point arithmetic, to reproduce the keystream. This creates a **computational asymmetry**: legitimate parties pay the simulation cost once, while attackers face the same cost for every guess.

Key innovations include:

- **Deterministic chaos as a one-way function** — The exponential divergence of nearby trajectories (quantified by the Lyapunov exponent) ensures that even microscopic differences in initial conditions produce completely different orbital states after sufficient steps. This maps naturally to a cryptographic one-way function: given the final state, recovering the initial configuration is computationally infeasible.

- **Platform-independent fixed-point arithmetic** — Kelvin uses Q32.64 fixed-point math instead of floating-point, guaranteeing bit-identical simulation results across all architectures (x86, ARM, WebAssembly, RISC-V). This is essential for a KDF — the same orbital configuration must produce the same keystream everywhere.

- **Lyapunov time as a security parameter** — The Lyapunov time quantifies the horizon beyond which the system becomes truly unpredictable. Kelvin's shadow orbit method estimates this horizon and rejects configurations that would produce unreliable keystreams, providing a rigorous bound on the security margin.

- **Entropy Extraction past the Lyapunov Horizon** — To ensure maximum uncertainty, Kelvin requires that the total simulation steps exceed the estimated Lyapunov time. This guarantees that the extractable entropy is fully randomized and decoupled from the initial configuration secrets.

- **Post-Quantum Hybrid Identity** — Kelvin bridges chaotic dynamics and Post-Quantum Cryptography. By applying **domain-separated hashing** to the orbital state, it derives uniform key pairs for **ML-DSA-65** (Quantum-Safe Signature), **ML-KEM-768** (Quantum-Safe KEM), and **Curve25519** (Classical). This allows a shared chaotic configuration to serve as a universally identifiable and quantum-resistant identity.

- **Negotiable physical constants** — Kelvin supports a dynamic gravitational constant ($G$), allowing communicating parties to initialize their chaotic environment with unique physical laws. This increases the configuration space and prevents pre-computation attacks based on fixed gravitational models. Strict validation bounds ($1.0 \le G \le 1000.0$) ensure the system remains within a chaotic yet numerically stable regime.

## How It Works

Kelvin's security rests on the unpredictability of chaotic n-body dynamics. The system follows a deterministic pipeline that transforms a shared orbital configuration into a cryptographic keystream:

1. **Configuration** — The shared secret is an `OrbitalConfig` specifying the number of bodies ($N \ge 3$), their initial positions/velocities, masses, the gravitational constant ($G$), time step, softening factor, and total simulation steps. This configuration is the equivalent of a cryptographic key — anyone with the same config will derive the same keystream.

2. **Lyapunov time estimation** — Before running the full simulation, Kelvin estimates the Lyapunov time of the system using the shadow orbit method. This quantifies the chaotic divergence rate and ensures the simulation runs within the predictable regime. If the requested step count exceeds the safe Lyapunov horizon, the system rejects the configuration.

3. **Orbital simulation** — The n-body system is evolved using a symplectic Verlet integrator that conserves energy and momentum. The deterministic fixed-point arithmetic ensures bit-identical results across all platforms (x86, ARM, WebAssembly, etc.).

4. **Seed extraction** — After simulation, the final orbital state is hashed with SHA3-512 using domain separation. This produces a cryptographically strong seed that is uniformly distributed and independent of the raw orbital coordinates.

5. **Key schedule** — The seed drives a deterministic key schedule that generates a continuous **Orbital Keystream**. The schedule automatically reseeds at configurable intervals by advancing the simulation, ensuring that the keystream remains tightly coupled to the physical evolution of the system.

6. **Encryption/Decryption** — Data is encrypted by XORing with the **Orbital One-Time Pad**. This produces a ciphertext that is computationally irreducible, resting on the fundamental unpredictability of the n-body problem. Decryption is the exact inverse operation, requiring the identical initial orbital configuration.

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

## Interactive 3D Visualization

Try the Kelvin orbital key generator in your browser:

<iframe src="examples/orbital_visualizer.html" width="100%" height="600" style="border: 1px solid #0f0; border-radius: 8px; background: #0a0a1a;" allowfullscreen></iframe>

*Drag to rotate, scroll to zoom, Space to pause. Load a `key.json` file to configure custom initial conditions.*

## Documentation

- **[Academic Citations](documentation/citations.md)** — Full academic context for every component of Kelvin, organized by pipeline stage. Each citation includes a summary of its relevance.
- **[Proof of Concept](documentation/proof_of_concept.md)** — Test results, benchmarks, and verification that Kelvin works as a functional cryptosystem.
- **[Project History](documentation/project_history.md)** — The 24-year evolution of the Kelvin cryptosystem, from celestial concept to hybrid post-quantum reality.
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
