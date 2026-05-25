//! Streaming API integration tests for the Kelvin cryptosystem.
//!
//! Tests the `StreamEncrypt`/`StreamDecrypt` traits across all 4 modes:
//! - Photon (V3) — HKDF→SHAKE256 XOR
//! - Quantum (H) — hybrid cache+XOR + orbital reseed
//! - Chaos (V2) — per-step SHAKE256 XOR
//! - Secure (V1) — ChaCha20 + BLAKE3 authentication
//!
//! These are integration tests (in `kelvin/tests/`) rather than unit tests
//! (in `kelvin/src/streaming.rs`) because they exercise the full public API
//! and use temp files for I/O tests.

use std::io::{Read, Write};

use kelvin::streaming::{
    decrypt_file_streaming, encrypt_file_streaming, ChaosDecryptor, ChaosEncryptor,
    PhotonDecryptor, PhotonEncryptor, QuantumDecryptor, QuantumEncryptor, SecureDecryptor,
    SecureEncryptor, StreamDecrypt, StreamEncrypt,
};
use kelvin::{
    Fixed, KelvinPhoton, KelvinQuantum, KelvinStreaming, OrbitalBody, OrbitalConfig, Vec3,
    PHOTON_BASE_SEED_SIZE, QUANTUM_BASE_SEED_SIZE,
};

// ============================================================================
// Helpers
// ============================================================================

fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
    let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

