# NIST Post-Quantum Digital Signature Standardization: Relevance to Kelvin

**Date:** 2026-05-23  
**Status:** NIST Round 3 (May 2026) — Additional Digital Signature Schemes  
**Cross-references:** [Quantum Resistance Analysis](quantum_analysis.md), [V3 OTP Study](v3_otp_study.md), [RustCrypto Integration Plan](rustcrypto_integration_plan.md)

---

## Part 1: Concise Overview

### 1. Executive Summary

In May 2026, NIST advanced nine candidates to the third round of its post-quantum digital signature standardization process. This announcement concerns **digital signature algorithms** — not general-purpose encryption or key establishment. For Kelvin's core OTP keystream, which relies on SHAKE256 (already NIST-standardized for post-quantum use), this announcement has **no direct impact**. However, it is highly relevant to Kelvin's optional authentication layer and provides strong validation that the project's post-quantum direction aligns with global cryptographic standards.

### 2. The Nine Candidates

| Category | Algorithms |
|----------|------------|
| **Lattice-Based** | HAWK, SNOVA |
| **Multivariate-Based** | MAYO, MQOM, QR-UOV, UOV |
| **Code-Based** | FAEST, SDitH |
| **Isogeny-Based** | SQIsign |

All nine are digital signature schemes. None are encryption or key-establishment mechanisms.

### 3. Relevance to Kelvin

#### 3.1 OTP Keystream (Symmetric Encryption) — Not Directly Relevant

Kelvin's core cipher is a symmetric OTP deriving its keystream from an entropy source (n-body chaos) and a KDF (SHAKE256). Its post-quantum security depends on:

- **SHAKE256** — Already standardized by NIST (FIPS 202) for post-quantum use
- **ChaCha20** — 256-bit key provides 128-bit post-quantum security against Grover's algorithm (see [Quantum Resistance Analysis](quantum_analysis.md))

No replacement of the core cipher is needed or warranted by this announcement.

#### 3.2 Authentication Module — Highly Relevant

Kelvin's architecture includes an optional KMAC module for symmetric authentication. A digital signature provides a **stronger form of authentication** (non-repudiation). Integrating one of these NIST finalists (e.g., HAWK or SNOVA) as an optional post-quantum signature layer would enable:

- **Non-repudiation** of encrypted data — proof of origin that a symmetric MAC cannot provide
- **Post-quantum authenticated encryption** — combining PQ confidentiality (OTP) with PQ authenticity (signature)
- **Identity binding** — signatures can be linked to a public key infrastructure

#### 3.3 Academic Validation

The existence of this NIST process strongly reinforces the central argument of Kelvin's research: **the future is post-quantum**. Citing this announcement demonstrates that Kelvin-Quantum is aligned with the direction of global cryptographic standards.

### 4. Immediate Actionable Recommendations

| Area | Action |
|------|--------|
| **OTP Core** | No change needed. SHAKE256 is already NIST-standardized for PQ use. |
| **Future Work / Related Work** | Add a paragraph noting NIST's PQ signature standardization and the potential for integration. |
| **Your Research** | Study HAWK as a candidate for optional PQ signature integration. |

### 5. Draft Paragraph for Study

> *"Concurrent with this research, NIST is advancing nine candidates to the third round of its post-quantum digital signature standardization process, including lattice-based schemes like HAWK and SNOVA. While Kelvin-Quantum's core OTP keystream relies on the already-standardized SHAKE256, its optional authentication layer currently uses symmetric KMAC. For applications requiring non-repudiation, integrating a NIST-standardized post-quantum signature scheme like HAWK would be a natural extension, creating a cryptosystem with post-quantum confidentiality (via OTP) and post-quantum authenticity (via signature)."*

---

## Part 2: Deep Technical Appendices

---

### Appendix A: HAWK — Lattice-Based Signatures from Module-LIP

#### A.1 Overview

HAWK is the only lattice-based signature scheme in NIST's second-round evaluation for additional post-quantum signatures. Unlike CRYSTALS-Dilithium (based on Module-LWE/Module-SIS), HAWK is built on the **Module Lattice Isomorphism Problem (Module-LIP)**.

