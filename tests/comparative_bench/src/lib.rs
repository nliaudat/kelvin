//! Shared utilities for comparative benchmarks.
//!
//! Provides:
//! - `test_buffer()` — fixed-size test data buffer filled with random bytes
//! - `dummy_seed_2048()` — a 2048-byte deterministic seed (useful for OTP modes)
//! - `aes_key()`, `aes_ctr_key()`, `ctr_nonce()` — deterministic keys/nonces for ring/dalek

use rand::Rng;
use rand::SeedableRng;

/// Size of the test buffer used in throughput benchmarks (1 MiB).
pub const BUFFER_SIZE: usize = 1 << 20; // 1 MiB

/// Generate a test buffer filled with deterministic pseudo-random bytes.
///
/// Using a deterministic seed ensures reproducible benchmarks.
pub fn test_buffer() -> Vec<u8> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut buf = vec![0u8; BUFFER_SIZE];
    rng.fill(&mut buf[..]);
    buf
}

/// Create a 2048-byte seed filled with deterministic pseudo-random bytes.
///
/// This simulates what would come from an orbital simulation, but avoids
/// the multi-second simulation overhead in throughput benchmarks.
pub fn dummy_seed_2048() -> [u8; 2048] {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut seed = [0u8; 2048];
    rng.fill(&mut seed[..]);
    seed
}

/// Create a 32-byte AES-256 key from deterministic random.
pub fn aes_key() -> [u8; 32] {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut key = [0u8; 32];
    rng.fill(&mut key[..]);
    key
}

/// Create a 32-byte X25519 secret key seed from deterministic random.
pub fn x25519_seed() -> [u8; 32] {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut seed = [0u8; 32];
    rng.fill(&mut seed[..]);
    seed
}

/// AES-256 key (32 bytes) for CTR mode.
pub fn aes_ctr_key() -> [u8; 32] {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut key = [0u8; 32];
    rng.fill(&mut key[..]);
    key
}

/// Nonce for AES-CTR (16 bytes).
pub fn ctr_nonce() -> [u8; 16] {
    let mut rng = rand::rngs::StdRng::seed_from_u64(7);
    let mut nonce = [0u8; 16];
    rng.fill(&mut nonce[..]);
    nonce
}
