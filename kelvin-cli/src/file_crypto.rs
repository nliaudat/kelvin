//! File-based encryption/decryption for all Kelvin modes.
//!
//! Provides per-mode processing functions that read from an input file,
//! encrypt/decrypt using the specified mode, and write to an output file.
//! Each mode has its own function to handle mode-specific buffer sizes,
//! tag handling, and authentication overhead.

use anyhow::{Context, Result};
use kelvin::{
    simulate_and_extract_seed_with_method, IntegrationMethod, Kelvin, KelvinPhoton,
    KelvinPhotonAuthenticated, KelvinQuantum, KelvinQuantumAuthenticated, KelvinStreaming,
    KelvinStreamingAuthenticated, OrbitalConfig, AUTH_OVERHEAD, PHOTON_DEFAULT_MAX_RESEEDS,
    STREAMING_CHUNK_SIZE,
};
use std::io::{Read, Write};

use crate::CryptoMode;

/// Dispatch to the correct processing function based on mode.
#[allow(clippy::too_many_arguments)]
pub fn process_file_mode(
    mode: &CryptoMode,
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    bytes_per_step: u64,
    method: IntegrationMethod,
    auth: bool,
) -> Result<()> {
    match mode {
        CryptoMode::Secure => {
            process_file_secure(config_path, input_path, output_path, encrypt, method)
        },
        CryptoMode::Chaos => process_file_chaos(
            config_path,
            input_path,
            output_path,
            encrypt,
            bytes_per_step,
            method,
            auth,
        ),
        CryptoMode::Photon => process_file_photon(
            config_path,
            input_path,
            output_path,
            encrypt,
            method,
            auth,
            bytes_per_step,
        ),
        CryptoMode::Quantum => process_file_quantum(
            config_path,
            input_path,
            output_path,
            encrypt,
            method,
            auth,
            bytes_per_step,
        ),
    }
}

/// V1 Secure: ChaCha20Poly1305 AEAD (original Kelvin).
///
/// ## AEAD Tag Handling
///
/// ChaCha20Poly1305 appends a 16-byte authentication tag to the ciphertext.
/// The buffer must have 16 extra bytes after the plaintext for the tag.
///
/// **Encryption flow:**
/// 1. Read `STREAMING_CHUNK_SIZE` bytes of plaintext into buffer[..chunk]
/// 2. `k.encrypt()` encrypts in-place, writing the 16-byte tag at buffer[chunk..chunk+16]
/// 3. Write buffer[..chunk + 16] to output (plaintext + tag)
///
/// **Decryption flow:**
/// 1. Read `STREAMING_CHUNK_SIZE + 16` bytes of ciphertext+tag into buffer
/// 2. `k.decrypt()` decrypts in-place, verifying the tag
/// 3. Write buffer[..chunk] to output (plaintext only, strip the tag)
fn process_file_secure(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    method: IntegrationMethod,
) -> Result<()> {
    let config_json = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    println!(
        "Initializing Kelvin Secure (V1, {} integration, this may take a few seconds)...",
        method_label
    );
    let mut k = Kelvin::new(config).context("Failed to initialize Kelvin")?;

    let mut input_file = std::fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file =
        std::fs::File::create(output_path).context("Failed to create output file")?;

    println!("Processing (ChaCha20Poly1305 AEAD)...");
    // Buffer layout: [plaintext/ciphertext | 16-byte AEAD tag]
    // encrypt_in_place() expects data.len() = plaintext_len + 16.
    // The tag is written at position plaintext_len..plaintext_len+16.
    let mut buffer = vec![0u8; STREAMING_CHUNK_SIZE + 16];
    let mut total_processed = 0u64;
    loop {
        if encrypt {
            // Encryption:
            // 1. Read STREAMING_CHUNK_SIZE bytes of plaintext into buffer[..chunk]
            // 2. Call encrypt(&mut buffer[..chunk + 16]) — the extra 16 bytes are
            //    zeroed and receive the AEAD tag at position chunk..chunk+16
            // 3. Write buffer[..chunk + 16] to output (ciphertext + tag)
            let bytes_read = input_file.read(&mut buffer[..STREAMING_CHUNK_SIZE])?;
            if bytes_read == 0 {
                break;
            }
            // Zero the tag area to ensure clean state
            buffer[bytes_read..bytes_read + 16].fill(0);
            k.encrypt(&mut buffer[..bytes_read + 16])?;
            output_file.write_all(&buffer[..bytes_read + 16])?;
            total_processed += bytes_read as u64;
        } else {
            // Decryption:
            // 1. Read STREAMING_CHUNK_SIZE + 16 bytes of ciphertext+tag into buffer
            // 2. Call decrypt(&mut buffer[..bytes_read]) — verifies the tag
            // 3. Write buffer[..bytes_read - 16] to output (plaintext only)
            let bytes_read = input_file.read(&mut buffer[..STREAMING_CHUNK_SIZE + 16])?;
            if bytes_read == 0 {
                break;
            }
            if bytes_read < 16 {
                anyhow::bail!("Invalid ciphertext: truncated data at offset {}", total_processed);
            }
            let plaintext_len = bytes_read - 16;
            k.decrypt(&mut buffer[..bytes_read])?;
            output_file.write_all(&buffer[..plaintext_len])?;
            total_processed += plaintext_len as u64;
        }

        if total_processed.is_multiple_of(1024 * 1024) {
            print!(".");
            let _ = std::io::stdout().flush();
        }
    }

    println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
    Ok(())
}

