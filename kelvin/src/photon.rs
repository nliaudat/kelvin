//! V3 Kelvin-Photon — Fast bulk OTP via HKDF→SHAKE256 XOR.
//!
//! ## Architecture
//!
//! Unlike V1 (which extracts only 32+12 bytes per HKDF call), V3 uses HKDF's
//! full capacity to derive a SHAKE256 XOF seed, then produces arbitrary-length
//! keystream:
//!
//! ```text
//! 2048B seed → HKDF-SHA512 → 64B XOF seed → SHAKE256 → unlimited keystream
//! ```
//!
//! This solves the original bottleneck: HKDF-SHA512 can output up to 16,320
//! bytes per call, but V1 only used 44 bytes (0.27%). V3 uses 64 bytes to
//! seed SHAKE256, which then produces unlimited keystream.
//!
//! ## Security
//!
//! - **No authentication**: XOR is malleable. Use with external MAC or
//!   in environments where malleability is acceptable.
//! - **Deterministic reseeding**: BLAKE3 reseed provides forward secrecy.
//! - **Quantum-resistant**: SHAKE256 provides 256-bit classical / 128-bit
//!   quantum security.
//!
//! ## References
//!
//! - Krawczyk, H., & Eronen, P. (2010). "HMAC-based Extract-and-Expand Key
//!   Derivation Function (HKDF)." RFC 5869.
//! - NIST FIPS PUB 202 (2015). "SHA-3 Standard."

use blake3::Hasher;
use hkdf::Hkdf;
use sha3::digest::{ExtendableOutput, XofReader};
use sha3::{Sha3_512, Shake256, Shake256Reader};
use zeroize::{Zeroize, Zeroizing};

use crate::error::KelvinError;
use crate::parameters::{
    DOMSEP_PHOTON_KEYSTREAM_V1, DOMSEP_PHOTON_RESEED_V1, KEYSTREAM_CHUNK_SIZE,
    PHOTON_BASE_SEED_SIZE, XOF_SEED_SIZE,
};

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
const PHOTON_RESEED_INTERVAL_BYTES: u64 = 64 * 1024 * 1024;

/// V3 Kelvin-Photon: fast bulk OTP via HKDF→SHAKE256 XOR.
///
/// Produces arbitrary-length keystream from a 2048-byte seed.
/// Each reseed derives a fresh 2048-byte pool via BLAKE3 for forward secrecy.
///
/// ## Performance
///
/// Unlike V1 (which runs HKDF per 44-byte key), V3 keeps a persistent SHAKE256
/// XOF reader alive across chunks. The reader is only re-created every 64 MiB
/// (when `PHOTON_RESEED_INTERVAL_BYTES` is reached), at which point HKDF +
/// BLAKE3 derive a fresh XOF seed. This eliminates the per-chunk HKDF overhead
/// that was the original bottleneck.
///
/// ## Chunk independence
///
/// The keystream within one reseed period is a pure function of
/// (seed, reseed_count). SHAKE256's XOF property provides position-independent
/// output — you can take any prefix of the keystream and it will match the
/// same prefix from any other call with the same (seed, reseed_count).
/// This means encryption and decryption are chunk-independent: splitting
/// data into different chunk sizes produces identical results.
///
/// ## Example
///
/// ```rust,ignore
/// use kelvin::KelvinPhoton;
///
/// let seed = [0u8; PHOTON_BASE_SEED_SIZE]; // From orbital simulation
/// let mut photon = KelvinPhoton::new(seed, 1000)?;
/// let mut data = b"Secret message".to_vec();
/// photon.encrypt(&mut data)?;
/// photon.decrypt(&mut data)?;
/// assert_eq!(&data, b"Secret message");
/// ```
pub struct KelvinPhoton {
    /// Current seed material (PHOTON_BASE_SEED_SIZE bytes).
    seed: [u8; PHOTON_BASE_SEED_SIZE],
    /// Current reseed counter.
    reseed_count: u64,
    /// Maximum reseeds before exhaustion.
    max_reseeds: u64,
    /// Total bytes processed (for tracking only, NOT used in keystream derivation).
    bytes_processed: u64,
    /// Persistent SHAKE256 XOF reader, kept alive across chunks.
    /// Re-created every `PHOTON_RESEED_INTERVAL_BYTES` via HKDF + BLAKE3 reseed.
    reader: Option<Shake256Reader>,
    /// Bytes generated since the last reseed (triggers HKDF + BLAKE3 refresh).
    bytes_since_reseed: u64,
}

