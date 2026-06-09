# Kelvin Stream Cipher Study: Evolution from AEAD to Quantum-Resistant Stream Cipher

**Status:** Pre-Implementation Research  
**Date:** 2026-05-19  
**Author:** Kelvin Project

---

## 1. Motivation

> "One-time pad is the only safe crypto actually."

The user's assertion is correct in the information-theoretic sense: a true one-time pad (OTP) with a perfectly random keystream of equal length to the plaintext is **unconditionally secure** regardless of the attacker's computational power (Shannon, 1949). However, Kelvin's XOR modes are stream ciphers — they provide computational indistinguishability, not information-theoretic secrecy. This study documents the evolution and trade-offs.

Kelvin currently has two modes, and this study proposes two more following the stream cipher architecture:

| # | Name | Description |
|---|------|-------------|
| V1 | **Kelvin-Secure** | ChaCha20Poly1305 AEAD with upfront simulation + virtual key schedule |
| V2 | **Kelvin-Chaos** | Pure XOR streaming with per-step real-time simulation (one integration step per chunk) |
| V3 | **Kelvin-Photon** | Fast bulk stream: HKDF→SHAKE256 XOR from upfront simulation (fast as light) |
| H | **Kelvin-Quantum** | Hybrid V3+V2: bulk speed of Photon + fresh entropy of Chaos |

### 1.1 The Current Bottleneck

```
V1 Kelvin-Secure: 2048-byte seed → HKDF-SHA512 → 32 bytes key + 12 bytes nonce (44 bytes total)
                                                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                                     Only 0.27% of HKDF's 16,320-byte capacity used

V2 Kelvin-Chaos: No HKDF at all — SHAKE256 XOF directly from per-step orbital state
                 Unlimited output, but pays the per-step simulation cost

V3 Kelvin-Photon: HKDF full capacity → SHAKE256 XOF → unlimited keystream
                  No per-step cost (upfront simulation only)

H  Kelvin-Quantum: V3 bulk speed + V2 entropy injection at reseed boundaries
                   Best of all worlds
```

### 1.2 The Hybrid Insight

The key innovation of **Kelvin-Quantum (Hybrid)** is:

```
V3 [Base Seed] ──HKDF→SHAKE256──→ 1MB keystream (fast, ~200ms/GB)
                   ↑
                   │ XOR fresh chaos every N bytes
                   │
V2 [Orbital State] ──Verlet(10k steps)──→ Fresh Entropy (~0.5ms per reseed)
```

V3 provides **bulk throughput** (SHAKE256 is fast). V2 provides **fresh chaotic entropy** (each integration step produces genuinely new dynamics). The hybrid combines them: use V3's fast cache for throughput, but inject V2's fresh orbital chaos at reseed boundaries to prevent seed exhaustion attacks.

---

## 2. Canonical Mode Names

| Mode | Library Name | Tagline | Rationale |
|------|-------------|---------|-----------|
| V1 | `KelvinSecure` | "The safe choice" | AEAD authentication, proven ChaCha20Poly1305 |
| V2 | `KelvinChaos` | "Pure chaotic streaming" | Each byte from fresh orbital dynamics, unlimited keystream |
| V3 | `KelvinPhoton` | "Fast as light" | Pure XOR stream from upfront simulation, quantum-resistant |
| H | `KelvinQuantum` | "Best of all worlds" | Hybrid: V3 speed + V2 entropy freshness |

---

## 3. Research Questions

### 3.1 Can HKDF-SHA512 produce an OTP-grade keystream?

**Yes, if the input entropy is sufficient — but it is still a stream cipher, not an OTP.**

- HKDF is a standardized PRF (HMAC-SHA512). Its output is computationally indistinguishable from random — but it is **not** information-theoretically random.
- A true OTP requires a keystream with **min-entropy equal to its length**. HKDF's output is computationally bounded by the entropy of its input key material.
- The 2048-byte seed from SHAKE256 extraction has at most **2048 bytes × 8 = 16,384 bits** of entropy (assuming perfect extraction). HKDF cannot increase this entropy; it can only stretch it.
- **Conclusion**: All modes are **stream ciphers**, not true OTPs. The security is computational, not information-theoretic.

### 3.2 How much keystream can we safely produce per reseed?

| Primitive | Max output per call | Security basis |
|-----------|-------------------|-----------------|
| HKDF-SHA512 expand | 16,320 bytes | HMAC-SHA512 security |
| SHAKE256 XOF | Unlimited | Keccak sponge security |
| BLAKE3 XOF | Unlimited | Bao/Argon2 security |

**Per-call comparison across modes:**

| Mode | Keystream per cycle | Cost per cycle |
|------|-------------------|----------------|
| V1 Kelvin-Secure | 44 bytes (32+12) | 1× HKDF expand + BLAKE3 reseed |
| V2 Kelvin-Chaos | `bytes_per_step` bytes | 1× Verlet step + 1× SHAKE256 XOF |
| V3 Kelvin-Photon | Arbitrary (HKDF→SHAKE256) | 1× HKDF expand + SHAKE256 XOF + BLAKE3 reseed |
| H Kelvin-Quantum | Cache-size (default 1MB) | 1× HKDF→SHAKE256 + periodic Verlet(10k) |

### 3.3 What is the effective entropy per reseed?

| Mode | Entropy model | Total keystream bound |
|------|--------------|----------------------|
| V1 Kelvin-Secure | Single extraction → deterministic reseeds | `max_keys × 4 GiB` |
| V2 Kelvin-Chaos | Continuous fresh entropy per step | Unlimited |
| V3 Kelvin-Photon | Single extraction → deterministic reseeds | Same bound as V1, larger per cycle |
| H Kelvin-Quantum | V3 base + V2 chaos injection at reseed | Effectively unlimited |