/// V2 Chaos: Per-step SHAKE256 XOR streaming.
///
/// When `auth` is true, uses [`KelvinStreamingAuthenticated`] to append a
/// 32-byte KMAC128 tag (NIST SP 800-185) to defeat ciphertext malleability.
fn process_file_chaos(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    bytes_per_step: u64,
    method: IntegrationMethod,
    auth: bool,
) -> Result<()> {
    let config_json = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    let auth_label = if auth { " + KMAC128" } else { "" };
    println!(
        "Initializing Kelvin Chaos (V2, {} integration, instant setup{})...",
        method_label, auth_label
    );

    if auth {
        let mut ks = KelvinStreamingAuthenticated::new(config, bytes_per_step)
            .context("Failed to initialize KelvinStreamingAuthenticated")?;

        let mut input_file =
            std::fs::File::open(input_path).context("Failed to open input file")?;
        let mut output_file =
            std::fs::File::create(output_path).context("Failed to create output file")?;

        println!("Processing (SHAKE256 XOR + KMAC128)...");
        let chunk_size = STREAMING_CHUNK_SIZE;
        // During encryption, each plaintext chunk produces ciphertext + auth overhead.
        // During decryption, we need to read ciphertext + auth overhead in one shot.
        let read_size = if encrypt { chunk_size } else { chunk_size + AUTH_OVERHEAD };
        let mut buffer = vec![0u8; read_size];
        let mut chunk = Vec::with_capacity(chunk_size + AUTH_OVERHEAD);
        let mut total_processed = 0u64;
        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            chunk.clear();
            chunk.extend_from_slice(&buffer[..bytes_read]);
            if encrypt {
                ks.encrypt(&mut chunk)?;
            } else {
                ks.decrypt(&mut chunk)?;
            }
            output_file.write_all(&chunk)?;
            total_processed += bytes_read as u64;
            if total_processed.is_multiple_of(1024 * 1024) {
                print!(".");
                let _ = std::io::stdout().flush();
            }
        }
        println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
        return Ok(());
    }

    let mut ks = KelvinStreaming::new(config, bytes_per_step)
        .context("Failed to initialize KelvinStreaming")?;

    let rate = ks.benchmark(100);
    let file_size = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    let (steps_needed, est_secs) = ks.estimate_time(file_size, rate);
    if file_size > 0 {
        println!("Estimated: {} steps, ~{:.1}s ({:.0} steps/sec)", steps_needed, est_secs, rate);
    }

    let mut input_file = std::fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file =
        std::fs::File::create(output_path).context("Failed to create output file")?;

    println!("Processing (SHAKE256 XOR)...");
    let mut buffer = vec![0u8; STREAMING_CHUNK_SIZE];
    let mut total_processed = 0u64;
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        if encrypt {
            ks.encrypt(&mut buffer[..bytes_read])?;
        } else {
            ks.decrypt(&mut buffer[..bytes_read])?;
        }

        output_file.write_all(&buffer[..bytes_read])?;
        total_processed += bytes_read as u64;
        if total_processed.is_multiple_of(1024 * 1024) {
            print!(".");
            let _ = std::io::stdout().flush();
        }
    }

    println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
    Ok(())
}

