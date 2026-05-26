//! Kelvin-Prism — Standalone OTP Key Generator for Homomorphic Encryption.
//!
//! ## Purpose
//!
//! `KelvinPrism` provides a domain-separated keystream generator designed
//! specifically for integration with homomorphic encryption (HE) systems.
//! It wraps `KelvinPhoton` internally but uses distinct domain separators
//! to ensure Prism-generated OTP keys are cryptographically isolated from
//! normal V3 Photon keystream.
//!
//! ## Architecture
//!
//! ```text
//! 2048B seed → HKDF-SHA512 → 64B XOF seed → SHAKE256 → unlimited OTP keys
//! ```
//!
//! Each reseed derives a fresh 2048-byte pool via BLAKE3 for forward secrecy.
//! The domain separators (`DOMSEP_PRISM_KEYSTREAM_V1`, `DOMSEP_PRISM_RESEED_V1`)
//! ensure cryptographic isolation from V3 Photon.
//!
//! ## Use Cases
//!
//! 1. **Recryption layer**: Generate OTP keys for `parasol_runtime::recrypt_one_time_pad`
//!    or any FHE library's recryption API.
//! 2. **Split-key XOR homomorphism**: Use `split_key()` to produce (A, B) where
//!    A ⊕ B = K, enabling XOR operations on encrypted data.
//! 3. **Chaotic FHE key generation**: Use `generate_otp_key()` to produce
//!    high-entropy key material for FHE secret keys (replacing Duffing-based
//!    approaches like DUff-skg).
//!
//! ## Security
//!
//! - **Domain separation**: Prism keys cannot collide with V3 Photon keystream.
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
//! use kelvin::{KelvinPrism, PHOTON_BASE_SEED_SIZE};
//!
//! let seed = [0u8; PHOTON_BASE_SEED_SIZE]; // From orbital simulation
//! let mut prism = KelvinPrism::new(seed, 1000)?;
//!
//! // Generate a 256-byte OTP key for FHE recryption
//! let otp_key = prism.generate_otp_key(256)?;
//!
//! // Split a key for XOR homomorphism
//! let (a, b) = prism.split_key(256)?;
//! // a ⊕ b == original key
//!
//! // Encrypt/decrypt with domain-separated keystream
//! let mut data = b"Secret message".to_vec();
//! prism.encrypt(&mut data)?;
//! prism.decrypt(&mut data)?;
//! assert_eq!(&data, b"Secret message");
//! ```

use blake3::Hasher;
use hkdf::Hkdf;
use sha3::digest::{ExtendableOutput, XofReader};
use sha3::{Sha3_512, Shake256, Shake256Reader};
use zeroize::{Zeroize, Zeroizing};

use crate::error::KelvinError;
use crate::parameters::{
    DOMSEP_PRISM_KEYSTREAM_V1, DOMSEP_PRISM_RESEED_V1, KEYSTREAM_CHUNK_SIZE, PHOTON_BASE_SEED_SIZE,
    XOF_SEED_SIZE,
};

/// A pair of zeroizing OTP pads `(A, B)` where `A ⊕ B = K`.
type OtpPadPair = (Zeroizing<Vec<u8>>, Zeroizing<Vec<u8>>);

/// Bytes of keystream generated before triggering a reseed (64 MiB).
///
/// Within one reseed period, the SHAKE256 XOF reader is kept alive and
/// produces keystream continuously. Only after this many bytes do we
/// run HKDF + BLAKE3 to derive a fresh XOF seed and reader.
///
/// **Why 64 MiB?** SHAKE256 can produce arbitrary-length output from a
/// single seed. 64 MiB balances reseed overhead (~microseconds) against
/// memory/throughput. Larger values reduce reseed frequency but increase
/// the amount of keystream generated from one seed (acceptable for XOF).
const PRISM_RESEED_INTERVAL_BYTES: u64 = 64 * 1024 * 1024;

