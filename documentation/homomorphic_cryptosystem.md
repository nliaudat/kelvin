# Can Kelvin Be Used as a Homomorphic Cryptosystem?

**Status:** Research Analysis  
**Date:** 2026-05-27  
**Author:** Kelvin Project

---

## Short Answer

**Yes, but not directly.** Kelvin's OTP core can be integrated into homomorphic encryption systems, but it doesn't provide homomorphic properties natively.

---

## 1. What Homomorphic Encryption Is

Homomorphic encryption (HE) allows computations on encrypted data without decryption:

```
Encrypt(a) ⊕ Encrypt(b) = Encrypt(a ⊕ b)   (additive homomorphic)
Encrypt(a) ⊗ Encrypt(b) = Encrypt(a ⊗ b)   (multiplicative homomorphic)
```

**Fully Homomorphic Encryption (FHE)** supports both operations, enabling things like:

- Cloud computing on encrypted medical records
- Private electronic voting
- Secure outsourced data analytics

---

## 2. Kelvin's Position: OTP is NOT Homomorphic

| Property | OTP (Kelvin's Core) | Homomorphic Encryption |
|----------|---------------------|------------------------|
| Operation | XOR (⊕) only | Addition AND multiplication |
| Key property | Same key for encryption/decryption | Public/private key or symmetric with evaluation key |
| Multiple operations | ❌ No (key changes each time) | ✅ Yes (unlimited on same ciphertext) |
| Perfect secrecy | ✅ Yes (information-theoretic) | ❌ No (computational security) |

**The fundamental limitation:** OTP keys can only be used once. After performing one XOR operation on ciphertexts, the result's security depends entirely on whether keys are reused.

---

## 3. How Kelvin CAN Be Used in HE Systems

### Strategy 1: OTP as a "Recryption" Layer

The `parasol_runtime` library explicitly implements OTP recryption:

```rust
// From parasol_runtime documentation
recrypt_one_time_pad(otp_key, ciphertext)
// Produces an FHE ciphertext containing XOR(plaintext, otp_key)
// The holder of the OTP key can then decrypt the final result
```

**How it works:**

1. You have an FHE ciphertext `Enc(m)`
2. You homomorphically XOR it with a one-time pad key `k` (requires FHE on the key)
3. Result is `Enc(m ⊕ k)` — still an FHE ciphertext
4. After FHE decryption, the owner of `k` gets `m ⊕ k`
5. Final XOR with `k` reveals `m`

**Kelvin's role:** Generate the one-time pad keys `k` using your orbital entropy source. This gives you information-theoretic security for the OTP layer while the FHE layer provides homomorphic computation.

#### Using `Prism mode` for Recryption

The `KelvinPrism` struct provides a dedicated API for generating OTP keys for
homomorphic encryption integration:

```rust
use kelvin::{KelvinPrism, PHOTON_BASE_SEED_SIZE};

// 1. Get a 2048-byte seed from orbital simulation
let (seed, _bodies) = kelvin::simulate_and_extract_seed(&config)?;

// 2. Create a Prism instance
let mut prism = KelvinPrism::new(seed, 100_000);

// 3. Generate an OTP key for FHE recryption
let otp_key = prism.generate_otp_key(32)?;

// 4. Pass to your FHE library's recryption API
// (e.g., parasol_runtime::recrypt_one_time_pad)
```

Key features of `KelvinPrism`:

- **Domain-separated keystream**: Prism keys are cryptographically isolated from
  normal V3 Photon keystream via `DOMSEP_PRISM_KEYSTREAM_V1` and
  `DOMSEP_PRISM_RESEED_V1`, preventing related-key attacks.
- **No FHE dependencies**: `KelvinPrism` is a standalone generator — it produces
  raw OTP key bytes that can be plugged into any FHE library.
- **Forward secrecy**: BLAKE3 reseeding ensures past keys are not recoverable
  from future state.
- **Quantum-resistant**: SHAKE256 provides 256-bit classical / 128-bit quantum
  security.
- **Static recryption**: The `KelvinPrism::recrypt(data, key)` static method
  provides a pure XOR operation matching the `parasol_runtime::recrypt_one_time_pad`
  concept.

### Strategy 2: Chaotic Key Generation for FHE (Kelvin-Flare)

> ⚠️ **Known Prior Art:** Chaotic key generation for FHE was previously proposed
> by Jawad (2025) [DUff-skg] using a Duffing oscillator (2-DOF) with RK4
> floating-point integration. Kelvin-Flare offers an alternative approach using
> 30-DOF n-body gravitational dynamics with fixed-point arithmetic and
> domain-separated extraction. See [Patent Review #3](patent_review_3.md) for
> full analysis.

A 2025 paper by Jawad proposes **DUff-skg**: generating FHE secret keys using chaotic Duffing equations.

| Aspect | DUff-skg (Jawad, 2025) | Kelvin-Flare |
|--------|------------------------|--------------|
| Chaos source | Duffing oscillator (2 DOF) | 5-body orbital simulation (30 DOF) |
| Key size | 2³²⁵ bits | 2¹⁹²⁰ bits |
| NIST tests | ✅ Passed | ✅ Passed (Kelvin's own tests) |
| Integration | BFV, CKKS, TFHE | Same standards |
| Forward secrecy | ❌ Not specified | ✅ BLAKE3 reseeding |
| Domain isolation | ❌ Single domain | ✅ Isolated from other Kelvin modes |

The paper's key equation:

```
sk = integer((x + y + 0.5) × 1000)
```

Where `(x, y)` come from modified Duffing equations.

**Kelvin's alternative:** The `KelvinFlare` struct uses 5-body orbital chaos (30 DOF) instead of the 2-DOF Duffing system, with fixed-point arithmetic and domain-separated SHAKE256 extraction.


#### Using `Flare mode` for FHE Key Generation

```rust
use kelvin::{KelvinFlare, FlareScheme, PHOTON_BASE_SEED_SIZE};

// 1. Get a 2048-byte seed from orbital simulation
let (seed, _bodies) = kelvin::simulate_and_extract_seed(&config)?;

// 2. Create a Flare instance
let mut flare = KelvinFlare::new(seed, 100_000);

// 3. Generate a raw secret key (256 bytes)
let secret_key = flare.generate_secret_key(256)?;

// 4. Generate a key formatted for a specific FHE scheme
let bfv_key = flare.generate_fhe_key(FlareScheme::Bfv, 256)?;
let ckks_key = flare.generate_fhe_key(FlareScheme::Ckks, 256)?;
let tfhe_key = flare.generate_fhe_key(FlareScheme::Tfhe, 256)?;
```

Key features of `KelvinFlare`:

- **Scheme-specific domain separation**: Keys for BFV, CKKS, and TFHE are
  cryptographically isolated via scheme-specific domain suffixes.
- **Higher-dimensional chaos**: 30 DOF vs 2 DOF (Duffing) provides richer
  entropy for FHE secret keys.
- **Forward secrecy**: BLAKE3 reseeding ensures past keys are not recoverable
  from future state.
- **Domain isolation**: Flare keys cannot collide with Split, Prism, or
  V3 Photon keystream.

### Strategy 3: Simple XOR-Based Partial Homomorphism (Kelvin-Split)

A forum post shows the simplest form: using OTP to enable XOR homomorphism:

> "Say you want to XOR two plaintexts together on a remote server without revealing the plaintexts:
>
> 1. Randomly split your OTP (K) into two pads (A, B) that XOR to your original pad, i.e. `A ⊕ B = K`.
> 2. Encrypt two plaintexts: `E1 = P1 ⊕ A`; `E2 = P2 ⊕ B`.
> 3. On server: `E3 = E1 ⊕ E2 = A ⊕ B ⊕ P1 ⊕ P2 = K ⊕ P1 ⊕ P2`.
> 4. Decrypt with K: `E3 ⊕ K = P1 ⊕ P2`."

#### Using `Split mode` for XOR Homomorphism

The `KelvinSplit` struct provides a dedicated API for the split-key XOR homomorphism:

```rust
use kelvin::{KelvinSplit, PHOTON_BASE_SEED_SIZE};

// 1. Get a 2048-byte seed from orbital simulation
let (seed, _bodies) = kelvin::simulate_and_extract_seed(&config)?;

// 2. Create a Split instance
let mut split = KelvinSplit::new(seed, 100_000);

// 3. Split a key for XOR homomorphism
let (a, b) = split.split_key(256)?;
// a ⊕ b == original master key K

// 4. Encrypt plaintexts with the split pads
let mut p1 = b"Secret message 1".to_vec();
let mut p2 = b"Secret message 2".to_vec();
split.encrypt(&mut p1)?; // Uses pad A
split.encrypt(&mut p2)?; // Uses pad B

// 5. On server: E3 = E1 ⊕ E2 = K ⊕ P1 ⊕ P2
// 6. Decrypt with K: E3 ⊕ K = P1 ⊕ P2
```

Key features of `KelvinSplit`:

- **Dedicated split-key API**: `split_key()` produces `(A, B)` where `A ⊕ B = K`.
- **Domain-separated keystream**: Split keys are cryptographically isolated from
  Prism, Flare, and V3 Photon via `DOMSEP_SPLIT_KEYSTREAM_V1` and
  `DOMSEP_SPLIT_RESEED_V1`.
- **Master key generation**: `generate_master_key()` produces the master key `K`
  directly.
- **Encryption/decryption**: `encrypt()`/`decrypt()` for domain-separated OTP
  encryption.

---

## 4. Recommended Architecture: Kelvin + FHE Library

Instead of making Kelvin homomorphic, integrate it with existing FHE libraries:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hybrid System Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐         ┌─────────────────┐               │
│  │   Kelvin        │         │   FHE Library   │               │
│  │   (OTP Layer)   │◄───────►│   (SEAL, Lattigo,│               │
│  │                 │         │    TFHE, etc.)  │               │
│  └─────────────────┘         └─────────────────┘               │
│           │                           │                          │
│           │ Generate OTP key           │ Homomorphic operations  │
│           │ (orbital entropy)          │ (addition/multiplication)│
│           ▼                           ▼                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  Use Case Specific                      │    │
│  ├─────────────────────────────────────────────────────────┤    │
│  │ • FHE uses Kelvin's keys for recryption layer           │    │
│  │ • Kelvin uses FHE for homomorphic properties            │    │
│  │ • Combined: information-theoretic + computational HE    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Implementation Example

```rust
// Hybrid: Kelvin OTP + FHE
use kelvin::{KelvinPrism, KelvinSplit, KelvinFlare, FlareScheme};
use fhe_library::{Ciphertext, Evaluator};

fn hybrid_encrypt_with_computation(
    prism: &mut KelvinPrism,
    split: &mut KelvinSplit,
    flare: &mut KelvinFlare,
    evaluator: &mut Evaluator,
    plaintexts: Vec<Vec<u8>>,
) -> Result<Vec<Ciphertext>> {
    let mut results = Vec::new();
    for plaintext in plaintexts {
        // Strategy 1: Generate OTP key for FHE recryption
        let otp_key = prism.generate_otp_key(plaintext.len())?;

        // Strategy 2: Encrypt with OTP locally
        let otp_ciphertext: Vec<u8> = plaintext.iter()
            .zip(otp_key.iter())
            .map(|(p, k)| p ^ k)
            .collect();

        // Strategy 3: Encrypt OTP ciphertext with FHE
        let fhe_ciphertext = evaluator.encrypt(&otp_ciphertext)?;

        results.push(fhe_ciphertext);
    }
    Ok(results)
    // Server can now perform homomorphic XOR on the FHE ciphertexts
    // without ever seeing plaintext or the OTP key!
}

// Generate FHE secret keys using Kelvin's orbital chaos
fn generate_fhe_keys(flare: &mut KelvinFlare) -> Result<()> {
    let bfv_sk = flare.generate_fhe_key(FlareScheme::Bfv, 64)?;
    let ckks_sk = flare.generate_fhe_key(FlareScheme::Ckks, 64)?;
    let tfhe_sk = flare.generate_fhe_key(FlareScheme::Tfhe, 64)?;
    Ok(())
}

// Split a key for XOR homomorphism
fn split_key_example(split: &mut KelvinSplit) -> Result<()> {
    let (a, b) = split.split_key(256)?;
    // a ⊕ b == original master key K
    Ok(())
}
```

---

## 5. Kelvin's Three HE Integration Modes

| Mode | Struct | Purpose | Domain Separators |
|------|--------|---------|-------------------|
| **Prism** | `KelvinPrism` | OTP key generation for FHE recryption | `DOMSEP_PRISM_KEYSTREAM_V1`, `DOMSEP_PRISM_RESEED_V1` |
| **Split** | `KelvinSplit` | XOR key-splitter for partial homomorphism | `DOMSEP_SPLIT_KEYSTREAM_V1`, `DOMSEP_SPLIT_RESEED_V1` |
| **Flare** | `KelvinFlare` | N-body gravitational FHE secret key generation | `DOMSEP_FLARE_KEYSTREAM_V1`, `DOMSEP_FLARE_RESEED_V1` |


All three modes are **domain-separated** from each other and from V3 Photon,
preventing related-key attacks when multiple modes are used in the same system.

---

## 6. Summary: Can Kelvin Be Used as Homomorphic?

| Use Case | Feasibility | Approach |
|----------|-------------|----------|
| Kelvin alone | ❌ No | OTP doesn't support multiplicative homomorphism |
| Kelvin + FHE recryption | ✅ Yes | Use `KelvinPrism` keys in recryption layer |
| Kelvin + chaotic FHE keygen | ✅ Yes | Use `KelvinFlare` as an alternative to Duffing-based keygen |

| Kelvin for XOR homomorphism | ✅ Yes | Use `KelvinSplit` split-key approach |

---

## 7. Bottom Line

Kelvin is **not** a homomorphic encryption system. But it can be a powerful component in hybrid systems:

> "Kelvin provides the information-theoretic OTP layer. FHE libraries provide homomorphic computation. Together, you get the best of both worlds: unconditional security for the OTP component plus full homomorphic capabilities for computation."

For a production implementation, integrate Kelvin with existing FHE frameworks like Microsoft SEAL, IBM HElib, or TFHE, using Kelvin's orbital chaos to generate keys for the recryption layer or as a high-entropy source for FHE key generation.

---

## 8. References

- Jawad (2025). "DUff-skg: FHE cryptographic systems with chaotic secret key generation." *Acta Scientiarum. Technology*, 47(1). [periodicos.uem.br](https://periodicos.uem.br)
- "A Noise-Free Homomorphic Encryption based on Chaotic System." *IEEE Xplore*, 2020.
- Shannon, C. E. (1949). "Communication Theory of Secrecy Systems." *Bell System Technical Journal*, 28(4), 656–715.
- Gentry, C. (2009). "Fully Homomorphic Encryption Using Ideal Lattices." *STOC 2009*.
- IBM. "HElib — An Implementation of homomorphic encryption." [github.com/homenc/HElib](https://github.com/homenc/HElib)
- Microsoft Research. "SEAL — Simple Encrypted Arithmetic Library." [github.com/microsoft/SEAL](https://github.com/microsoft/SEAL)
- Chillotti, I., et al. "TFHE: Fast Fully Homomorphic Encryption Library." [tfhe.github.io/tfhe](https://tfhe.github.io/tfhe)
