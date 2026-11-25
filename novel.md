# Contribution Analysis: The Kelvin N-Body KDF

> **An objective assessment of the project's technical contributions to chaos-based cryptography, independent of its suitability as a production encryption primitive.**

**Cross-reference:** See the [Compliance Register](COMPLIANCE.md) for per-item production-readiness status and residual risks (formerly `bad.md`, removed 2026-10-10 after its fixable limbs were fixed).

---

## Abstract

This document evaluates the Kelvin project's contribution to chaos-based cryptography. The project addresses a specific open problem — cross-platform deterministic chaotic simulation for key derivation — which has limited the reproducibility and practical applicability of chaos-based cryptosystems since their inception in the 1990s. Kelvin's primary contributions are: (1) a Q32.64 fixed-point arithmetic engine that produces bit-identical chaotic trajectories on the tested toolchain (cross-architecture bit-identity follows from integer-only arithmetic by design; no multi-platform matrix currently covers x86_64/aarch64/riscv64/WASM); (2) machine-checked proofs of a limited set of implementation facts (bounded add/sub/mul ranges, division identities, Verlet kick schedule, pairwise momentum cancellation, domain-separation framing) using Kani and Lean 4 — the C1–C5 security claims themselves are not machine-checked; and (3) a domain-separated multi-mode architecture that isolates the chaotic KDF from downstream cipher operations. The cryptographic security of the system is bounded by SHAKE256 (128-bit post-quantum), which is equivalent to existing standards. The contribution is to the engineering practice of chaos-based cryptography, not to cryptographic theory. Measured basis: 215 library unit tests, 8/8 golden vectors, 12 DudeCT benches with |t| < 5, and a full release gate green 2026-10-10.

---

## 1. Introduction: The Reproducibility Problem in Chaos-Based Cryptography

Chaos-based cryptography has been studied since the 1990s (Kocarev, 2001; Jakimoski & Kocarev, 2001; Alvarez & Li, 2006). A recurring criticism — articulated, for example, by Li et al. (2005) and Arroyo et al. (2009) — is that the field suffers from a systematic reproducibility deficit. Most published chaos-based cryptosystems are implemented in IEEE 754 floating-point arithmetic, which means that simulation results depend on:

1. **Compiler optimization level:** Different levels (-O2 vs -O3 vs -Os) can reorder floating-point operations, producing different results.
2. **Target architecture:** x86, ARM, and WASM implementations of transcendental functions differ, and fused multiply-add (FMA) behaviour varies.
3. **Runtime library:** glibc, musl, and MSVC implement different numeric approximation strategies for sin/cos/sqrt.

Arroyo et al. (2009) propose a framework for cryptanalysing digital chaos-based ciphers, motivated by the many published schemes later shown to be weak; reproducibility problems from platform-dependent floating-point behaviour compound this. (An earlier draft quoted a "22 of 37 flawed schemes" figure that could not be traced to a source; it has been removed.)

Kelvin addresses one specific sub-problem of this larger failure mode: **cross-platform determinism of chaotic simulations.**

---

## 2. Related Work and Context

### 2.1 Chaos-Based Stream Ciphers

The chaotic cryptographic literature contains proposals based on logistic maps, Lorenz attractors, Hénon maps, and Chua's circuits (Kocarev, 2001; Yang et al., 2010). These systems typically operate on 1–3 degrees of freedom, making them potentially vulnerable to phase-space reconstruction attacks (Álvarez et al., 2003; Short, 1994). Kelvin uses a gravitational n-body model with 48 degrees of freedom at the standard level (8 bodies × 6; 30 in the 5-body analysis configuration), which increases the dimensionality of the phase space but does not provide a formal security reduction.

### 2.2 Deterministic Fixed-Point Chaos

Cang et al. (2021) proposed the Finite Precision Period Calculation (FPPC) algorithm to detect periodicity in discrete implementations of chaotic systems. Their Sprott-A PRNG used 64-bit floating-point precision and demonstrated that finite-precision degradation is a measurable concern even at 64 bits. Kelvin uses 128-bit Q32.64 fixed-point arithmetic, which provides a larger state space per variable but does not eliminate the periodicity concern — it only shifts the bound.