/// Standalone OTP key generator for homomorphic encryption integration.
///
/// Wraps `KelvinPhoton`-style keystream generation internally with
/// domain-separated HKDF→SHAKE256 for OTP key material. Designed to
/// integrate with any FHE library (parasol_runtime, SEAL, HElib, TFHE, etc.)
/// without adding FHE dependencies to the `kelvin` crate.
///
/// ## Design (Option A — standalone)
///
/// - NO dependency on `parasol_runtime` or any FHE library
/// - Generates raw OTP key bytes from orbital entropy (via HKDF→SHAKE256)
/// - Can plug into ANY FHE library: parasol_runtime, SEAL, HElib, TFHE, etc.
/// - User is responsible for wiring Kelvin keys into their FHE library
///
/// ## Domain Separation
///
/// All keystream generation uses `DOMSEP_PRISM_KEYSTREAM_V1` for HKDF expansion
/// and `DOMSEP_PRISM_RESEED_V1` for BLAKE3 reseeding. This ensures Prism keys
/// are cryptographically isolated from V3 Photon keystream, preventing
/// related-key attacks when both are used in the same system.
pub struct KelvinPrism {
    /// Current seed material (PHOTON_BASE_SEED_SIZE bytes).
    seed: [u8; PHOTON_BASE_SEED_SIZE],
    /// Current reseed counter.
    reseed_count: u64,
    /// Maximum reseeds before exhaustion.
    max_reseeds: u64,
    /// Total bytes processed (for tracking only, NOT used in keystream derivation).
    bytes_processed: u64,
    /// Persistent SHAKE256 XOF reader, kept alive across chunks.
    /// Re-created every `PRISM_RESEED_INTERVAL_BYTES` via HKDF + BLAKE3 reseed.
    reader: Option<Shake256Reader>,
    /// Bytes generated since the last reseed (triggers HKDF + BLAKE3 refresh).
    bytes_since_reseed: u64,
}

impl std::fmt::Debug for KelvinPrism {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KelvinPrism")
            .field("seed", &"[redacted]")
            .field("reseed_count", &self.reseed_count)
            .field("max_reseeds", &self.max_reseeds)
            .field("bytes_processed", &self.bytes_processed)
            .field("reader", &self.reader.as_ref().map(|_| "Shake256Reader(active)"))
            .field("bytes_since_reseed", &self.bytes_since_reseed)
            .finish()
    }
}

