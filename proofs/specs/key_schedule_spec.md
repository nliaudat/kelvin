# Key Schedule Specification

## Overview

The key schedule derives a sequence of independent AES-256-GCM or
ChaCha20-Poly1305 keys from the orbital seed, using a two-layer
key derivation function (KDF) for domain separation and forward secrecy.

## Architecture

```
Orbital Seed (32 bytes from SHAKE256 extraction)
    ↓
HKDF-SHA256 (extract phase)
    ↓
PRK (32 bytes pseudorandom key)
    ↓
HKDF-SHA256 (expand phase, per-key)
    ↓
Key Material (32 bytes key + 12 bytes nonce)
    ↓
Blake3 (reseed, every N keys)
    ↓
Next PRK
```

## Layer 1: HKDF-SHA256

### Extract Phase

```
PRK = HMAC-SHA256(salt=domain_separator, ikm=orbital_seed)
```

Domain separator: `"KELVIN-KEY-SCHEDULE-v1"`

### Expand Phase (per key)

```
key_material_i = HMAC-SHA256(PRK, info_i || 0x01)
```

Where `info_i` encodes the key index:
```
info_i = "KELVIN-KEY-" || u64_to_le_bytes(i)
```

### Output

Each expansion produces 44 bytes:
- 32 bytes: AES-256 key (or ChaCha20 key)
- 12 bytes: nonce (96-bit, NIST-compliant)

## Layer 2: Blake3 Reseed

After `MAX_KEYS_PER_SEED` keys (default: 2^20), the schedule reseeds:

```
PRK_new = Blake3(PRK || "KELVIN-RESEED-v1" || counter)
```

This provides:
- **Forward secrecy**: Compromising the current PRK does not reveal past keys
- **Key independence**: Keys before and after reseed are cryptographically
  independent
- **Exhaustion prevention**: The schedule can generate up to 2^48 keys
  (2^20 × 2^28 reseeds)

## Properties

### Determinism

The same seed always produces the same key sequence:
```
∀ seed, i: next_key(seed, i) = next_key(seed, i)
```

### Key Independence

Knowledge of key `i` reveals nothing about key `j ≠ i`:
```
∀ i ≠ j: I(K_i; K_j) ≈ 0
```

### Forward Secrecy

Knowledge of PRK after reseed `r` reveals nothing about keys before reseed `r`:
```
∀ i < reseed_point: I(K_i; PRK_after_reseed) ≈ 0
```

### Exhaustion

The schedule tracks remaining bytes and returns `None` when exhausted:
```
remaining_bytes = max_bytes - keys_generated × 44
```

## Invariants

1. **Key uniqueness**: No two calls to `next_key` return the same
   (key, nonce) pair for the same seed.
2. **Nonce uniqueness**: Each nonce is unique within the key schedule's
   lifetime.
3. **Reset**: `reset(new_seed)` produces an independent key sequence.
4. **Step counter**: `step()` returns the number of keys generated.

## References

- Krawczyk, H. (2010). "Cryptographic extraction and key derivation:
  The HKDF scheme." *CRYPTO 2010*.
- NIST SP 800-56C Rev. 2 (2020). *Recommendation for Key-Derivation
  Methods in Key-Establishment Schemes*.
- Aumasson, J.-P., et al. (2013). "BLAKE2: simpler, smaller, fast as MD5."
  *ACNS 2013*.
