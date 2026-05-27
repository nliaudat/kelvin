//! Kelvin-Split — Dedicated XOR Key-Splitter for Homomorphic Encryption.
//!
//! ## Purpose
//!
//! `KelvinSplit` provides a domain-separated keystream generator designed
//! specifically for the **split-key XOR homomorphism**:
//!
//! ```text
//! split_key(len) → (A, B) where A ⊕ B = K
//! ```
//!
//! This enables XOR operations on encrypted data without revealing plaintexts:
//!
//! 1. Generate master key `K` and split into `(A, B)` where `A ⊕ B = K`
//! 2. Encrypt plaintexts: `E1 = P1 ⊕ A`, `E2 = P2 ⊕ B`
//! 3. On server: `E3 = E1 ⊕ E2 = K ⊕ P1 ⊕ P2`
//! 4. Decrypt with `K`: `E3 ⊕ K = P1 ⊕ P2`
//!
//! ## Architecture
//!
//! ```text
//! 2048B seed → HKDF-SHA512 → 64B XOF seed → SHAKE256 → unlimited OTP keys
//! ```
//!
//! Each reseed derives a fresh 2048-byte pool via BLAKE3 for forward secrecy.
//! The domain separators (`DOMSEP_SPLIT_KEYSTREAM_V1`, `DOMSEP_SPLIT_RESEED_V1`)
//! ensure cryptographic isolation from Prism, Flare, and V3 Photon.
//!
//! ## Use Cases
//!
//! 1. **Split-key XOR homomorphism**: Use `split_key()` to produce (A, B) where
//!    A ⊕ B = K, enabling XOR operations on encrypted data.
//! 2. **Master key generation**: Use `generate_master_key()` to produce the
//!    master key K directly.
//! 3. **Encryption/decryption**: Use `encrypt()`/`decrypt()` for domain-separated
//!    OTP encryption.
//!
//! ## Security
//!
//! - **Domain separation**: Split keys cannot collide with Prism, Flare, or
//!   V3 Photon keystream.
//! - **Forward secrecy**: BLAKE3 reseeding ensures past keys are not recoverable
//!   from future state.
//! - **Quantum-resistant**: SHAKE256 provides 256-bit classical / 128-bit
//!   quantum security.
//! - **No authentication**: XOR is malleable. Use with external MAC or in
//!   environments where malleability is acceptable.
//!
//! ## Example
//!
//! ```rust,ignore
//! use kelvin::{KelvinSplit, PHOTON_BASE_SEED_SIZE};
//!
//! let seed = [0u8; PHOTON_BASE_SEED_SIZE]; // From orbital simulation
//! let mut split = KelvinSplit::new(seed, 1000)?;
//!
//! // Split a key for XOR homomorphism
//! let (a, b) = split.split_key(256)?;
//! // a ⊕ b == original master key K
//!
//! // Generate a master key directly
//! let master_key = split.generate_master_key(256)?;
//!
//! // Encrypt/decrypt with domain-separated keystream
//! let mut data = b"Secret message".to_vec();
//! split.encrypt(&mut data)?;
//! split.decrypt(&mut data)?;
//! assert_eq!(&data, b"Secret message");
//! ```

use blake3::Hasher;
use hkdf::Hkdf;
use sha3::digest::{ExtendableOutput, XofReader};
use sha3::{Sha3_512, Shake256, Shake256Reader};
use zeroize::{Zeroize, Zeroizing};

use crate::error::KelvinError;
use crate::parameters::{
    DOMSEP_SPLIT_KEYSTREAM_V1, DOMSEP_SPLIT_RESEED_V1, KEYSTREAM_CHUNK_SIZE, PHOTON_BASE_SEED_SIZE,
    XOF_SEED_SIZE,
};

/// A pair of zeroizing OTP pads `(A, B)` where `A ⊕ B = K`.
type SplitPadPair = (Zeroizing<Vec<u8>>, Zeroizing<Vec<u8>>);

/// Bytes of keystream generated before triggering a reseed (64 MiB).
///
/// Within one reseed period, the SHAKE256 XOF reader is kept alive and
/// produces keystream continuously. Only after this many bytes do we
/// run HKDF + BLAKE3 to derive a fresh XOF seed and reader.
const SPLIT_RESEED_INTERVAL_BYTES: u64 = 64 * 1024 * 1024;