fn test_quantum_seed() -> [u8; QUANTUM_BASE_SEED_SIZE] {
    let mut seed = [0u8; QUANTUM_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

fn streaming_config() -> OrbitalConfig {
    let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
    let planet1 = OrbitalBody::new(
        Fixed::from_raw(1 << 54),
        Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
        Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
    );
    let planet2 = OrbitalBody::new(
        Fixed::from_raw(1 << 53),
        Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
        Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
    );
    let planet3 = OrbitalBody::new(
        Fixed::from_raw(1 << 52),
        Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
        Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
    );
    let planet4 = OrbitalBody::new(
        Fixed::from_raw(1 << 51),
        Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
        Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
    );
    OrbitalConfig::new(
        vec![sun, planet1, planet2, planet3, planet4],
        1000,
        100,
        kelvin::DEFAULT_DT,
        Fixed::from_raw(1 << 44),
        kelvin::DEFAULT_G,
    )
    .unwrap()
}

/// Create a temp file with the given content, return the path.
fn create_temp_file(_content: &[u8], suffix: &str) -> (std::path::PathBuf, std::fs::File) {
    let mut path = std::env::temp_dir();
    path.push(format!("kelvin_test_{}_{}", std::process::id(), suffix));
    let file = std::fs::File::create(&path).unwrap();
    (path, file)
}

// ============================================================================
// Photon (V3) Streaming Tests
// ============================================================================

#[test]
fn test_photon_streaming_round_trip() {
    let seed = test_seed();
    let mut encryptor = PhotonEncryptor::new(seed, 1000);
    let mut decryptor = PhotonDecryptor::new(seed, 1000);

    let plaintext = b"Hello, Kelvin V3 Photon streaming!";
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    let tag = encryptor.finalize().unwrap();
    assert!(tag.is_empty(), "Photon should not produce a tag");

    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&[]).unwrap();
    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_photon_streaming_multi_chunk() {
    let seed = test_seed();
    let mut encryptor = PhotonEncryptor::new(seed, 1000);
    let mut decryptor = PhotonDecryptor::new(seed, 1000);

    let chunks = vec![b"First chunk".to_vec(), b"Second chunk".to_vec(), b"Third".to_vec()];
    let mut all_ciphertext = Vec::new();
    for chunk in &chunks {
        encryptor.update(chunk, &mut all_ciphertext).unwrap();
    }
    encryptor.finalize().unwrap();

    let mut all_plaintext = Vec::new();
    decryptor.update(&all_ciphertext, &mut all_plaintext).unwrap();
    decryptor.finalize(&[]).unwrap();

    let expected: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
    assert_eq!(all_plaintext, expected);
}

#[test]
fn test_photon_streaming_empty() {
    let seed = test_seed();
    let mut encryptor = PhotonEncryptor::new(seed, 1000);
    let mut output = Vec::new();
    encryptor.update(&[], &mut output).unwrap();
    assert!(output.is_empty());
    let tag = encryptor.finalize().unwrap();
    assert!(tag.is_empty());
}

#[test]
fn test_photon_streaming_determinism() {
    let seed = test_seed();
    let mut e1 = PhotonEncryptor::new(seed, 1000);
    let mut e2 = PhotonEncryptor::new(seed, 1000);

    let data = b"Determinism test data";
    let mut c1 = Vec::new();
    let mut c2 = Vec::new();
    e1.update(data, &mut c1).unwrap();
    e2.update(data, &mut c2).unwrap();
    assert_eq!(c1, c2);
}

#[test]
fn test_photon_streaming_chunk_independence() {
    let seed = test_seed();
    let data = vec![0xBBu8; 100];

    // Encrypt as one chunk
    let mut e1 = PhotonEncryptor::new(seed, 1000);
    let mut c1 = Vec::new();
    e1.update(&data, &mut c1).unwrap();
    e1.finalize().unwrap();

    // Encrypt as two chunks
    let mut e2 = PhotonEncryptor::new(seed, 1000);
    let mut c2 = Vec::new();
    e2.update(&data[..50], &mut c2).unwrap();
    e2.update(&data[50..], &mut c2).unwrap();
    e2.finalize().unwrap();

    assert_eq!(c1, c2, "Photon should produce same ciphertext regardless of chunking");
}

#[test]
fn test_photon_streaming_vs_batch_consistency() {
    let seed = test_seed();
    let data = vec![0xAAu8; 1000];

    // Batch mode
    let mut batch = KelvinPhoton::new(seed, 1000);
    let mut batch_result = data.clone();
    batch.encrypt(&mut batch_result).unwrap();

    // Streaming mode
    let mut stream = PhotonEncryptor::new(seed, 1000);
    let mut stream_result = Vec::new();
    stream.update(&data, &mut stream_result).unwrap();
    stream.finalize().unwrap();

    assert_eq!(
        batch_result, stream_result,
        "Photon streaming and batch should produce identical ciphertext"
    );
}

// ============================================================================
// Quantum (H) Streaming Tests
// ============================================================================

#[test]
fn test_quantum_streaming_round_trip() {
    let seed = test_quantum_seed();
    let mut encryptor = QuantumEncryptor::new(seed, 1000);
    let mut decryptor = QuantumDecryptor::new(seed, 1000);

    let plaintext = b"Hello, Kelvin H Quantum streaming!";
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    let tag = encryptor.finalize().unwrap();
    assert!(tag.is_empty(), "Quantum should not produce a tag");

    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&[]).unwrap();
    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_quantum_streaming_determinism() {
    let seed = test_quantum_seed();
    let mut e1 = QuantumEncryptor::new(seed, 1000);
    let mut e2 = QuantumEncryptor::new(seed, 1000);

    let data = b"Determinism test data";
    let mut c1 = Vec::new();
    let mut c2 = Vec::new();
    e1.update(data, &mut c1).unwrap();
    e2.update(data, &mut c2).unwrap();
    assert_eq!(c1, c2);
}

#[test]
fn test_quantum_streaming_vs_batch_consistency() {
    let seed = test_quantum_seed();
    let data = vec![0xAAu8; 1000];

    // Batch mode
    let mut batch = KelvinQuantum::new(seed, 1000);
    let mut batch_result = data.clone();
    batch.encrypt(&mut batch_result).unwrap();

    // Streaming mode
    let mut stream = QuantumEncryptor::new(seed, 1000);
    let mut stream_result = Vec::new();
    stream.update(&data, &mut stream_result).unwrap();
    stream.finalize().unwrap();

    assert_eq!(
        batch_result, stream_result,
        "Quantum streaming and batch should produce identical ciphertext"
    );
}

// ============================================================================
// Chaos (V2) Streaming Tests
// ============================================================================

#[test]
fn test_chaos_streaming_round_trip() {
    let config = streaming_config();
    let mut encryptor = ChaosEncryptor::new(config.clone(), 64).unwrap();
    let mut decryptor = ChaosDecryptor::new(config, 64).unwrap();

    let plaintext = b"Hello, Kelvin V2 Chaos streaming!";
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    let tag = encryptor.finalize().unwrap();
    assert!(tag.is_empty(), "Chaos should not produce a tag");

    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&[]).unwrap();
    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_chaos_streaming_multi_chunk() {
    let config = streaming_config();
    let mut encryptor = ChaosEncryptor::new(config.clone(), 32).unwrap();
    let mut decryptor = ChaosDecryptor::new(config, 32).unwrap();

    let chunks = vec![b"First chunk".to_vec(), b"Second chunk".to_vec(), b"Third".to_vec()];
    let all_plaintext: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
    let mut all_ciphertext = Vec::new();
    encryptor.update(&all_plaintext, &mut all_ciphertext).unwrap();
    encryptor.finalize().unwrap();

    let mut decrypted = Vec::new();
    decryptor.update(&all_ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&[]).unwrap();
    assert_eq!(decrypted, all_plaintext);
}

#[test]
fn test_chaos_streaming_determinism() {
    let config = streaming_config();
    let mut e1 = ChaosEncryptor::new(config.clone(), 64).unwrap();
    let mut e2 = ChaosEncryptor::new(config, 64).unwrap();

    let data = b"Determinism test data";
    let mut c1 = Vec::new();
    let mut c2 = Vec::new();
    e1.update(data, &mut c1).unwrap();
    e2.update(data, &mut c2).unwrap();
    assert_eq!(c1, c2);
}

#[test]
fn test_chaos_streaming_chunk_independence() {
    let config = streaming_config();
    let data = vec![0xBBu8; 128]; // Multiple of 64

    // Encrypt as one chunk
    let mut e1 = ChaosEncryptor::new(config.clone(), 64).unwrap();
    let mut c1 = Vec::new();
    e1.update(&data, &mut c1).unwrap();
    e1.finalize().unwrap();

    // Encrypt as two chunks
    let mut e2 = ChaosEncryptor::new(config, 64).unwrap();
    let mut c2 = Vec::new();
    e2.update(&data[..64], &mut c2).unwrap();
    e2.update(&data[64..], &mut c2).unwrap();
    e2.finalize().unwrap();

    assert_eq!(c1, c2, "Chaos should produce same ciphertext regardless of chunking");
}

#[test]
fn test_chaos_streaming_vs_batch_consistency() {
    let config = streaming_config();
    let data = vec![0xAAu8; 200]; // Multiple of 64 to align with bytes_per_step

    // Batch mode
    let mut batch = KelvinStreaming::new(config.clone(), 64).unwrap();
    let mut batch_result = data.clone();
    batch.encrypt(&mut batch_result).unwrap();

    // Streaming mode
    let mut stream = ChaosEncryptor::new(config, 64).unwrap();
    let mut stream_result = Vec::new();
    stream.update(&data, &mut stream_result).unwrap();
    stream.finalize().unwrap();

    assert_eq!(
        batch_result, stream_result,
        "Chaos streaming and batch should produce identical ciphertext"
    );
}

// ============================================================================
// Secure (V1) Streaming Tests
// ============================================================================

#[test]
fn test_secure_streaming_round_trip() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let mut encryptor = SecureEncryptor::new(key, nonce);
    let mut decryptor = SecureDecryptor::new(key, nonce);

    let plaintext = b"Hello, Kelvin V1 Secure streaming!";
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    let tag = encryptor.finalize().unwrap();
    assert!(!tag.is_empty(), "Secure should produce a tag");

    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&tag).unwrap();
    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_secure_streaming_multi_chunk() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let mut encryptor = SecureEncryptor::new(key, nonce);
    let mut decryptor = SecureDecryptor::new(key, nonce);

    let chunks = vec![b"First chunk".to_vec(), b"Second chunk".to_vec(), b"Third".to_vec()];
    let mut all_ciphertext = Vec::new();
    for chunk in &chunks {
        encryptor.update(chunk, &mut all_ciphertext).unwrap();
    }
    let tag = encryptor.finalize().unwrap();

    let mut all_plaintext = Vec::new();
    decryptor.update(&all_ciphertext, &mut all_plaintext).unwrap();
    decryptor.finalize(&tag).unwrap();

    let expected: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
    assert_eq!(all_plaintext, expected);
}