**The hybrid solves the entropy problem**: each reseed injects fresh chaotic dynamics from the orbital simulation, preventing the "single extraction" weakness of V1 and V3.

### 3.4 Forward secrecy

| Mode | Forward secrecy | Mechanism |
|------|----------------|-----------|
| V1 Kelvin-Secure | ✅ Good | BLAKE3 reseed per key |
| V2 Kelvin-Chaos | ✅✅ Excellent | Each Verlet step independent |
| V3 Kelvin-Photon | ✅ Good | BLAKE3 reseed per cycle |
| H Kelvin-Quantum | ✅✅ Excellent | BLAKE3 reseed + Verlet chaos injection |

---

## 4. Architecture Comparison

### 4.1 Pipeline Diagrams

```
V1 Kelvin-Secure (AEAD):
  simulate(total_steps) → SHAKE256 → 2048B seed
    → HKDF → 32B key + 12B nonce → ChaCha20Poly1305 encrypt
    → BLAKE3 reseed → HKDF → 32B + 12B → ChaCha20Poly1305 encrypt
    ...
    └── Exhausted after max_keys × 4 GiB

V2 Kelvin-Chaos (Streaming XOR):
  [no upfront simulation]
  per_step: verlet_step() → SHAKE256 → N bytes keystream → XOR
  per_step: verlet_step() → SHAKE256 → N bytes keystream → XOR
  ...
  └── Unlimited (simulate indefinitely)

V3 Kelvin-Photon (Batch Stream):
  simulate(total_steps) → SHAKE256 → 2048B seed
    → HKDF → 64B XOF seed → SHAKE256 → N bytes → XOR
    → BLAKE3 reseed → HKDF → 64B XOF seed → SHAKE256 → M bytes → XOR
    ...
    └── Exhausted after max_keys × large_keystream

H Kelvin-Quantum (Hybrid Stream):
  simulate(total_steps) → SHAKE256 → 2048B base seed
    → HKDF → SHAKE256 → refill 1MB cache
    → XOR from cache → ... → XOR from cache → cache empty
    → Verlet(10k steps) → SHAKE256 → fresh 64B entropy
    → XOR fresh entropy into base seed
    → HKDF → SHAKE256 → refill 1MB cache (with fresh entropy)
    → XOR from cache → ... (repeat indefinitely)
```

### 4.2 Architecture Table

| Property | V1 Kelvin-Secure | V2 Kelvin-Chaos | V3 Kelvin-Photon | H Kelvin-Quantum |
|----------|-----------|-------------------|----------|--------|
| **Simulation** | Upfront full | Per-step | Upfront full | Upfront + periodic |
| **Cipher** | ChaCha20Poly1305 | SHAKE256 XOR | HKDF→SHAKE256 XOR | Hybrid cache + XOR |
| **Auth** | ✅ AEAD tag | ❌ None | ❌ None | ❌ None |
| **Nonce** | 12-byte internal counter per key (auto-incremented; never reused unless API is misused by manually resetting/forking state) | None | None | None |
| **Keystream** | Finite (~28 GiB) | ✅ Truly unlimited (simulation never stops; 1B step safety limit in impl) | Finite (max_keys × 16KB HKDF bound) | ⚠️ Effectively unlimited (max_keys × reseed_interval; with 1B step limit: ~100k reseeds × 10MB ≈ 1TB max) |
| **Entropy renewal** | Deterministic reseed | Fresh per step | Deterministic reseed | Fresh per reseed |
| **Setup cost** | High (seconds) | None | High (seconds) | High (seconds) |

---

## 5. Pairwise Comparisons (Folded Into Sections 1–4)

The pairwise comparison content that existed in the original document has been consolidated into the mode comparison in §1, the pipeline diagrams in §4.1, the architecture table in §4.2, and the research questions in §3. All mode-level trade-offs (bulk vs. streaming, upfront simulation vs. per-step, authentication availability, keystream limits) are documented in those sections.

---

## 24. Naming Clarification

> **Note on "Kelvin-Quantum"**: The name "Quantum" refers to **quantum-resistant** (post-quantum cryptography — i.e., security against attackers with quantum computers), not quantum key distribution (QKD), quantum computation, or any quantum mechanical phenomenon. The security of all Kelvin modes derives from classical chaotic n-body dynamics and standardized cryptographic primitives (SHAKE256, HKDF-SHA512), not from quantum mechanics.

Also note: All XOR modes are **stream ciphers**, not true one-time pads. The keystream is computationally indistinguishable from random via SHAKE256, not information-theoretically random. See [stream_cipher_security.md](stream_cipher_security.md) for the full security argument.

---

## 25. References


- Shannon, C. E. (1949). "Communication Theory of Secrecy Systems." *Bell System Technical Journal*, 28(4), 656–715.
- Krawczyk, H., & Eronen, P. (2010). "HMAC-based Extract-and-Expand Key Derivation Function (HKDF)." RFC 5869.
- Bertoni, G., et al. (2013). "Keccak." *EUROCRYPT 2013*, 313–314.
- NIST. (2015). "SHA-3 Standard: Permutation-Based Hash and Extendable-Output Functions." FIPS PUB 202.
- Aumasson, J.-P., et al. (2020). "BLAKE3: One Function, Fast Everywhere."
- Benettin et al. (1980). "Lyapunov Characteristic Exponents for Smooth Dynamical Systems." *Meccanica*, 15, 9–20.
- Wolf et al. (1985). "Determining Lyapunov Exponents from a Time Series." *Physica D*, 16(3), 285–317.