# Kelvin — Orbital Chaos KDF Cryptosystem
### **Backronym:** **K**ey derivation from n-body **E**lliptic **L**yapunov **V**ortex **IN**stability
### *An n-body simulation based Key Derivation Function*

[Proof of Concept](documentation/proof_of_concept.md) | [Usage Guide](documentation/usage.md) | [Quantum Analysis](documentation/quantum_analysis.md)

---

### *Three bodies. Infinite chaos...*

[![Kelvin Orbital Key Generator](https://img.shields.io/badge/🚀-Launch_3D_Orbital_Visualizer-00ff00?style=for-the-badge)](kelvin-demo/orbital_visualizer.html)

*Drag to rotate, scroll to zoom, Space to pause. Load a `key.json` file to configure custom initial conditions.*

**EXPERIMENTAL — NOT FOR PRODUCTION USE.**

================================================================================

Kelvin is an experimental cryptosystem that derives cryptographic keys from the chaotic evolution of an n-body gravitational system. It combines:

- **Q32.64 fixed-point arithmetic** — deterministic across all platforms
- **Symplectic Verlet integrator** — energy-conserving n-body simulation (default)
- **Euler integrator** — numerically unstable, faster chaos amplification (`--euler` flag)
- **Lyapunov time estimation** — shadow orbit method for chaos quantification
- **SHAKE256 XOF entropy extraction** — 2048-byte domain-separated hashing of orbital state
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

- **Deep Physical Binding** — Every cryptographic seed is cryptographically bound to the physical laws of the simulation. By hashing the gravitational constant ($G$), softening factor, and **instantaneous force vectors** (accelerations) into the seed, Kelvin ensures that the keyspace is tethered to the physical reality of the N-body system, preventing "shortcut" attacks that ignore the dynamics.

- **Post-Quantum Hybrid Identity** — Kelvin bridges chaotic dynamics and Post-Quantum Cryptography. By applying **domain-separated hashing (SHAKE256)** to the orbital state, it derives uniform key pairs for **ML-DSA-65** (Quantum-Safe Signature), **ML-KEM-768** (Quantum-Safe KEM), and **Curve25519** (Classical). This allows a shared chaotic configuration to serve as a universally identifiable and quantum-resistant identity.

- **Negotiable physical constants** — Kelvin supports a dynamic gravitational constant ($G$), allowing communicating parties to initialize their chaotic environment with unique physical laws. This increases the configuration space and prevents pre-computation attacks based on fixed gravitational models. Strict validation bounds ($1.0 \le G \le 1000.0$) ensure the system remains within a chaotic yet numerically stable regime.

## How It Works

Kelvin's security rests on the unpredictability of chaotic n-body dynamics. The system follows a deterministic pipeline that transforms a shared orbital configuration into a cryptographic keystream:

1. **Configuration** — The shared secret is an `OrbitalConfig` specifying the number of bodies ($N \ge 3$), their initial positions/velocities, masses, the gravitational constant ($G$), time step, softening factor, and total simulation steps. This configuration is the equivalent of a cryptographic key — anyone with the same config will derive the same keystream.

2. **Lyapunov time estimation** — Before running the full simulation, Kelvin estimates the Lyapunov time of the system using the shadow orbit method. This quantifies the chaotic divergence rate and ensures the simulation runs within the predictable regime. If the requested step count exceeds the safe Lyapunov horizon, the system rejects the configuration.

3. **Orbital simulation** — The n-body system is evolved using a symplectic Verlet integrator (default, energy-conserving) or an explicit Euler integrator (`--euler` flag, numerically unstable). Verlet conserves energy and momentum for physically realistic trajectories; Euler's numerical instability amplifies chaos ~10x faster for maximum entropy per step. Both use deterministic fixed-point arithmetic for bit-identical results across all platforms (x86, ARM, WebAssembly, etc.).

4. **Seed extraction** — After simulation, the final orbital state is hashed with **SHAKE256 (XOF)** using domain separation. This process incorporates the full physical state: positions, velocities, masses, the gravitational constant ($G$), the softening factor, and the **instantaneous gravitational force vectors** acting on every body. This produces a **2048-byte** cryptographically strong seed that is physically bound to the simulation's reality.

5. **Key schedule** — The seed drives a deterministic key schedule that generates a continuous **Orbital Keystream**. The schedule automatically reseeds at configurable intervals using **SHAKE256** to advance the entropy pool, ensuring that the keystream remains tightly coupled to the physical evolution of the system. Each reseed event incorporates fresh force vector data from the current orbital state.

6. **Encryption/Decryption** — Data is encrypted by XORing with the **ChaCha20 Orbital Keystream**, seeded by the chaotic orbital state. Decryption is the exact inverse operation, requiring the identical initial orbital configuration.

## Architecture

```
kelvin-core/     — Fixed-point math, Vec3, OrbitalBody, Verlet/Euler integrator
kelvin-kdf/      — OrbitalConfig, LyapunovEstimator, SHAKE256 XOF extractor, KeySchedule
kelvin-stream/   — ChaCha20 wrapper with StreamCipher trait
kelvin/          — Top-level struct with 5 modes: Secure (V1), Chaos (V2),
                   Photon (V3), Quantum (H), Prism (HE OTP key generator)
kelvin-cli/      — CLI tool (keygen, encrypt/decrypt with --mode, benchmark, identify)
kelvin-ffi/      — C FFI bindings for iOS/Android/embedded
kelvin-demo/     — Demo kit: 3D orbital visualizer, test binaries, sample keys
kelvin-test-client/ — Integration test client (self-test + test vector verification)
kelvin-test-server/ — Test vector generation server
```

## Cryptographic Modes

Kelvin provides four cryptographic modes, each optimized for different use cases. All modes support both Verlet (default) and Euler (`--euler`) integration.

| Parameter | V1 `Secure` | V2 `Chaos` | V3 `Photon` | H `Quantum` |
| :--- | :--- | :--- | :--- | :--- |
| **Tagline** | *"The safe choice"* | *"Pure chaotic streaming"* | *"Fast as light"* | *"Best of all worlds"* |
| **Engine** | `Kelvin` | `KelvinStreaming` | `KelvinPhoton` | `KelvinQuantum` |
| **Simulation** | Upfront (Verlet/Euler) | Per-step (Verlet/Euler) | Upfront (Verlet/Euler) | Upfront + periodic reseed |
| **Cipher** | ChaCha20Poly1305 AEAD | SHAKE256 XOR per-step | HKDF→SHAKE256 XOR | Hybrid cache+XOR + orbital reseed |
| **Authentication** | ✅ Built-in AEAD | ❌ XOR only (add `--auth`) | ❌ XOR only (add `--auth`) | ❌ XOR only (add `--auth`) |
| **Authenticated engine** | N/A (AEAD built-in) | `KelvinStreamingAuthenticated` | `KelvinPhotonAuthenticated` | `KelvinQuantumAuthenticated` |
| **Auth method** | ChaCha20Poly1305 tag | KMAC128 tag (32 bytes, NIST SP 800-185) | KMAC128 tag (32 bytes, NIST SP 800-185) | KMAC128 tag (32 bytes, NIST SP 800-185) |
| **Keystream** | Finite (~28 GiB) | ✅ Unlimited | Finite (key schedule bound) | ✅ Effectively unlimited |
| **Setup time** | Seconds–minutes | Instant | Seconds–minutes | Seconds–minutes |
| **First byte** | After setup | Milliseconds | After setup | After setup |
| **Bulk throughput** | ~500 MB/s | Slow (O(N) sim/chunk) | ~200 MB/s | ~500 MB/s |
| **Ideal use case** | Storage / authenticated channels | Lightweight real-time streams | 1 MB–1 GB batch encryption | Large bulk data requiring fresh entropy |
| **CLI mode flag** | `--mode secure` (default) | `--mode chaos` | `--mode photon` | `--mode quantum` |
| **Auth flag** | *(ignored)* | `--auth` | `--auth` | `--auth` |
| **Integration method** | `--euler` available | `--euler` available | `--euler` available | `--euler` available |

> **Note:** All modes use the same `OrbitalConfig` shared secret. The integration method (Verlet/Euler) must match between encryption and decryption.
>
> **Authentication:** Append `--auth` to encrypt/decrypt commands for chaos, photon, or quantum modes to append a 32-byte KMAC128 tag (NIST SP 800-185), defeating ciphertext malleability. The secure mode has built-in AEAD and ignores the flag.

### Prism Mode — OTP Key Generator for Homomorphic Encryption

**Prism** (`KelvinPrism`) is a standalone OTP key generator designed specifically for integration with homomorphic encryption (HE) systems. It is not an encryption mode itself — it produces domain-separated OTP key material that can be plugged into any FHE library (SEAL, HElib, TFHE, etc.).

| Property | Prism |
|----------|-------|
| **Engine** | `KelvinPrism` |
| **Purpose** | Generate OTP keys for FHE recryption, split-key XOR homomorphism, chaotic FHE keygen |
| **Keystream** | HKDF→SHAKE256 XOR (domain-separated from V3 Photon) |
| **Forward secrecy** | ✅ BLAKE3 reseeding |
| **Quantum resistance** | ✅ SHAKE256 (256-bit classical / 128-bit quantum) |
| **Key features** | `generate_otp_key()`, `split_key()`, `recrypt()` |

See the [Homomorphic Cryptosystem Analysis](documentation/homomorphic_cryptosystem.md) for full details on integrating Kelvin with FHE systems.

## Security Levels

| Level    | Bodies | Steps     | Sun Mass Var. | Raw Keyspace | Setup Time |
|----------|--------|-----------|---------------|--------------|------------|
| Standard | 5      | 1,000,000 | ±25%          | $\approx 2^{1287}$ | ~1s        |
| Paranoid | 5      | 10,000,000| ±25%          | $\approx 2^{1287}$ | ~10s       |
| Maximum  | 10     | 100,000,000| ±25%         | $\approx 2^{2744}$ | ~2min      |

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
- **NIST FIPS PUB 202 (2015)** — SHAKE256 Extendable-Output Function (XOF) standard [doi:10.6028/NIST.FIPS.202]
- **Bertoni et al. (2013)** — Keccak sponge construction [doi:10.1007/978-3-642-38348-9_19]

### Stream Ciphers
- **Bernstein (2008)** — ChaCha20 specification (SASC 2008)
- **Nir & Langley (2018)** — ChaCha20 IETF standard (RFC 8439) [doi:10.17487/RFC8439]

### Related Work
- **Song et al. (2025)** — CryptoChaos: hybrid chaos-based cryptographic framework (arXiv:2504.08618)
- **Cang, Kang & Wang (2021)** — PRNG based on generalized conservative Sprott-A system [doi:10.1007/s11071-021-06310-9]
- **Halayka (2012)** — N-body dynamics for PRNG (computationally expensive vs. LFSR)
- **Vuckovac (2021)** — N-body puzzles for PoW (no full cryptosystem implemented)
- **Kraicha et al. (2025)** — Orbital-inspired encryption using Phobos/Deimos positions (metaphorical, not simulated)

### Security & Side Channels
- **Koeune & Standaert (2005)** — Side-channel attack methodology [doi:10.1007/11554578_3]

### Reproducibility
- **Gent (2017)** — Recomputation Manifesto [doi:10.1145/3105966]

## License

Licensed under the Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License. See [licence.md](licence.md) for details.

## Citation

If you use Kelvin in academic work, please cite:

```bibtex
@misc{liaudat2026kelvin,
  author    = {Nicolas Liaudat},
  title     = {{Kelvin}: Orbital Chaos {KDF} Cryptosystem},
  year      = {2026},
  howpublished = {\url{https://github.com/nliaudat/kelvin}}
}
```
