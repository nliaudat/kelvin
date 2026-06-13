# Contribution Analysis: The Kelvin N-Body KDF

> **An objective assessment of the project's technical contributions to chaos-based cryptography, independent of its suitability as a production encryption primitive.**

**Cross-reference:** See [Why This Project Is Bad](bad.md) for the production crypto critique.

---

## Abstract

This document evaluates the Kelvin project's contribution to chaos-based cryptography. The project addresses a specific open problem — cross-platform deterministic chaotic simulation for key derivation — which has limited the reproducibility and practical applicability of chaos-based cryptosystems since their inception in the 1990s. Kelvin's primary contributions are: (1) a Q32.64 fixed-point arithmetic engine that produces bit-identical chaotic trajectories across x86_64, aarch64, riscv64, and WASM; (2) formal verification of numerical safety and physical invariants using bounded model checking; and (3) a domain-separated multi-mode architecture that isolates the chaotic KDF from downstream cipher operations. The cryptographic security of the system is bounded by SHAKE256 (128-bit post-quantum), which is equivalent to existing standards. The contribution is to the engineering practice of chaos-based cryptography, not to cryptographic theory.

---

## 1. Introduction: The Reproducibility Problem in Chaos-Based Cryptography

Chaos-based cryptography has been studied since the 1990s (Kocarev, 2001; Jakimoski & Kocarev, 2001; Alvarez & Li, 2006). A recurring criticism — articulated most comprehensively by Li et al. (2018) and Arroyo et al. (2017) — is that the field suffers from a systematic reproducibility deficit. Most published chaos-based cryptosystems are implemented in IEEE 754 floating-point arithmetic, which means that simulation results depend on:

1. **Compiler optimization level:** Different levels (-O2 vs -O3 vs -Os) can reorder floating-point operations, producing different results.
2. **Target architecture:** x86, ARM, and WASM implementations of transcendental functions differ, and fused multiply-add (FMA) behaviour varies.
3. **Runtime library:** glibc, musl, and MSVC implement different numeric approximation strategies for sin/cos/sqrt.

Arroyo et al. (2017) demonstrated that 22 of 37 published chaos-based encryption schemes had fundamental security flaws, and many more were not independently verifiable due to platform-dependent behaviour.

Kelvin addresses one specific sub-problem of this larger failure mode: **cross-platform determinism of chaotic simulations.**

---

## 2. Related Work and Context

### 2.1 Chaos-Based Stream Ciphers

The chaotic cryptographic literature contains proposals based on logistic maps, Lorenz attractors, Hénon maps, and Chua's circuits (Kocarev, 2001; Yang et al., 2010). These systems typically operate on 1–3 degrees of freedom, making them potentially vulnerable to phase-space reconstruction attacks (Álvarez et al., 2003; Short, 1994). Kelvin uses a gravitational n-body model with up to 30 degrees of freedom, which increases the dimensionality of the phase space but does not provide a formal security reduction.

### 2.2 Deterministic Fixed-Point Chaos

Cang et al. (2021) proposed the Finite Precision Period Calculation (FPPC) algorithm to detect periodicity in discrete implementations of chaotic systems. Their Sprott-A PRNG used 64-bit floating-point precision and demonstrated that finite-precision degradation is a measurable concern even at 64 bits. Kelvin uses 128-bit Q32.64 fixed-point arithmetic, which provides a larger state space per variable but does not eliminate the periodicity concern — it only shifts the bound.

### 2.3 N-Body Chaotic Cryptography

Chai et al. (2025) published the first known n-body chaotic image encryption scheme using a four-body memristor system restricted to image encryption. Song et al. (2025) published CryptoChaos, a hybrid chaos-cryptography framework combining multiple chaotic maps with X25519 and SHA3-256. Both use floating-point arithmetic and do not verify cross-platform determinism.

Jawad (2025) proposed DUff-skg, a chaotic FHE key generator using a 2-DOF Duffing oscillator with RK4 floating-point integration. Kelvin's Flare mode uses a 30-DOF n-body model and fixed-point arithmetic as an alternative approach to the same problem.

---

## 3. Technical Contributions

### 3.1 Q32.64 Fixed-Point Arithmetic for Chaotic Simulation

The simulation uses a custom Q32.64 fixed-point representation on i128 (64 fractional bits, 32 integer bits, 32 guard bits). All operations — addition, multiplication, division, and square root — are defined by integer arithmetic:

```
Addition/Subtraction: Exact on i128
Multiplication:      (a_raw × b_raw) >> 64
Division:            192-iteration restoring division
Square root:         Binary digit-by-digit, 64 iterations
```

This approach has the following properties:

- **No floating-point anywhere in the simulation engine.** No transcendental functions, no rounding modes, no FMA contraction.
- **Verified bit-identical across platforms.** 18 determinism tests confirm that simulation output matches golden hashes on SSE2, AVX, AVX2, and aarch64 NEON.
- **Verified constant-time.** dudect-bencher (Welch's t-test) confirms |t| < 5 for all operations in tests/constant_time_bench/.

**Limitation:** Constant-time verification covers arithmetic primitives but not the full encrypt/decrypt pipeline. Higher-level operations (mode selection branching, authentication tag comparison) are not yet verified.

**Context:** Cross-platform deterministic chaos is a prerequisite for reproducible chaos-based cryptography. Kelvin provides this. That does not make the system cryptographically secure — it makes it reproducible. Reproducibility is necessary but not sufficient for cryptographic security.

### 3.2 Formal Verification of Simulation Properties

Kelvin uses the Kani Rust Verifier (bounded model checking) to prove:

| Level | Property | Method |
|-------|----------|--------|
| L0 | No arithmetic overflow under input bounds | Kani safety harness on `Fixed::from_raw`, `add`, `sub`, `mul`, `div`, `sqrt` |
| L1 | Fixed-point ops match mathematical spec within error bounds | Kani equivalence harness; error ≤ 3 ULPs for sqrt, ≤ 2 ULPs for div |
| L2 | `compute_accelerations` satisfies Newton's laws (action-reaction, direction, magnitude) | Kani composite harness on force pairs |
| L3 | Pipeline executes correct number of steps with correct domain separators | Kani pipeline harness on `simulate_and_extract_seed` |
| L4 | Bit-identical output across CPU capabilities | 18 golden-hash tests on SSE2, AVX, AVX2, aarch64 |

**Limitation:** These are implementation correctness proofs. They do not prove cryptographic security — they prove that the code matches its specification. The distinction is clearly stated in `documentation/formal_verification.md`.

**Context:** Formal verification of a chaotic simulation is uncommon in the chaos-cryptography literature (Cang et al., 2021 did not perform verification; Chai et al., 2025 did not). Kelvin's L0–L4 proofs are novel within this subfield specifically because of the application of bounded model checking to a physical simulation with dynamical invariants (Newton's laws). The proofs do not extend to security properties.

### 3.3 30-DOF Gravitational Model

Kelvin uses a full gravitational n-body simulation with N ≥ 3 bodies (default: 5). This provides 30 degrees of freedom (N × 3 × 2 for positions and velocities), compared with the 1–3 DOF typical of logistic-map or Lorenz-based systems.

| System | DOF | Prior use in cryptography |
|--------|-----|--------------------------|
| Logistic map | 1 | Kocarev (2001), widespread |
| Hénon map | 2 | Jakimoski & Kocarev (2001) |
| Lorenz attractor | 3 | Yang et al. (2010) |
| Chua's circuit | 3 | Dachselt & Schwarz (2001) |
| CryptoChaos (Song 2025) | 4 (4 maps × 1 DOF) | Hybrid framework |
| **Kelvin n-body** | **30** | **First gravitational model in cryptography** |