/// Dedicated XOR key-splitter for homomorphic encryption integration.
///
/// `KelvinSplit` wraps HKDF→SHAKE256 keystream generation with domain
/// separation for the split-key XOR homomorphism. It is designed to be
/// used alongside `KelvinPrism` and `KelvinFlare` without key collision.
///
/// ## Domain Separation
///
/// All keystream generation uses `DOMSEP_SPLIT_KEYSTREAM_V1` for HKDF expansion
/// and `DOMSEP_SPLIT_RESEED_V1` for BLAKE3 reseeding. This ensures Split keys
/// are cryptographically isolated from Prism, Flare, and V3 Photon keystream.
pub struct KelvinSplit {
    /// Current seed material (PHOTON_BASE_SEED_SIZE bytes).
    seed: [u8; PHOTON_BASE_SEED_SIZE],
    /// Current reseed counter.
    reseed_count: u64,
    /// Maximum reseeds before exhaustion.
    max_reseeds: u64,
    /// Total bytes processed (for tracking only, NOT used in keystream derivation).
    bytes_processed: u64,
    /// Persistent SHAKE256 XOF reader, kept alive across chunks.
    reader: Option<Shake256Reader>,
    /// Bytes generated since the last reseed.
    bytes_since_reseed: u64,
}

impl std::fmt::Debug for KelvinSplit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KelvinSplit")
            .field("seed", &"[redacted]")
            .field("reseed_count", &self.reseed_count)
            .field("max_reseeds", &self.max_reseeds)
            .field("bytes_processed", &self.bytes_processed)
            .field("reader", &self.reader.as_ref().map(|_| "Shake256Reader(active)"))
            .field("bytes_since_reseed", &self.bytes_since_reseed)
            .finish()
    }
}