#[test]
fn test_secure_streaming_tamper_detection() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let mut encryptor = SecureEncryptor::new(key, nonce);

    let plaintext = b"Tamper test data";
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    let tag = encryptor.finalize().unwrap();

    // Tamper with the ciphertext
    if !ciphertext.is_empty() {
        ciphertext[0] ^= 0x01;
    }

    let mut decryptor = SecureDecryptor::new(key, nonce);
    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    assert!(decryptor.finalize(&tag).is_err());
}

#[test]
fn test_secure_streaming_empty() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let mut encryptor = SecureEncryptor::new(key, nonce);
    let mut output = Vec::new();
    encryptor.update(&[], &mut output).unwrap();
    assert!(output.is_empty());
    let tag = encryptor.finalize().unwrap();
    assert!(!tag.is_empty(), "Secure should produce a tag even for empty input");
}

#[test]
fn test_secure_streaming_wrong_tag() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let mut encryptor = SecureEncryptor::new(key, nonce);

    let plaintext = b"Wrong tag test";
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    let _tag = encryptor.finalize().unwrap();

    // Try to decrypt with a wrong tag
    let mut decryptor = SecureDecryptor::new(key, nonce);
    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    let wrong_tag = [0xFFu8; 16];
    assert!(decryptor.finalize(&wrong_tag).is_err());
}

