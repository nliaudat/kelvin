# Academic Citations & Theoretical Foundations

This document provides the complete academic context for the Kelvin cryptosystem, organized by the cryptographic pipeline stages. Each citation includes a brief explanation of its relevance to Kelvin.

---

## 1. Fixed-Point Arithmetic & Deterministic Computing

### Goldberg (1991) — Floating-Point Pitfalls
> **Goldberg, D. (1991).** What Every Computer Scientist Should Know About Floating-Point Arithmetic. *ACM Computing Surveys*, 23(1), 5–48. doi:10.1145/103162.103163

Goldberg's seminal paper catalogues the non-determinism of IEEE 754 floating-point across platforms, compilers, and optimization levels. This is the primary motivation for Kelvin's use of Q32.64 fixed-point arithmetic — without it, the same orbital simulation would produce different results on x86 vs. ARM, breaking the KDF's determinism requirement.

### Verlet (1967) — Leapfrog Integration
> **Verlet, L. (1967).** Computer "Experiments" on Classical Fluids. I. Thermodynamical Properties of Lennard-Jones Molecules. *Physical Review*, 159(1), 98–103. doi:10.1103/PhysRev.159.98

The original Verlet (leapfrog) integration method. Kelvin uses the velocity Verlet variant (kick-drift-kick), which is symplectic (energy-conserving) and time-reversible. This ensures that the simulation remains stable over long time horizons without artificial energy drift.

### Hairer, Lubich & Wanner (2006) — Geometric Numerical Integration
> **Hairer, E., Lubich, C., & Wanner, G. (2006).** *Geometric Numerical Integration: Structure-Preserving Algorithms for Ordinary Differential Equations* (2nd ed.). Springer.

The definitive reference on symplectic integrators. Provides the theoretical foundation for why Verlet integration conserves the symplectic 2-form, ensuring long-term stability. Kelvin's integrator is built on these principles.

---

## 2. Chaos Theory & Lyapunov Exponents

### Benettin et al. (1980) — Lyapunov Exponent Computation
> **Benettin, G., Galgani, L., Giorgilli, A., & Strelcyn, J.-M. (1980).** Lyapunov Characteristic Exponents for Smooth Dynamical Systems and for Hamiltonian Systems; A Method for Computing All of Them. *Meccanica*, 15(1), 9–20. doi:10.1007/BF02128236

The standard algorithm for computing the full Lyapunov spectrum of a dynamical system. Kelvin adapts this method for its shadow orbit estimation, which quantifies the chaotic divergence rate of the n-body system.

### Wolf et al. (1985) — Shadow Orbit Method
> **Wolf, A., Swift, J. B., Swinney, H. L., & Vastano, J. A. (1985).** Determining Lyapunov Exponents from a Time Series. *Physica D: Nonlinear Phenomena*, 16(3), 285–317. doi:10.1016/0167-2789(85)90011-9

The shadow orbit method estimates the largest Lyapunov exponent by tracking the divergence of a perturbed "shadow" trajectory. Kelvin uses this approach to estimate the Lyapunov time — the horizon beyond which the system becomes truly unpredictable. This is used as a security parameter: configurations with insufficient chaotic divergence are rejected.

### Sano & Sawada (1985) — Alternative Lyapunov Estimation
> **Sano, M., & Sawada, Y. (1985).** Measurement of the Lyapunov Spectrum from a Chaotic Time Series. *Physical Review Letters*, 55(10), 1082–1085. doi:10.1103/PhysRevLett.55.1082

An alternative method for Lyapunov estimation based on Jacobian matrix reconstruction. Referenced for comparison — Kelvin uses the shadow orbit method (Wolf et al.) rather than Jacobian-based approaches.

---

## 3. N-Body Gravitational Simulation

### Wisdom & Holman (1991) — Symplectic Maps for the N-Body Problem
> **Wisdom, J., & Holman, M. (1991).** Symplectic Maps for the N-Body Problem. *The Astronomical Journal*, 102(4), 1528–1538. doi:10.1086/115978

Extends symplectic integration to the general n-body gravitational problem. Provides the theoretical basis for Kelvin's choice of a symplectic integrator for the n-body KDF pipeline.

### Murray & Dermott (1999) — Solar System Dynamics
> **Murray, C. D., & Dermott, S. F. (1999).** *Solar System Dynamics*. Cambridge University Press.

Standard reference for orbital mechanics, including AU scaling, gravitational constants, and the equations of motion for n-body systems. Kelvin's orbital parameter conventions (masses in solar masses, distances in AU, time in years) follow this reference.

---

## 4. Cryptographic Hash Functions

