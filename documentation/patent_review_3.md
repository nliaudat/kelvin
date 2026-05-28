# Patent Landscape Review #3 — IEEE Xplore NPL Deep-Dive & Updated Assessment

**Status:** Third Review — Expanded NPL Search  
**Date:** 2026-05-28  
**Author:** Kelvin Project  
**Cross-references:** [Patent Review #1](patent_review_1.md), [Patent Review #2](patent_review_2.md), [REFERENCES.bib](../REFERENCES.bib)

---

## Résumé

A systematic search of the **IEEE Xplore** digital library (2000–2025) across 7 query categories revealed **5 new prior art papers** that significantly reshape the novelty landscape for the Kelvin cryptosystem.

| Query | Result | Impact on Kelvin |
|-------|--------|:----------------:|
| **Q1** — N-body cryptography | ✅ **Chai et al. (2025)** — Four-body memristor 7D chaotic image encryption (JCICE 2025, Harbin) | 🔴 **Direct prior art** — first known n-body derived chaotic cryptosystem |
| **Q2** — Chaotic stream cipher (XOR) | ✅ **Weng et al. (2009)** — Orbit perturbation for continuous-time chaotic stream ciphers (CISE 2009) | 🟡 Prior art confirmed for orbit perturbation concept |
| **Q3** — Lyapunov in cryptography | ⚠️ Lyapunov used only as validation metric across all found papers | 🟢 No direct claim blocking — but weakens "novel use" argument |
| **Q4** — Fixed-point + n-body + crypto | ❌ **No prior art found** in IEEE Xplore, arXiv, or IACR ePrint | ✅ **Strongest novelty claim remains intact** |
| **Q5** — HE + chaos + key generation | ✅ **4 papers found** including DUff-skg (Jawad, 2025) for Duffing+FHE key generation | 🔴 **New RED risk** — Flare mode claims directly overlap with DUff-skg |
| **Q6** — Orbit perturbation encryption | ✅ **Song (2012)** — Chaotic orbit perturbation in diffusion (IWCFTA 2012) | 🟡 Prior art confirmed for orbit perturbation in encryption |
| **Q7** — Chaotic reseeding | ❌ **No peer-reviewed prior art** (only unverified mailing list post) | ✅ **Novel** — candidate for strongest remaining claim |

### Key Takeaways

1. **Fixed-point n-body + cryptography (Q4)** remains the strongest novelty claim — no prior art in any database searched.
2. **Chaotic HE key generation (Q5)** is no longer novel — DUff-skg (Jawad, 2025) directly covers Duffing chaos + BFV/CKKS FHE key generation. Kelvin must distinguish via 30-DOF n-body vs. 2-DOF Duffing, fixed-point vs. RK4 floating, and domain separation.
3. **Chaotic reseeding (Q7)** has no peer-reviewed prior art — this is a newly strengthened novelty claim.
4. **N-body cryptography (Q1)** now has direct prior art beyond Apple '559 — Chai et al. (2025) published a four-body memristor chaotic image encryption system. Kelvin must distinguish via full gravitational simulation (vs. restricted four-body), fixed-point (vs. floating), and general-purpose (vs. image-specific).

---

## Table of Contents

1. [Résumé](#résumé)
2. [Executive Summary](#1-executive-summary)
3. [IEEE Xplore NPL Search Results by Query](#2-ieee-xplore-npl-search-results-by-query)
4. [HE + Chaos Prior Art Deep-Dive](#3-he--chaos-prior-art-deep-dive)
5. [Patent Status Verification](#4-patent-status-verification)
6. [Updated Novelty Assessment](#5-updated-novelty-assessment)
7. [Updated Risk Assessment](#6-updated-risk-assessment)
8. [Updated Patent Strategy Recommendations](#7-updated-patent-strategy-recommendations)
9. [References](#8-references)

---

## 1. Executive Summary

### 1.1 Scope of This Review

This third review incorporates results from a systematic **IEEE Xplore** digital library search conducted across 7 query categories, covering publications from 2000–2025. The search targeted non-patent literature (NPL) that may constitute prior art for Kelvin's patent claims. Results from arXiv and IACR ePrint (documented in Patent Review #2) are incorporated for completeness.

### 1.2 Critical Changes from Previous Reviews

| Finding | Previous Status (v2) | New Status (v3) | Change |
|---------|---------------------|-----------------|--------|
| N-body cryptography prior art | Apple '559 (expired) only | + **Chai et al. (2025)** — four-body memristor system | 🔴 New direct prior art |
| Chaotic HE key generation | No prior art found | + **DUff-skg (2025)** + 3 other papers | 🔴 New RED risk |
| Fixed-point n-body + crypto | Novel — no prior art | **Still novel** — no prior art | ✅ Confirmed |
| Chaotic reseeding | Novel — no prior art | **Still novel** — no peer-reviewed prior art | ✅ Strengthened |
| Lyapunov as crypto parameter | Novel — no prior art | ⚠️ **Weakened** — widely used as validation metric | 🟡 Downgraded |

### 1.3 Updated Landscape Summary

| Category | Status | Key Risk |
|----------|--------|----------|
| **N-body chaotic cryptography** (broad) | ❌ **Blocked** — Apple '559 + Chai et al. (2025) | Highest risk |
| **Chaotic stream cipher** (general) | ❌ **Blocked** — Toshiba '445 + Weng et al. (2009) | High risk |
| **HE + chaotic key generation** | ❌ **Prior art exists** — DUff-skg (2025) + 3 papers | High risk |
| **Orbit perturbation encryption** | ⚠️ **Prior art exists** — Song (2012), Weng et al. (2009) | Moderate risk |
| **Chirikov map + Julia set KDF** | ⚠️ **Prior art exists** — OrBIT-KDF (2026) | Moderate risk |
| **Fixed-point n-body for crypto** | ✅ **Novel** — no prior art in any database | Low risk |
| **Chaotic reseeding for forward secrecy** | ✅ **Novel** — no peer-reviewed prior art | Low risk |
| **Multi-mode integrated architecture** | ✅ **Novel** — no prior art found | Low risk |
| **Domain-separated keystream isolation** | ✅ **Novel** — no prior art found | Low risk |

---

## 2. IEEE Xplore NPL Search Results by Query

### 2.1 Query 1: N-body Cryptography

**Search:** `(n-body OR "N body") AND (cryptograph* OR encrypt* OR cipher)`

#### Result: Prior Art FOUND

| Field | Detail |
|-------|--------|
| **Title** | Chaotic Image Encryption Based on an Improved Seven-Dimensional Memristor-based Restricted Four-Body System |
| **Source** | IEEE Xplore / 2025 4th International Joint Conference on Information and Communication Engineering (JCICE) |
| **Date** | July 25–27, 2025 |
| **URL** | [https://ieeexplore.ieee.org/document/11182029](https://ieeexplore.ieee.org/document/11182029) |
| **DOI** | 10.1109/JCICE66205.2025.11182029 |
| **Author** | CHAI Zhijun et al. (Heilongjiang University, Harbin, China) |
| **ISBN** | 979-8-3315-7632-5 |

**Abstract:** *"This study develops an improved memristor-restricted four-star seven-dimensional chaotic system based on the constrained N-body problem. Dynamics simulations demonstrate that the system exhibits satisfactory chaotic characteristics. Building upon this system, we propose a novel image encryption scheme. Numerical simulation results demonstrate that proposed encryption scheme can effectively encrypt the image while maintaining robust defense capabilities against external violent attacks."*

**Key Technical Details:**
- Based on **constrained N-body problem** dynamics (restricted four-body)
- Memristor-restricted four-body system
- 7-dimensional chaotic system
- Designed specifically for **image encryption**
- Conference held July 2025 in Harbin, China

**Relevance to Kelvin:** This is the most directly relevant prior art found. It explicitly uses an N-body derived system for chaotic encryption. Kelvin must distinguish on the following grounds:

| Distinction | Chai et al. (2025) | Kelvin |
|-------------|-------------------|--------|
| **N-body type** | Restricted four-body (constrained) | Full 10-body gravitational simulation |
| **Dimensionality** | 7D chaotic system | 30 DOF (10 bodies × 3D) |
| **Arithmetic** | Floating-point (presumed) | Q32.64 fixed-point |
| **Application** | Image encryption only | General-purpose encryption (7 modes) |
| **Extraction** | Not specified | SHAKE256 XOF with domain separation |
| **Lyapunov monitoring** | Not specified | Shadow orbit method |
| **Integrator** | Not specified | Verlet (symplectic) / Euler |

---

### 2.2 Query 2: Chaotic Stream Ciphers (XOR Focus)

**Search:** `(chaos OR chaotic) AND (stream cipher OR keystream) AND (XOR)`

#### Result: Prior Art FOUND

| Field | Detail |
|-------|--------|
| **Title** | A Novel Orbit Perturbation Method for Continuous-Time Chaos to Obtain Dynamic Seeds-Key Stream Cipher |
| **Source** | IEEE Xplore / 2009 International Conference on Computational Intelligence and Software Engineering (CISE) |
| **Date** | December 11, 2009 |
| **URL** | [https://ieeexplore.ieee.org/document/5366879](https://ieeexplore.ieee.org/document/5366879) |
| **DOI** | 10.1109/CISE.2009.5366879 |
| **Authors** | Weng Yifang, Zheng Rong, Chen Yi |
| **ISBN** | 978-1-4244-4507-3 |

**Abstract:** *"It is possible for continuous-time chaos based stream ciphers to have larger seeds-key space and longer seeds-key than discrete-time chaos based ones. However, they are not applicable because of the many long-length run-courses and very large departure time. The orbit perturbation method to chaotic stranger attractor is proposed to solve the problems. It is proved that the systems keep chaotic after their parameters are overlaid by some chaotic variables. As a result, the run-course closes to the ideal values, and the departure time is decreased greatly. It is verified by Lorenz, Rossler and Chen systems. Using the orbit perturbation method, the chaotic stream ciphers with dynamic seeds-key could be obtained."*

**Key Technical Details:**
- Orbit perturbation method for continuous-time chaotic systems
- Validated on Lorenz, Rössler, and Chen systems
- Solves "long-length run-courses and very large departure time" problems
- Produces dynamic seeds-key stream ciphers

**Relevance to Kelvin:** This is foundational prior art for orbit perturbation in continuous-time chaotic stream ciphers. Kelvin's Quantum (H) mode reseeding mechanism touches this concept. Distinction: Kelvin uses discrete-time n-body simulation (not continuous-time Lorenz/Rossler/Chen), fixed-point arithmetic, and SHAKE256 extraction.

---

### 2.3 Query 3: Lyapunov in Cryptography

**Search:** `(Lyapunov) AND (cryptograph* OR encrypt* OR key)`

#### Result: Lyapunov appears primarily as validation method

| Finding | Source | Detail |
|---------|--------|--------|
| Lyapunov used for chaos validation | Various | Lyapunov exponents are typically employed as objective functions or security validation metrics rather than direct cryptographic primitives |
| Example | DUff-skg paper (Jawad, 2025) | "Lyapunov exponent analysis demonstrates robust chaos for encryption" |

**Relevance to Kelvin:** While no patents claim Lyapunov directly, academic papers consistently use Lyapunov analysis to validate chaotic systems for cryptography. This weakens the argument that "using Lyapunov exponents in cryptography" is novel. However, Kelvin's specific use of Lyapunov exponent estimation as a **security parameter** (determining safe encryption horizon via shadow orbit method) remains potentially novel — no prior art uses Lyapunov time as both a minimum and maximum bound for key derivation.

---

### 2.4 Query 4: Fixed-Point + N-Body + Cryptography

**Search:** `(fixed-point OR "fixed point") AND (n-body OR simulation) AND (cryptograph*)`

#### Result: NO PRIOR ART FOUND

**Conclusion:** This combination remains novel. No academic papers or conference proceedings combine fixed-point arithmetic/numerics with N-body simulation in a cryptographic context across IEEE Xplore, arXiv, or IACR ePrint (2000–2025).

**This is Kelvin's strongest novelty claim.**

---

### 2.5 Query 5: Homomorphic Encryption + Chaos + Key Generation

**Search:** `(homomorphic) AND (chaos OR chaotic) AND (key generation)`

#### Result: Multiple prior art papers found (2021–2025)

See Section 3 for full deep-dive.

---

### 2.6 Query 6: Orbit Perturbation Encryption

**Search:** `(orbit*) AND (perturb*) AND (encrypt* OR cipher)`

#### Result: Multiple prior art papers found (2009–2012)

**Paper 1: Chaotic Permutation and Perturbation Mechanism**

| Field | Detail |
|-------|--------|
| **Title** | A Novel Digital Image Cryptosystem with Chaotic Permutation and Perturbation Mechanism |
| **Source** | IEEE Xplore / 2012 Fifth International Workshop on Chaos-fractals Theories and Applications (IWCFTA) |
| **Date** | October 18, 2012 |
| **URL** | [https://ieeexplore.ieee.org/document/6383210](https://ieeexplore.ieee.org/document/6383210) |
| **DOI** | 10.1109/IWCFTA.2012.51 |
| **Author** | Tao Song |
| **ISBN** | 978-1-4673-2825-8 |

**Abstract:** *"Symmetric stream cipher with chaotic maps have been proven to be feasible and secure for digital image encryption. A new permutation method with perturbation strategy is proposed, which is used to enhance the performance of Cat map. Moreover, a chaotic orbit perturbation mechanism is introduced in diffusion procedure to further enhance the security of the cryptosystem."*

**Key Technical Details:**
- Uses 2D Cat map and Baker map for pixel permutation
- Chaotic orbit perturbation in diffusion procedure
- Correlation and logistic map analysis
- Pages 202–206

**Paper 2: Continuous-Time Chaos Orbit Perturbation** (same as Query 2 result)

| Field | Detail |
|-------|--------|
| **Title** | A Novel Orbit Perturbation Method for Continuous-Time Chaos to Obtain Dynamic Seeds-Key Stream Cipher |
| **Source** | IEEE Xplore / CISE 2009 |
| **Date** | December 2009 |
| **URL** | [https://ieeexplore.ieee.org/document/5366879](https://ieeexplore.ieee.org/document/5366879) |
| **DOI** | 10.1109/CISE.2009.5366879 |
| **Authors** | Weng Yifang, Zheng Rong, Chen Yi |

**Relevance to Kelvin:** Both papers establish prior art for orbit perturbation in chaotic cryptosystems. Kelvin's Quantum (H) mode orbital reseed mechanism should be distinguished from these by emphasizing: (1) n-body gravitational dynamics vs. Cat map / Lorenz systems, (2) fixed-point determinism, (3) SHAKE256 extraction with domain separation, (4) Lyapunov-based stability monitoring during reseeding.

---

### 2.7 Query 7: Chaotic Reseeding

**Search:** `(reseed* OR re-seed*) AND (chaos OR chaotic) AND (cryptograph*)`

#### Result: NO PEER-REVIEWED PAPERS FOUND

| Field | Detail |
|-------|--------|
| **Source** | Cryptography Mailing List (Marc.info) |
| **Date** | October 30, 2020 |
| **URL** | https://marc.info/?l=cryptography&m=160410553023081&w=3 |
| **Author** | Natanael |

**Content Summary:** *"A simple RNG reseeding tweak to prevent malicious but passive sources from introducing bias... using VDF - verifiable delay functions... The malicious source can not have determined what the final output from any other source will have been at the point in time when it is forced to submit its own contribution."*

**Note:** This is a mailing list post, not peer-reviewed. The specific combination of **chaotic reseeding** (reseeding a keystream cache from chaotic orbital dynamics) appears absent from the formal literature.

**Relevance to Kelvin:** This strengthens the novelty of Kelvin's Quantum (H) mode — the hybrid deterministic cache with periodic chaotic reseeding. No peer-reviewed prior art covers this specific mechanism.

---

## 3. HE + Chaos Prior Art Deep-Dive

### 3.1 Paper 1: DUff-skg (Most Direct Prior Art)

| Field | Detail |
|-------|--------|
| **Title** | FHE cryptographic systems with using chaotic secret key generation (DUff-skg) |
| **Source** | Boletim da Sociedade Paranaense de Matemática (UEM, Brazil) |
| **Date** | 2025 |
| **URL** | [https://www.ojs.uem.br/ojs/index.php/BSocParanMat/article/view/77855](https://www.ojs.uem.br/ojs/index.php/BSocParanMat/article/view/77855) |
| **Author** | Nibras Hadi Jawad |
| **Key Space** | 2³²³²⁵ bits (exceeds 2¹²⁸ minimum) |

**Abstract:** *"This research discusses the integration of chaotic systems with fully homomorphic encryption systems to produce strong secret keys it is called DUff-skg. The idea uses a chaotic duffing scheme to provide high-quality randomness for FHE keys, combining key unpredictability with mathematical security. The results confirm the effectiveness of the DUff-skg proposed key in homomorphic encryption systems (BFV and CKKS)."*

**Technical Details:**
- Uses modified **Duffing oscillator** equations (2-DOF)
- **RK4** numerical solution (step size 0.01, 1000 iterations)
- NIST statistical test suite validation
- Implemented in **BFV and CKKS** FHE schemes
- Secret key extraction: `sk = integer((x + y + 0.5) × 1000)`

#### Kelvin vs. DUff-skg Distinction

| Aspect | DUff-skg (Jawad, 2025) | Kelvin (Flare mode) |
|--------|------------------------|---------------------|
| **Chaos source** | Duffing oscillator (2-DOF) | N-body gravitational simulation (30-DOF) |
| **Numerical method** | RK4 floating-point | Verlet/Euler fixed-point (Q32.64) |
| **Key extraction** | Simple integer formula | SHAKE256 XOF with domain separation |
| **FHE schemes** | BFV, CKKS | BFV, CKKS, TFHE (via Prism/Split/Flare) |
| **Key isolation** | Single key per scheme | Domain-separated keys per mode |
| **Determinism** | Floating-point (platform-dependent) | Fixed-point (cross-platform identical) |
| **Lyapunov monitoring** | Validation only | Active security parameter |

---

### 3.2 Paper 2: Chaos-Scattering with Partial Homomorphic Encryption

| Field | Detail |
|-------|--------|
| **Title** | Privacy preserving algorithm using Chaos-scattering of partial homomorphic encryption |
| **Source** | Journal of Physics: Conference Series (IOP Publishing) |
| **Date** | 2021 |
| **Authors** | Mohammed, S. J., and Taha, D. B. |
| **DOI** | 10.1088/1742-6596/1963/1/012154 |

**Abstract:** *"A Chao-scattering-based privacy-preserving technique for partial homomorphic encryption (PHE). In this research, the Lorenz chaotic system is used to generate pseudo-random sequences to improve the encryption. The algorithm claims that key length and randomization determine encryption effectiveness."*

**Technical Details:**
- Lorenz chaotic system for PRNG
- Partial homomorphic encryption (PHE) — not fully homomorphic
- Key length and randomization as security metrics

**Distinction from Kelvin:** Uses Lorenz system (3-DOF continuous) vs. n-body gravitational simulation (30-DOF discrete). Partial HE vs. Kelvin's full FHE/OTP modes. No fixed-point, no domain separation, no Lyapunov monitoring.

---

### 3.3 Paper 3: Chaotic Extreme Learning Machine + FHE

| Field | Detail |
|-------|--------|
| **Title** | Privacy-Preserving Chaotic Extreme Learning Machine with Fully Homomorphic Encryption |
| **Source** | arXiv e-prints |
| **Date** | 2022 |
| **Authors** | Imtiaz Ahamed, S., and Ravi, V. |
| **arXiv ID** | arXiv:2208 |

**Abstract:** *"A Privacy-Preserving Chaotic Extreme Learning Machine (ELM) utilizing Fully Homomorphic Encryption. The recommended method generates weights and biases using a chaotic logistic map, improving model efficacy and data privacy. Chaotic ELM outperforms normal ELM in many datasets, especially healthcare applications."*

**Technical Details:**
- Chaotic logistic map for weight/bias generation
- FHE for model privacy
- Tested on healthcare and finance datasets

**Distinction from Kelvin:** Application-specific (ML weights/biases) vs. general-purpose encryption. Uses simple logistic map (1-D) vs. n-body (30-DOF). Not a general key derivation system.

---

### 3.4 Paper 4: Fractal-Based Hybrid with Homomorphic Encryption

| Field | Detail |
|-------|--------|
| **Title** | Fractal-Based Hybrid Cryptosystem: Enhancing Image Encryption with RSA, Homomorphic Encryption, and Chaotic Maps |
| **Source** | Entropy (MDPI) |
| **Date** | October 25, 2023 |
| **URL** | [https://www.mdpi.com/1099-4300/25/11/1478](https://www.mdpi.com/1099-4300/25/11/1478) |
| **DOI** | 10.3390/e25111478 |

**Abstract:** *"This paper introduces a novel approach to enhance image encryption by combining the strengths of the RSA algorithm, homomorphic encryption, and chaotic maps, specifically the sine and logistic map, alongside the self-similar properties of the fractal Sierpinski triangle. The proposed fractal-based hybrid cryptosystem leverages Paillier encryption for maintaining security and privacy, while the chaotic maps introduce randomness, periodicity, and robustness."*

**Technical Details:**
- Integrates Paillier homomorphic encryption
- Logistic map and sine map for chaos
- Fractal Sierpinski triangle for key space expansion
- RSA as additional security layer

**Distinction from Kelvin:** Image-specific, uses Paillier (additive HE) vs. Kelvin's BFV/CKKS/TFHE. Uses simple 1D chaotic maps vs. n-body simulation. No fixed-point, no domain separation.

---

## 4. Patent Status Verification

### 4.1 Previously Identified Patents — Status Confirmed

| Patent | Previous Status | Current Status | Change? |
|--------|----------------|----------------|---------|
| Apple US 6,587,559 | Expired 2019-02-20 | Expired | ✅ Confirmed |
| Toshiba US 6,014,445 | Expired 2016-10-22 | Expired | ✅ Confirmed |
| CN102360488B | Expired (fee related) | Expired | ✅ Confirmed |
| Nantes US 8,781,124 | Lapsed 2022 | Lapsed | ✅ Confirmed |

### 4.2 New Patent Searches — No New Blocking Patents Found

All patent searches (Google Patents, USPTO, WIPO) returned zero results for queries related to n-body cryptography, gravitational key derivation, orbital key generation, chaotic KDF, simulation-based key generation, and celestial mechanics encryption.

**Conclusion:** No new blocking patents have been filed. The patent landscape remains clear for Kelvin's specific implementation details.

---

## 5. Updated Novelty Assessment

### 5.1 What Remains Novel After This Review

| Invention | Novelty Status | Confidence | Rationale |
|-----------|---------------|------------|-----------|
| Fixed-point n-body as deterministic one-way function | ✅ **Novel** | High | No prior art in any database (Q4) |
| Chaotic reseeding for forward secrecy | ✅ **Novel** | High | No peer-reviewed prior art (Q7) |
| Multi-mode integrated chaotic cryptosystem | ✅ **Novel** | High | No prior art found |
| Domain-separated keystream isolation | ✅ **Novel** | High | No prior art found |
| Hybrid deterministic-chaotic stream cipher (Quantum H) | ✅ **Novel** | High | Chaotic reseeding + fixed-point n-body combination is novel |
| Lyapunov horizon as cryptographic security parameter | ⚠️ **Weakened** | Moderate | Lyapunov widely used as validation metric (Q3), but use as active security parameter remains novel |

### 5.2 What Is Now Established as Prior Art

| Concept | Prior Art Source | Impact |
|---------|-----------------|--------|
| N-body derived chaotic encryption | Chai et al. (2025) — four-body memristor system | 🔴 Direct overlap with broad concept |
| Chaotic key generation for FHE | DUff-skg (Jawad, 2025) — Duffing + BFV/CKKS | 🔴 Direct overlap with Flare mode |
| Orbit perturbation in chaotic ciphers | Weng et al. (2009), Song (2012) | 🟡 Prior art for reseeding concept |
| Chaotic stream cipher (general) | Toshiba '445 + Weng et al. (2009) | 🟡 Prior art for V2 mode |
| Hybrid chaos-crypto framework | CryptoChaos (2025) | 🟡 Prior art for multi-mode concept |
| Chirikov map + Julia set KDF | OrBIT-KDF (2026) | 🟡 Prior art for chaotic map + hash KDF |

### 5.3 Updated Novelty Matrix

| Kelvin Feature | Apple '559 | Chai 2025 | DUff-skg 2025 | Weng 2009 | Song 2012 | OrBIT-KDF | CryptoChaos |
|---------------|:----------:|:---------:|:-------------:|:---------:|:---------:|:---------:|:-----------:|
| N-body gravitational simulation | ⚠️ Broad | 🔴 Direct | ❌ | ❌ | ❌ | ❌ | ❌ |
| Fixed-point arithmetic (Q32.64) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Lyapunov horizon as security param | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| SHAKE256 XOF extraction | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Domain-separated keystream | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Chaotic reseeding (Quantum H) | ❌ | ❌ | ❌ | ⚠️ Partial | ⚠️ Partial | ❌ | ❌ |
| HE key generation (Flare) | ❌ | ❌ | 🔴 Direct | ❌ | ❌ | ❌ | ❌ |
| Multi-mode integrated architecture | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ⚠️ Partial |
| Formal verification (Kani) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

**Legend:** 🔴 Direct overlap | ⚠️ Partial overlap | ❌ No overlap

---

## 6. Updated Risk Assessment

### 6.1 Risk Matrix (Updated with NPL Findings)

| Risk | Level | Mitigation |
|------|-------|------------|
| Apple '559 + Chai (2025) blocking broad n-body claims | 🔴 **High** | Narrow claims to full gravitational n-body (not restricted), fixed-point, general-purpose |
| DUff-skg (2025) blocking Flare HE key generation | 🔴 **High** | Emphasize 30-DOF vs. 2-DOF, fixed-point vs. RK4, domain separation |
| OrBIT-KDF prior art for "chaotic map + hash = KDF" | 🟡 **Moderate** | Kelvin uses n-body, not Chirikov/Julia |
| CryptoChaos prior art for "hybrid chaos-crypto" | 🟡 **Moderate** | Kelvin's architecture is distinct (integrated modes, domain separation) |
| Weng (2009) + Song (2012) for orbit perturbation | 🟡 **Moderate** | Kelvin uses n-body, fixed-point, Lyapunov monitoring |
| Toshiba '445 for chaotic stream cipher | 🟡 **Moderate** | V2 mode exposed; patent expired |
| Lyapunov as validation metric (prior art) | 🟡 **Moderate** | Emphasize active security parameter use, not just validation |
| Fixed-point n-body crypto (Q4) | 🟢 **Low** | Strong patent position — no prior art |
| Chaotic reseeding (Q7) | 🟢 **Low** | Strong patent position — no peer-reviewed prior art |
| Multi-mode integrated architecture | 🟢 **Low** | No prior art found |

### 6.2 Updated Warning Flags

1. **🔴 Chai et al. (2025) is the most dangerous new prior art** — Published July 2025 in IEEE Xplore (JCICE conference, Harbin). This is the first known paper to explicitly use an N-body derived system for chaotic encryption. While it uses a restricted four-body memristor system (not full gravitational simulation), a patent examiner could argue that the broad concept of "n-body chaotic cryptography" was known before Kelvin.

2. **🔴 DUff-skg (Jawad, 2025) directly overlaps with Flare mode** — This paper explicitly uses chaotic dynamics (Duffing oscillator) for FHE key generation in BFV and CKKS schemes. Kelvin's Flare mode claims must be carefully distinguished: 30-DOF n-body vs. 2-DOF Duffing, fixed-point vs. RK4 floating, domain-separated extraction vs. simple integer formula.

3. **🟡 Lyapunov novelty is weakened** — While no prior art uses Lyapunov exponents as an active security parameter, their widespread use as a validation metric means the "novelty" argument is weaker. Claims should emphasize the specific mechanism (shadow orbit method, minimum/maximum bounds) rather than the general concept.

4. **🟢 Fixed-point n-body + crypto remains the strongest claim** — No prior art found in any database. This is Kelvin's most defensible patent position.

5. **🟢 Chaotic reseeding is a newly strengthened claim** — No peer-reviewed prior art. The Quantum (H) mode's hybrid cache + periodic orbital reseed is novel.

---

## 7. Updated Patent Strategy Recommendations

### 7.1 Priority Order (Updated with NPL Findings)

| Priority | Invention | Rationale | Status Change |
|----------|-----------|-----------|---------------|
| **1** | Fixed-point n-body as deterministic one-way function | Strongest novelty — no prior art in any database (Q4) | ✅ Unchanged |
| **2** | Hybrid deterministic-chaotic stream cipher with chaotic reseeding (Quantum H) | Novel — no peer-reviewed prior art for chaotic reseeding (Q7) | ⬆️ **Upgraded** |
| **3** | Lyapunov horizon as cryptographic security parameter | Weakened but still potentially novel as active parameter | ⬇️ **Downgraded** |
| **4** | Multi-mode integrated architecture with domain separation | Novel integration pattern | ✅ Unchanged |
| **5** | Chaotic key generation for homomorphic encryption | **Prior art found** — DUff-skg covers this. Only patent if distinguishing features are strong enough. | ⬇️ **Downgraded** |

### 7.2 What to Emphasize in Claims (Updated)

1. **Full gravitational n-body simulation** — Not "restricted" or "constrained" n-body (distinction from Chai et al.). Emphasize: 10-body, Newtonian gravity, Verlet/Euler integration, 30 DOF.

2. **Fixed-point arithmetic (Q32.64)** — Cross-platform determinism. No prior art in any database.

3. **Chaotic reseeding mechanism** — Periodic injection of fresh orbital entropy into a deterministic keystream cache. No peer-reviewed prior art.

4. **Lyapunov exponent as active security parameter** — Shadow orbit method determining both minimum and maximum bounds for key derivation. Distinguish from passive validation use.

5. **Domain-separated multi-mode architecture** — Integration of 7 modes with shared seed derivation but isolated keystreams.

### 7.3 What to Avoid in Claims (Updated)

1. **"N-body chaotic cryptography" broadly** — Blocked by Apple '559 + Chai et al. (2025)
2. **"Chaotic stream cipher" broadly** — Blocked by Toshiba '445 + Weng et al. (2009)
3. **"Chaotic key generation for homomorphic encryption" broadly** — Blocked by DUff-skg (2025)
4. **"Orbit perturbation" broadly** — Blocked by Weng et al. (2009) + Song (2012)
5. **"Hybrid chaos-cryptography framework"** — Covered by CryptoChaos (2025)
6. **"Key derivation using chaotic system"** — Covered by OrBIT-KDF (2026)

### 7.4 Recommended Next Steps (Updated)

1. ✅ **Initial patent review** — Complete (Patent Review #1)
2. ✅ **Web-based prior art search** — Complete (Patent Review #2)
3. ✅ **IEEE Xplore NPL deep-dive** — Complete (this document)
4. 🔲 **Professional prior art search** — Pay a patent attorney for a formal prior art search ($2,000–$5,000). Include IEEE Xplore, ACM Digital Library, and Scopus.
5. 🔲 **Freedom-to-operate opinion** — Legal opinion on commercialization, specifically addressing Chai et al. (2025) and DUff-skg (2025)
6. 🔲 **Provisional patent application** — Draft and file within 6 months. Priority: fixed-point n-body + chaotic reseeding.
7. 🔲 **Defensive publication** — Consider publishing non-patentable innovations on arXiv or GitHub to establish prior art against competitors

---

## 8. References

### Newly Identified IEEE Xplore References

1. **Chai, Z. et al.** (2025). "Chaotic Image Encryption Based on an Improved Seven-Dimensional Memristor-based Restricted Four-Body System." *2025 4th International Joint Conference on Information and Communication Engineering (JCICE)*, Harbin, China. DOI: 10.1109/JCICE66205.2025.11182029.

2. **Weng, Y., Zheng, R., & Chen, Y.** (2009). "A Novel Orbit Perturbation Method for Continuous-Time Chaos to Obtain Dynamic Seeds-Key Stream Cipher." *2009 International Conference on Computational Intelligence and Software Engineering (CISE)*. DOI: 10.1109/CISE.2009.5366879.

3. **Song, T.** (2012). "A Novel Digital Image Cryptosystem with Chaotic Permutation and Perturbation Mechanism." *2012 Fifth International Workshop on Chaos-fractals Theories and Applications (IWCFTA)*, pp. 202–206. DOI: 10.1109/IWCFTA.2012.51.

### Newly Identified HE + Chaos References

4. **Jawad, N. H.** (2025). "FHE cryptographic systems with using chaotic secret key generation (DUff-skg)." *Boletim da Sociedade Paranaense de Matemática*, UEM, Brazil.

5. **Mohammed, S. J., & Taha, D. B.** (2021). "Privacy preserving algorithm using Chaos-scattering of partial homomorphic encryption." *Journal of Physics: Conference Series*, IOP Publishing. DOI: 10.1088/1742-6596/1963/1/012154.

6. **Imtiaz Ahamed, S., & Ravi, V.** (2022). "Privacy-Preserving Chaotic Extreme Learning Machine with Fully Homomorphic Encryption." *arXiv e-prints*, arXiv:2208.

7. **Fractal-Based Hybrid Cryptosystem** (2023). "Enhancing Image Encryption with RSA, Homomorphic Encryption, and Chaotic Maps." *Entropy*, MDPI, 25(11), 1478. DOI: 10.3390/e25111478.

### Previously Identified References (from Patent Reviews #1 and #2)

8. **US 6,587,559** — Apple Inc. "Cryptographic System Using Chaotic Dynamics" (expired 2019)
9. **US 6,014,445** — Toshiba. "Stream Cipher Using Chaos" (expired 2016)
10. **CN102360488B** — "Image Encryption Based on Chaotic Orbit Perturbation" (lapsed)
11. **US 8,781,124** — Université de Nantes. "Chaotic Sequence Generator" (lapsed 2022)
12. **OrBIT-KDF** (2026). GitHub: cozmobaut-hub/OrBIT-KDF.
13. **Song et al.** (2025). "CryptoChaos." arXiv:2504.08618.
14. **Halayka** (2012). "N-body dynamics for PRNG." viXra.
15. **Vuckovac** (2021). "Cryptographic Puzzles and Complex Systems." *Complex Systems*.
16. **Kraicha et al.** (2025). "Orbital-Inspired Encryption Using Phobos and Deimos." *JISA*.
17. **Cang, Kang & Wang** (2021). "Conservative Sprott-A PRNG." *Nonlinear Dynamics*.

---

*This document is part of the Kelvin Cryptosystem documentation suite.  
It represents an updated patent landscape analysis incorporating IEEE Xplore NPL search results (2000–2025).  
This should not be construed as legal advice. Consult a qualified patent attorney before filing any patent application.*

*Last Updated: 2026-05-28*