### 2.3 N-Body Chaotic Cryptography

Kwong et al. (2025) published the first known n-body chaotic image encryption scheme using a seven-dimensional memristor-restricted four-body system for image encryption. Song et al. (2025) published CryptoChaos, a hybrid chaos-cryptography framework combining four chaotic maps (Logistic, Chebyshev, Tent, Hénon) with SHA3-256, X25519, HKDF and AES-GCM. Neither is described as verifying cross-platform determinism.

Jawad (2025) proposed DUff-skg, a chaotic FHE key generator using a 2-DOF Duffing oscillator with RK4 floating-point integration. Kelvin's Flare mode uses a 48-DOF (standard level) n-body model and fixed-point arithmetic as an alternative approach to the same problem.

---

## 3. Technical Contributions

### 3.1 Q32.64 Fixed-Point Arithmetic for Chaotic Simulation

The simulation uses a custom Q32.64 fixed-point representation on i128 (64 fractional bits, 32 integer bits; the upper 32 bits are unused sign extension, not guard bits). All operations — addition, multiplication, division, and square root — are defined by integer arithmetic:

```
Addition/Subtraction: Exact on i128
Multiplication:      (a_raw × b_raw) >> 64
Division:            192-iteration restoring division
Square root:         Binary digit-by-digit, 64 iterations
```

This approach has the following properties:

