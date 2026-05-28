# SHAKE256 Entropy Extraction Specification

## Overview

The entropy extraction pipeline converts the chaotic orbital state into
cryptographic key material. It is the bridge between the n-body simulation
and the symmetric cipher.

## Pipeline

```
OrbitalState (bodies, step counter)
    ↓
feed_orbital_state() — serialize positions, velocities, masses, step
    ↓
SHAKE256 XOF — absorb serialized state
    ↓
extract_seed() — squeeze 32 bytes → domain-separated seed
    ↓
KeySchedule — HKDF-SHA256 → Blake3 → AES-256-GCM / ChaCha20-Poly1305 keys
```

## Stage 1: feed_orbital_state

Serializes the orbital state into a byte buffer for SHAKE256 absorption:

```
buffer = []
for each body i:
    buffer ← buffer || body[i].position.x.to_le_bytes()  // 16 bytes
    buffer ← buffer || body[i].position.y.to_le_bytes()  // 16 bytes
    buffer ← buffer || body[i].position.z.to_le_bytes()  // 16 bytes
    buffer ← buffer || body[i].velocity.x.to_le_bytes()  // 16 bytes
    buffer ← buffer || body[i].velocity.y.to_le_bytes()  // 16 bytes
    buffer ← buffer || body[i].velocity.z.to_le_bytes()  // 16 bytes
    buffer ← buffer || body[i].mass.to_le_bytes()        // 16 bytes
buffer ← buffer || step_counter.to_le_bytes()            // 8 bytes
```

Total: `N_bodies × 7 × 16 + 8` bytes.

## Stage 2: SHAKE256 Absorption

The serialized buffer is absorbed into SHAKE256 with domain separation:

```
shake256 = SHAKE256::new()
shake256.absorb(b"KELVIN-EXTRACT-v1")     // domain separator
shake256.absorb(buffer)                    // orbital state
```

## Stage 3: Seed Squeezing

The seed is squeezed from the SHAKE256 XOF:

```
seed = shake256.squeeze(32)                // 32-byte seed
```

## Stage 4: Extended Extraction

For key schedules requiring more entropy:

```
extended = shake256.squeeze(64)            // 64-byte extended seed
```

## Properties

### Determinism

The same orbital state always produces the same seed:
```
∀ state: extract_seed(state) = extract_seed(state)
```

### Avalanche Effect

A single-bit change in any input produces ~50% bit flips in the output:
```
∀ state, bit: popcount(extract_seed(state) ⊕ extract_seed(state ⊕ bit)) ≈ 128
```

### Domain Separation

Different domain strings produce uncorrelated outputs:
```
∀ state: extract_seed(state, "KELVIN-EXTRACT-v1")
       ≠ extract_seed(state, "KELVIN-EXTRACT-v2")
```

### One-Wayness

Given the seed, it is infeasible to recover the orbital state:
```
seed → state: computationally infeasible (SHAKE256 preimage resistance)
```

## Invariants

1. **Length**: `extract_seed` always returns exactly 32 bytes.
2. **No panic**: `extract_seed` never panics, even with empty body list.
3. **Step sensitivity**: Different step counts produce different seeds.
4. **Body sensitivity**: Different body configurations produce different seeds.

## References

- NIST FIPS 202 (2015). *SHA-3 Standard: Permutation-Based Hash and
  Extendable-Output Functions*.
- Bertoni, G., et al. (2011). "Keccak sponge function family main document."
  *SHA-3 submission, updated*.