#### A.2 Algorithm Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HAWK SIGNATURE FLOW                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  KEY GENERATION:                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │ 1. Choose ring R = ℤ[ζ] where ζ^n = -1 (n = 512 or 1024)            │   │
│  │ 2. Sample random U ∈ GL₂(R) (invertible 2×2 matrix over R)          │   │
│  │ 3. Compute Q = U* × U (the "public" lattice basis)                   │   │
│  │ 4. Private key = U (or its components), Public key = Q               │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│  SIGNING:                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │ 1. Hash message to a target vector                                   │   │
│  │ 2. Use private key U to find a short vector representation           │   │
│  │ 3. Sample from discrete Gaussian distribution (two fixed cosets)     │   │
│  │ 4. Output compact signature (555 bytes for HAWK-512)                 │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│  VERIFICATION:                                                               │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │ 1. Check that signature satisfies quadratic form Q                   │   │
│  │ 2. Verify hash matches the reconstructed value                       │   │
│  │ 3. SUF-CMA security reduces to "one more SVP" (omSVP) problem        │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### A.3 The Hard Problem: Module-LIP

The security of HAWK rests on the **Module Lattice Isomorphism Problem**:

- **What is LIP?** Two lattices L(B₁) and L(B₂) are isomorphic if there exists an orthogonal transformation mapping one to the other. The search version asks: given two lattices, find that transformation.
- **For HAWK specifically:**
  - Key recovery = solving "search module LIP" (smLIP) in rank 2
  - Signature forgery = solving "one more SVP" (omSVP), which reduces to finding additional short vectors
  - "Recover HAWK private key ⇔ solve 'search module LIP' (smLIP) in rank 2"

#### A.4 Advantages

| Advantage | Details |
|-----------|---------|
| **Very fast** | Sign/verify < 0.1ms on modern CPUs; KeyGen ~3.5ms |
| **Compact signatures** | 555 bytes for HAWK-512, 1221 bytes for HAWK-1024 |
| **Small memory** | ≤12 KiB RAM, runs on ARM Cortex-M0+ (16 KiB SRAM devices) |
| **No floating point** | Works on embedded devices without FPU |
| **Isochronous** | Time independent of secret data (side-channel resistant by design) |
| **BUFF security** | Achieves binding properties natively |

#### A.5 Limitations

| Limitation | Details |
|------------|---------|
| **New problem** | Module-LIP has less cryptanalysis history than LWE/SIS |
| **Side-channel concerns** | Recent attacks (April 2026) show Gaussian sampler leakage can recover keys with ~14–400 signatures |
| **Theoretical uncertainty** | Recent work shows rank-2 module-LIP may be reducible to simpler problems |

#### A.6 NIST Status

HAWK advanced to Round 3 of NIST's additional signatures process (May 2026). NIST views its performance and compactness favorably, but continues scrutinizing the security assumptions.

---

### Appendix B: SNOVA — Multivariate Signatures from Matrix-Structured UOV

#### B.1 Overview

SNOVA belongs to the **multivariate quadratic (MQ)** family and is a structured variant of the classic UOV (Unbalanced Oil and Vinegar) scheme from 1999. The core innovation: **replace numbers with matrices** to reduce public key size while preserving the UOV trapdoor.

#### B.2 Classic UOV Foundation

```
OIL VARIABLES (o):  The "secret" variables — can be solved for
VINEGAR VARIABLES (v): The "random" variables — chosen freely

Classic UOV:
- Public key: o quadratic equations in (o+v) variables
- Private key: Knowledge of which variables are "oil" vs "vinegar"
- Signing: Choose vinegar variables at random, solve linear system for oil variables
- Security: MQ problem is NP-hard (can't distinguish oil from vinegar)

Problem with classic UOV: Public keys are enormous (e.g., 70+ KB even at low security).
```

#### B.3 SNOVA's Matrix Structure Innovation

SNOVA introduces a **block-ring structure** using matrices over finite fields:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SNOVA STRUCTURE                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Instead of variables in F_q, variables are MATRICES of size ℓ×ℓ over F_q   │
│                                                                              │
│  Parameters:                                                                │
│  - q: field size (typically 16)                                            │
│  - ℓ: matrix dimension (2, 3, or 4)                                        │
│  - m: number of matrix equations                                           │
│  - n: number of matrix variables                                           │
│  - S: symmetric ℓ×ℓ matrix with irreducible polynomial                     │
│                                                                              │
│  Public key structure:                                                      │
│  p_i(U) = Σ A_α · Uᵀ · Q_{α,1}⊗ⁿ · P_i · Q_{α,2}⊗ⁿ · U · B_α               │
│           ↑           ↑                    ↑                    ↑          │
│         random   powers of S         fixed matrices       random           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