/// V3 Photon: Fast HKDF→SHAKE256 XOR OTP from upfront simulation.
///
/// When `auth` is true, uses [`KelvinPhotonAuthenticated`] to append a
/// 32-byte KMAC128 tag (NIST SP 800-185) to defeat ciphertext malleability.
fn process_file_photon(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    method: IntegrationMethod,
    auth: bool,
    bytes_per_step: u64,
) -> Result<()> {
    let config_json = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    let auth_label = if auth { " + KMAC128" } else { "" };
    println!(
        "Initializing Kelvin Photon (V3, {} integration, running orbital simulation{})...",
        method_label, auth_label
    );
    let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
        .context("Failed to run orbital simulation")?;

    let mut input_file = std::fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file =
        std::fs::File::create(output_path).context("Failed to create output file")?;

    let cipher_label = if auth { "HKDF→SHAKE256 XOR + KMAC128" } else { "HKDF→SHAKE256 XOR" };
    println!("Processing ({})...", cipher_label);

    if auth {
        let chunk_size = bytes_per_step as usize;
        let mut photon = KelvinPhotonAuthenticated::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
        // During encryption, each plaintext chunk produces ciphertext + auth overhead.
        // During decryption, we need to read ciphertext + auth overhead in one shot.
        let read_size = if encrypt { chunk_size } else { chunk_size + AUTH_OVERHEAD };
        let mut buffer = vec![0u8; read_size];
        let mut chunk = Vec::with_capacity(chunk_size + AUTH_OVERHEAD);
        let mut total_processed = 0u64;
        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            chunk.clear();
            chunk.extend_from_slice(&buffer[..bytes_read]);
            if encrypt {
                photon.encrypt(&mut chunk)?;
            } else {
                photon.decrypt(&mut chunk)?;
            }
            output_file.write_all(&chunk)?;
            total_processed += bytes_read as u64;
            if total_processed.is_multiple_of(1024 * 1024) {
                print!(".");
                let _ = std::io::stdout().flush();
            }
        }
        println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
        return Ok(());
    }

    let chunk_size = bytes_per_step as usize;
    let mut photon = KelvinPhoton::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
    let mut buffer = vec![0u8; chunk_size];
    let mut total_processed = 0u64;
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        if encrypt {
            photon.encrypt(&mut buffer[..bytes_read])?;
        } else {
            photon.decrypt(&mut buffer[..bytes_read])?;
        }

        output_file.write_all(&buffer[..bytes_read])?;
        total_processed += bytes_read as u64;
        if total_processed.is_multiple_of(1024 * 1024) {
            print!(".");
            let _ = std::io::stdout().flush();
        }
    }

    println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
    Ok(())
}

/// H Quantum: Hybrid V3 bulk speed + V2 orbital entropy reseed.
///
/// When `auth` is true, uses [`KelvinQuantumAuthenticated`] to append a
/// 32-byte KMAC128 tag (NIST SP 800-185) to defeat ciphertext malleability.
fn process_file_quantum(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    method: IntegrationMethod,
    auth: bool,
    bytes_per_step: u64,
) -> Result<()> {
    let config_json = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    let auth_label = if auth { " + KMAC128" } else { "" };
    println!(
        "Initializing Kelvin Quantum (H, {} integration, running orbital simulation{})...",
        method_label, auth_label
    );
    let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
        .context("Failed to run orbital simulation")?;

    let mut input_file = std::fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file =
        std::fs::File::create(output_path).context("Failed to create output file")?;

    let cipher_label = if auth {
        "Hybrid cache+XOR + orbital reseed + KMAC128"
    } else {
        "Hybrid cache+XOR + orbital reseed"
    };
    println!("Processing ({})...", cipher_label);

    if auth {
        let chunk_size = bytes_per_step as usize;
        let mut quantum = KelvinQuantumAuthenticated::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
        // During encryption, each plaintext chunk produces ciphertext + auth overhead.
        // During decryption, we need to read ciphertext + auth overhead in one shot.
        let read_size = if encrypt { chunk_size } else { chunk_size + AUTH_OVERHEAD };
        let mut buffer = vec![0u8; read_size];
        let mut chunk = Vec::with_capacity(chunk_size + AUTH_OVERHEAD);
        let mut total_processed = 0u64;
        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            chunk.clear();
            chunk.extend_from_slice(&buffer[..bytes_read]);
            if encrypt {
                quantum.encrypt(&mut chunk)?;
            } else {
                quantum.decrypt(&mut chunk)?;
            }
            output_file.write_all(&chunk)?;
            total_processed += bytes_read as u64;
            if total_processed.is_multiple_of(1024 * 1024) {
                print!(".");
                let _ = std::io::stdout().flush();
            }
        }
        println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
        return Ok(());
    }

    let chunk_size = bytes_per_step as usize;
    let mut quantum = KelvinQuantum::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);

    let mut buffer = vec![0u8; chunk_size];
    let mut total_processed = 0u64;
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        if encrypt {
            quantum.encrypt(&mut buffer[..bytes_read])?;
        } else {
            quantum.decrypt(&mut buffer[..bytes_read])?;
        }

        output_file.write_all(&buffer[..bytes_read])?;
        total_processed += bytes_read as u64;
        if total_processed.is_multiple_of(1024 * 1024) {
            print!(".");
            let _ = std::io::stdout().flush();
        }
    }

    println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
    Ok(())
}