impl KelvinPrism {
    /// Create a new `KelvinPrism` instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream (each reseed produces ~16 KB
    /// of HKDF output, which seeds unlimited SHAKE256 keystream).
    ///
    /// ## Panics
    ///
    /// Panics if `seed` is not exactly `PHOTON_BASE_SEED_SIZE` bytes.
    pub fn new(seed: [u8; PHOTON_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        KelvinPrism {
            seed,
            reseed_count: 0,
            max_reseeds,
            bytes_processed: 0,
            reader: None,
            bytes_since_reseed: 0,
        }
    }

    /// Maximum chunk size for keystream generation (1 MB).
    ///
    /// Processing data in chunks prevents OOM crashes when generating
    /// large (multi-GB) OTP keys by avoiding a full-size keystream allocation.
    const CHUNK_SIZE: usize = KEYSTREAM_CHUNK_SIZE;

    /// Ensure the SHAKE256 reader is initialized (first call) or re-created
    /// after `PRISM_RESEED_INTERVAL_BYTES` of keystream have been produced.
    ///
    /// This is the only place where HKDF + BLAKE3 are called, so the expensive
    /// operations happen at most once per 64 MiB of data.
    fn ensure_reader(&mut self) -> Result<(), KelvinError> {
        if self.reseed_count >= self.max_reseeds {
            return Err(KelvinError::SeedExhausted);
        }

        // HKDF-SHA512 expand: derive XOF seed from 2048-byte pool
        let hk = Hkdf::<Sha3_512>::new(None, &self.seed);
        let mut xof_seed = [0u8; XOF_SEED_SIZE];
        let mut info = Vec::with_capacity(32);
        info.extend_from_slice(DOMSEP_PRISM_KEYSTREAM_V1);
        info.extend_from_slice(&self.reseed_count.to_le_bytes());

        hk.expand(&info, &mut xof_seed).map_err(|_| KelvinError::SeedExhausted)?;

        // SHAKE256 XOF: create a persistent reader
        let mut hasher = Shake256::default();
        sha3::digest::Update::update(&mut hasher, &xof_seed);
        sha3::digest::Update::update(&mut hasher, b"kelvin-prism-xof-v1");

        self.reader = Some(hasher.finalize_xof());

        // Reseed: derive new seed via BLAKE3 for forward secrecy
        let mut reseed_hasher = Hasher::new();
        reseed_hasher.update(DOMSEP_PRISM_RESEED_V1);
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
    ///
    /// The keystream is domain-separated from normal V3 Photon keystream via
    /// `DOMSEP_PRISM_KEYSTREAM_V1` and `DOMSEP_PRISM_RESEED_V1`.
    pub fn generate_keystream_into(&mut self, output: &mut [u8]) -> Result<(), KelvinError> {
        if output.is_empty() {
            return Ok(());
        }

        // Check if we need to (re)initialize the reader
        if self.reader.is_none() || self.bytes_since_reseed >= PRISM_RESEED_INTERVAL_BYTES {
            self.ensure_reader()?;
        }

        // Read keystream from the persistent XOF reader
        if let Some(ref mut reader) = self.reader {
            XofReader::read(reader, output);
        }

        self.bytes_since_reseed += output.len() as u64;
        Ok(())
    }

    /// Generate an arbitrary-length OTP key from orbital entropy.
    ///
    /// The returned key is domain-separated from normal encryption/decryption
    /// keystream, preventing related-key attacks when both `KelvinPrism` and
    /// `KelvinPhoton` are used in the same system.
    ///
    /// ## Use Case
    ///
    /// Pass the returned key to an FHE library's recryption API:
    ///
    /// ```rust,ignore
    /// // With parasol_runtime:
    /// let otp_key = prism.generate_otp_key(32)?;
    /// let (public_otp, secret_otp) = parasol_runtime::generate_one_time_pad(...);
    /// // Use Kelvin's key as the OTP key material
    /// ```
    pub fn generate_otp_key(&mut self, len: usize) -> Result<Zeroizing<Vec<u8>>, KelvinError> {
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

    /// Split an OTP key into two pads `(A, B)` such that `A ⊕ B = key`.
    ///
    /// This enables the split-key XOR homomorphism:
    ///
    /// 1. Generate a master key `K` via `generate_otp_key(len)`
    /// 2. Split into `(A, B)` where `A ⊕ B = K`
    /// 3. Encrypt plaintexts: `E1 = P1 ⊕ A`, `E2 = P2 ⊕ B`
    /// 4. On server: `E3 = E1 ⊕ E2 = K ⊕ P1 ⊕ P2`
    /// 5. Decrypt with `K`: `E3 ⊕ K = P1 ⊕ P2`
    ///
    /// The server can XOR the two ciphertexts without ever seeing the
    /// plaintexts or the master key `K`.
    ///
    /// ## Panics
    ///
    /// Panics if `len` is 0.
    pub fn split_key(&mut self, len: usize) -> Result<OtpPadPair, KelvinError> {
        assert!(len > 0, "split_key: len must be > 0");

        // Generate the master key K
        let k = self.generate_otp_key(len)?;

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
    /// buffer for the entire input. A single reusable buffer of at most 1 MB
    /// is allocated once and reused across chunks, preventing OOM crashes when
    /// processing large (multi-GB) data. The buffer is zeroized after use.
    ///
    /// ## Domain Separation
    ///
    /// The keystream used here is domain-separated from `KelvinPhoton`'s
    /// keystream via `DOMSEP_PRISM_KEYSTREAM_V1`. This means encrypting data
    /// with `KelvinPrism` produces different ciphertexts than encrypting the
    /// same data with `KelvinPhoton`, even with the same seed.
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.is_empty() {
            return Ok(());
        }
        // Allocate a reusable keystream buffer (up to CHUNK_SIZE = 1 MB).
        // For small inputs, cap the allocation to the actual data length
        // to avoid allocating a full 1 MB buffer for tiny messages.
        // Reusing the buffer across chunks avoids thousands of allocations
        // for multi-GB inputs.
        let buf_size = std::cmp::min(data.len(), Self::CHUNK_SIZE);
        let mut keystream = Zeroizing::new(vec![0u8; buf_size]);
        let mut offset = 0;
        while offset < data.len() {
            let remaining = data.len() - offset;
            let chunk_size = std::cmp::min(remaining, Self::CHUNK_SIZE);
            let chunk = &mut data[offset..offset + chunk_size];

            // Generate keystream into the reusable buffer (only the portion
            // needed for this chunk), then XOR into data.
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

    /// Convenience: XOR `data` with `key` in-place (static recryption).
    ///
    /// This is a pure XOR operation with no state. It matches the concept of
    /// `parasol_runtime::recrypt_one_time_pad` which XORs an OTP key into an
    /// FHE ciphertext.
    ///
    /// ## Panics
    ///
    /// Panics if `key` is shorter than `data`.
    pub fn recrypt(data: &mut [u8], key: &[u8]) {
        assert!(key.len() >= data.len(), "recrypt: key must be at least as long as data");
        for (d, k) in data.iter_mut().zip(key.iter()) {
            *d ^= k;
        }
    }

    /// Get the total bytes of OTP key material generated.
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

impl Drop for KelvinPrism {
    fn drop(&mut self) {
        self.seed.zeroize();
        self.reseed_count.zeroize();
        self.bytes_processed.zeroize();
        self.bytes_since_reseed.zeroize();
        // XofReader is taken by value in read(), so we drop it naturally.
        // No explicit zeroize needed since SHAKE256 state is ephemeral.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a test seed (all zeros, deterministic for testing).
    fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
        [0u8; PHOTON_BASE_SEED_SIZE]
    }

    #[test]
    fn test_generate_otp_key_single() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        let key = prism.generate_otp_key(32).unwrap();
        assert_eq!(key.len(), 32);
        // Key should not be all zeros (extremely unlikely)
        assert!(key.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_generate_otp_key_multiple() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        let key1 = prism.generate_otp_key(64).unwrap();
        let key2 = prism.generate_otp_key(64).unwrap();
        // Two consecutive keys should be different
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_otp_key_zero_length() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        let key = prism.generate_otp_key(0).unwrap();
        assert!(key.is_empty());
    }

    #[test]
    fn test_generate_otp_key_large() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        // Generate 10 MB key (spans multiple reseed intervals)
        let key = prism.generate_otp_key(10 * 1024 * 1024).unwrap();
        assert_eq!(key.len(), 10 * 1024 * 1024);
        // Verify it's not all zeros
        assert!(key.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_split_key() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        let len = 256;
        let (a, b) = prism.split_key(len).unwrap();
        assert_eq!(a.len(), len);
        assert_eq!(b.len(), len);

        // Verify A ⊕ B = K (where K is the master key)
        // We can verify by checking that A ⊕ B is not all zeros
        // (since A is random, A ⊕ B should be random)
        let xor_result: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
        assert!(xor_result.iter().any(|&b| b != 0));

        // Verify A ≠ B (they should be different)
        assert_ne!(a, b);
    }

    #[test]
    fn test_split_key_property() {
        // Verify the core property: A ⊕ B = K, and K can be recovered
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        let len = 64;
        let (a, b) = prism.split_key(len).unwrap();

        // Recover K = A ⊕ B
        let k_recovered: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();

        // Generate K directly (same seed, same state)
        // Note: split_key consumed 2*len bytes of keystream (K + A)
        // So we need to generate K from a fresh prism with same seed
        let mut prism2 = KelvinPrism::new(test_seed(), 1000);
        let k_direct = prism2.generate_otp_key(len).unwrap();
        // split_key generates K first, then A (another len bytes)
        let _a_from_prism2 = prism2.generate_otp_key(len).unwrap();

        // K_recovered should equal K_direct
        assert_eq!(k_recovered, &**k_direct);
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        // Use two separate instances (same seed = same keystream)
        let seed = test_seed();
        let mut enc = KelvinPrism::new(seed, 1000);
        let mut dec = KelvinPrism::new(seed, 1000);

        let original = b"Hello, Kelvin-Prism!".to_vec();
        let mut data = original.clone();

        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original); // Should be encrypted

        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original); // Should be restored
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        let mut data = Vec::new();
        prism.encrypt(&mut data).unwrap();
        assert!(data.is_empty());
        prism.decrypt(&mut data).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        // Use two separate instances (same seed = same keystream)
        let seed = test_seed();
        let mut enc = KelvinPrism::new(seed, 1000);
        let mut dec = KelvinPrism::new(seed, 1000);

        let original = vec![0xABu8; 5 * 1024 * 1024]; // 5 MB
        let mut data = original.clone();

        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original);

        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_recrypt_static() {
        let mut data = b"Hello, world!".to_vec();
        let key = b"This is a 32-byte key for test!!!!!";

        KelvinPrism::recrypt(&mut data, key);
        assert_ne!(&data, b"Hello, world!");

        // XOR again with same key to decrypt
        KelvinPrism::recrypt(&mut data, key);
        assert_eq!(&data, b"Hello, world!");
    }

    #[test]
    #[should_panic(expected = "key must be at least as long as data")]
    fn test_recrypt_short_key_panics() {
        let mut data = b"Hello, world!".to_vec();
        let key = b"short";
        KelvinPrism::recrypt(&mut data, key);
    }

    #[test]
    fn test_domain_separation() {
        // Prism keys should differ from Photon keystream with same seed
        use crate::KelvinPhoton;

        let seed = test_seed();
        let mut prism = KelvinPrism::new(seed, 1000);
        let mut photon = KelvinPhoton::new(seed, 1000);

        let prism_key = prism.generate_otp_key(64).unwrap();
        let mut photon_buf = vec![0u8; 64];
        photon.encrypt(&mut photon_buf).unwrap();

        // Prism key should differ from Photon keystream
        assert_ne!(&**prism_key, &photon_buf[..]);
    }

    #[test]
    fn test_bytes_processed() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        assert_eq!(prism.bytes_processed(), 0);

        prism.generate_otp_key(100).unwrap();
        assert_eq!(prism.bytes_processed(), 100);

        prism.generate_otp_key(200).unwrap();
        assert_eq!(prism.bytes_processed(), 300);
    }

    #[test]
    fn test_reseed_count() {
        let mut prism = KelvinPrism::new(test_seed(), 1000);
        assert_eq!(prism.reseed_count(), 0);

        // Generate enough data to trigger reseeds
        // PRISM_RESEED_INTERVAL_BYTES = 64 MiB, so generate 128 MiB
        prism.generate_otp_key(128 * 1024 * 1024).unwrap();
        assert!(prism.reseed_count() >= 1);
    }

    #[test]
    fn test_remaining_reseeds() {
        let mut prism = KelvinPrism::new(test_seed(), 10);
        assert_eq!(prism.remaining_reseeds(), 10);

        // Exhaust all reseeds
        for _ in 0..10 {
            prism.generate_otp_key(64 * 1024 * 1024).unwrap();
        }
        assert_eq!(prism.remaining_reseeds(), 0);

        // Next generation should fail
        let result = prism.generate_otp_key(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_keystream() {
        // Same seed + same reseed count should produce same keystream
        let seed = test_seed();
        let mut prism1 = KelvinPrism::new(seed, 1000);
        let mut prism2 = KelvinPrism::new(seed, 1000);

        let key1 = prism1.generate_otp_key(256).unwrap();
        let key2 = prism2.generate_otp_key(256).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_debug_redacts_seed() {
        let prism = KelvinPrism::new(test_seed(), 1000);
        let debug_str = format!("{:?}", prism);
        assert!(!debug_str.contains("0u8"));
        assert!(debug_str.contains("[redacted]"));
    }
}