// ============================================================================
// File I/O Streaming Tests
// ============================================================================

#[test]
fn test_file_streaming_photon_round_trip() {
    let seed = test_seed();
    let plaintext = b"File-based Photon streaming test data!";

    let (in_path, mut in_file) = create_temp_file(plaintext, "photon_in.bin");
    in_file.write_all(plaintext).unwrap();
    in_file.flush().unwrap();

    let out_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_photon_out.bin", std::process::id()));
        p
    };
    let dec_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_photon_dec.bin", std::process::id()));
        p
    };

    // Encrypt
    let encryptor = PhotonEncryptor::new(seed, 1000);
    encrypt_file_streaming(encryptor, &in_path, &out_path, 64).unwrap();

    // Decrypt
    let decryptor = PhotonDecryptor::new(seed, 1000);
    decrypt_file_streaming(decryptor, &out_path, &dec_path, 64, 0).unwrap();

    // Verify
    let mut decrypted = Vec::new();
    std::fs::File::open(&dec_path).unwrap().read_to_end(&mut decrypted).unwrap();
    assert_eq!(&decrypted, plaintext);

    // Cleanup
    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_file(&dec_path);
}

#[test]
fn test_file_streaming_secure_round_trip() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let plaintext = b"File-based Secure streaming test data!";

    let (in_path, mut in_file) = create_temp_file(plaintext, "secure_in.bin");
    in_file.write_all(plaintext).unwrap();
    in_file.flush().unwrap();

    let out_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_secure_out.bin", std::process::id()));
        p
    };
    let dec_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_secure_dec.bin", std::process::id()));
        p
    };

    // Encrypt
    let encryptor = SecureEncryptor::new(key, nonce);
    encrypt_file_streaming(encryptor, &in_path, &out_path, 64).unwrap();

    // Decrypt (tag is 32 bytes for BLAKE3)
    let decryptor = SecureDecryptor::new(key, nonce);
    decrypt_file_streaming(decryptor, &out_path, &dec_path, 64, 32).unwrap();

    // Verify
    let mut decrypted = Vec::new();
    std::fs::File::open(&dec_path).unwrap().read_to_end(&mut decrypted).unwrap();
    assert_eq!(&decrypted, plaintext);

    // Cleanup
    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_file(&dec_path);
}

