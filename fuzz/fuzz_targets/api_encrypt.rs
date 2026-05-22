//! Property-based fuzz test for top-level `kelvin` API entry points.
//!
//! Generates random seeds, data chunks, and configuration parameters,
//! then exercises the encrypt/decrypt round-trip on all lightweight
//! API types (those that don't require expensive orbital simulation setup).
//!
//! Tested APIs:
//! - `KelvinPhoton::new(seed, max_reseeds)` → encrypt/decrypt
//! - `KelvinQuantum::with_config(seed, max_reseeds, ...)` → encrypt/decrypt
//! - `KelvinPhotonAuthenticated::new(seed, max_reseeds)` → encrypt/decrypt
//! - `KelvinQuantumAuthenticated::with_config(seed, max_reseeds, ...)` → encrypt/decrypt
//!
//! Run with: `cargo test -p kelvin-fuzz --test api_encrypt`
//! Heavy run: `PROPTEST_CASES=100000 cargo test -p kelvin-fuzz --test api_encrypt`

use proptest::prelude::*;
use kelvin::{KelvinPhoton, KelvinQuantum, KelvinPhotonAuthenticated, KelvinQuantumAuthenticated};

/// Build a 2048-byte seed from a vector of bytes (padded/repeated as needed).
fn make_seed(bytes: Vec<u8>) -> [u8; 2048] {
    let mut seed = [0u8; 2048];
    if bytes.is_empty() {
        return seed;
    }
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = bytes[i % bytes.len()];
    }
    seed
}

proptest! {
    #[test]
    fn fuzz_api_encrypt(
        seed_bytes in proptest::collection::vec(0u8..=255u8, 1..=64),
        max_reseeds in 0u64..100u64,
        data_bytes in proptest::collection::vec(0u8..=255u8, 0..=4096),
    ) {
        let seed = make_seed(seed_bytes);

        // ── KelvinPhoton ──
        // new() panics if max_reseeds == 0, so skip that case
        if max_reseeds > 0 {
            let mut photon = KelvinPhoton::new(seed, max_reseeds);
            let mut buf = data_bytes.clone();
            let original = buf.clone();
            if photon.encrypt(&mut buf).is_ok() {
                // Round-trip: decrypt with a new instance (same seed = same keystream)
                let mut photon2 = KelvinPhoton::new(seed, max_reseeds);
                photon2.decrypt(&mut buf).expect("Decryption failed");
                assert_eq!(buf, original, "Round-trip data mismatch for KelvinPhoton");
            }
        }

        // ── KelvinQuantum ──
        // with_config returns Err if max_reseeds == 0
        if let Ok(mut quantum) = KelvinQuantum::with_config(seed, max_reseeds, 1024, 100, 1024) {
            let mut buf = data_bytes.clone();
            let original = buf.clone();
            if quantum.encrypt(&mut buf).is_ok() {
                // Round-trip: decrypt with a new instance (same seed = same keystream)
                let mut quantum2 = KelvinQuantum::with_config(seed, max_reseeds, 1024, 100, 1024)
                    .expect("Second instance creation should succeed");
                quantum2.decrypt(&mut buf).expect("Decryption failed");
                assert_eq!(buf, original, "Round-trip data mismatch for KelvinQuantum");
            }
        }

        // ── KelvinPhotonAuthenticated ──
        // new() panics if max_reseeds == 0
        if max_reseeds > 0 {
            let mut auth_photon = KelvinPhotonAuthenticated::new(seed, max_reseeds);
            let mut buf = data_bytes.clone();
            let original = buf.clone();
            if auth_photon.encrypt(&mut buf).is_ok() {
                // Authenticated decrypt verifies tag, then strips it
                // Use a new instance (same seed = same keystream + same MAC key)
                let mut auth_photon2 = KelvinPhotonAuthenticated::new(seed, max_reseeds);
                auth_photon2.decrypt(&mut buf).expect("Authenticated decryption failed");
                assert_eq!(buf, original, "Round-trip data mismatch for KelvinPhotonAuthenticated");
            }
        }

        // ── KelvinQuantumAuthenticated ──
        // with_config returns Err if max_reseeds == 0
        if let Ok(mut auth_quantum) = KelvinQuantumAuthenticated::with_config(
            seed, max_reseeds, 1024, 100, 1024,
        ) {
            let mut buf = data_bytes.clone();
            let original = buf.clone();
            if auth_quantum.encrypt(&mut buf).is_ok() {
                // Round-trip: decrypt with a new instance (same seed = same keystream + same MAC key)
                let mut auth_quantum2 = KelvinQuantumAuthenticated::with_config(
                    seed, max_reseeds, 1024, 100, 1024,
                ).expect("Second instance creation should succeed");
                auth_quantum2.decrypt(&mut buf).expect("Authenticated decryption failed");
                assert_eq!(buf, original, "Round-trip data mismatch for KelvinQuantumAuthenticated");
            }
        }
    }
}
