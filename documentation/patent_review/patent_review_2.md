# Patent Landscape Review #2 — Updated Prior Art Analysis & New Findings

**Status:** Second Review — Updated  
**Date:** 2026-05-28  
**Author:** Kelvin Project  
**Cross-references:** [Patent Review #1](patent_review_1.md), [REFERENCES.bib](../REFERENCES.bib)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Newly Identified Similar Projects](#2-newly-identified-similar-projects)
3. [Newly Identified Academic References](#3-newly-identified-academic-references)
4. [Patent Status Verification](#4-patent-status-verification)
5. [Updated Novelty Assessment](#5-updated-novelty-assessment)
6. [Updated Risk Assessment](#6-updated-risk-assessment)
7. [Updated Patent Strategy Recommendations](#7-updated-patent-strategy-recommendations)
8. [References](#8-references)

---

## 1. Executive Summary

### 1.1 Key Findings

This review updates the initial patent landscape analysis (Patent Review #1, 2026-05-27) with new web-based searches conducted on 2026-05-28 across Google Patents, arXiv, IACR ePrint, and GitHub.

**Critical findings:**

1. **OrBIT-KDF** (GitHub, created 2026-01-30) — A directly similar project using Chirikov standard map + Julia set iteration for key derivation. This is the closest known prior art to Kelvin. Licensed under Apache-2.0.

2. **CryptoChaos** (arXiv:2504.08618, 2025) — A hybrid chaos-based cryptographic framework combining four discrete chaotic maps with X25519 and AES-GCM. Demonstrates that hybrid chaos-crypto frameworks are an active research area.

3. **No new blocking patents found** — All patent searches returned zero results for queries related to n-body cryptography, gravitational key derivation, orbital key generation, chaotic KDF, simulation-based key generation, and celestial mechanics encryption.

4. **No IACR ePrint papers found** — Searches for "gravitational", "n-body", "orbital", "chaos key derivation", and "physical simulation cryptography" returned zero results on IACR ePrint.

5. **Existing patent status confirmed** — All four previously identified patents (Apple US 6,587,559, Toshiba US 6,014,445, CN102360488B, Nantes US 8,781,124) remain expired or lapsed.

### 1.2 Updated Landscape Summary

| Category | Status | Key Risk |
|----------|--------|----------|
| **N-body chaotic KDF** (broad concept) | ❌ **Blocked** — Apple US 6,587,559 (expired) + OrBIT-KDF prior art | Highest risk |
| **Chaotic stream cipher** (general method) | ❌ **Blocked** — Toshiba US 6,014,445 (expired) | High risk |
| **Chirikov map + Julia set KDF** | ⚠️ **Prior art exists** — OrBIT-KDF (2026-01-30) | Moderate risk |
| **Hybrid multi-map chaotic framework** | ⚠️ **Prior art exists** — CryptoChaos (2025) | Moderate risk |
| **Kelvin's specific architecture** (fixed-point n-body, Lyapunov, multi-mode, HE integration) | ✅ **Potentially novel** | Low risk |

---

## 2. Newly Identified Similar Projects

### 2.1 OrBIT-KDF (GitHub: cozmobaut-hub/OrBIT-KDF)

| Field | Detail |
|-------|--------|
| **Repository** | [https://github.com/cozmobaut-hub/OrBIT-KDF](https://github.com/cozmobaut-hub/OrBIT-KDF) |
| **Description** | "OrBIT is an experimental key derivation function that uses chaotic fractal dynamics plus a cryptographic hash to turn user credentials into high-entropy keys." |
| **Language** | Python |
| **License** | Apache-2.0 |
| **Stars** | 1 |
| **Created** | 2026-01-30 |
| **Last Updated** | 2026-02-11 |
| **Size** | 119 KB |

#### Architecture

OrBIT-KDF uses a **Chirikov standard map** on the torus combined with a **Julia iteration** in the complex plane:

1. **Username → Chirikov state**: SHA-512(username) → initial (x₀, p₀) in T²
2. **Chirikov dynamics**: pₙ₊₁ = pₙ + K·sin(xₙ) (mod 2π), xₙ₊₁ = xₙ + pₙ₊₁ (mod 2π)
3. **Username → Julia parameter**: From final Chirikov point, derive Julia parameter c
4. **Password → Julia state**: SHA-512(password) → initial complex condition z₀
5. **Julia iteration**: zₙ₊₁ = zₙ² + c
6. **Output**: Encode triples (xₙ, pₙ, zₙ) as 64-bit floats → SHA-512 → 512-bit digest

#### Sub-modules (from repository structure):
- `Chirikov&Julia/` — Standard map + Julia set
- `DeJong&Julia/` — De Jong attractor + Julia set
- `Henon&Julia/` — Hénon map + Julia set
- `Lorenz/` — Lorenz attractor
- `FullKDF/` — Complete KDF implementation
- `SFX/` — Sound effects / visualization

#### Comparison with Kelvin

| Aspect | OrBIT-KDF | Kelvin |
|--------|-----------|--------|
| **Core dynamics** | Chirikov standard map + Julia set | N-body gravitational simulation (Verlet/Euler) |
| **Chaos source** | 2D area-preserving map + complex iteration | 30-DOF (10 bodies × 3 dimensions) gravitational dynamics |
| **Extraction** | SHA-512 | SHAKE256 XOF (extendable output) |
| **Fixed-point** | No (floating-point) | Yes (Q32.64 fixed-point) |
| **Lyapunov monitoring** | No | Yes (shadow orbit method) |
| **Multi-mode** | Single KDF mode | 7 modes (Secure, Chaos, Photon, Quantum, Prism, Split, Flare) |
| **Domain separation** | No | Yes (domain-separated hashing per mode) |
| **HE integration** | No | Yes (Prism/Split/Flare) |
| **Formal verification** | No | Yes (Kani proof harnesses) |
| **Language** | Python | Rust |
| **License** | Apache-2.0 | Proprietary / MIT |

#### Risk Assessment

**Low risk.** OrBIT-KDF is a small experimental project (1 star, 119 KB, last updated February 2026) that uses fundamentally different chaotic dynamics (Chirikov map + Julia set) compared to Kelvin's n-body gravitational simulation. The architecture, extraction method, and security mechanisms are distinct. However, it establishes prior art for the general concept of "chaotic map + cryptographic hash = KDF" and specifically for "Chirikov map + Julia set" combinations.

---

### 2.2 CryptoChaos (arXiv:2504.08618)

| Field | Detail |
|-------|--------|
| **Authors** | Song et al. |
| **Published** | 2025-04-11 |
| **Link** | [https://arxiv.org/abs/2504.08618](https://arxiv.org/abs/2504.08618) |
| **Description** | "A Hybrid Chaos-Based Cryptographic Framework for Post-Quantum Secure Communications" |

#### Architecture

CryptoChaos combines four discrete chaotic maps with standard cryptographic primitives:
- Logistic map, Tent map, Sine map, Chebyshev map
- X25519 key exchange
- AES-GCM encryption
- Chaotic S-box generation

#### Comparison with Kelvin

| Aspect | CryptoChaos | Kelvin |
|--------|-------------|--------|
| **Chaos source** | 4 discrete 1D maps | N-body gravitational simulation |
| **Key exchange** | X25519 | N-body derived keys |
| **Encryption** | AES-GCM | Multi-mode (ChaCha20, SHAKE256, etc.) |
| **Chaos purpose** | S-box generation, key mixing | Full keystream generation |
| **Dimensionality** | 1D maps | 30-DOF (10 bodies × 3D) |

#### Risk Assessment

**Low risk.** CryptoChaos uses fundamentally different chaotic primitives (discrete 1D maps) and a different architecture (chaos-assisted standard crypto vs. Kelvin's chaos-as-primary-keystream). It establishes prior art for "hybrid chaos-crypto frameworks" broadly, but does not specifically cover n-body gravitational dynamics.

---

### 2.3 Other GitHub Projects Found

| Project | Stars | Description | Relevance |
|---------|-------|-------------|-----------|
| RachanaJayaram/Image-Encryption-Chaos-Maps | 123 | Image encryption using various chaos maps | Low — image-specific, uses standard logistic/tent maps |
| aarya-arun/Chaotic-Map-Image-Encryption | 13 | Image encryption based on chaos maps in Python | Low — image-specific |
| parth721/chaos-based_cryptography | 3 | Logistic map + XOR diffusion | Low — simple logistic map |
| souradipp76/PostQuantum_Crypto | 3 | Chaos-based Post Quantum Cryptography Algorithm | Low — no details available |

**None of these projects use n-body gravitational simulation for key derivation.**

---

## 3. Newly Identified Academic References

### 3.1 arXiv Papers

The following papers were found through systematic arXiv searches and are relevant to Kelvin's prior art landscape:

| Paper | Date | Relevance to Kelvin |
|-------|------|---------------------|
| **Chaotic flux cipher based on random cubic family** (arXiv:2603.20937) | 2026-03 | Symmetric stream cipher using random cubic mappings in complex plane. Related: uses complex dynamics for keystream generation, similar in spirit to Kelvin's chaos-based approach but different mathematical foundation. |
| **CryptoChaos** (arXiv:2504.08618) | 2025-04 | Hybrid chaos-crypto framework (see Section 2.2). |
| **Chaos in violent relaxation dynamics** (arXiv:2503.22479) | 2025-03 | N-body simulation chaos analysis. Not cryptographic, but relevant to understanding chaotic properties of n-body systems. |
| **Partial suppression of chaos in relativistic three-body problems** (arXiv:2410.15410) | 2024-10 | Studies chaos in gravitational n-body (3 ≤ N ≤ 10³). Relevant to understanding chaos properties of Kelvin's core dynamics. |
| **CS-PRNG using Robust Chaotic Tent Map** (arXiv:2408.05580) | 2024-08 | Chaotic PRNG design. General prior art for chaos-based RNG. |
| **Stability analysis via second-order Rényi entropy** (arXiv:2210.09417) | 2022-10 | N-body stability analysis using entropy. Relevant to Kelvin's stability monitoring. |
| **WHFast symplectic integrator** (arXiv:1506.01084) | 2015-06 | Fast Wisdom-Holman integrator for gravitational simulations. Relevant to Kelvin's integrator choices. |

### 3.2 IACR ePrint

**No relevant papers found.** Searches for "gravitational", "n-body", "orbital", "chaos key derivation", and "physical simulation cryptography" returned zero results on IACR ePrint. This confirms that the intersection of gravitational n-body simulation and cryptography has not been explored in the mainstream cryptography literature.

### 3.3 Updated Academic Landscape

| Category | Prior Art Status |
|----------|------------------|
| **N-body simulation for PRNG** | Halayka (2012) — proposed but found computationally expensive |
| **N-body for proof-of-work** | Vuckovac (2021) — suggested but not implemented |
| **Orbital positions for encryption** | Kraicha et al. (2025) — metaphorical, not simulated |
| **Chaotic stream cipher (general)** | Extensive prior art (Toshiba '445, many academic papers) |
| **Chirikov map + Julia set KDF** | OrBIT-KDF (2026) — direct prior art |
| **Hybrid chaos-crypto framework** | CryptoChaos (2025) — prior art |
| **Fixed-point n-body for crypto** | **No prior art found** — novel |
| **Lyapunov exponent as crypto parameter** | **No prior art found** — novel |
| **N-body KDF for homomorphic encryption** | **No prior art found** — novel |
| **Multi-mode integrated chaotic cryptosystem** | **No prior art found** — novel |

---

## 4. Patent Status Verification

### 4.1 Previously Identified Patents — Status Confirmed

All four previously identified patents were re-verified via Google Patents:

| Patent | Previous Status | Current Status | Change? |
|--------|----------------|----------------|---------|
| Apple US 6,587,559 | Expired 2019-02-20 | Expired | ✅ Confirmed |
| Toshiba US 6,014,445 | Expired 2016-10-22 | Expired | ✅ Confirmed |
| CN102360488B | Expired (fee related) | Expired | ✅ Confirmed |
| Nantes US 8,781,124 | Lapsed 2022 | Lapsed | ✅ Confirmed |

### 4.2 New Patent Searches — No New Blocking Patents Found

Systematic searches on Google Patents for the following queries returned **zero results**:

| Search Query | Result |
|-------------|--------|
| `"n-body" "key derivation" cryptography` | No results |
| `gravitational simulation cryptography key` | No results |
| `orbital mechanics key generation` | No results |
| `chaos based key derivation function` | No results |
| `n body simulation cryptographic key` | No results |
| `gravitational key derivation` | No results |
| `celestial mechanics cryptography` | No results |
| `verlet integration key generation` | No results |
| `chaotic key derivation function` | No results |
| `simulation based key generation` | No results |
| `celestial mechanics encryption` | No results |

**Conclusion:** No new blocking patents have been filed since the initial review. The patent landscape remains clear for Kelvin's specific implementation details.

### 4.3 OrBIT-KDF Patent Risk

OrBIT-KDF is released under Apache-2.0 license and has no associated patent filings (based on repository inspection). It is a small experimental project with no commercial backing. **No patent risk from OrBIT-KDF.**

---

## 5. Updated Novelty Assessment

### 5.1 What Remains Novel After This Review

| Invention | Novelty Status | Confidence |
|-----------|---------------|------------|
| Fixed-point n-body as deterministic one-way function | ✅ **Novel** — no prior art found | High |
| Lyapunov horizon as cryptographic security parameter | ✅ **Novel** — no prior art found | High |
| Chaotic key generation for homomorphic encryption | ✅ **Novel** — no prior art found | High |
| Multi-mode integrated chaotic cryptosystem | ✅ **Novel** — no prior art found | High |
| Hybrid deterministic-chaotic stream cipher (Quantum H) | ✅ **Novel** — no prior art found | High |
| Domain-separated keystream isolation | ✅ **Novel** — no prior art found | High |
| BLAKE3 reseeding for forward secrecy in chaotic systems | ✅ **Novel** — no prior art found | High |

### 5.2 What Is Now Established as Prior Art

| Concept | Prior Art Source | Impact |
|---------|-----------------|--------|
| Chirikov map + Julia set KDF | OrBIT-KDF (2026) | ⚠️ Kelvin does not use Chirikov or Julia dynamics |
| Hybrid chaos-crypto framework | CryptoChaos (2025) | ⚠️ Kelvin's architecture is fundamentally different |
| Chaotic stream cipher (general) | Toshiba '445 + academic papers | ⚠️ Already known, Kelvin's V2 mode is exposed |
| N-body for PRNG (general concept) | Halayka (2012) | ⚠️ Already known, Apple '559 covers this |

### 5.3 Key Distinction: Kelvin vs. OrBIT-KDF

The most important distinction for patent purposes:

| Kelvin Feature | OrBIT-KDF | Why It Matters |
|---------------|-----------|----------------|
| **N-body gravitational simulation** | Chirikov map + Julia set | Fundamentally different dynamical system |
| **30 DOF (10 bodies × 3D)** | 2D map + complex plane | Higher-dimensional chaos space |
| **Fixed-point arithmetic** | Floating-point | Cross-platform determinism |
| **Lyapunov exponent monitoring** | Not present | Security parameter, collapse/ejection detection |
| **SHAKE256 XOF** | SHA-512 | Extendable output, NIST-standard |
| **Multi-mode architecture** | Single KDF | 7 modes with domain separation |
| **HE integration** | Not present | Prism/Split/Flare modes |
| **Formal verification** | Not present | Kani proof harnesses |
| **Rust implementation** | Python | Memory safety, performance |

---

## 6. Updated Risk Assessment

### 6.1 Risk Matrix (Updated)

| Risk | Level | Mitigation |
|------|-------|------------|
| Apple '559 prior art blocking broad n-body claims | 🔴 **High** | Narrow claims to specific implementation details |
| OrBIT-KDF prior art for "chaotic map + hash = KDF" | 🟡 **Moderate** | Kelvin uses n-body, not Chirikov/Julia |
| CryptoChaos prior art for "hybrid chaos-crypto" | 🟡 **Moderate** | Kelvin's architecture is distinct |
| Toshiba '445 prior art for chaotic stream cipher | 🟡 **Moderate** | V2 mode exposed; patent expired |
| No prior art for fixed-point n-body crypto | 🟢 **Low** | Strong patent position |
| No prior art for Lyapunov crypto parameter | 🟢 **Low** | Strong patent position |
| No prior art for chaotic HE key generation | 🟢 **Low** | Strongest patent position |

### 6.2 Updated Warning Flags

1. **⚠️ OrBIT-KDF establishes prior art for "chaotic map + hash = KDF"** — While Kelvin uses n-body instead of Chirikov/Julia, a patent examiner could argue that the general concept of "iterating a chaotic system and hashing the result to produce a key" is obvious in light of OrBIT-KDF + Apple '559. Claims must be narrowly tailored to n-body gravitational dynamics specifically.

2. **⚠️ CryptoChaos establishes prior art for "hybrid chaos-crypto"** — Kelvin's multi-mode architecture could be seen as an obvious extension of the hybrid approach. Emphasize that Kelvin's modes are integrated (shared seed derivation, domain separation) rather than separate systems.

3. **⚠️ No new blocking patents, but the field is active** — The existence of OrBIT-KDF (created January 2026) and CryptoChaos (April 2025) shows that chaos-based cryptography is gaining attention. File the provisional patent application promptly to establish priority.

---

## 7. Updated Patent Strategy Recommendations

### 7.1 Priority Order (Updated)

| Priority | Invention | Rationale |
|----------|-----------|-----------|
| **1** | Fixed-point n-body as deterministic one-way function | Strongest novelty, no prior art |
| **2** | Lyapunov horizon as cryptographic security parameter | Strong novelty, no prior art |
| **3** | Chaotic key generation for homomorphic encryption | Strong novelty, no prior art |
| **4** | Hybrid deterministic-chaotic stream cipher (Quantum H) | Novel combination, no prior art |
| **5** | Multi-mode integrated architecture with domain separation | Novel integration pattern |

### 7.2 What to Emphasize in Claims

Based on the new findings, patent claims should emphasize:

1. **N-body gravitational simulation specifically** — Not "chaotic dynamics" broadly. The n-body problem has specific properties (gravitational force law, symplectic structure, 30 DOF) that distinguish it from Chirikov maps, Julia sets, logistic maps, etc.

2. **Fixed-point arithmetic** — The Q32.64 fixed-point solution for cross-platform determinism is a specific technical contribution not found in any prior art.

3. **Lyapunov exponent estimation** — Using shadow orbit method to determine safe encryption horizon is unique.

4. **Domain-separated multi-mode architecture** — The integration of 7 modes with shared seed derivation but isolated keystreams is novel.

### 7.3 What to Avoid in Claims

1. **"Chaotic map" or "chaotic dynamics" broadly** — Too broad, covered by Apple '559 and OrBIT-KDF prior art
2. **"Chaotic stream cipher" broadly** — Covered by Toshiba '445
3. **"Hybrid chaos-cryptography framework"** — Covered by CryptoChaos
4. **"Key derivation using chaotic system"** — Covered by OrBIT-KDF

### 7.4 Recommended Next Steps (Updated)

1. ✅ **Initial patent review** — Complete (Patent Review #1)
2. ✅ **Web-based prior art search** — Complete (this document)
3. 🔲 **Professional prior art search** — Pay a patent attorney for a formal prior art search ($2,000–$5,000)
4. 🔲 **Freedom-to-operate opinion** — Legal opinion on commercialization
5. 🔲 **Provisional patent application** — Draft and file within 6 months
6. 🔲 **Defensive publication** — Consider publishing non-patentable innovations on arXiv or GitHub to establish prior art against competitors (especially relevant given OrBIT-KDF's existence)

---

## 8. References

### Newly Identified References

1. **OrBIT-KDF** (2026). GitHub: cozmobaut-hub/OrBIT-KDF. [https://github.com/cozmobaut-hub/OrBIT-KDF](https://github.com/cozmobaut-hub/OrBIT-KDF)
2. **Song et al.** (2025). "A Hybrid Chaos-Based Cryptographic Framework for Post-Quantum Secure Communications." arXiv:2504.08618.
3. **Chaotic flux cipher** (2026). "A chaotic flux cipher based on the random cubic family." arXiv:2603.20937.
4. **Chaos in violent relaxation** (2025). arXiv:2503.22479.
5. **Relativistic three-body chaos** (2024). arXiv:2410.15410.
6. **CS-PRNG using Robust Chaotic Tent Map** (2024). arXiv:2408.05580.
7. **WHFast symplectic integrator** (2015). arXiv:1506.01084.

### Previously Identified References (from Patent Review #1)

8. **US 6,587,559** — Apple Inc. "Cryptographic System Using Chaotic Dynamics" (expired 2019)
9. **US 6,014,445** — Toshiba. "Stream Cipher Using Chaos" (expired 2016)
10. **CN102360488B** — "Image Encryption Based on Chaotic Orbit Perturbation" (lapsed)
11. **US 8,781,124** — Université de Nantes. "Chaotic Sequence Generator" (lapsed 2022)
12. **Halayka** (2012). "N-body dynamics for PRNG." viXra.
13. **Vuckovac** (2021). "Cryptographic Puzzles and Complex Systems." Complex Systems.
14. **Kraicha et al.** (2025). "Orbital-Inspired Encryption Using Phobos and Deimos." JISA.
15. **Cang, Kang & Wang** (2021). "Conservative Sprott-A PRNG." Nonlinear Dynamics.

---

*This document is part of the Kelvin Cryptosystem documentation suite.  
It represents an updated patent landscape analysis based on web searches conducted 2026-05-28.  
This should not be construed as legal advice. Consult a qualified patent attorney before filing any patent application.*

*Last Updated: 2026-05-28*