impl KelvinSplit {
    /// Create a new `KelvinSplit` instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    ///
    /// ## Panics
    ///
    /// Panics if `seed` is not exactly `PHOTON_BASE_SEED_SIZE` bytes.
    pub fn new(seed: [u8; PHOTON_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        KelvinSplit {
            seed,
            reseed_count: 0,
            max_reseeds,
            bytes_processed: 0,
            reader: None,
            bytes_since_reseed: 0,
        }
    }

    /// Maximum chunk size for keystream generation (1 MB).
    const CHUNK_SIZE: usize = KEYSTREAM_CHUNK_SIZE;

    /// Ensure the SHAKE256 reader is initialized or re-created after
    /// `SPLIT_RESEED_INTERVAL_BYTES` of keystream.
    fn ensure_reader(&mut self) -> Result<(), KelvinError> {
        if self.reseed_count >= self.max_reseeds {
            return Err(KelvinError::SeedExhausted);
        }

        // HKDF-SHA512 expand: derive XOF seed from 2048-byte pool
        let hk = Hkdf::<Sha3_512>::new(None, &self.seed);
        let mut xof_seed = [0u8; XOF_SEED_SIZE];
        let mut info = [0u8; 33];
        info[..25].copy_from_slice(DOMSEP_SPLIT_KEYSTREAM_V1);
        info[25..].copy_from_slice(&self.reseed_count.to_le_bytes());

        hk.expand(&info, &mut xof_seed).map_err(|_| KelvinError::SeedExhausted)?;

        // SHAKE256 XOF: create a persistent reader
        let mut hasher = Shake256::default();
        sha3::digest::Update::update(&mut hasher, &xof_seed);
        sha3::digest::Update::update(&mut hasher, b"kelvin-split-xof-v1");

        self.reader = Some(hasher.finalize_xof());

        // Reseed: derive new seed via BLAKE3 for forward secrecy
        let mut reseed_hasher = Hasher::new();
        reseed_hasher.update(DOMSEP_SPLIT_RESEED_V1);
        reseed_hasher.update(&self.seed[..]);
        reseed_hasher.update(&self.reseed_count.to_le_bytes());
        let mut reseed_buf = [0u8; PHOTON_BASE_SEED_SIZE];
        reseed_hasher.finalize_xof().fill(&mut reseed_buf);
        self.seed = reseed_buf;
        reseed_buf.zeroize();

        self.reseed_count += 1;
        self.bytes_since_reseed = 0;
        xof_seed.zeroize();

        Ok(())
    }

    /// Generate keystream and write it directly into `output`.
    ///
    /// Uses the persistent SHAKE256 reader. If the reader is not yet initialized
    /// or has exhausted its reseed interval, `ensure_reader()` is called first.
    pub fn generate_keystream_into(&mut self, output: &mut [u8]) -> Result<(), KelvinError> {
        if output.is_empty() {
            return Ok(());
        }

        if self.reader.is_none() || self.bytes_since_reseed >= SPLIT_RESEED_INTERVAL_BYTES {
            self.ensure_reader()?;
        }

        if let Some(ref mut reader) = self.reader {
            XofReader::read(reader, output);
        }

        self.bytes_since_reseed += output.len() as u64;
        Ok(())
    }

    /// Generate a master key `K` of arbitrary length.
    ///
    /// The returned key is domain-separated from Prism, Flare, and V3 Photon
    /// keystream, preventing related-key attacks when all are used in the
    /// same system.
    pub fn generate_master_key(&mut self, len: usize) -> Result<Zeroizing<Vec<u8>>, KelvinError> {
        if len == 0 {
            return Ok(Zeroizing::new(Vec::new()));
        }

        let mut key = Zeroizing::new(vec![0u8; len]);
        let mut offset = 0;
        while offset < len {
            let remaining = len - offset;
            let chunk_size = std::cmp::min(remaining, Self::CHUNK_SIZE);
            self.generate_keystream_into(&mut key[offset..offset + chunk_size])?;
            offset += chunk_size;
        }
        self.bytes_processed += len as u64;
        Ok(key)
    }

    /// Split a master key into two pads `(A, B)` such that `A ⊕ B = K`.
    ///
    /// This enables the split-key XOR homomorphism:
    ///
    /// 1. Generate a master key `K` via `generate_master_key(len)`
    /// 2. Split into `(A, B)` where `A ⊕ B = K`
    /// 3. Encrypt plaintexts: `E1 = P1 ⊕ A`, `E2 = P2 ⊕ B`
    /// 4. On server: `E3 = E1 ⊕ E2 = K ⊕ P1 ⊕ P2`
    /// 5. Decrypt with `K`: `E3 ⊕ K = P1 ⊕ P2`
    ///
    /// The server can XOR the two ciphertexts without ever seeing the
    /// plaintexts or the master key `K`.
    ///
    pub fn split_key(&mut self, len: usize) -> Result<SplitPadPair, KelvinError> {
        if len == 0 {
            return Ok((Zeroizing::new(Vec::new()), Zeroizing::new(Vec::new())));
        }

        // Generate the master key K
        let k = self.generate_master_key(len)?;

        // Generate pad A (random), then B = A ⊕ K
        let mut a = Zeroizing::new(vec![0u8; len]);
        let mut offset = 0;
        while offset < len {
            let remaining = len - offset;
            let chunk_size = std::cmp::min(remaining, Self::CHUNK_SIZE);
            self.generate_keystream_into(&mut a[offset..offset + chunk_size])?;
            offset += chunk_size;
        }
        self.bytes_processed += len as u64;

        // B = A ⊕ K
        let mut b = a.clone();
        for (b_byte, k_byte) in b.iter_mut().zip(k.iter()) {
            *b_byte ^= k_byte;
        }

        Ok((a, b))
    }

    /// Encrypt data in-place using domain-separated OTP keystream.
    ///
    /// XOR is its own inverse, so encryption and decryption are the same operation.
    ///
    /// Processes data in 1 MB chunks to avoid allocating a full-size keystream
    /// buffer for the entire input.
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.is_empty() {
            return Ok(());
        }
        let buf_size = std::cmp::min(data.len(), Self::CHUNK_SIZE);
        let mut keystream = Zeroizing::new(vec![0u8; buf_size]);
        let mut offset = 0;
        while offset < data.len() {
            let remaining = data.len() - offset;
            let chunk_size = std::cmp::min(remaining, Self::CHUNK_SIZE);
            let chunk = &mut data[offset..offset + chunk_size];

            self.generate_keystream_into(&mut keystream[..chunk_size])?;
            for (d, k) in chunk.iter_mut().zip(keystream[..chunk_size].iter()) {
                *d ^= k;
            }

            offset += chunk_size;
        }
        self.bytes_processed += data.len() as u64;
        Ok(())
    }

    /// Decrypt data in-place (same as encrypt, XOR is its own inverse).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.encrypt(data)
    }

    /// Get the total bytes of key material generated.
    pub fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    /// Get the current reseed count.
    pub fn reseed_count(&self) -> u64 {
        self.reseed_count
    }

    /// Get the remaining reseeds before exhaustion.
    pub fn remaining_reseeds(&self) -> u64 {
        self.max_reseeds.saturating_sub(self.reseed_count)
    }
}