#[test]
fn test_file_streaming_chaos_round_trip() {
    let config = streaming_config();
    let plaintext = b"File-based Chaos streaming test data!";

    let (in_path, mut in_file) = create_temp_file(plaintext, "chaos_in.bin");
    in_file.write_all(plaintext).unwrap();
    in_file.flush().unwrap();

    let out_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_chaos_out.bin", std::process::id()));
        p
    };
    let dec_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_chaos_dec.bin", std::process::id()));
        p
    };

    // Encrypt
    let encryptor = ChaosEncryptor::new(config.clone(), 64).unwrap();
    encrypt_file_streaming(encryptor, &in_path, &out_path, 64).unwrap();

    // Decrypt
    let decryptor = ChaosDecryptor::new(config, 64).unwrap();
    decrypt_file_streaming(decryptor, &out_path, &dec_path, 64, 0).unwrap();

    // Verify
    let mut decrypted = Vec::new();
    std::fs::File::open(&dec_path).unwrap().read_to_end(&mut decrypted).unwrap();
    assert_eq!(&decrypted, plaintext);

    // Cleanup
    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_file(&dec_path);
}

#[test]
fn test_file_streaming_secure_tamper_detection() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let plaintext = b"Tamper detection via file I/O";

    let (in_path, mut in_file) = create_temp_file(plaintext, "tamper_in.bin");
    in_file.write_all(plaintext).unwrap();
    in_file.flush().unwrap();

    let out_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_tamper_out.bin", std::process::id()));
        p
    };
    let tampered_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_tamper_tampered.bin", std::process::id()));
        p
    };

    // Encrypt
    let encryptor = SecureEncryptor::new(key, nonce);
    encrypt_file_streaming(encryptor, &in_path, &out_path, 64).unwrap();

    // Tamper with the encrypted file
    let mut encrypted_data = Vec::new();
    std::fs::File::open(&out_path).unwrap().read_to_end(&mut encrypted_data).unwrap();
    if !encrypted_data.is_empty() {
        encrypted_data[0] ^= 0x01;
    }
    std::fs::write(&tampered_path, &encrypted_data).unwrap();

    // Decrypt should fail
    let dec_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_tamper_dec.bin", std::process::id()));
        p
    };
    let decryptor = SecureDecryptor::new(key, nonce);
    let result = decrypt_file_streaming(decryptor, &tampered_path, &dec_path, 64, 32);
    assert!(result.is_err(), "Tampered file should fail decryption");

    // Cleanup
    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_file(&tampered_path);
    let _ = std::fs::remove_file(&dec_path);
}