This structure massively compresses the public key: SNOVA public keys range from **1 KB to 70 KB** depending on security level.

#### B.4 Parameter Sets

| Security Level | Parameters (q,ℓ,m,n) | Signature Size | Public Key Size | Claimed Security |
|----------------|----------------------|----------------|-----------------|------------------|
| I (lowest) | (16,2,17,54) | 124 B | 9.6 KB | 151 bits |
| I (lowest) | (16,3,8,33) | 164.5 B | 2.3 KB | 159 bits |
| I (lowest) | (16,4,5,29) | 248 B | 1.0 KB | 175 bits |
| III (medium) | (16,2,25,81) | 178 B | 31 KB | 215 bits |
| III (medium) | (16,3,11,60) | 286 B | 5.8 KB | 213 bits |
| V (highest) | (16,2,33,108) | 232 B | 70 KB | 279 bits |
| V (highest) | (16,3,15,81) | 380.5 B | 15 KB | 285 bits |
| V (highest) | (16,4,10,70) | 576 B | 7.8 KB | 335 bits |

SNOVA achieves signature sizes as small as **124 bytes** — among the smallest of any post-quantum signature scheme.

#### B.5 Advantages

| Advantage | Details |
|-----------|---------|
| **Tiny signatures** | As low as 124 bytes (HAWK is 555 bytes) |
| **Small public keys** | As low as 1 KB (with ℓ=4 parameter set) |
| **Fast verification** | ~218K cycles on ARM Cortex-M4 |
| **Proven UOV core** | UOV has remained unbroken since 1999 |

#### B.6 Limitations (Critical)

| Limitation | Details |
|------------|---------|
| **Multiple serious attacks** | Wedge product attack broke 6 of 11 parameter sets |
| **Round 2 parameters broken** | "All parameters of SNOVA updated for Round 2 are now broken" (NIST, Sept 2025) |
| **NIST security concerns** | "The novelty of SNOVA and its history of attacks lead NIST to question its security even more so than other structured-UOV schemes" |
| **New parameters submitted** | Designers submitted new parameters after attacks, but confidence is eroded |

#### B.7 NIST Status

SNOVA advanced to Round 3 despite these attacks, with the understanding that new parameters have been submitted to address the vulnerabilities. However, NIST explicitly notes its concerns about security in NIST.IR.8528.

---

### Appendix C: Head-to-Head Comparison

| Dimension | HAWK | SNOVA |
|-----------|------|-------|
| **Mathematical family** | Lattice (Module-LIP) | Multivariate (UOV with matrix structure) |
| **Security basis** | Relatively new (but actively studied) | MQ problem (NP-hard, 25+ years) |
| **Signature size** | 555–1221 bytes | 124–576 bytes (smaller) |
| **Public key size** | 1024–2440 bytes | 1–70 KB (wider range) |
| **Signing speed** | ~85K cycles | ~608K cycles (slower) |
| **Verification speed** | ~148K cycles | ~218K cycles |
| **Side-channel resistance** | Isochronous by design | Not a focus |
| **Attack history** | Theoretical progress; side-channel leaks possible | Multiple breaks (6 parameter sets) |
| **NIST confidence** | Cautiously optimistic | Explicitly questioned |
| **RAM usage** | ≤12 KiB | Not specified |
| **Embedded suitability** | Excellent (ARM Cortex-M0+) | Unknown |

#### Verdict: HAWK is Preferred for Kelvin

For integration into Kelvin's authentication layer, **HAWK is the more suitable candidate**:

1. **NIST confidence** — Even with side-channel concerns, NIST hasn't questioned HAWK's core security the way they have SNOVA's
2. **Performance profile** — Extremely fast signing (85K cycles) fits Kelvin's high-performance goals
3. **Hardware friendly** — No floating point, runs on constrained devices
4. **Small RAM** — 12 KiB fits well with embedded use cases
5. **BUFF security** — Achieves binding properties natively
6. **Active research community** — Problems like module-LIP are being actively studied; recent attacks are on implementation (side-channel), not mathematics

