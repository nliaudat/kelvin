# Stream Cipher Specifications

## Overview

Kelvin supports two AEAD stream ciphers for the final encryption layer:
AES-256-GCM and ChaCha20-Poly1305. Both are NIST-recommended and provide
authenticated encryption with associated data (AEAD).

## AES-256-GCM

```
struct AesGcmStream {
    key: [u8; 32],         // AES-256 key
    nonce: [u8; 12],       // 96-bit nonce (NIST-compliant)
    position: u64,         // Current position in keystream
    max_bytes: u64,        // Maximum safe bytes (2^39 - 256)
    exhausted: bool,       // Whether the stream is exhausted
}
```

### Algorithm

AES-256-GCM uses:
- **AES-256**: 256-bit key, 14 rounds
- **GCM mode**: Galois/Counter Mode for authenticated encryption
- **Nonce**: 96-bit (12 bytes), randomly generated per key
- **Tag**: 128-bit (16 bytes) authentication tag
- **Max plaintext**: 2^39 - 256 bytes per key/nonce pair

### Operations

```
fn encrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error>
fn decrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error>
```

Both operations:
1. Check that the stream is not exhausted.
2. Encrypt/decrypt the buffer in-place.
3. Append/prepend the 16-byte authentication tag.
4. Update the position counter.

### Properties

- **Authenticated**: Tampering is detected via the MAC tag.
- **Deterministic**: Same key + nonce → same keystream.
- **Position-tracked**: Tracks bytes processed.
- **Exhaustion-safe**: Refuses to encrypt beyond max safe bytes.

### Feature: aes-ni

When the `aes-ni` feature is enabled, AES-GCM uses hardware-accelerated
AES-NI instructions for improved performance.

## ChaCha20-Poly1305

```
struct ChaChaStream {
    key: [u8; 32],         // ChaCha20 key
    nonce: [u8; 12],       // 96-bit nonce (IETF variant)
    position: u64,         // Current position in keystream
    max_bytes: u64,        // Maximum safe bytes (2^38)
    exhausted: bool,       // Whether the stream is exhausted
}
```

### Algorithm

ChaCha20-Poly1305 uses:
- **ChaCha20**: 256-bit key, 20 rounds, stream cipher
- **Poly1305**: Wegman-Carter MAC for authentication
- **Nonce**: 96-bit (12 bytes), IETF variant (RFC 8439)
- **Tag**: 128-bit (16 bytes) authentication tag
- **Max plaintext**: 2^38 bytes per key/nonce pair

### Operations

```
fn encrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error>
fn decrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error>
```

Both operations:
1. Check that the stream is not exhausted.
2. Encrypt/decrypt the buffer in-place.
3. Append/prepend the 16-byte authentication tag.
4. Update the position counter.

### Properties

- **Authenticated**: Tampering is detected via the MAC tag.
- **Deterministic**: Same key + nonce → same keystream.
- **Position-tracked**: Tracks bytes processed.
- **Exhaustion-safe**: Refuses to encrypt beyond max safe bytes.
- **Constant-time**: Resistant to timing side-channel attacks.

## Comparison

| Property | AES-256-GCM | ChaCha20-Poly1305 |
|----------|-------------|-------------------|
| Key size | 256 bits | 256 bits |
| Nonce size | 96 bits | 96 bits |
| Tag size | 128 bits | 128 bits |
| Rounds | 14 (AES-256) | 20 (ChaCha20) |
| Max plaintext | 2^39 - 256 bytes | 2^38 bytes |
| Hardware accel. | AES-NI (optional) | None needed |
| Software perf. | Fast with AES-NI | Fast on all CPUs |
| Side-channel | Table-based (varies) | Constant-time |

## References

- NIST FIPS 197 (2001). *Advanced Encryption Standard (AES)*.
- NIST SP 800-38D (2007). *Recommendation for Block Cipher Modes of
  Operation: Galois/Counter Mode (GCM) and GMAC*.
- Bernstein, D. J. (2008). "ChaCha, a variant of Salsa20."
  *Workshop Record of SASC 2008*.
- Nir, Y., & Langley, A. (2018). "ChaCha20-Poly1305 for Transport Layer
  Security (TLS)." *RFC 8439*.
