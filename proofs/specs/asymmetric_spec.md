# Asymmetric Key Derivation Specification

## Overview

The asymmetric key derivation system generates post-quantum (ML-DSA) and
classical (Ed25519) signing key pairs from the orbital simulation state.
This enables the Kelvin cryptosystem to provide authenticated key exchange
and digital signatures.

## Key Derivation

### From Orbital Bodies

```
fn from_bodies(bodies: &[OrbitalBody], steps: u64, domain: &str) -> OrbitalKeyPair
```

1. Simulate the n-body system for `steps` iterations.
2. Extract entropy from the final orbital state via SHAKE256.
3. Use the extracted seed to deterministically generate:
   - ML-DSA-65 key pair (post-quantum)
   - Ed25519 key pair (classical)

### From OrbitalConfig

```
fn derive(config: &OrbitalConfig) -> Result<OrbitalKeyPair, AsymmetricError>
```

1. Validate the config.
2. Create initial bodies from config.
3. Call `from_bodies` with config parameters.

## Key Pair Structure

```
struct OrbitalKeyPair {
    ml_dsa: MlDsaKeyPair,    // ML-DSA-65 (FIPS 204)
    ed25519: Ed25519KeyPair, // Ed25519 (RFC 8032)
    seed: [u8; 32],          // Zeroized on drop
}
```

## Signing

### ML-DSA (Post-Quantum)

```
fn sign_ml_dsa(&self, msg: &[u8]) -> Vec<u8>
fn verify_ml_dsa(&self, msg: &[u8], signature: &[u8]) -> Result<(), signature::Error>
```

- Algorithm: ML-DSA-65 (FIPS 204, security category 3)
- Signature size: 3309 bytes
- Public key size: 1952 bytes
- Secret key size: 4032 bytes

### Ed25519 (Classical)

```
fn sign_ed25519(&self, msg: &[u8]) -> ed25519_dalek::Signature
fn verify_ed25519(&self, msg: &[u8], signature: &ed25519_dalek::Signature) -> Result<(), signature::Error>
```

- Algorithm: Ed25519 (RFC 8032)
- Signature size: 64 bytes
- Public key size: 32 bytes
- Secret key size: 32 bytes

## Properties

### Determinism

The same orbital state always produces the same key pair:
```
∀ bodies, steps, domain: from_bodies(bodies, steps, domain) = from_bodies(bodies, steps, domain)
```

### Config Sensitivity

Different configs produce different key pairs:
```
∀ config₁ ≠ config₂: derive(config₁) ≠ derive(config₂)
```

### Domain Separation

Different domain strings produce different key pairs:
```
∀ domain₁ ≠ domain₂: from_bodies(bodies, steps, domain₁) ≠ from_bodies(bodies, steps, domain₂)
```

## Security

- ML-DSA-65 provides security against quantum adversaries (category 3).
- Ed25519 provides compatibility with existing PKI infrastructure.
- The seed is zeroized on drop to prevent key recovery.
- Both key types are derived from the same orbital entropy, ensuring
  that possession of one key type does not reveal the other.

## References

- NIST FIPS 204 (2024). *Module-Lattice-Based Digital Signature Standard*.
- Bernstein, D. J., et al. (2012). "High-speed high-security signatures."
  *Journal of Cryptographic Engineering*, 2(2), 77–89.