- **No floating-point in the simulation engine itself** (keygen sampling and display helpers use f64 outside the deterministic path). No transcendental functions, no rounding modes, no FMA contraction in the core loop.
- **Bit-identical on the tested toolchain.** 18 determinism tests (golden hashes + intra-process repeatability) pass; SSE2/AVX/AVX2/NEON matrix coverage is asserted by design, not automated.
- **Measured constant-time behaviour (statistical, not proved).** dudect-bencher (Welch's t-test) shows |t| < 5 for the 12 benches in `tests/constant_time_bench/`; `verlet_step` showed |t| ≈ 75 in one configuration and is under investigation (see `COMPLIANCE.md`).

**Limitation:** Constant-time verification covers arithmetic primitives but not the full encrypt/decrypt pipeline. Higher-level operations (mode selection branching, authentication tag comparison) are not yet verified.

**Context:** Cross-platform deterministic chaos is a prerequisite for reproducible chaos-based cryptography. Kelvin provides this. That does not make the system cryptographically secure — it makes it reproducible. Reproducibility is necessary but not sufficient for cryptographic security.

### 3.2 Formal Verification of Simulation Properties

Status is tracked in [`documentation/formal_verification.md`](documentation/formal_verification.md) and [`documentation/formal_verification/PROOF_INDEX.md`](documentation/formal_verification/PROOF_INDEX.md). In summary (2026-11-25):

| Level | Target | Status |
|-------|--------|--------|
| L0 | No overflow for bounded add/sub/mul | Proved (Kani `fixed_math.rs`; Lean `FixedPoint`) |
| L1 | Fixed-point ops match the mathematical spec | Partial — division/remainder identities only |
| L2 | `compute_accelerations` satisfies Newton's laws | Partial — Lean pairwise momentum cancellation only |
| L3 | Pipeline executes the right steps with the right domain separators | Not proved (domain framing is proved in Lean/Kani; the pipeline is tested) |
| L4 | Bit-identical output on the tested toolchain | Tested (18 golden-hash and repeatability tests), not proved |

37 Kani harnesses are compiled into `kelvin-core`; other files under `proofs/kani/` are uncompiled templates.

**Limitation:** These are implementation-correctness checks on a narrow set of facts. They do not prove cryptographic security, and there is no verified refinement from the Lean models to the Rust code.

**Context:** Applying bounded model checking and theorem proving to a chaotic simulation is uncommon in the chaos-cryptography literature (Cang et al., 2021 and Kwong et al., 2025 did not), but the coverage here is partial and should not be read as a verified simulator.

### 3.3 Higher-DOF Gravitational Model

Kelvin uses a full gravitational n-body simulation with N ≥ 3 bodies (standard: 8 bodies, i.e. 48 DOF; maximum: 10 bodies, 60 DOF; the 30-DOF figure refers to the 5-body analysis configuration). This provides an order of magnitude more degrees of freedom (N × 3 × 2 for positions and velocities) compared with the 1–3 DOF typical of logistic-map or Lorenz-based systems.

| System | DOF | Prior use in cryptography |
|--------|-----|--------------------------|
| Logistic map | 1 | Kocarev (2001), widespread |
| Hénon map | 2 | Jakimoski & Kocarev (2001) |
| Lorenz attractor | 3 | Yang et al. (2010) |
| Chua's circuit | 3 | Dachselt & Schwarz (2001) |
| CryptoChaos (Song 2025) | 4 (4 maps × 1 DOF) | Hybrid framework |
| **Kelvin n-body** | **48** (standard; 30 at N=5) | **Gravitational n-body model; first known use as a KDF** |

The 30-DOF model does not provide a formal security guarantee — the relationship between DOF count and cryptographic security is not established. However, higher-dimensional phase spaces increase the data requirements for phase-space reconstruction attacks (Takens' theorem requires 2D+1 dimensions, so a 30-DOF analysis configuration needs at least 61 dimensions (97 at the 48-DOF standard level) of embedded data for reconstruction — a practical barrier for finite keystream observations).

**Limitation:** No formal proof links DOF count to attack resistance. This is a plausibility argument, not a security reduction.

### 3.4 Domain-Separated Multi-Mode Architecture

The system defines 7 distinct operational modes, each isolated by a cryptographic domain separator in the SHAKE256 extraction step:

| Mode | Keystream Domain Separator | Purpose |
|------|-----------------|---------|
| V1 Secure | DOMSEP_ORBITAL_STATE_V1 (seed extraction; AEAD cipher) | ChaCha20Poly1305 AEAD |
| V2 Chaos | `kelvin-streaming-v2-v1-000000000` (+ `DOMSEP_STREAMING_MAC_KEY_V1` for the auth tag key) | Per-step streaming stream cipher |
| V3 Photon | DOMSEP_PHOTON_KEYSTREAM_V1 | HKDF→SHAKE256 batch stream cipher |
| H Quantum | DOMSEP_QUANTUM_KEYSTREAM (+ perturb/reseed/cache domains) | Hybrid cache with orbital reseeding |
| Prism | DOMSEP_PRISM_KEYSTREAM_V1 | HE key generation |
| Split | DOMSEP_SPLIT_KEYSTREAM_V1 | XOR key splitting |
| Flare | DOMSEP_FLARE_KEYSTREAM_V1 | FHE secret key generation |

Domain separation between derived keys is a standard cryptographic practice (Krawczyk, 2010; NIST SP 800-185). Kelvin applies it correctly. The architecture cleanly separates the chaotic KDF (which produces a seed) from the downstream cipher operations (which consume it via standard SHAKE256 XOR).

**Limitation:** Domain separation prevents cross-mode key collision. It does not provide cryptographic security where none exists. The KDF's security is a conjecture, not a reduction.

---

## 4. Position Within the Field

Kelvin is positioned within the chaos-based cryptography subfield, not within mainstream symmetric cryptography. The relevant comparison is to other chaos-based systems, not to AES or ChaCha20.

| Prior System | Deterministic | Verified | DOF | Key Use |
|-------------|---------------|----------|-----|---------|
| Kwong et al. (2025) | ❌ Float | ❌ | 7-D (restricted four-body) | Image encryption only |
| Song et al. (2025) | ❌ Float | ❌ | 4× 1D maps | Hybrid stream |
| Cang et al. (2021) | ❌ Float | ❌ | 3 | PRNG only |
| Jawad (2025) | ❌ Float (RK4) | ❌ | 2 | FHE keys only |
| **Kelvin** | **✅ Fixed-point** | **⚠️ Partial (see §3.2)** | **48 (standard)** | **General-purpose** |

---

## 5. Limitations

1. **The cryptographic security is bounded by SHAKE256 (128-bit post-quantum).** The n-body KDF does not raise this bound. Formal verification C3 and C4 contain errors in their bound calculations (see `documentation/adversarial_critique/adversarial_critique_v4.md`).

2. **No formal reduction to a known hard problem exists.** The n-body inversion problem is not known to be as hard as factoring, discrete log, or LWE. This is a conjecture.

3. **Finite-precision periodicity is an open problem.** No concrete lower bound on the period length of the Q32.64 n-body simulation has been established.

4. **No independent cryptanalysis has been performed.** The system has not been reviewed by the cryptographic community.

5. **The key generator is RNG-limited.** The reachable keyspace (2^256) is smaller than the representable configuration space (2^1920).

---

## 6. Conclusion

Kelvin's technical contributions are to the engineering practice of chaos-based cryptography: it solves the cross-platform determinism problem, it applies formal verification to a chaotic simulation, and it provides a clean architectural separation between key derivation and cipher operations. These contributions are real and measurable. They do not constitute a security proof, a production-ready cipher, or a replacement for established cryptographic standards.

The project is correctly described as: *"A research implementation of a chaotic n-body KDF with golden-hash determinism tests and formal proofs of implementation correctness, accompanied by reference stream cipher modes that demonstrate seed consumption via SHAKE256 XOR. The KDF's security is a conjecture; the cipher security is standard SHAKE256."*

---

## References

- Arroyo, D., Alvarez, G., Li, S., Li, C., & Fernandez, V. (2009). "A basic framework for the cryptanalysis of digital chaos-based cryptography." *2009 3rd International Conference on Signals, Circuits and Systems (SCS)*. [doi:10.1109/SSD.2009.4956652](https://doi.org/10.1109/SSD.2009.4956652)
- Alvarez, G. & Li, S. (2006). "Some basic cryptographic requirements for chaos-based cryptosystems." *International Journal of Bifurcation and Chaos*, 16(8), 2129–2151.
- Cang, S., Kang, Z., & Wang, Z. (2021). "Pseudo-random number generator based on a generalized conservative Sprott-A system." *Nonlinear Dynamics*, 104, 827–844. [doi:10.1007/s11071-021-06310-9](https://doi.org/10.1007/s11071-021-06310-9)
- Kwong, H., Chai, Z., & Wu, T. (2025). "Chaotic Image Encryption Based on an Improved Seven-Dimensional Memristor-based Restricted Four-Body System." *2025 4th International Joint Conference on Information and Communication Engineering (JCICE)*. [doi:10.1109/JCICE66205.2025.11182029](https://doi.org/10.1109/JCICE66205.2025.11182029)
- Jakimoski, G. & Kocarev, L. (2001). "Chaos and cryptography: Block encryption ciphers based on chaotic maps." *IEEE Transactions on Circuits and Systems I*, 48(2), 163–169.
- Jawad, N. H. (2025). "FHE cryptographic systems with using chaotic secret key generation (DUff-skg)." *Boletim da Sociedade Paranaense de Matemática*. [doi:10.5269/bspm.77855](https://doi.org/10.5269/bspm.77855)
- Kocarev, L. (2001). "Chaos-based cryptography: A brief overview." *IEEE Circuits and Systems Magazine*, 1(3), 6–21.
- Li, S., Chen, G., & Mou, X. (2005). "On the dynamical degradation of digital piecewise linear chaotic maps." *International Journal of Bifurcation and Chaos*, 15(10), 3119–3151.
- NIST SP 800-185 (2016). "SHA-3 Derived Functions: cSHAKE, KMAC (Keccak Message Authentication Code), TupleHash, and ParallelHash."
- Short, K. M. (1994). "Steps toward unmasking secure communications." *International Journal of Bifurcation and Chaos*, 4(4), 959–977.
- Song, K., Imran, N., Chen, J. Y., & Dobbins, A. C. (2025). "A Hybrid Chaos-Based Cryptographic Framework for Post-Quantum Secure Communications" (CryptoChaos). [arXiv:2504.08618](https://arxiv.org/abs/2504.08618).
- Takens, F. (1981). "Detecting strange attractors in turbulence." *Dynamical Systems and Turbulence*, 366–381.
- Yang, J., et al. (2010). "A new chaotic image encryption scheme using Lorenz system." *International Journal of Bifurcation and Chaos*, 20(11), 3715–3730.