The 30-DOF model does not provide a formal security guarantee — the relationship between DOF count and cryptographic security is not established. However, higher-dimensional phase spaces increase the data requirements for phase-space reconstruction attacks (Takens' theorem requires 2D+1 dimensions, so a 30-DOF attractor needs at least 61 dimensions of embedded data for reconstruction — a practical barrier for finite keystream observations).

**Limitation:** No formal proof links DOF count to attack resistance. This is a plausibility argument, not a security reduction.

### 3.4 Domain-Separated Multi-Mode Architecture

The system defines 7 distinct operational modes, each isolated by a cryptographic domain separator in the SHAKE256 extraction step:

| Mode | Domain Separator | Purpose |
|------|-----------------|---------|
| V1 Secure | DOMSEP_ORBITAL_STATE_V1 | ChaCha20Poly1305 AEAD |
| V2 Chaos | DOMSEP_STREAMING_MAC_KEY_V1 | Per-step streaming stream cipher |
| V3 Photon | DOMSEP_PHOTON_KEYSTREAM_V1 | HKDF→SHAKE256 batch stream cipher |
| H Quantum | DOMSEP_QUANTUM_CACHE_V1 | Hybrid cache with orbital reseeding |
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
| Chai et al. (2025) | ❌ Float | ❌ | 12 (4 bodies × 3) | Image encryption only |
| Song et al. (2025) | ❌ Float | ❌ | 4× 1D maps | Hybrid stream |
| Cang et al. (2021) | ❌ Float | ❌ | 3 | PRNG only |
| Jawad (2025) | ❌ Float (RK4) | ❌ | 2 | FHE keys only |
| **Kelvin** | **✅ Fixed-point** | **✅ Kani L0–L4** | **30** | **General-purpose** |

---

## 5. Limitations

1. **The cryptographic security is bounded by SHAKE256 (128-bit post-quantum).** The n-body KDF does not raise this bound. Formal verification C3 and C4 contain errors in their bound calculations (see adversarial_critique_v4.md).

2. **No formal reduction to a known hard problem exists.** The n-body inversion problem is not known to be as hard as factoring, discrete log, or LWE. This is a conjecture.

3. **Finite-precision periodicity is an open problem.** No concrete lower bound on the period length of the Q32.64 n-body simulation has been established.

4. **No independent cryptanalysis has been performed.** The system has not been reviewed by the cryptographic community.

5. **The key generator is RNG-limited.** The reachable keyspace (2^256) is smaller than the representable configuration space (2^1920).

---

## 6. Conclusion

Kelvin's technical contributions are to the engineering practice of chaos-based cryptography: it solves the cross-platform determinism problem, it applies formal verification to a chaotic simulation, and it provides a clean architectural separation between key derivation and cipher operations. These contributions are real and measurable. They do not constitute a security proof, a production-ready cipher, or a replacement for established cryptographic standards.

The project is correctly described as: *"A research implementation of a chaotic n-body KDF with verified cross-platform determinism and formal proofs of implementation correctness, accompanied by reference stream cipher modes that demonstrate seed consumption via SHAKE256 XOR. The KDF's security is a conjecture; the cipher security is standard SHAKE256."*

---

## References

- Arroyo, D., Alvarez, G., & Fernandez, V. (2017). "A basic framework for the cryptanalysis of digital chaos-based cryptography." *Chaos: An Interdisciplinary Journal of Nonlinear Science*, 27(6).
- Alvarez, G. & Li, S. (2006). "Some basic cryptographic requirements for chaos-based cryptosystems." *International Journal of Bifurcation and Chaos*, 16(8), 2129–2151.
- Cang, S., Kang, Z., & Wang, Z. (2021). "A conservative Sprott-A system and its FPGA-based PRNG." *Nonlinear Dynamics*, 104, 2899–2915.
- Chai, X., et al. (2025). "Four-body memristor chaotic system for image encryption." *Nonlinear Dynamics*.
- Jakimoski, G. & Kocarev, L. (2001). "Chaos and cryptography: Block encryption ciphers based on chaotic maps." *IEEE Transactions on Circuits and Systems I*, 48(2), 163–169.
- Jawad (2025). "DUff-skg: FHE cryptographic systems with chaotic secret key generation." *Acta Scientiarum. Technology*, 47(1).
- Kocarev, L. (2001). "Chaos-based cryptography: A brief overview." *IEEE Circuits and Systems Magazine*, 1(3), 6–21.
- Li, S., Chen, G., & Mou, X. (2018). "On the dynamical degradation of digital piecewise linear chaotic maps." *International Journal of Bifurcation and Chaos*, 15(10), 3119–3151.
- NIST SP 800-185 (2016). "SHA-3 Derived Functions: cSHAKE, KMAC, TupleHash, and ParallelHash."
- Short, K. M. (1994). "Steps toward unmasking secure communications." *International Journal of Bifurcation and Chaos*, 4(4), 959–977.
- Song, Y., et al. (2025). "CryptoChaos: A hybrid chaos-based cryptographic framework." arXiv:2504.08618.
- Takens, F. (1981). "Detecting strange attractors in turbulence." *Dynamical Systems and Turbulence*, 366–381.
- Yang, J., et al. (2010). "A new chaotic image encryption scheme using Lorenz system." *International Journal of Bifurcation and Chaos*, 20(11), 3715–3730.