//! In-memory benchmark mode for all Kelvin modes.
//!
//! Processes `size` bytes entirely in memory without file I/O, isolating
//! crypto throughput from disk I/O. Supports all modes with and without
//! authentication tags.
//!
//! ## Tag handling for authenticated modes
//!
//! For modes that append authentication tags (Secure: 16-byte AEAD tag;
//! Chaos/Photon/Quantum with `--auth`: 32-byte KMAC128 tag), a separate
//! output vector is used to avoid out-of-bounds panics when the tag
//! expansion exceeds the fixed-size data buffer.

use anyhow::{Context, Result};
use kelvin::{
    simulate_and_extract_seed_with_method, IntegrationMethod, Kelvin, KelvinPhoton,
    KelvinPhotonAuthenticated, KelvinQuantum, KelvinQuantumAuthenticated, KelvinStreaming,
    KelvinStreamingAuthenticated, OrbitalConfig, AUTH_OVERHEAD, PHOTON_DEFAULT_MAX_RESEEDS,
    STREAMING_CHUNK_SIZE,
};

use crate::CryptoMode;

/// Process `size` bytes in memory using the specified mode.
///
/// Allocates a buffer of `size` bytes, encrypts/decrypts it, and reports
/// throughput. Uses a separate output vector for authenticated modes to
/// safely handle tag expansion.
#[allow(clippy::too_many_arguments)]
pub fn process_in_memory(
    mode: &CryptoMode,
    config_path: &str,
    encrypt: bool,
    bytes_per_step: u64,
    method: IntegrationMethod,
    auth: bool,
    size: u64,
) -> Result<()> {
    let config_json = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let mode_label = format!("{:?}", mode);
    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    let auth_label = if auth { " + KMAC128" } else { "" };
    let op_label = if encrypt { "Encrypt" } else { "Decrypt" };

    println!(
        "In-memory {} ({} mode, {} integration{})...",
        op_label, mode_label, method_label, auth_label
    );

    // Constants for tag sizes
    const SECURE_TAG_LEN: usize = 16; // ChaCha20Poly1305 AEAD tag

    // Allocate the full data buffer in memory
    let mut data = vec![0xABu8; size as usize];

    // Initialize the crypto engine (includes orbital simulation for photon/quantum)
    match mode {
        CryptoMode::Secure => {
            let mut k = Kelvin::new(config).context("Failed to initialize Kelvin")?;
            println!("  Setup complete. Processing {} bytes...", size);
            let start = std::time::Instant::now();
            let chunk_size = STREAMING_CHUNK_SIZE;
            // Secure mode uses ChaCha20Poly1305 which needs 16 extra bytes for the AEAD tag.
            // Use a separate output vector to avoid out-of-bounds panics when the tag
            // expansion exceeds the fixed-size data buffer.
            let mut buf = vec![0u8; chunk_size + SECURE_TAG_LEN];
            let mut output = Vec::with_capacity(size as usize + SECURE_TAG_LEN);
            let mut offset = 0;
            while offset < data.len() {
                let remaining = data.len() - offset;
                if encrypt {
                    let chunk = std::cmp::min(remaining, chunk_size);
                    // Encryption: copy plaintext, encrypt (appends tag), extend output
                    buf[..chunk].copy_from_slice(&data[offset..offset + chunk]);
                    buf[chunk..chunk + SECURE_TAG_LEN].fill(0);
                    k.encrypt(&mut buf[..chunk + SECURE_TAG_LEN])?;
                    output.extend_from_slice(&buf[..chunk + SECURE_TAG_LEN]);
                    offset += chunk;
                } else {
                    if remaining < SECURE_TAG_LEN {
                        anyhow::bail!("Invalid ciphertext: truncated data at offset {}", offset);
                    }
                    let chunk = std::cmp::min(remaining - SECURE_TAG_LEN, chunk_size);
                    // Decryption: copy ciphertext + tag, decrypt (verifies tag), extend output with plaintext
                    buf[..chunk + SECURE_TAG_LEN]
                        .copy_from_slice(&data[offset..offset + chunk + SECURE_TAG_LEN]);
                    k.decrypt(&mut buf[..chunk + SECURE_TAG_LEN])?;
                    output.extend_from_slice(&buf[..chunk]);
                    offset += chunk + SECURE_TAG_LEN;
                }
            }
            let elapsed = start.elapsed().as_secs_f64();
            report_throughput(op_label, size, elapsed);
        },

        CryptoMode::Chaos => {
            if auth {
                let mut ks = KelvinStreamingAuthenticated::new(config, bytes_per_step)
                    .context("Failed to initialize KelvinStreamingAuthenticated")?;
                println!("  Setup complete. Processing {} bytes...", size);
                let start = std::time::Instant::now();
                let chunk_size = STREAMING_CHUNK_SIZE;
                // For auth modes, we use a separate working buffer that can grow/shrink
                // with the tag. The `data` buffer stays at `size` bytes (plaintext size).
                // During encryption, we process `chunk` bytes of plaintext, encrypt produces
                // `chunk + AUTH_OVERHEAD` bytes, and we write those to a separate output buffer.
                // During decryption, we read `chunk + AUTH_OVERHEAD` bytes from the ciphertext,
                // decrypt produces `chunk` bytes of plaintext, and we write those back.
                let read_chunk_size = if encrypt { chunk_size } else { chunk_size + AUTH_OVERHEAD };
                let mut output = Vec::with_capacity(size as usize + AUTH_OVERHEAD);
                let mut offset = 0;
                while offset < data.len() {
                    let remaining = data.len() - offset;
                    let chunk = std::cmp::min(remaining, read_chunk_size);
                    let mut buf = data[offset..offset + chunk].to_vec();
                    if encrypt {
                        ks.encrypt(&mut buf)?;
                    } else {
                        ks.decrypt(&mut buf)?;
                    }
                    output.extend_from_slice(&buf);
                    offset += chunk;
                }
                let elapsed = start.elapsed().as_secs_f64();
                report_throughput(op_label, size, elapsed);
            } else {
                let mut ks = KelvinStreaming::new(config, bytes_per_step)
                    .context("Failed to initialize KelvinStreaming")?;
                println!("  Setup complete. Processing {} bytes...", size);
                let start = std::time::Instant::now();
                let chunk_size = STREAMING_CHUNK_SIZE;
                let mut offset = 0;
                while offset < data.len() {
                    let remaining = data.len() - offset;
                    let chunk = std::cmp::min(remaining, chunk_size);
                    ks.encrypt(&mut data[offset..offset + chunk])?;
                    offset += chunk;
                }
                let elapsed = start.elapsed().as_secs_f64();
                report_throughput(op_label, size, elapsed);
            }
        },
        CryptoMode::Photon => {
            let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
                .context("Failed to run orbital simulation")?;
            println!("  Orbital simulation complete. Processing {} bytes...", size);
            if auth {
                let mut photon = KelvinPhotonAuthenticated::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
                let start = std::time::Instant::now();
                let chunk_size = bytes_per_step as usize;
                let read_chunk_size = if encrypt { chunk_size } else { chunk_size + AUTH_OVERHEAD };
                // Use a separate output buffer to avoid overwriting issues with tag expansion
                let mut output = Vec::with_capacity(size as usize + AUTH_OVERHEAD);
                let mut offset = 0;
                while offset < data.len() {
                    let remaining = data.len() - offset;
                    let chunk = std::cmp::min(remaining, read_chunk_size);
                    let mut buf = data[offset..offset + chunk].to_vec();
                    if encrypt {
                        photon.encrypt(&mut buf)?;
                    } else {
                        photon.decrypt(&mut buf)?;
                    }
                    output.extend_from_slice(&buf);
                    offset += chunk;
                }
                let elapsed = start.elapsed().as_secs_f64();
                report_throughput(op_label, size, elapsed);
            } else {
                let mut photon = KelvinPhoton::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
                let start = std::time::Instant::now();
                let chunk_size = bytes_per_step as usize;
                let mut offset = 0;
                while offset < data.len() {
                    let remaining = data.len() - offset;
                    let chunk = std::cmp::min(remaining, chunk_size);
                    photon.encrypt(&mut data[offset..offset + chunk])?;
                    offset += chunk;
                }
                let elapsed = start.elapsed().as_secs_f64();
                report_throughput(op_label, size, elapsed);
            }
        },
        CryptoMode::Quantum => {
            let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
                .context("Failed to run orbital simulation")?;
            println!("  Orbital simulation complete. Processing {} bytes...", size);
            if auth {
                let mut quantum = KelvinQuantumAuthenticated::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
                let start = std::time::Instant::now();
                let chunk_size = bytes_per_step as usize;
                let read_chunk_size = if encrypt { chunk_size } else { chunk_size + AUTH_OVERHEAD };
                // Use a separate output buffer to avoid overwriting issues with tag expansion
                let mut output = Vec::with_capacity(size as usize + AUTH_OVERHEAD);
                let mut offset = 0;
                while offset < data.len() {
                    let remaining = data.len() - offset;
                    let chunk = std::cmp::min(remaining, read_chunk_size);
                    let mut buf = data[offset..offset + chunk].to_vec();
                    if encrypt {
                        quantum.encrypt(&mut buf)?;
                    } else {
                        quantum.decrypt(&mut buf)?;
                    }
                    output.extend_from_slice(&buf);
                    offset += chunk;
                }
                let elapsed = start.elapsed().as_secs_f64();
                report_throughput(op_label, size, elapsed);
            } else {
                let mut quantum = KelvinQuantum::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
                let start = std::time::Instant::now();
                let chunk_size = bytes_per_step as usize;
                let mut offset = 0;
                while offset < data.len() {
                    let remaining = data.len() - offset;
                    let chunk = std::cmp::min(remaining, chunk_size);
                    quantum.encrypt(&mut data[offset..offset + chunk])?;
                    offset += chunk;
                }
                let elapsed = start.elapsed().as_secs_f64();
                report_throughput(op_label, size, elapsed);
            }
        },
    }

    Ok(())
}

/// Report throughput for in-memory benchmark.
pub fn report_throughput(op_label: &str, size: u64, elapsed_secs: f64) {
    let size_gb = size as f64 / (1024.0 * 1024.0 * 1024.0);
    let throughput_gbs = size_gb / elapsed_secs;
    let throughput_mbs = throughput_gbs * 1024.0;
    println!(
        "  {} complete: {:.3}s, {:.2} GB/s ({:.0} MB/s)",
        op_label, elapsed_secs, throughput_gbs, throughput_mbs
    );
}