---

### Appendix D: Integration Concept

#### D.1 Feature Flag Design

A natural integration point would be an optional `pq-signatures` feature in the `kelvin` crate:

```toml
# kelvin/Cargo.toml
[features]
pq-signatures = ["dep:hawk"]  # hypothetical HAWK crate
```

#### D.2 Code Sketch

```rust
/// Kelvin-Quantum with HAWK signatures (post-quantum non-repudiation)
/// 
/// This combines Kelvin's OTP confidentiality with HAWK's post-quantum
/// digital signatures for authenticated encryption with non-repudiation.
#[cfg(feature = "pq-signatures")]
pub struct KelvinQuantumAuthenticated {
    cipher: KelvinQuantum,      // OTP core (SHAKE256 + ChaCha20)
    hawk_signer: HawkSigner,    // HAWK for post-quantum signatures
}

#[cfg(feature = "pq-signatures")]
impl KelvinQuantumAuthenticated {
    /// Encrypt and sign with post-quantum security.
    ///
    /// 1. OTP encryption using Kelvin's chaotic KDF + SHAKE256 keystream
    /// 2. HAWK signature over the ciphertext (555 bytes for HAWK-512)
    ///
    /// Returns the HAWK signature for separate transmission or storage.
    pub fn encrypt_and_sign(&mut self, data: &mut [u8]) -> Result<Vec<u8>> {
        // Phase 1: OTP encryption (Kelvin core)
        let keystream = self.cipher.keystream(data.len())?;
        xor_in_place(data, &keystream);
        
        // Phase 2: HAWK signature over ciphertext
        let signature = self.hawk_signer.sign(data)?;
        
        Ok(signature)
    }
    
    /// Verify signature and decrypt.
    pub fn verify_and_decrypt(
        &mut self,
        data: &mut [u8],
        signature: &[u8],
        public_key: &HawkPublicKey,
    ) -> Result<bool> {
        // Verify first (fail fast if tampered)
        let valid = self.hawk_signer.verify(data, signature, public_key)?;
        if !valid {
            return Ok(false);
        }
        
        // Decrypt
        let keystream = self.cipher.keystream(data.len())?;
        xor_in_place(data, &keystream);
        
        Ok(true)
    }
}
```

#### D.3 Security Properties

| Property | Mechanism |
|----------|-----------|
| **Confidentiality** | OTP keystream (SHAKE256 + ChaCha20, 256-bit key) |
| **Non-repudiation** | HAWK digital signature (Module-LIP, 128-bit PQ security) |
| **Authentication** | Signature verification before decryption |
| **Integrity** | Signature covers ciphertext; tampering detected before decryption |

---

### Appendix E: Full Candidate Reference

| Algorithm | Family | Key Feature | Signature Size | Potential for Kelvin |
|-----------|--------|-------------|----------------|---------------------|
| **HAWK** | Lattice | Fast, small signatures | 555–1221 B | **Excellent** — modular integration for non-repudiation |
| **SNOVA** | Lattice (UOV variant) | Efficient, balanced | 124–576 B | Good — similar to HAWK, but NIST security concerns |
| **FAEST** | Code-based (AES-based) | Conservative security | ~5–10 KB | Moderate — larger signatures |
| **SQIsign** | Isogeny | Smallest signatures | ~112 B | Poor — computationally very slow |
| **MAYO** | Multivariate | Niche optimizations | ~500 B | Unlikely — too specialized |
| **MQOM** | Multivariate | Niche optimizations | ~1–2 KB | Unlikely — too specialized |
| **QR-UOV** | Multivariate | Structured UOV variant | ~1–2 KB | Unlikely — too specialized |
| **UOV** | Multivariate | Classic UOV | ~1–2 KB | Unlikely — too specialized |
| **SDitH** | Code-based | Syndrome decoding | ~5–10 KB | Unlikely — too specialized |

---

*This document is part of the Kelvin Cryptosystem documentation suite.  
See also: [Quantum Resistance Analysis](quantum_analysis.md), [V3 OTP Study](v3_otp_study.md), [RustCrypto Integration Plan](rustcrypto_integration_plan.md)*
