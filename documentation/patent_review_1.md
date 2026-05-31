# Patent Landscape Review #1 — Prior Art Analysis & Novelty Assessment

**Status:** First Review  
**Date:** 2026-05-27  
**Author:** Kelvin Project  
**Cross-references:** [Mode Flowcharts](mode_flowcharts.md), [Homomorphic Cryptosystem Analysis](homomorphic_cryptosystem.md), [Quantum Resistance Analysis](quantum_analysis.md), [Project History](project_history.md), [Proof of Concept](proof_of_concept.md), [Keyspace Analysis](keyspace_analysis.md)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Identified Blocking Patents](#2-identified-blocking-patents)
3. [Patent Claim Mapping to Kelvin's Architecture](#3-patent-claim-mapping-to-kelvins-architecture)
4. [Novelty Assessment Per Mode](#4-novelty-assessment-per-mode)
5. [Patentable Inventions Identified](#5-patentable-inventions-identified)
6. [Risk Assessment & Warning Flags](#6-risk-assessment--warning-flags)
7. [Recommended Patent Strategy](#7-recommended-patent-strategy)
8. [Prior Art Search Recommendations](#8-prior-art-search-recommendations)
9. [References](#9-references)

---

## 1. Executive Summary

### 1.1 Core Finding

**You can likely file a patent (brevet), but the broad concept of using "chaos" and "n-body" dynamics for cryptography is already in the prior art.** Your unique contribution lies in the specific implementation details — the architecture, the integration patterns, and the novel combinations that no single prior patent covers.

### 1.2 Patent Expiration Summary

| Patent | Filing Date | Expiration | Status | Enforceable? |
|--------|-------------|------------|--------|:------------:|
| **Apple US 6,587,559** (N-body crypto) | 2001-01-04 | **2019-02-20** | Expired — Fee Related | ❌ No |
| **Toshiba US 6,014,445** (Chaos stream cipher) | 1996-10-22 | **2016-10-22** | Expired — Lifetime | ❌ No |
| **CN102360488B** (Orbit perturbation) | 2011-09-29 | **2031-09-29** | Expired — Fee Related (lapsed) | ❌ No |
| **Nantes US 8,781,124** (Chaos seq. generator) | 2011-03-28 | **2031-07-22** | Expired — Fee Related (lapsed 2022) | ❌ No |

> **All four identified blocking patents are expired or lapsed.** The two most critical (Apple '559 and Toshiba '445) expired in 2019 and 2016 respectively. The Chinese and Nantes patents lapsed for non-payment of maintenance fees. However, they remain prior art for novelty purposes — narrow claim drafting is still essential.

### 1.3 The Landscape at a Glance

| Category | Status | Key Risk |
|----------|--------|----------|
| **N-body chaotic KDF** (broad concept) | ❌ **Blocked** — Apple US 6,587,559 | Highest risk |
| **Chaotic stream cipher** (general method) | ❌ **Blocked** — Toshiba US 6,014,445 | High risk |
| **Chaotic orbit perturbation** (reseeding) | ⚠️ **Constrained** — CN102360488B | Moderate risk |
| **Chaotic sequence generator** | ⚠️ **Constrained** — US 8,781,124 (Nantes) | Moderate risk |
| **Hybrid multi-mode architecture** | ✅ **Potentially novel** | Low risk |
| **Kelvin-Quantum (H) hybrid model** | ✅ **Potentially novel** | Low risk |
| **HE integration (Prism/Split/Flare)** | ✅ **Strong novelty** | Very low risk |
| **Fixed-point n-body as one-way function** | ✅ **Potentially novel** | Low risk |
| **Lyapunov horizon as security parameter** | ✅ **Strong novelty** | Very low risk |

### 1.4 Key Takeaway

> **Do not patent the "n-body crypto" idea. Patent your specific "machine" — the Kelvin-Kit architecture, the hybrid Quantum (H) model, and the HE integration modes.**

---

## 2. Identified Blocking Patents

### 2.1 Apple US 6,587,559 — "Cryptographic System Using Chaotic Dynamics"

| Field | Detail |
|-------|--------|
| **Patent No.** | US 6,587,559 |
| **Assignee** | Apple Inc. |
| **Priority Date** | 1998-05-27 |
| **Filing Date** | 2001-01-04 |
| **Grant Date** | 2003-05-06 |
| **Expiration Date** | **2019-02-20** (adjusted expiration) |
| **Status** | **Expired — Fee Related** (not renewed) |
| **Source** | [Google Patents: US6558759](https://patents.google.com/patent/US6558759/en) — verified 2026-05-27 |

#### Core Claim

The patent explicitly covers a cryptographic system where the chaotic dynamics are based on the **"N-body" problem** to provide cryptographic security. The system uses chaotic dynamics to generate cryptographic keys or for encryption.

#### Relevance to Kelvin

This is the **most critical blocking patent** for the Kelvin project. The broad claim covers:

- Using an N-body simulation to generate cryptographic keys
- Using chaotic orbital dynamics for encryption
- Any system where the chaotic state of an N-body system is used as a cryptographic primitive

#### Mitigating Factors

| Factor | Kelvin's Position |
|--------|-------------------|
| **Patent expiry** | **Expired 2019-02-20** — confirmed via Google Patents. The patent is no longer enforceable. This completely eliminates the infringement risk for the broad concept. |
| **Floating-point vs. fixed-point** | Apple's patent predates Kelvin's Q32.64 fixed-point solution. The patent describes floating-point implementations, which were the only option at the time. Kelvin's fixed-point approach solves a problem (cross-platform determinism) that Apple's patent does not address. |
| **Lyapunov enforcement** | Apple's patent does not mention Lyapunov exponent estimation or chaotic regime enforcement. Kelvin's mandatory Lyapunov horizon check is a novel security mechanism. |
| **Deep Physical Binding** | Apple's patent does not describe hashing force vectors, gravitational constants, or domain-separated extraction. |
| **Multi-mode architecture** | Apple's patent describes a single-mode system. Kelvin's 7-mode integrated framework is architecturally distinct. |

#### ⚠️ Warning

Even if the patent has expired, the existence of this prior art means that **any new patent claiming "n-body chaotic cryptography" as a broad concept will be rejected**. Claims must be narrowly tailored to specific implementation details.

---

### 2.2 Toshiba US 6,014,445 — "Stream Cipher Using Chaos"

| Field | Detail |
|-------|--------|
| **Patent No.** | US 6,014,445 |
| **Assignee** | Kabushiki Kaisha Toshiba |
| **Priority Date** | 1995-10-23 (JP27429295A) |
| **Filing Date** | 1996-10-22 |
| **Grant Date** | 2000-01-11 |
| **Expiration Date** | **2016-10-22** (anticipated expiration) |
| **Status** | **Expired — Lifetime** (fully expired) |
| **Source** | [Google Patents: US6014445](https://patents.google.com/patent/US6014445/en) — verified 2026-05-27 |

#### Core Claim

The patent covers a fundamental method for creating a stream cipher using chaos:

1. Generate a chaotic sequence
2. Convert it to a binary keystream (e.g., using thresholds)
3. Perform a logic operation (like XOR) with plaintext to create ciphertext

#### Relevance to Kelvin

This patent is very close to the architecture of Kelvin's **Chaos (V2)** and **Photon (V3)** modes:

- **Chaos (V2):** Per-step SHAKE256 XOR — generates chaotic sequence from orbital simulation, converts to keystream via SHAKE256 XOF, XORs with data
- **Photon (V3):** HKDF→SHAKE256 XOR — generates keystream from seed, XORs with data

#### Mitigating Factors

| Factor | Kelvin's Position |
|--------|-------------------|
| **Patent expiry** | **Expired 2016-10-22** — confirmed via Google Patents. Fully expired ("Expired - Lifetime"). No infringement risk. |
| **Specific mechanism** | Toshiba's patent describes threshold-based binary conversion. Kelvin uses SHAKE256 XOF — a fundamentally different, NIST-standardized extraction method. |
| **Domain separation** | Kelvin's domain-separated hashing (`b"kelvin-streaming-v2-v1-000000000"`, etc.) is not described in the Toshiba patent. |
| **Forward secrecy** | BLAKE3 reseeding for forward secrecy is not described. |

---

### 2.3 CN102360488B — "Image Encryption Based on Chaotic Orbit Perturbation"

| Field | Detail |
|-------|--------|
| **Patent No.** | CN102360488B |
| **Assignee** | Chinese filing |
| **Priority Date** | 2011-09-29 |
| **Filing Date** | 2011-09-29 |
| **Grant Date** | 2013-02-13 |
| **Expiration Date** | **2031-09-29** (anticipated expiration) |
| **Status** | **Expired — Fee Related** (lapsed for non-payment of maintenance fees) |
| **Source** | [Google Patents: CN102360488B](https://patents.google.com/patent/CN102360488B/en) — verified 2026-05-27 |

#### Core Claim

An image encryption method based on **"chaotic orbit perturbation"** — a technique related to Kelvin's reseeding process where the chaotic system's trajectory is perturbed to refresh entropy.

#### Relevance to Kelvin

This patent covers a technique related to Kelvin's **Quantum (H) mode** reseeding mechanism, where the orbital simulation is advanced by N steps to inject fresh chaotic entropy into the base seed.

#### Mitigating Factors

| Factor | Kelvin's Position |
|--------|-------------------|
| **Jurisdiction** | Chinese patent — does not apply in US/Europe |
| **Specific application** | Image encryption specifically — Kelvin's application is general-purpose |
| **Mechanism** | Kelvin's reseeding uses SHAKE256 XOF extraction + BLAKE3 mixing, not the specific perturbation method described |

---

### 2.4 US 8,781,124 (US20130170641A1) — "Chaotic Sequence Generator for Encryption Keys"

| Field | Detail |
|-------|--------|
| **Patent No.** | US 8,781,124 (published as US20130170641A1) |
| **Assignee** | Université de Nantes |
| **Priority Date** | 2010-03-29 (FR1052295) |
| **Filing Date** | 2011-03-28 |
| **Grant Date** | 2014-07-15 |
| **Expiration Date** | **2031-07-22** (adjusted expiration) |
| **Status** | **Expired — Fee Related** (lapsed 2022 for failure to pay maintenance fees) |
| **Source** | [Google Patents: US20130170641A1](https://patents.google.com/patent/US20130170641A1/en) — verified 2026-05-27 |

#### Core Claim

A generator for creating chaotic sequences to be used as encryption keys.

#### Relevance to Kelvin

This covers the general concept of using a chaotic system to generate key material — a concept that overlaps with all of Kelvin's modes.

#### Mitigating Factors

| Factor | Kelvin's Position |
|--------|-------------------|
| **Patent expiry** | **Lapsed 2022** for failure to pay maintenance fees. Technically expired before its 2031 term. However, the prior art still exists for novelty purposes. |
| **Specific generator** | Describes a specific electronic circuit implementation — Kelvin is software-based |
| **N-body specificity** | Does not describe n-body gravitational dynamics specifically |

---

## 3. Patent Claim Mapping to Kelvin's Architecture

This table maps each identified patent's claims to specific Kelvin modes, showing which modes are directly affected and which are potentially novel.

| Kelvin Mode | Apple '559 (N-body) | Toshiba '445 (Chaos stream) | CN102360488B (Orbit perturb.) | Nantes '0641A1 (Seq. gen.) | **Overall Risk** |
|-------------|:-------------------:|:---------------------------:|:-----------------------------:|:--------------------------:|:----------------:|
| **Secure (V1)** — ChaCha20 + BLAKE3 | ⚠️ Seed derivation uses n-body | ❌ Not a chaotic stream cipher | ❌ No orbit perturbation | ⚠️ Key generation from chaos | **Low** (standard cipher) |
| **Chaos (V2)** — Per-step SHAKE256 XOR | 🔴 Direct overlap | 🔴 Direct overlap | ❌ No perturbation | 🔴 Direct overlap | **HIGH** |
| **Photon (V3)** — HKDF→SHAKE256 XOR | ⚠️ Seed uses n-body | ⚠️ Chaotic sequence → XOR | ❌ No perturbation | ⚠️ Key generation | **Moderate** |
| **Quantum (H)** — Hybrid cache + orbital reseed | ⚠️ Seed uses n-body | ⚠️ Chaotic sequence → XOR | ⚠️ Orbit perturbation reseed | ⚠️ Key generation | **Moderate** (but novel combination) |
| **Prism** — HE stream key generator | ⚠️ Seed uses n-body | ❌ Not a stream cipher | ❌ No perturbation | ❌ Key generation for HE | **Low** (novel application) |
| **Split** — XOR key-splitter | ⚠️ Seed uses n-body | ❌ Not a stream cipher | ❌ No perturbation | ❌ Key generation for HE | **Low** (novel application) |
| **Flare** — FHE secret key generator | ⚠️ Seed uses n-body | ❌ Not a stream cipher | ❌ No perturbation | ❌ Key generation for HE | **Low** (novel application) |

**Legend:**
- 🔴 **Direct overlap** — The mode directly implements the patented concept
- ⚠️ **Partial overlap** — The mode uses the concept as one component among others
- ❌ **No overlap** — The mode does not implement the patented concept

### Key Insight

The **Chaos (V2)** mode is the most exposed — it directly implements all four patented concepts. However, V2 is also the mode with the strongest security properties (true per-step chaotic keystream). The patent strategy should **not** focus on V2 as a standalone invention, but rather on the **integrated system** that includes V2 as one component among many.

---

## 4. Novelty Assessment Per Mode

### 4.1 Secure (V1) — ChaCha20 + BLAKE3 Authentication

| Aspect | Assessment |
|--------|------------|
| **ChaCha20 cipher** | Standard, well-known. No patent issues. |
| **BLAKE3 authentication** | Standard, well-known. No patent issues. |
| **Seed derivation from n-body** | ⚠️ Touches Apple '559. But seed derivation is a one-time setup step, not the core encryption mechanism. |
| **Verdict** | **Low risk.** The mode uses standard cryptographic primitives. The n-body seed derivation is a setup step, not the encryption method itself. |

### 4.2 Chaos (V2) — Per-Step SHAKE256 XOR

| Aspect | Assessment |
|--------|------------|
| **N-body chaotic keystream** | 🔴 Directly touches Apple '559 |
| **Chaotic sequence → XOR cipher** | 🔴 Directly touches Toshiba '445 |
| **SHAKE256 extraction** | Standard XOF — no patent issues |
| **Domain separation** | ✅ Novel — not described in prior art |
| **Stability monitoring** | ✅ Novel — Lyapunov-based collapse/ejection detection |
| **Verdict** | **HIGH risk as standalone invention.** Do not patent V2 alone. Patent it only as part of the integrated multi-mode system. |

### 4.3 Photon (V3) — HKDF→SHAKE256 XOR

| Aspect | Assessment |
|--------|------------|
| **HKDF key derivation** | Standard, well-known. No patent issues. |
| **SHAKE256 XOF keystream** | Standard, NIST-standardized. No patent issues. |
| **BLAKE3 reseeding** | ✅ Novel combination — forward secrecy via BLAKE3 |
| **Domain separation** | ✅ Novel — not described in prior art |
| **Verdict** | **Low risk.** The mode uses standard KDF primitives. The BLAKE3 reseeding mechanism is a potentially novel contribution. |

### 4.4 Quantum (H) — Hybrid Cache + Orbital Reseed

| Aspect | Assessment |
|--------|------------|
| **BLAKE3 → SHAKE256 cache** | Standard XOF usage — no patent issues |
| **Orbital reseed mechanism** | ⚠️ Touches CN102360488B (orbit perturbation) |
| **Hybrid architecture** | ✅ **Strong novelty** — combining fast deterministic PRNG with periodic chaotic reseeding |
| **Configurable reseed interval** | ✅ Novel — adaptive security/performance tradeoff |
| **Domain separation** | ✅ Novel — not described in prior art |
| **Verdict** | **Best patent candidate among encryption modes.** The hybrid architecture (fast cache + periodic orbital reseed) is a unique solution to the degradation problem in chaotic systems. |

### 4.5 Prism / Split / Flare — HE Integration Modes

| Aspect | Assessment |
|--------|------------|
| **Stream key generation for FHE recryption** | ✅ **Strong novelty** — no prior art found |
| **Domain-separated key isolation** | ✅ Novel — prevents cross-mode key reuse |
| **Chaotic FHE key generation (Flare)** | ✅ Novel extension of DUff-skg (Jawad, 2025) — 30 DOF vs 2 DOF |
| **XOR key-splitter (Split)** | ✅ Novel application of split-key concept |
| **Verdict** | **Strongest patent candidates.** The HE integration modes represent a novel application of chaotic KDF to homomorphic encryption — an area with minimal prior art. |

---

## 5. Patentable Inventions Identified

Based on the analysis above, the following inventions have the strongest novelty and are recommended for patent filing:

### 5.1 Invention #1: Hybrid Multi-Mode Chaotic Cryptosystem

**Title:** *Integrated Multi-Mode Cryptographic System Using Chaotic Orbital Dynamics*

**Novelty:** A single cryptographic framework that seamlessly integrates multiple encryption modes (Secure, Chaos, Photon, Quantum, Prism, Split, Flare) with domain-separated keystream isolation, allowing the same orbital configuration to serve multiple cryptographic purposes without key reuse.

**Key Claims:**
- A system with ≥3 encryption modes sharing a common chaotic seed derivation
- Domain-separated keystream generation preventing cross-mode key recovery
- Method for switching between modes within a single encryption session

**Prior Art Gap:** No existing patent describes an integrated multi-mode chaotic cryptosystem with domain-separated keystream isolation.

---

### 5.2 Invention #2: Hybrid Deterministic-Chaotic Stream Cipher (Kelvin-Quantum)

**Title:** *Hybrid Stream Cipher with Deterministic Cache and Periodic Chaotic Reseeding*

**Novelty:** A stream cipher that combines a fast deterministic keystream cache (BLAKE3 → SHAKE256 XOF) with periodic reseeding from a chaotic n-body simulation. This solves the dynamical degradation problem identified by Cang et al. (2021) — the cache provides performance, the orbital reseed provides fresh entropy and breaks periodicity.

**Key Claims:**
- A keystream cache refilled from a deterministic XOF
- Periodic reseeding from a chaotic n-body simulation
- Configurable reseed interval balancing performance and entropy freshness
- Forward secrecy via BLAKE3 reseeding after each orbital injection

**Prior Art Gap:** No existing patent combines a fast XOF cache with periodic n-body chaotic reseeding. The CN102360488B patent covers orbit perturbation but not in the context of a hybrid cache-based stream cipher.

---

### 5.3 Invention #3: Chaotic Key Generation for Homomorphic Encryption

**Title:** *System and Method for Generating Homomorphic Encryption Keys Using Chaotic N-Body Simulation*

**Novelty:** Using a high-dimensional (30 DOF) chaotic n-body simulation to generate keys for homomorphic encryption systems, including stream keys for recryption (Prism), split keys for XOR homomorphism (Split), and FHE secret keys (Flare).

**Key Claims:**
- Generating FHE recryption keys from chaotic orbital state
- Domain-separated key generation for different FHE schemes (BFV, CKKS, TFHE)
- Split-key XOR homomorphism using chaotic stream keys
- Higher-dimensional chaos (30 DOF) vs. prior art (2 DOF Duffing)

**Prior Art Gap:** The DUff-skg paper (Jawad, 2025) uses a 2-DOF Duffing oscillator for FHE key generation. Kelvin's 30-DOF n-body approach is a significant extension. No patent covers n-body chaotic key generation for FHE.

---

### 5.4 Invention #4: Lyapunov Horizon as Cryptographic Security Parameter

**Title:** *Cryptographic Security Parameter Determination Using Lyapunov Exponent Estimation*

**Novelty:** Using Lyapunov exponent estimation (shadow orbit method) to determine the safe encryption horizon of a chaotic cryptosystem. The Lyapunov time serves as both a lower bound (minimum steps to enter chaos) and an upper bound (maximum steps before predictability degrades).

**Key Claims:**
- Estimating Lyapunov time via shadow orbit method
- Rejecting configurations with insufficient chaotic divergence
- Using Lyapunov horizon as both minimum and maximum bound for key derivation
- Runtime stability monitoring (collapse/ejection detection)

**Prior Art Gap:** No existing patent uses Lyapunov exponent estimation as a cryptographic security parameter. The concept is known in chaos theory but has not been applied to cryptographic key derivation.

---

### 5.5 Invention #5: Fixed-Point N-Body Simulation as Deterministic One-Way Function

**Title:** *Deterministic Cryptographic One-Way Function Using Fixed-Point N-Body Simulation*

**Novelty:** Using Q32.64 fixed-point arithmetic for a cross-platform deterministic n-body simulation that serves as a cryptographic one-way function. Unlike floating-point implementations (Apple '559), fixed-point guarantees bit-identical results across all architectures.

**Key Claims:**
- Fixed-point arithmetic (Q32.64) for deterministic n-body simulation
- Cross-platform bit-identical keystream generation
- Deep Physical Binding (force vectors, constants, domain separation in hash chain)
- Sequential simulation as anti-parallelization measure

**Prior Art Gap:** Apple '559 describes floating-point implementations. The fixed-point determinism solution is a distinct technical contribution that solves a problem (cross-platform reproducibility) not addressed by the prior art.

---

## 6. Risk Assessment & Warning Flags

### 6.1 ⚠️ "Harvest Now, Decrypt Later" (HNDL) — NOT Patentable

The concept of "Harvest Now, Decrypt Later" is a **well-known threat model** in post-quantum cryptography. It describes a problem, not a solution. You cannot patent:

- The observation that encrypted data can be stored now and decrypted later with quantum computers
- The general idea of using post-quantum cryptography to defend against HNDL
- The term "Harvest Now, Decrypt Later" itself (it is prior art terminology)

**Recommendation:** Do not include HNDL as a patent claim. It can be mentioned as motivation in the patent specification but must not appear in the claims.

### 6.2 ⚠️ Apple '559 — The Blocking Patent

Even if expired, Apple's US 6,587,559 is the most significant prior art. Any patent application must:

1. **Explicitly distinguish** from Apple '559 in the specification
2. **Narrow claims** to avoid the broad "n-body chaotic cryptography" concept
3. **Emphasize** the specific technical contributions (fixed-point, Lyapunov, multi-mode, HE integration) that Apple '559 does not cover

### 6.3 ⚠️ Toshiba '445 — Chaotic Stream Cipher Prior Art

The Toshiba patent covers the general method of "chaotic sequence → binary keystream → XOR with plaintext." Kelvin's Chaos (V2) mode directly implements this pattern. To distinguish:

- Emphasize the **specific extraction method** (SHAKE256 XOF vs. threshold-based binary conversion)
- Emphasize **domain separation** and **forward secrecy** (not described in Toshiba)
- Do **not** claim the broad "chaotic stream cipher" concept

### 6.4 ⚠️ Prior Art in Academic Literature

Several academic papers describe chaos-based cryptography and may constitute prior art:

| Paper | Year | Relevance |
|-------|------|-----------|
| Song et al. — CryptoChaos | 2025 | Hybrid chaos framework (4 maps + AES-GCM) |
| Cang, Kang & Wang — Sprott-A PRNG | 2021 | Conservative chaotic PRNG + FPPC |
| Halayka — N-body PRNG | 2012 | N-body dynamics for PRNG |
| Vuckovac — N-body PoW puzzles | 2021 | N-body for proof-of-work |
| Kraicha et al. — Orbital encryption | 2025 | Phobos/Deimos positions for encryption |

These papers do **not** describe Kelvin's specific architecture but establish that chaos-based cryptography is an active research area. Patent examiners may cite these as evidence that the general concept is well-known.

### 6.5 ✅ Low-Risk Areas

The following areas have **minimal prior art risk**:

- **Fixed-point n-body simulation** for cryptographic determinism
- **Lyapunov horizon** as a cryptographic security parameter
- **Multi-mode integrated architecture** with domain separation
- **HE integration** (Prism/Split/Flare) using chaotic KDF
- **BLAKE3 reseeding** for forward secrecy in chaotic systems

---

## 7. Recommended Patent Strategy

### 7.1 What to Patent (Priority Order)

| Priority | Invention | Jurisdiction | Estimated Strength |
|----------|-----------|-------------|-------------------|
| **1** | Hybrid deterministic-chaotic stream cipher (Quantum H) | US, EP, JP | Strong |
| **2** | Chaotic key generation for HE (Prism/Split/Flare) | US, EP | Strongest |
| **3** | Lyapunov horizon as security parameter | US | Strong |
| **4** | Fixed-point n-body as one-way function | US, EP | Moderate |
| **5** | Integrated multi-mode architecture | US | Moderate |

### 7.2 What NOT to Patent

| Concept | Reason |
|---------|--------|
| "N-body chaotic cryptography" (broad) | Blocked by Apple '559 |
| "Chaotic stream cipher" (broad) | Blocked by Toshiba '445 |
| "Harvest Now, Decrypt Later" defense | Prior art terminology |
| "Post-quantum cryptography" (broad) | Well-known concept |
| "Chaotic orbit perturbation" | Covered by CN102360488B |

### 7.3 Filing Strategy

1. **Provisional patent application (US)** — File first to establish priority date. Include all 5 inventions in a single provisional. Cost: ~$1,000–$3,000 (filing fees + attorney).

2. **PCT international application** — Within 12 months of provisional. Covers US, EP, JP, CN. Cost: ~$5,000–$15,000.

3. **National phase entries** — Within 30 months of priority date. Selectively enter US, EP, JP based on market interest.

### 7.4 Claim Drafting Strategy

| Claim Type | Example |
|------------|---------|
| **System claim** | "A cryptographic system comprising: a fixed-point n-body simulator; a Lyapunov estimator; a SHAKE256 XOF extractor; a keystream cache; and a reseeding module configured to periodically inject chaotic entropy..." |
| **Method claim** | "A method for generating cryptographic keystream comprising: simulating an n-body system using fixed-point arithmetic; estimating a Lyapunov exponent; extracting entropy via SHAKE256; caching keystream; and periodically reseeding from orbital state..." |
| **Apparatus claim** | "A cryptographic apparatus comprising a processor configured to execute fixed-point n-body simulation and a memory storing domain-separated keystream buffers..." |
| **Computer-readable medium claim** | "A non-transitory computer-readable medium storing instructions that, when executed, cause a processor to..." |

### 7.5 Recommended Next Steps

1. ✅ **This review** — Complete (this document)
2. 🔲 **Professional prior art search** — Pay a patent attorney for a formal prior art search ($2,000–$5,000)
3. 🔲 **Freedom-to-operate opinion** — Legal opinion on whether Kelvin can be commercialized without infringing existing patents
4. 🔲 **Provisional patent application** — Draft and file within 6 months
5. 🔲 **Open-source strategy** — Consider defensive publication for non-patentable innovations (publish in arXiv or GitHub to establish prior art against competitors)

---

## 8. Prior Art Search Recommendations

### 8.1 Recommended Search Queries

For a professional prior art search, use the following queries:

| Query | Target |
|-------|--------|
| `(n-body OR "N body") AND (cryptograph* OR encrypt* OR cipher)` | N-body cryptography |
| `(chaos OR chaotic) AND (stream cipher OR keystream) AND (XOR)` | Chaotic stream ciphers |
| `(Lyapunov) AND (cryptograph* OR encrypt* OR key)` | Lyapunov in cryptography |
| `(fixed-point OR "fixed point") AND (n-body OR simulation) AND (cryptograph*)` | Fixed-point crypto simulation |
| `(homomorphic) AND (chaos OR chaotic) AND (key generation)` | HE + chaos |
| `(orbit*) AND (perturb*) AND (encrypt* OR cipher)` | Orbit perturbation encryption |
| `(reseed* OR re-seed*) AND (chaos OR chaotic) AND (cryptograph*)` | Chaotic reseeding |

### 8.2 Databases to Search

| Database | Coverage |
|----------|----------|
| **USPTO** (patft.uspto.gov) | US patents and applications |
| **EPO** (worldwide.espacenet.com) | European and global patents |
| **WIPO PATENTSCOPE** (patentscope.wipo.int) | PCT applications |
| **Google Patents** (patents.google.com) | Cross-database search |
| **JPO** (j-platpat.inpit.go.jp) | Japanese patents |
| **CNIPA** (english.cnipa.gov.cn) | Chinese patents |

### 8.3 Non-Patent Literature (NPL) to Search

| Source | Coverage |
|--------|----------|
| **arXiv** (arxiv.org) | Cryptography, chaos theory preprints |
| **IACR ePrint** (eprint.iacr.org) | Cryptography research |
| **IEEE Xplore** | Conference and journal papers |
| **ACM Digital Library** | Conference and journal papers |
| **Google Scholar** | Cross-disciplinary search |

---

## 9. References

### Patents Cited

1. **US 6,587,559** — Apple Inc. "Cryptographic System Using Chaotic Dynamics" (2003)
2. **US 6,014,445** — Kabushiki Kaisha Toshiba. "Stream Cipher Using Chaos" (2000)
3. **CN102360488B** — "Image Encryption Based on Chaotic Orbit Perturbation" (2011)
4. **US 8,781,124** (published as US20130170641A1) — Université de Nantes. "Chaotic Sequence Generator for Encryption Keys" (granted 2014, lapsed 2022)

### Academic References

5. Song et al. (2025). "A Hybrid Chaos-Based Cryptographic Framework for Post-Quantum Secure Communications." *arXiv:2504.08618*.
6. Cang, Kang & Wang (2021). "Pseudo-random number generator based on a generalized conservative Sprott-A system." *Nonlinear Dynamics*, 104, 827–844. doi:10.1007/s11071-021-06310-9
7. Halayka (2012). "On leveraging the chaotic and combinatorial nature of deterministic n-body dynamics on the unit m-sphere in order to implement a pseudo-random number generator." *viXra*.
8. Vuckovac (2021). "Cryptographic Puzzles and Complex Systems." *Complex Systems*, 30(3), 375–390. doi:10.25088/ComplexSystems.30.3.375
9. Kraicha et al. (2025). "Orbital-Inspired Encryption Using Phobos and Deimos Positions." *Journal of Information Security and Applications*, 88, 103912.
10. Jawad (2025). "DUff-skg: FHE cryptographic systems with chaotic secret key generation." *Acta Scientiarum. Technology*, 47(1).

### Kelvin Documentation

11. [Mode Flowcharts](mode_flowcharts.md) — Kelvin encryption mode architecture
12. [Homomorphic Cryptosystem Analysis](homomorphic_cryptosystem.md) — HE integration details
13. [Quantum Resistance Analysis](quantum_analysis.md) — Post-quantum security assessment
14. [Project History](project_history.md) — 24-year evolution of the system
15. [Proof of Concept](proof_of_concept.md) — Test results and verification
16. [Keyspace Analysis](keyspace_analysis.md) — Keyspace size estimation

---

*This document is part of the Kelvin Cryptosystem documentation suite.  
It represents a preliminary patent landscape analysis and should not be construed as legal advice. Consult a qualified patent attorney before filing any patent application.*

*Last Updated: 2026-05-27*