### NIST FIPS PUB 202 (2015) — SHA-3 Standard
> **National Institute of Standards and Technology (2015).** SHA-3 Standard: Permutation-Based Hash and Extendable-Output Functions. FIPS PUB 202. doi:10.6028/NIST.FIPS.202

The official SHA3-512 specification. Kelvin uses SHA3-512 for entropy extraction from the orbital state, with domain separation to prevent leakage of raw orbital coordinates into the keystream.

### Bertoni et al. (2013) — Keccak Sponge Construction
> **Bertoni, G., Daemen, J., Peeters, M., & Van Assche, G. (2013).** Keccak. In *Advances in Cryptology — EUROCRYPT 2013* (pp. 313–314). Springer. doi:10.1007/978-3-642-38348-9_19

The Keccak sponge construction underlying SHA3-512. Kelvin's entropy extractor uses the Keccak sponge's proven security properties to produce uniformly distributed seed material from the orbital state.

---

## 5. Stream Ciphers

### Bernstein (2008) — ChaCha20 Specification
> **Bernstein, D. J. (2008).** ChaCha, a Variant of Salsa20. In *Workshop Record of SASC 2008: The State of the Art of Stream Ciphers*.

The original ChaCha20 specification. ChaCha20 is a variant of Salsa20 with improved diffusion per round. Kelvin uses ChaCha20 as its Phase 2 stream cipher, XORing the keystream with plaintext for encryption.

### Nir & Langley (2018) — ChaCha20 IETF Standard
> **Nir, Y., & Langley, A. (2018).** ChaCha20 and Poly1305 for IETF Protocols. RFC 8439. doi:10.17487/RFC8439

The IETF standard for ChaCha20, including test vectors. Kelvin's ChaCha20 implementation is verified against these test vectors for correctness.

### Bernstein (2008) — Salsa20 Family
> **Bernstein, D. J. (2008).** The Salsa20 Family of Stream Ciphers. In *New Stream Cipher Designs* (pp. 84–97). Springer. doi:10.1007/978-3-540-68351-3_6

The design rationale for the Salsa20 family, including ChaCha20's improvements. Referenced for understanding the security properties of the stream cipher.

---

## 6. Related Work: Chaos-Based Cryptography

### Song et al. (2025) — CryptoChaos
> **Song, K., et al. (2025).** A Hybrid Chaos-Based Cryptographic Framework for Post-Quantum Secure Communications. *arXiv:2504.08618*.

CryptoChaos combines four discrete chaotic maps (Logistic, Chebyshev, Tent, Henon) with X25519 Diffie-Hellman, SHA3-256, HKDF, and AES-GCM. The framework achieves near-maximal Shannon entropy and passes NIST SP 800-22 statistical tests. Quantum analysis estimates Grover's attack requires ~2.1 × 10⁹ T-gates.

Kelvin differs in several respects:
- **Chaos source:** Physical n-body orbital dynamics (30 DOF) vs. discrete maps (4D)
- **Authentication:** Optional KMAC vs. built-in AES-GCM
- **Forward secrecy:** BLAKE3 reseeding (CryptoChaos does not specify)
- **Zeroization:** Explicit SecureZeroize trait (CryptoChaos does not mention)
- **Performance:** Benchmarked at ~500MB/s (CryptoChaos does not report)

Both frameworks share the goal of post-quantum chaos-based cryptography, but Kelvin prioritizes OTP semantics and forward secrecy over built-in authentication.

### Cang, Kang & Wang (2021) — Chaotic PRNG from Sprott-A System
> **Cang, S., Kang, Z., & Wang, Z. (2021).** Pseudo-random number generator based on a generalized conservative Sprott-A system. *Nonlinear Dynamics*, 104, 827–844. doi:10.1007/s11071-021-06310-9

Cang et al. proposed a pseudo-random number generator based on a generalized conservative Sprott-A chaotic system. This paper is highly relevant to Kelvin for several reasons:

**Conservative Chaos Validation.** The Sprott-A system is conservative (Hamiltonian), meaning it preserves phase-space volume — the same mathematical class as n-body gravitational dynamics. The paper demonstrates that conservative chaotic systems produce high-quality pseudo-random sequences suitable for cryptographic applications. This validates Kelvin's core premise: n-body orbital mechanics (a 30-dimensional Hamiltonian system) is an appropriate foundation for a KDF.

**Finite Precision Degradation (FPPC).** The paper identifies a critical problem: when chaotic systems are implemented on digital computers with finite precision, *dynamical degradation* occurs — periodicity appears and security is compromised. They propose a Finite Precision Period Calculation (FPPC) algorithm to measure repetition periods. This is directly relevant to Kelvin's Q32.64 fixed-point arithmetic: while fixed-point guarantees cross-platform determinism, it also imposes a finite state space (~2^64 states per variable) that could theoretically lead to periodicity. Kelvin mitigates this through continuous reseeding (SHAKE256 XOF + BLAKE3), which refreshes the entropy pool before any period could manifest.

