# RustCrypto Integration Plan — Kelvin

> **Status**: Planning Phase  
> **Target**: Incrementally replace custom cryptographic plumbing with RustCrypto-ecosystem crates  
> **Design Principle**: The core n-body simulation (`kelvin-core`) remains dependency-free. RustCrypto integration happens in the cryptographic layers only.

---

## Table of Contents

1. [Current Architecture Overview](#1-current-architecture-overview)
2. [Dependency Map](#2-dependency-map)
3. [Phase 0 — Authenticated Encryption via ChaCha20Poly1305](#3-phase-0--authenticated-encryption-via-chacha20poly1305)
4. [Phase 1 — Standardized Key Derivation via HKDF](#4-phase-1--standardized-key-derivation-via-hkdf)
5. [Phase 2 — Fast Reseeding via BLAKE3](#5-phase-2--fast-reseeding-via-blake3)
6. [Phase 3 — Adopt RustCrypto Traits for Interoperability](#6-phase-3--adopt-rustcrypto-traits-for-interoperability)
7. [Phase 4 — Additional Cryptographic Options](#7-phase-4--additional-cryptographic-options)
8. [Files Modified Per Phase](#8-files-modified-per-phase)
9. [Backward Compatibility and Migration](#9-backward-compatibility-and-migration)
10. [Testing and Verification Strategy](#10-testing-and-verification-strategy)

---

## 1. Current Architecture Overview

```
                                    ┌────────────────────┐
                                    │   kelvin-core       │
                                    │  (no_std, no deps)  │
                                    │                     │
                                    │  Fixed Q32.64       │
                                    │  Vec3, OrbitalBody  │
                                    │  Verlet integrator  │
                                    │  Stability checks   │
                                    └─────────┬───────────┘
                                              │
                                              ▼
                                    ┌────────────────────┐
                                    │   kelvin-kdf        │
                                    │                     │
                                    │  OrbitalConfig      │
                                    │  LyapunovEstimator  │
                                    │                     │
                                    │  ┌───────────────┐  │
                                    │  │  ExtractSeed   │──│── SHA3-512 / SHAKE256 (RustCrypto ✓)
                                    │  └───────────────┘  │
                                    │  ┌───────────────┐  │
                                    │  │  KeySchedule   │──│── Custom SHA3-512 KDF + SHAKE256 reseed
                                    │  └───────────────┘  │
                                    │  ┌───────────────┐  │
                                    │  │  Asymmetric   │──│── curve25519-dalek / x25519-dalek / ml-kem / ml-dsa
                                    │  └───────────────┘  │
                                    └─────────┬───────────┘
                                              │
                    ┌─────────────────────────┼─────────────────────────┐
                    │                         │                         │
                    ▼                         ▼                         ▼
           ┌───────────────┐       ┌──────────────────┐      ┌──────────────────┐
           │  kelvin-stream│       │  kelvin (top)    │      │  kelvin-cli       │
           │               │       │                  │      │                   │
           │  ChaCha20 ✓   │       │  Encrypt/Decrypt │      │  keygen/enc/dec   │
           │  AES-256-CTR  │       │  Asymmetric      │      │  benchmark        │
           └───────────────┘       └──────────────────┘      └──────────────────┘
```

**Currently using RustCrypto**: `sha3`, `chacha20`, `aes` (optional), `ctr` (optional).  
**Not using RustCrypto**: `hkdf`, `chacha20poly1305`, `blake3`, `signature`, `aead`, `ed25519-dalek`, `argon2`.

---

## 2. Dependency Map

### Current Dependencies (RustCrypto-blessed)

| Crate | Version | Used In | Purpose |
|-------|---------|---------|---------|
| `sha3` | 0.10 | `kelvin-kdf` | SHA3-512 + SHAKE256 entropy extraction |
| `chacha20` | 0.9 | `kelvin-stream` | ChaCha20 IETF stream cipher |
| `aes` | 0.9 (optional) | `kelvin-stream` | AES-256-CTR fallback |
| `ctr` | 0.10 (optional) | `kelvin-stream` | CTR mode for AES |

### Current Dependencies (Non-RustCrypto)

| Crate | Version | Used In | Purpose |
|-------|---------|---------|---------|
| `curve25519-dalek` | 4 | `kelvin-kdf` | ECC scalar operations |
| `x25519-dalek` | 2 | `kelvin-kdf` | X25519 key exchange |
| `ml-kem` | 0.3 | `kelvin-kdf` | ML-KEM-768 post-quantum KEM |
| `ml-dsa` | 0.1.0-rc.4 | `kelvin-kdf` | ML-DSA-65 post-quantum signatures |

### Proposed New Dependencies

| Crate | Version | Phase | Purpose |
|-------|---------|-------|---------|
| `chacha20poly1305` | 0.10 | Phase 0 | Authenticated encryption (AEAD) |
| `hkdf` | 0.12 | Phase 1 | RFC 5869 standardized key derivation |
| `blake3` | 1.5 | Phase 2 | Fast key schedule reseeding |
| `signature` | 2.2 | Phase 3 | Standardized Signer/Verifier traits |
| `aead` | 0.5 | Phase 3 | Standardized AEAD trait |
| `ed25519-dalek` | 2 | Phase 3 | Ed25519 classical signatures |
| `argon2` | 0.5 | Phase 4 | Password-based config derivation |

---

## 3. Phase 0 — Authenticated Encryption via ChaCha20Poly1305

> **Priority**: Highest  
> **Security Impact**: Critical — adds authentication preventing chosen-ciphertext attacks  
> **Effort**: Low (modify ~3 files)  
> **Backward Compatible**: No (changes keystream format)

### 3.1 Problem

Kelvin currently uses raw ChaCha20 XOR (`kelvin-stream/src/chacha.rs`):

```rust
// CURRENT: No authentication tag
self.cipher.apply_keystream(data);
```

This provides **no integrity protection**. An attacker with write access can flip arbitrary bits in the ciphertext, and the decryption will silently produce corrupted plaintext (or worse, inject controlled plaintext).

### 3.2 Solution

Replace `ChaChaStream` with `ChaCha20Poly1305` AEAD (Authenticated Encryption with Associated Data).

### 3.3 Implementation Steps

#### Step 3.3.1 — `kelvin-stream/Cargo.toml`

```toml
[dependencies]
chacha20 = "0.9"
chacha20poly1305 = "0.10"   # NEW: AEAD mode
zeroize = { version = "1.6", features = ["zeroize_derive"] }

[features]
aes = ["dep:aes"]
ctr = ["dep:ctr"]
aes-ni = ["dep:aes", "dep:ctr"]
```

#### Step 3.3.2 — `kelvin-stream/src/traits.rs`

Replace the custom `StreamCipher` trait with an adapter that wraps RustCrypto's `Aead` trait:

```rust
// BEFORE: Custom StreamCipher trait
pub trait StreamCipher: core::fmt::Debug {
    fn xor_in_place(&mut self, data: &mut [u8]);
    fn position(&self) -> u64;
    fn max_safe_bytes(&self) -> u64;
}

// AFTER: Keep the trait but delegate to AEAD internally
// The trait stays for backward compatibility with Kelvin's architecture
// but the ChaCha implementation uses ChaCha20Poly1305 internally.
```

**Decision**: Keep the `StreamCipher` trait for now (avoids changing the entire architecture in one go). Phase 3 can adopt the `aead` trait directly.

#### Step 3.3.3 — `kelvin-stream/src/chacha.rs`

**Key change**: Switch from `chacha20::ChaCha20` to `chacha20poly1305::ChaCha20Poly1305`.

| Aspect | Current | New |
|--------|---------|-----|
| Algorithm | `ChaCha20` (plain XOR) | `ChaCha20Poly1305` (AEAD) |
| Key size | 32 bytes | 32 bytes (same) |
| Nonce size | 12 bytes | 12 bytes (same, IETF variant) |
| Output size | `len(plain)` | `len(plain) + 16` (Poly1305 tag) |
| Auth tag | None | 16-byte Poly1305 tag appended |
| Associated Data | N/A | Empty `&[]` by default (configurable) |

**Method signature change**:

```rust
// BEFORE
pub fn xor_in_place(&mut self, data: &mut [u8]) -> ();

// AFTER — returns Result with tag verification
pub fn encrypt_in_place(
    &mut self,
    buffer: &mut [u8],
) -> Result<(), aead::Error>;

pub fn decrypt_in_place(
    &mut self,
    buffer: &mut [u8],
) -> Result<(), aead::Error>;
```

#### Step 3.3.4 — `kelvin/src/lib.rs`

Update the `Kelvin` struct's encrypt/decrypt methods to handle AEAD:

```rust
// BEFORE
pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
    self.stream.xor_in_place(data);
    self.bytes_processed += data.len() as u64;
    Ok(())
}

// AFTER
pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
    // AEAD encrypt_in_place needs space for the 16-byte tag
    // This is an overloaded API: data[..len] is plaintext, data[len..] gets the tag
    self.stream.encrypt_in_place(data)
        .map_err(|_| KelvinError::EncryptionFailed)?;
    self.bytes_processed += data.len() as u64;
    Ok(())
}

pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
    // AEAD decrypt_in_place verifies the tag and strips it
    self.stream.decrypt_in_place(data)
        .map_err(|_| KelvinError::AuthenticationFailed)?;
    self.bytes_processed += data.len() as u64;
    Ok(())
}
```

**Important**: AEAD `encrypt_in_place` requires `buffer.len() >= plaintext_len + 16` for the tag. The caller must allocate the extra 16 bytes. This is a **breaking protocol change** — ciphertexts will be 16 bytes longer.

#### Step 3.3.5 — `kelvin-stream/src/traits.rs` (Updated)

```rust
/// Trait for authenticated stream ciphers.
pub trait StreamCipher: core::fmt::Debug {
    /// Encrypt data in-place using AEAD.
    /// `buffer[..plaintext_len]` contains plaintext.
    /// On success, `buffer[..plaintext_len]` contains ciphertext + appended tag.
    fn encrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error>;

    /// Decrypt data in-place using AEAD.
    /// `buffer[..ciphertext_len]` contains ciphertext + appended tag.
    /// On success, `buffer[..ciphertext_len - 16]` contains plaintext.
    fn decrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error>;

    fn position(&self) -> u64;
    fn max_safe_bytes(&self) -> u64;
}
```

#### Step 3.3.6 — `kelvin-stream/src/aes_ctr.rs`

AES-256-CTR does not have a standard AEAD mode in the current codebase. Options:
1. Add AES-GCM via `aes-gcm` crate (adds authentication)
2. Keep raw AES-256-CTR as-is, document the lack of authentication
3. Remove the AES-NI feature (simplify)

**Recommendation**: Add `aes-gcm` as the authenticated AES mode and deprecate the raw CTR mode.

### 3.4 Phase 0 Test Plan

| Test | Description |
|------|-------------|
| `test_aead_round_trip` | Encrypt then decrypt, verify plaintext matches |
| `test_aead_tag_verification` | Corrupt ciphertext, verify decryption fails |
| `test_aead_deterministic` | Same key+nonce produces same ciphertext+tag |
| `test_aead_different_keys` | Different keys produce different outputs |
| `test_aead_associated_data` | Test with non-empty associated data |
| `test_too_small_buffer` | Buffer smaller than tag, verify error |

---

## 4. Phase 1 — Standardized Key Derivation via HKDF

> **Priority**: High  
> **Security Impact**: Medium — replaces ad-hoc KDF with RFC 5869 standard  
> **Effort**: Low (modify ~2 files)  
> **Backward Compatible**: No (changes derived key values)

### 4.1 Problem

In `kelvin-kdf/src/schedule.rs`, the `KeySchedule::next_key()` method derives keys using a custom construction:

```rust
// CURRENT: Ad-hoc KDF (SHA3-512 of seed + counter)
let mut hasher = Sha3_512::new();
Digest::update(&mut hasher, b"kelvin-key-derivation-v1");
Digest::update(&mut hasher, &self.seed[..]);
Digest::update(&mut hasher, &self.keys_generated.to_le_bytes());
let hash = hasher.finalize();
// Then split into key[..32] + nonce[32..48]
```

This is essentially a custom KDF construction. While SHA3-512 is sound, a standardized KDF like HKDF provides:
- Proven security reduction
- Domain separation via `info` parameter
- Efficient extraction + expansion phases
- Auditability against a known standard

### 4.2 Solution

Replace the custom key derivation with HKDF-SHA512 (HKDF instantiated with SHA-512).

### 4.3 Implementation Steps

#### Step 4.3.1 — `kelvin-kdf/Cargo.toml`

```toml
[dependencies]
sha3 = { version = "0.10", default-features = false }
hkdf = "0.12"                          # NEW: RFC 5869 HKDF
```

#### Step 4.3.2 — `kelvin-kdf/src/schedule.rs` — Key Derivation

```rust
// BEFORE — custom KDF
use sha3::{Digest, Sha3_512, Shake256};

pub fn next_key(&mut self) -> Option<([u8; 32], [u8; 16])> {
    // ...
    let mut hasher = Sha3_512::new();
    Digest::update(&mut hasher, b"kelvin-key-derivation-v1");
    Digest::update(&mut hasher, &self.seed[..]);
    Digest::update(&mut hasher, &self.keys_generated.to_le_bytes());
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash[..32]);
    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&hash[32..48]);
    // ...
}

// AFTER — HKDF-SHA512
use sha3::Sha3_512;
use hkdf::Hkdf;

pub fn next_key(&mut self) -> Option<([u8; 32], [u8; 16])> {
    // ...
    let hk = Hkdf::<Sha3_512>::new(None, &self.seed);

    // Domain-separated key derivation
    let mut key_info = Vec::new();
    key_info.extend_from_slice(b"kelvin-hkdf-key-v1");
    key_info.extend_from_slice(&self.keys_generated.to_le_bytes());

    let mut nonce_info = Vec::new();
    nonce_info.extend_from_slice(b"kelvin-hkdf-nonce-v1");
    nonce_info.extend_from_slice(&self.keys_generated.to_le_bytes());

    let mut key = [0u8; 32];
    hk.expand(&key_info, &mut key)
        .expect("HKDF expand should not fail for valid output length");

    let mut nonce = [0u8; 16];
    hk.expand(&nonce_info, &mut nonce)
        .expect("HKDF expand should not fail for valid output length");
    // ...
}
```

#### Step 4.3.3 — `kelvin-kdf/src/schedule.rs` — Reseeding

The reseeding path (SHAKE256 of current seed + step) is a **XOF-based reseed**, which is different from HKDF. Keep SHAKE256 for reseeding. This is justiï¬ed because:
- SHAKE256 is a standardized XOF designed for arbitrary-length output
- The reseed generates a full 2048-byte pool, which is not a standard HKDF usage pattern
- SHAKE256 provides domain separation via the context string

### 4.4 Phase 1 Test Plan

| Test | Description |
|------|-------------|
| `test_hkdf_deterministic` | Same seed + counter produces same key |
| `test_hkdf_key_nonce_different` | Key and nonce differ from same seed |
| `test_hkdf_domain_separation` | Different info strings → different outputs |
| `test_hkdf_multiple_keys` | Sequential keys are all different |
| `test_hkdf_seed_avalanche` | 1-bit seed change → completely different keys |

---

## 5. Phase 2 — Fast Reseeding via BLAKE3

> **Priority**: Medium  
> **Security Impact**: Low — reseeding is performance-critical, SHAKE256 is slower  
> **Effort**: Low (modify ~2 files)  
> **Backward Compatible**: No (changes reseed output)

### 5.1 Problem

In `kelvin-kdf/src/schedule.rs`, reseeding uses SHAKE256:

```rust
// CURRENT: SHAKE256 reseed (secure but slow)
let mut reseed_hasher = Shake256::default();
sha3::digest::Update::update(&mut reseed_hasher, b"kelvin-reseed-v1");
sha3::digest::Update::update(&mut reseed_hasher, &self.seed[..]);
sha3::digest::Update::update(&mut reseed_hasher, &self.step.to_le_bytes());
let mut reader = reseed_hasher.finalize_xof();
XofReader::read(&mut reader, &mut reseed_buf);
```

SHAKE256 is a sponge-based XOF — secure and standardized, but computationally expensive for generating 2048-byte pools. This reseed happens every `reseed_interval` simulation steps, which could be frequent.

### 5.2 Solution

Replace SHAKE256 reseeding with BLAKE3, which is:
- ~10x faster than SHAKE256 for large outputs
- A XOF by design (native arbitrary-length output)
- Simpler API — no separate XOF reader pattern
- Cryptographically secure (3.5x safety margin over SHA3)

### 5.3 Implementation Steps

#### Step 5.3.1 — `kelvin-kdf/Cargo.toml`

```toml
[dependencies]
sha3 = { version = "0.10", default-features = false }
blake3 = "1.5"                         # NEW: Fast XOF reseeding
hkdf = "0.12"
```

#### Step 5.3.2 — `kelvin-kdf/src/schedule.rs` — Reseed

```rust
// BEFORE — SHAKE256 reseed
use sha3::{Digest, Sha3_512, Shake256};

let mut reseed_buf = [0u8; 2048];
let mut reseed_hasher = Shake256::default();
sha3::digest::Update::update(&mut reseed_hasher, b"kelvin-reseed-v1");
sha3::digest::Update::update(&mut reseed_hasher, &self.seed[..]);
sha3::digest::Update::update(&mut reseed_hasher, &self.step.to_le_bytes());
let mut reader = reseed_hasher.finalize_xof();
XofReader::read(&mut reader, &mut reseed_buf);
self.seed = reseed_buf;

// AFTER — BLAKE3 reseed
use blake3::Hasher;

let mut reseed_hasher = Hasher::new();
reseed_hasher.update(b"kelvin-reseed-v1");
reseed_hasher.update(&self.seed[..]);
reseed_hasher.update(&self.step.to_le_bytes());
let mut reseed_buf = [0u8; 2048];
reseed_hasher.finalize_xof().fill(&mut reseed_buf);
self.seed = reseed_buf;
```

#### 5.3.3 Validation: Why BLAKE3 for Reseeding but SHAKE256 for Entropy Extraction?

| Criterion | Entropy Extraction | Key Schedule Reseed |
|-----------|-------------------|-------------------|
| **Frequency** | Once per simulation | Every reseed_interval (many times) |
| **NIST Standard?** | Required (audit) | Not required (internal) |
| **Output size** | 2048 bytes | 2048 bytes |
| **Performance** | Not critical (1x) | Critical (1000x+) |
| **Security margin** | Single hash layer | Inside KDF pipeline |

**Decision**: Keep SHAKE256 for the initial entropy extraction from orbital state (Phase 0-style, NIST-audited). Use BLAKE3 for the internal reseeding (performance-sensitive, inside the KDF).

### 5.4 Phase 2 Test Plan

| Test | Description |
|------|-------------|
| `test_blake3_deterministic` | Same seed produces same reseed output |
| `test_blake3_different_seed` | Different seeds produce different outputs |
| `test_blake3_step_aware` | Different step counters produce different outputs |
| `test_blake3_entropy` | Statistical tests on reseed output uniformity |
| `test_blake3_performance` | Benchmark vs SHAKE256 reseed |

---

## 6. Phase 3 — Adopt RustCrypto Traits for Interoperability

> **Priority**: Low  
> **Security Impact**: Low — ecosystem alignment  
> **Effort**: Moderate (modify ~4 files)  
> **Backward Compatible**: Yes (additive changes)

### 6.1 Problem

Kelvin defines custom traits (`StreamCipher`) and custom types (`OrbitalKeyPair`) that don't interoperate with the broader Rust cryptographic ecosystem. Adopting RustCrypto's standard traits would allow Kelvin to work with:

- TLS implementations (`rustls`, `tls-api`)
- Message serialization frameworks
- Signature verification in other tools

### 6.2 Implementation Steps

#### 6.2.1 — Adopt `signature` Trait for ML-DSA-65

```rust
// kelvin-kdf/src/signature_adapter.rs
use signature::{Signer, Verifier};
use ml_dsa::{MlDsa65, SigningKey, VerifyingKey};

/// Adapter implementing RustCrypto's Signer/Verifier traits.
pub struct KelvinSigner(pub SigningKey<MlDsa65>);
pub struct KelvinVerifier(pub VerifyingKey<MlDsa65>);

impl Signer<Vec<u8>> for KelvinSigner {
    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.0.sign(msg).to_vec()
    }
}

impl Verifier<Vec<u8>> for KelvinVerifier {
    fn verify(&self, msg: &[u8], signature: &[u8]) -> Result<(), signature::Error> {
        self.0
            .verify(msg, &ml_dsa::Signature::try_from(signature).map_err(|_| signature::Error::new())?)
            .map_err(|_| signature::Error::new())
    }
}
```

#### 6.2.2 — Adopt `aead` Trait for ChaChaStream (as alternative)

```rust
// kelvin-stream/src/aead_adapter.rs
use aead::{Aead, KeyInit, Payload};
use chacha20poly1305::ChaCha20Poly1305;

impl Aead for KelvinAeadStream {
    type NonceSize = <ChaCha20Poly1305 as Aead>::NonceSize;
    type TagSize = <ChaCha20Poly1305 as Aead>::TagSize;
    type CiphertextOverhead = <ChaCha20Poly1305 as Aead>::CiphertextOverhead;

    fn encrypt<'msg, 'aad>(
        &self,
        nonce: &aead::Nonce<Self::NonceSize>,
        plaintext: impl Into<Payload<'msg, 'aad>>,
    ) -> Result<Vec<u8>, aead::Error> {
        self.inner.encrypt(nonce, plaintext)
    }

    fn decrypt<'msg, 'aad>(
        &self,
        nonce: &aead::Nonce<Self::NonceSize>,
        ciphertext: impl Into<Payload<'msg, 'aad>>,
    ) -> Result<Vec<u8>, aead::Error> {
        self.inner.decrypt(nonce, ciphertext)
    }
}
```

**Note**: The `aead` trait uses a different API pattern (nonce-generic) than Kelvin's current design. This adapter should be additive — Kelvin's internal API stays the same, but can now also expose AEAD-compatible interfaces.

#### 6.2.3 — Add Ed25519 Classical Signatures

```rust
// kelvin-kdf/src/asymmetric.rs — additional method

use ed25519_dalek::{SigningKey as EdSigningKey, VerifyingKey as EdVerifyingKey};

impl OrbitalKeyPair {
    /// Derive an Ed25519 sub-key from the same orbital state.
    pub fn derive_ed25519(&self, bodies: &[OrbitalBody], step: u64, g: Fixed, softening: Fixed) -> (EdSigningKey, EdVerifyingKey) {
        let seed_ed = extract_seed(bodies, step, g, softening, b"kelvin-ed25519-v1");
        let mut ed_seed = [0u8; 32];
        ed_seed.copy_from_slice(&seed_ed[..32]);
        let signing_key = EdSigningKey::from_bytes(&ed_seed);
        let verifying_key = signing_key.verifying_key();
        (signing_key, verifying_key)
    }
}
```

### 6.3 Phase 3 Test Plan

| Test | Description |
|------|-------------|
| `test_signature_trait_ml_dsa` | Sign and verify via `signature::Signer`/`Verifier` |
| `test_signature_trait_ed25519` | Sign and verify with Ed25519 |
| `test_aead_trait_compatibility` | Encrypt/decrypt via `aead::Aead` trait |
| `test_cross_ecosystem` | Verify Kelvin ML-DSA sig in external tool |

---

## 7. Phase 4 — Additional Cryptographic Options

> **Priority**: Optional  
> **Security Impact**: Low  
> **Effort**: Low (modify ~3 files)

### 7.1 Argon2 Password-to-Config

Kelvin is key-based (requires an `OrbitalConfig`), not password-based. An optional password-to-config path would improve usability:

```text
Password → Argon2id → Deterministic seed → OrbitalConfig::from_seed()
```

This would use:
- `argon2` crate (RustCrypto-aligned) — memory-hard KDF
- A deterministic function to map the 32-byte Argon2 output to orbital parameters (masses, positions, velocities)

```rust
// kelvin-cli/ or kelvin-kdf/src/password.rs

use argon2::{Argon2, PasswordHash, PasswordHasher, Params};
use argon2::password_hash::SaltString;

pub fn password_to_config(password: &str, salt: &[u8]) -> Result<OrbitalConfig, ConfigError> {
    // 1. Derive 32-byte key from password
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(65536, 3, 4, Some(32))?,
    );
    let salt = SaltString::encode_b64(salt)?;
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    let key: [u8; 32] = hash.hash.unwrap().as_bytes().try_into()?;

    // 2. Deterministically map to orbital parameters
    OrbitalConfig::from_seed(&key)
}

impl OrbitalConfig {
    /// Create an orbital configuration from a 32-byte seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        // Use seed bytes to deterministically generate:
        // - Number of bodies (3-10 based on seed[0])
        // - Masses, positions, velocities using seeded RNG
        // - G, dt, softening from seed
        // This is an area of active research — mapping bits to
        // orbital configurations is a non-trivial problem.
    }
}
```

This would add:
- `argon2 = "0.5"` to `kelvin-kdf` or `kelvin-cli` dependencies
- `rsrand::Rng` with the seed for deterministic parameter generation

---

## 8. Files Modified Per Phase

| Phase | Files Modified | Files Added |
|-------|---------------|-------------|
| **Phase 0** | `kelvin-stream/Cargo.toml`<br>`kelvin-stream/src/chacha.rs`<br>`kelvin-stream/src/traits.rs`<br>`kelvin-stream/src/aes_ctr.rs`<br>`kelvin/src/lib.rs`<br>`kelvin/src/error.rs` | None |
| **Phase 1** | `kelvin-kdf/Cargo.toml`<br>`kelvin-kdf/src/schedule.rs` | None |
| **Phase 2** | `kelvin-kdf/Cargo.toml`<br>`kelvin-kdf/src/schedule.rs` | None |
| **Phase 3** | `kelvin-stream/Cargo.toml`<br>`kelvin-kdf/Cargo.toml`<br>`kelvin-kdf/src/asymmetric.rs` | `kelvin-kdf/src/signature_adapter.rs`<br>`kelvin-stream/src/aead_adapter.rs` |
| **Phase 4** | `kelvin-kdf/Cargo.toml`<br>`kelvin-cli/Cargo.toml` | `kelvin-kdf/src/password.rs` |

### Files Never Modified

| File | Reason |
|------|--------|
| `kelvin-core/*` | No-dependency, no_std engine — RustCrypto integration is a cryptographic concern |
| `kelvin-ffi/*` | C FFI bindings — not impacted by Rust dependency changes |
| `libs/*` | Foreign language bindings — would need separate updates |

---

## 9. Backward Compatibility and Migration

### 9.1 Breaking Changes by Phase

| Phase | Breaking? | Impact |
|-------|-----------|--------|
| Phase 0 | **Yes** | Ciphertext format changes (added auth tag, 16 bytes longer) |
| Phase 1 | **Yes** | Derived keys change (different KDF algorithm) |
| Phase 2 | **Yes** | Reseed values change (different XOF algorithm) |
| Phase 3 | **No** | Additive traits, existing API unchanged |
| Phase 4 | **No** | New feature, existing API unchanged |

### 9.2 Migration Strategy

1. **Version bumps**: Each breaking phase increments the minor version (0.2.0 → 0.3.0 → 0.4.0)
2. **Feature flags**: Some changes can be feature-gated:
   - `#[cfg(feature = "aead")]` for Phase 0
   - `#[cfg(feature = "hkdf")]` for Phase 1
   - `#[cfg(feature = "blake3-reseed")]` for Phase 2
3. **Coexistence**: During transition, old and new code paths could coexist under feature flags, allowing users to migrate at their own pace.

### 9.3 Migration Path

```text
Phase 0 release (v0.2.0): aead feature flag
  - Default: ChaCha20Poly1305
  - Feature "classic-xor": legacy raw ChaCha20 (for pre-0.2.0 file access)
  
Phase 1 release (v0.3.0): hkdf feature flag
  - Default: HKDF-SHA512
  - Feature "legacy-kdf": old custom SHA3-512 KDF
  
Phase 2 release (v0.4.0): blake3-reseed feature flag
  - Default: BLAKE3 reseed
  - Feature "shake256-reseed": old SHAKE256 reseed
```

---

## 10. Testing and Verification Strategy

### 10.1 Per-Phase Testing

Each phase must pass:

1. **Determinism tests**: Same inputs → same outputs (regardless of platform)
2. **Round-trip tests**: Encrypt → decrypt → original plaintext
3. **Avalanche tests**: 1-bit input change → >50% output bit difference
4. **Known-answer tests**: Hardcoded test vectors specific to each primitive
5. **Interop tests**: Ensure Phase 0 + Phase 1 + Phase 2 work together

### 10.2 Performance Benchmarks

```text
Benchmark suite (all phases combined vs current):
  - entropy_extraction    (SHAKE256 vs SHAKE256 — unchanged)
  - key_derivation        (custom SHA3-512 vs HKDF-SHA512)
  - reseeding             (SHAKE256 vs BLAKE3)
  - encryption_1kb        (raw ChaCha20 vs ChaCha20Poly1305)
  - encryption_1mb        (raw ChaCha20 vs ChaCha20Poly1305)
  - full_pipeline_100k    (Complete encrypt with Standard security)
```

### 10.3 Security Audit Checklist

- [ ] ChaCha20Poly1305 nonce reuse protection (rekeying ensures fresh nonces)
- [ ] HKDF salt handling (using `None` salt — evaluate if a salt mechanism is needed)
- [ ] BLAKE3 context string domain separation (ensuring uniqueness)
- [ ] AEAD tag verification on ALL decryption paths
- [ ] No secret-dependent branches or timing variations
- [ ] Zeroization of temporary key material after use
- [ ] no_std compatibility maintained for `kelvin-core`

---

## Summary

```
Phase  Priority  Effort  Breaking  What
─────  ────────  ──────  ────────  ──────────────────────────────
0      HIGH      LOW     YES       ChaCha20Poly1305 AEAD
1      HIGH      LOW     YES       HKDF-SHA512 standard KDF
2      MEDIUM    LOW     YES       BLAKE3 fast reseeding
3      LOW       MOD     NO        RustCrypto traits (signature, aead, ed25519)
4      OPTIONAL  LOW     NO        Argon2 password path
```

Total new dependencies: **5–7 crates** (depending on phases chosen)  
Total files modified: **~8 source files + 4 Cargo.toml**  
Lines of code changed: **~200-400** (estimated)

The core innovation of Kelvin — n-body simulation as a KDF — remains untouched. RustCrypto integration replaces the cryptographic *plumbing* without changing the novel *engine*.