impl Drop for KelvinSplit {
    fn drop(&mut self) {
        self.seed.zeroize();
        self.reseed_count.zeroize();
        self.bytes_processed.zeroize();
        self.bytes_since_reseed.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
        [0u8; PHOTON_BASE_SEED_SIZE]
    }

    #[test]
    fn test_generate_master_key_single() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        let key = split.generate_master_key(32).unwrap();
        assert_eq!(key.len(), 32);
        assert!(key.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_generate_master_key_multiple() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        let key1 = split.generate_master_key(64).unwrap();
        let key2 = split.generate_master_key(64).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_master_key_zero_length() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        let key = split.generate_master_key(0).unwrap();
        assert!(key.is_empty());
    }

    #[test]
    fn test_split_key() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        let len = 256;
        let (a, b) = split.split_key(len).unwrap();
        assert_eq!(a.len(), len);
        assert_eq!(b.len(), len);

        let xor_result: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
        assert!(xor_result.iter().any(|&b| b != 0));
        assert_ne!(a, b);
    }

    #[test]
    fn test_split_key_property() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        let len = 64;
        let (a, b) = split.split_key(len).unwrap();

        let k_recovered: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();

        let mut split2 = KelvinSplit::new(test_seed(), 1000);
        let k_direct = split2.generate_master_key(len).unwrap();
        let _a_from_split2 = split2.generate_master_key(len).unwrap();

        assert_eq!(k_recovered, &**k_direct);
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let seed = test_seed();
        let mut enc = KelvinSplit::new(seed, 1000);
        let mut dec = KelvinSplit::new(seed, 1000);

        let original = b"Hello, Kelvin-Split!".to_vec();
        let mut data = original.clone();

        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original);

        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        let mut data = Vec::new();
        split.encrypt(&mut data).unwrap();
        assert!(data.is_empty());
        split.decrypt(&mut data).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let seed = test_seed();
        let mut enc = KelvinSplit::new(seed, 1000);
        let mut dec = KelvinSplit::new(seed, 1000);

        let original = vec![0xABu8; 5 * 1024 * 1024]; // 5 MB
        let mut data = original.clone();

        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original);

        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_domain_separation_from_prism() {
        use crate::KelvinPrism;

        let seed = test_seed();
        let mut split = KelvinSplit::new(seed, 1000);
        let mut prism = KelvinPrism::new(seed, 1000);

        let split_key = split.generate_master_key(64).unwrap();
        let prism_key = prism.generate_otp_key(64).unwrap();

        assert_ne!(&**split_key, &**prism_key);
    }

    #[test]
    fn test_domain_separation_from_photon() {
        use crate::KelvinPhoton;

        let seed = test_seed();
        let mut split = KelvinSplit::new(seed, 1000);
        let mut photon = KelvinPhoton::new(seed, 1000);

        let split_key = split.generate_master_key(64).unwrap();
        let mut photon_buf = vec![0u8; 64];
        photon.encrypt(&mut photon_buf).unwrap();

        assert_ne!(&**split_key, &photon_buf[..]);
    }

    #[test]
    fn test_bytes_processed() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        assert_eq!(split.bytes_processed(), 0);

        split.generate_master_key(100).unwrap();
        assert_eq!(split.bytes_processed(), 100);

        split.generate_master_key(200).unwrap();
        assert_eq!(split.bytes_processed(), 300);
    }

    #[test]
    fn test_reseed_count() {
        let mut split = KelvinSplit::new(test_seed(), 1000);
        assert_eq!(split.reseed_count(), 0);

        split.generate_master_key(128 * 1024 * 1024).unwrap();
        assert!(split.reseed_count() >= 1);
    }

    #[test]
    fn test_remaining_reseeds() {
        let mut split = KelvinSplit::new(test_seed(), 10);
        assert_eq!(split.remaining_reseeds(), 10);

        for _ in 0..10 {
            split.generate_master_key(64 * 1024 * 1024).unwrap();
        }
        assert_eq!(split.remaining_reseeds(), 0);

        let result = split.generate_master_key(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_keystream() {
        let seed = test_seed();
        let mut split1 = KelvinSplit::new(seed, 1000);
        let mut split2 = KelvinSplit::new(seed, 1000);

        let key1 = split1.generate_master_key(256).unwrap();
        let key2 = split2.generate_master_key(256).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_debug_redacts_seed() {
        let split = KelvinSplit::new(test_seed(), 1000);
        let debug_str = format!("{:?}", split);
        assert!(!debug_str.contains("0u8"));
        assert!(debug_str.contains("[redacted]"));
    }
}