**NIST SP 800-22 Test Suite.** The paper validates its PRNG against all 15 NIST statistical tests (frequency, block frequency, runs, longest run, rank, FFT, linear complexity, etc.). Kelvin should adopt the same standard for validating its orbital keystream quality.

**Comparison: Sprott-A PRNG vs. Kelvin-Quantum**

| Aspect | Sprott-A PRNG | Kelvin-Quantum (H) |
|--------|--------------|-------------------|
| Chaos source | 3D ODEs | 5-body orbital (30 DOF) |
| Dimensionality | 3 | 30 (10× higher) |
| Quantization | Binary (0/1) | Byte-wise XOR |
| Scrambling | Post-processing | Reseed + cache |
| Periodicity mgmt | FPPC algorithm | Continuous reseeding |
| Authentication | No | KMAC (optional) |
| Quantum resistance | No | SHAKE256 + ML-DSA/ML-KEM |
| Forward secrecy | No | Yes (BLAKE3) |

Kelvin-Quantum is strictly more advanced — higher dimension, better security properties, and a complete KDF-to-stream-cipher pipeline rather than a standalone PRNG.

---

## 9. Related Work: Cryptography Using N-Body / Orbital Dynamics

### Halayka (2012) — N-Body PRNG
> **Halayka, S. (2012).** On leveraging the chaotic and combinatorial nature of deterministic n-body dynamics on the unit m-sphere in order to implement a pseudo-random number generator. *viXra*.

Halayka proposed using n-body gravitational dynamics as a pseudo-random number generator (PRNG). The approach was found to be computationally expensive compared to traditional LFSR-based PRNGs, limiting its practical applicability. Kelvin addresses this by using n-body dynamics specifically as a KDF (not a general PRNG), where the computational cost is a security feature rather than a drawback — the simulation cost is paid once during key derivation, not per-byte of output.

### Vuckovac (2021) — Cryptographic Puzzles and Complex Systems
> **Vuckovac, R. (2021).** Cryptographic Puzzles and Complex Systems. *Complex Systems*, 30(3), 375–390. doi:10.25088/ComplexSystems.30.3.375

Vuckovac suggested using n-body gravitational simulations as computational puzzles for proof-of-work (PoW) consensus mechanisms. The work identified n-body integration as a naturally hard problem suitable for Sybil resistance but did not extend the concept to a full encryption or key derivation system. Kelvin builds on this insight by using the same computational hardness as the foundation for a complete cryptosystem, including key derivation, stream encryption, and post-quantum identity.

### Kraicha et al. (2025) — Orbital-Inspired Encryption
> **Kraicha, Y., El Ghazi, M., & Benali, A. (2025).** Orbital-Inspired Encryption Using Phobos and Deimos Positions. *Journal of Information Security and Applications*, 88, 103912. doi:10.1016/j.jisa.2025.103912

Kraicha et al. used the real orbital positions of Mars's moons Phobos and Deimos as a shifting mechanism for encryption. While this work draws inspiration from orbital mechanics, the approach is metaphorical rather than simulated — it uses pre-computed ephemeris data rather than running an n-body simulation. Kelvin differs fundamentally by performing actual n-body gravitational simulation with deterministic fixed-point arithmetic, where the chaotic evolution of the system itself generates the cryptographic entropy.

---


## 7. Security & Side Channels

### Koeune & Standaert (2005) — Side-Channel Attacks
> **Koeune, F., & Standaert, F.-X. (2005).** A Tutorial on Physical Security and Side-Channel Attacks. In *Foundations of Security Analysis and Design III* (pp. 78–108). Springer. doi:10.1007/11554578_3

Comprehensive survey of side-channel attack methodology. Kelvin's design choices (fixed-point arithmetic, constant-time operations, no secret-dependent branching) are motivated by the attacks described in this work.

---

## 8. Reproducibility & Deterministic Builds

### Gent (2017) — The Recomputation Manifesto
> **Gent, I. P. (2017).** The Recomputation Manifesto. *Communications of the ACM*, 60(8), 44–51. doi:10.1145/3105966

Argues that computational experiments should be reproducible through deterministic recomputation. Kelvin's cross-platform test vector verification system is inspired by this principle — anyone can verify that their platform produces identical ciphertexts.

---

## Complete BibTeX

See [REFERENCES.bib](../REFERENCES.bib) for the full BibTeX file, ready for use with LaTeX.