#[test]
fn test_file_streaming_large_buffer() {
    let seed = test_seed();
    let plaintext = vec![0xABu8; 256 * 1024]; // 256 KB

    let (in_path, mut in_file) = create_temp_file(&plaintext, "large_in.bin");
    in_file.write_all(&plaintext).unwrap();
    in_file.flush().unwrap();

    let out_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_large_out.bin", std::process::id()));
        p
    };
    let dec_path = {
        let mut p = std::env::temp_dir();
        p.push(format!("kelvin_test_{}_large_dec.bin", std::process::id()));
        p
    };

    // Encrypt with 64 KB buffer
    let encryptor = PhotonEncryptor::new(seed, 1000);
    encrypt_file_streaming(encryptor, &in_path, &out_path, 65536).unwrap();

    // Decrypt
    let decryptor = PhotonDecryptor::new(seed, 1000);
    decrypt_file_streaming(decryptor, &out_path, &dec_path, 65536, 0).unwrap();

    // Verify
    let mut decrypted = Vec::new();
    std::fs::File::open(&dec_path).unwrap().read_to_end(&mut decrypted).unwrap();
    assert_eq!(decrypted, plaintext);

    // Cleanup
    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_file(&dec_path);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_secure_streaming_double_finalize() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let mut encryptor = SecureEncryptor::new(key, nonce);

    encryptor.update(b"test", &mut Vec::new()).unwrap();
    let _tag = encryptor.finalize().unwrap();

    // Second finalize should fail
    assert!(encryptor.finalize().is_err());
}

#[test]
fn test_secure_streaming_update_after_finalize() {
    let key = [0xABu8; 32];
    let nonce = [0xCDu8; 12];
    let mut encryptor = SecureEncryptor::new(key, nonce);

    encryptor.update(b"test", &mut Vec::new()).unwrap();
    let _tag = encryptor.finalize().unwrap();

    // Update after finalize should fail
    let mut out = Vec::new();
    assert!(encryptor.update(b"more data", &mut out).is_err());
}

#[test]
fn test_chaos_streaming_misaligned_chunks() {
    let config = streaming_config();
    let mut encryptor = ChaosEncryptor::new(config.clone(), 64).unwrap();
    let mut decryptor = ChaosDecryptor::new(config, 64).unwrap();

    // Data not a multiple of bytes_per_step (64)
    let plaintext = b"Data that is not aligned to 64 bytes!!!";
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    encryptor.finalize().unwrap();

    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&[]).unwrap();
    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_photon_streaming_large_data() {
    let seed = test_seed();
    let mut encryptor = PhotonEncryptor::new(seed, 1000);
    let mut decryptor = PhotonDecryptor::new(seed, 1000);

    // 1 MB of data to exercise chunking
    let plaintext = vec![0xCDu8; 1024 * 1024];
    let mut ciphertext = Vec::new();
    encryptor.update(&plaintext, &mut ciphertext).unwrap();
    let tag = encryptor.finalize().unwrap();
    assert!(tag.is_empty());

    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&[]).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_quantum_streaming_max_reseed_exhaustion() {
    let seed = test_quantum_seed();
    // Only 1 reseed allowed
    let mut encryptor = QuantumEncryptor::new(seed, 1);

    // First call should succeed (initializes reader)
    let mut out = Vec::new();
    encryptor.update(b"test", &mut out).unwrap();
    encryptor.finalize().unwrap();

    // Second call with new instance should also work (same seed)
    let mut encryptor2 = QuantumEncryptor::new(seed, 1);
    let mut out2 = Vec::new();
    encryptor2.update(b"test", &mut out2).unwrap();
    encryptor2.finalize().unwrap();
}

#[test]
fn test_streaming_trait_object_dispatch() {
    // Verify that trait objects work for StreamEncrypt/StreamDecrypt
    let seed = test_seed();
    let plaintext = b"Trait object dispatch test";

    // Box<dyn StreamEncrypt>
    let mut encryptor: Box<dyn StreamEncrypt> = Box::new(PhotonEncryptor::new(seed, 1000));
    let mut ciphertext = Vec::new();
    encryptor.update(plaintext, &mut ciphertext).unwrap();
    let tag = encryptor.finalize().unwrap();
    assert!(tag.is_empty());

    // Box<dyn StreamDecrypt>
    let mut decryptor: Box<dyn StreamDecrypt> = Box::new(PhotonDecryptor::new(seed, 1000));
    let mut decrypted = Vec::new();
    decryptor.update(&ciphertext, &mut decrypted).unwrap();
    decryptor.finalize(&[]).unwrap();
    assert_eq!(&decrypted, plaintext);
}
