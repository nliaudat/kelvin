# Cipher Modes Specification

## Overview

Kelvin provides multiple cipher modes, each offering different security
properties and use cases. All modes derive their keystream from the
orbital chaos simulation via SHAKE256 extraction.

## Mode Comparison

| Mode | Auth | PQ-Safe | Key Type | Use Case |
|------|------|---------|----------|----------|
| **Photon** | No | No | Symmetric | High-speed streaming |
| **Quantum** | No | Yes | Symmetric | PQ-safe streaming |
| **Prism** | No | No | OTP | One-time pad |
| **Split** | No | No | Split-key | Secret sharing |
| **Flare** | No | No | FHE | Homomorphic encryption |
| **Authenticated** | Yes | Yes | AEAD | Authenticated encryption |
| **Streaming** | Yes | Yes | AEAD | File streaming |

## Photon Mode

```
struct KelvinPhoton {
    seed: [u8; 32],       // Base seed
    keystream: Vec<u8>,   // Cached keystream
    position: u64,        // Current position in keystream
    reseed_count: u64,    // Number of reseeds performed
    max_reseeds: u64,     // Maximum reseeds before exhaustion
}
```

- Uses ChaCha20 for keystream generation.
- No authentication tag.
- Deterministic: same seed → same keystream.
- Domain-separated from other modes.

## Quantum Mode

```
struct KelvinQuantum {
    seed: [u8; 64],       // Extended seed (PQ-safe)
    config: OrbitalConfig, // Orbital simulation config
    keystream: Vec<u8>,   // Cached keystream
    position: u64,        // Current position in keystream
    reseed_count: u64,    // Number of reseeds performed
    max_reseeds: u64,     // Maximum reseeds before exhaustion
}
```

- Uses orbital chaos simulation for reseeding.
- Post-quantum safe: reseeds from chaotic n-body dynamics.
- Deterministic: same seed → same keystream.
- Domain-separated from other modes.

## Prism Mode

```
struct KelvinPrism {
    seed: [u8; 32],       // Base seed
    keystream: Vec<u8>,   // Cached keystream
    position: u64,        // Current position in keystream
    reseed_count: u64,    // Number of reseeds performed
    max_reseeds: u64,     // Maximum reseeds before exhaustion
}
```

- Generates one-time pad (OTP) keys.
- `generate_otp_key(len)` → returns `len` bytes of keystream.
- `split_key(len)` → returns two pads `(pad_a, pad_b)` where `pad_a ⊕ pad_b = key`.
- `recrypt(data, key)` → applies XOR with key in-place.
- Domain-separated from other modes.

## Split Mode

```
struct KelvinSplit {
    seed: [u8; 32],       // Base seed
    keystream: Vec<u8>,   // Cached keystream
    position: u64,        // Current position in keystream
    reseed_count: u64,    // Number of reseeds performed
    max_reseeds: u64,     // Maximum reseeds before exhaustion
}
```

- Generates split keys for secret sharing.
- `generate_master_key(len)` → returns `len` bytes of master key.
- `split_key(len)` → returns two pads `(pad_a, pad_b)` where `pad_a ⊕ pad_b = master_key`.
- Domain-separated from Prism and Photon modes.

## Flare Mode

```
struct KelvinFlare {
    seed: [u8; 32],       // Base seed
    keystream: Vec<u8>,   // Cached keystream
    position: u64,        // Current position in keystream
    reseed_count: u64,    // Number of reseeds performed
    max_reseeds: u64,     // Maximum reseeds before exhaustion
}
```

- Generates keys for fully homomorphic encryption (FHE).
- `generate_secret_key(len)` → returns `len` bytes of secret key material.
- `generate_fhe_key(scheme, len)` → returns key for specified FHE scheme.
- Supported FHE schemes: BFV, CKKS, TFHE.
- Domain-separated from all other modes.

## Authenticated Modes

### PhotonAuthenticated

```
struct KelvinPhotonAuthenticated {
    seed: [u8; 32],       // Base seed
    mac_key: [u8; 32],    // Derived MAC key
    position: u64,        // Current position in keystream
    reseed_count: u64,    // Number of reseeds performed
    max_reseeds: u64,     // Maximum reseeds before exhaustion
}
```

- Encrypt-then-MAC: encrypt with ChaCha20, then MAC with derived key.
- Tag size: 32 bytes (SHA256).
- Domain-separated from unauthenticated modes.

### QuantumAuthenticated

```
struct KelvinQuantumAuthenticated {
    seed: [u8; 64],       // Extended seed (PQ-safe)
    mac_key: [u8; 32],    // Derived MAC key
    position: u64,        // Current position in keystream
    reseed_count: u64,    // Number of reseeds performed
    max_reseeds: u64,     // Maximum reseeds before exhaustion
}
```

- Post-quantum authenticated encryption.
- Encrypt-then-MAC with orbital reseeding.
- Domain-separated from other modes.

### StreamingAuthenticated

```
struct KelvinStreamingAuthenticated {
    config: OrbitalConfig,  // Orbital simulation config
    mac_key: [u8; 32],     // Derived MAC key from bodies
    position: u64,         // Current position in keystream
    bytes_per_step: u64,   // Bytes processed per simulation step
}
```

- Authenticated streaming for large files.
- MAC key derived from orbital body state.
- Each chunk is encrypted and MAC'd independently.

## Domain Separation

Each mode uses a unique domain suffix to ensure cryptographic isolation:

| Mode | Domain Suffix |
|------|---------------|
| Photon | `KELVIN-PHOTON-v1` |
| Quantum | `KELVIN-QUANTUM-v1` |
| Prism | `KELVIN-PRISM-v1` |
| Split | `KELVIN-SPLIT-v1` |
| Flare | `KELVIN-FLARE-v1` |
| PhotonAuthenticated | `KELVIN-PHOTON-AUTH-v1` |
| QuantumAuthenticated | `KELVIN-QUANTUM-AUTH-v1` |
| StreamingAuthenticated | `KELVIN-STREAMING-AUTH-v1` |

## Properties

1. **Domain isolation**: Different modes produce uncorrelated keystreams
   even with the same seed.
2. **Determinism**: Same seed + same mode → same keystream.
3. **Reseed exhaustion**: Each mode has a maximum number of reseeds
   before it is exhausted.
4. **Zeroize on drop**: All sensitive state is cleared on drop.

## References

- Bernstein, D. J. (2008). "ChaCha, a variant of Salsa20."
  *Workshop Record of SASC 2008*.
- NIST SP 800-38D (2007). *Recommendation for Block Cipher Modes of
  Operation: Galois/Counter Mode (GCM) and GMAC*.