impl std::fmt::Debug for KelvinPhoton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KelvinPhoton")
            .field("seed", &"[redacted]")
            .field("reseed_count", &self.reseed_count)
            .field("max_reseeds", &self.max_reseeds)
            .field("bytes_processed", &self.bytes_processed)
            .field("reader", &self.reader.as_ref().map(|_| "Shake256Reader(active)"))
            .field("bytes_since_reseed", &self.bytes_since_reseed)
            .finish()
    }
}

impl KelvinPhoton {
    /// Create a new Kelvin-Photon instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream (each reseed produces ~16KB
    /// of HKDF output, which seeds unlimited SHAKE256 keystream).
    pub fn new(seed: [u8; PHOTON_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        KelvinPhoton {
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
    /// Processing data in chunks prevents OOM crashes when encrypting
    /// large (multi-GB) inputs by avoiding a full-size keystream allocation.
    const CHUNK_SIZE: usize = KEYSTREAM_CHUNK_SIZE;

    /// Ensure the SHAKE256 reader is initialized (first call) or re-created
    /// after `PHOTON_RESEED_INTERVAL_BYTES` of keystream have been produced.
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
        info.extend_from_slice(DOMSEP_PHOTON_KEYSTREAM_V1);
        info.extend_from_slice(&self.reseed_count.to_le_bytes());

        hk.expand(&info, &mut xof_seed).map_err(|_| KelvinError::SeedExhausted)?;

        // SHAKE256 XOF: create a persistent reader
        let mut hasher = Shake256::default();
        sha3::digest::Update::update(&mut hasher, &xof_seed);
        sha3::digest::Update::update(&mut hasher, b"kelvin-photon-xof-v1");

        self.reader = Some(hasher.finalize_xof());

        // Reseed: derive new seed via BLAKE3 for forward secrecy
        let mut reseed_hasher = Hasher::new();
        reseed_hasher.update(DOMSEP_PHOTON_RESEED_V1);
        reseed_hasher.update(&self.seed[..]);
        reseed_hasher.update(&self.reseed_count.to_le_bytes());
        let mut reseed_buf = [0u8; PHOTON_BASE_SEED_SIZE];
        reseed_hasher.finalize_xof().fill(&mut reseed_buf);
        self.seed = reseed_buf;

        self.reseed_count += 1;
        self.bytes_since_reseed = 0;
        xof_seed.zeroize();

        Ok(())
    }

    /// Generate keystream and write it directly into `output`.
    ///
    /// Uses the persistent SHAKE256 reader. If the reader is not yet initialized
    /// or has exhausted its reseed interval, `ensure_reader()` is called first.
    fn generate_keystream_into(&mut self, output: &mut [u8]) -> Result<(), KelvinError> {
        if output.is_empty() {
            return Ok(());
        }

        // Check if we need to (re)initialize the reader
        if self.reader.is_none() || self.bytes_since_reseed >= PHOTON_RESEED_INTERVAL_BYTES {
            self.ensure_reader()?;
        }

        // Read keystream from the persistent XOF reader
        if let Some(ref mut reader) = self.reader {
            XofReader::read(reader, output);
        }

        self.bytes_since_reseed += output.len() as u64;
        Ok(())
    }

    /// Encrypt data in-place using XOR with the keystream.
    ///
    /// XOR is its own inverse, so encryption and decryption are the same operation.
    ///
    /// Processes data in 1 MB chunks to avoid allocating a full-size keystream
    /// buffer for the entire input. A single reusable buffer of at most 1 MB
    /// is allocated once and reused across chunks, preventing OOM crashes when
    /// processing large (multi-GB) data. The buffer is zeroized after use.
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

    /// Get the total bytes processed.
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

impl Drop for KelvinPhoton {
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

    fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
        let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        seed
    }

    #[test]
    fn test_round_trip_small() {
        let mut photon = KelvinPhoton::new(test_seed(), 1000);
        let mut data = b"Hello, Kelvin V3 Photon!".to_vec();
        let original = data.clone();

        photon.encrypt(&mut data).unwrap();
        assert_ne!(data, original, "encrypted data should differ from plaintext");

        // Decrypt with new instance (same seed = same keystream)
        let mut photon2 = KelvinPhoton::new(test_seed(), 1000);
        photon2.decrypt(&mut data).unwrap();
        assert_eq!(data, original, "round-trip should restore original");
    }

    #[test]
    fn test_determinism() {
        let mut p1 = KelvinPhoton::new(test_seed(), 1000);
        let mut p2 = KelvinPhoton::new(test_seed(), 1000);

        let mut data1 = b"Determinism test".to_vec();
        let mut data2 = data1.clone();

        p1.encrypt(&mut data1).unwrap();
        p2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2, "two instances should produce identical ciphertext");
    }

    #[test]
    fn test_empty_data() {
        let mut photon = KelvinPhoton::new(test_seed(), 1000);
        let mut empty: Vec<u8> = vec![];
        photon.encrypt(&mut empty).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_large_data() {
        let mut photon = KelvinPhoton::new(test_seed(), 1000);
        let mut data = vec![0xABu8; 100_000]; // 100KB
        let original = data.clone();

        photon.encrypt(&mut data).unwrap();
        assert_ne!(data, original);

        let mut photon2 = KelvinPhoton::new(test_seed(), 1000);
        photon2.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_bytes_processed() {
        let mut photon = KelvinPhoton::new(test_seed(), 1000);
        assert_eq!(photon.bytes_processed(), 0);

        let mut data = vec![0u8; 100];
        photon.encrypt(&mut data).unwrap();
        assert_eq!(photon.bytes_processed(), 100);

        let mut data2 = vec![0u8; 50];
        photon.encrypt(&mut data2).unwrap();
        assert_eq!(photon.bytes_processed(), 150);
    }

    #[test]
    fn test_reseed_count_increments() {
        let mut photon = KelvinPhoton::new(test_seed(), 1000);
        assert_eq!(photon.reseed_count(), 0);

        // First call initializes the reader (reseed_count becomes 1)
        let mut data = vec![0u8; 1];
        photon.encrypt(&mut data).unwrap();
        assert_eq!(photon.reseed_count(), 1);

        // Second call uses the persistent reader (no reseed)
        photon.encrypt(&mut data).unwrap();
        assert_eq!(photon.reseed_count(), 1);
    }

    #[test]
    fn test_exhaustion() {
        let mut photon = KelvinPhoton::new(test_seed(), 3);
        // Encrypt enough to trigger 3 reseeds (each reseed lasts 64 MiB)
        // 3 reseeds = 192 MiB of keystream
        let mut data = vec![0u8; 64 * 1024 * 1024 + 1]; // 64 MiB + 1 byte

        // First 64 MiB + 1 byte: triggers reseed at 64 MiB boundary
        assert!(photon.encrypt(&mut data).is_ok());
        assert_eq!(photon.reseed_count(), 2); // initial + 1 reseed

        // Second 64 MiB + 1 byte: triggers another reseed
        assert!(photon.encrypt(&mut data).is_ok());
        assert_eq!(photon.reseed_count(), 3); // exhausted

        // Third call should fail (max_reseeds = 3, so reseed_count 3 = exhausted)
        assert!(photon.encrypt(&mut data).is_err());
    }

    #[test]
    fn test_remaining_reseeds() {
        let mut photon = KelvinPhoton::new(test_seed(), 10);
        assert_eq!(photon.remaining_reseeds(), 10);

        // First call initializes the reader (consumes 1 reseed)
        let mut data = vec![0u8; 1];
        photon.encrypt(&mut data).unwrap();
        assert_eq!(photon.remaining_reseeds(), 9);

        // Second call uses persistent reader (no reseed consumed)
        photon.encrypt(&mut data).unwrap();
        assert_eq!(photon.remaining_reseeds(), 9);
    }

    #[test]
    fn test_avalanche() {
        // 1-bit change in seed should produce completely different keystream
        let mut seed2 = test_seed();
        seed2[0] ^= 0x01;

        let mut p1 = KelvinPhoton::new(test_seed(), 1000);
        let mut p2 = KelvinPhoton::new(seed2, 1000);

        let mut data1 = b"Avalanche test data".to_vec();
        let mut data2 = data1.clone();

        p1.encrypt(&mut data1).unwrap();
        p2.encrypt(&mut data2).unwrap();

        let diff_bits: u32 =
            data1.iter().zip(data2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
        assert!(diff_bits > 50, "Too few differing bits: {}", diff_bits);
    }
}
