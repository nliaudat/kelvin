//! Kelvin CLI — Orbital Chaos KDF Cryptosystem
//!
//! Provides a command-line interface for:
//! - Generating orbital configurations (Keygen)
//! - Encrypting/Decrypting files (with mode selection)
//! - Inspecting public keys (Identify)
//! - Benchmarking
//!
//! ## Modes
//!
//! | Mode | Engine (no `--auth`) | Engine (with `--auth`) | Cipher |
//! |------|---------------------|------------------------|--------|
//! | `secure` (V1, default) | `Kelvin` | `Kelvin` (AEAD built-in) | ChaCha20Poly1305 AEAD |
//! | `chaos` (V2) | `KelvinStreaming` | `KelvinStreamingAuthenticated` | SHAKE256 XOR per-step |
//! | `photon` (V3) | `KelvinPhoton` | `KelvinPhotonAuthenticated` | HKDF→SHAKE256 XOR |
//! | `quantum` (H) | `KelvinQuantum` | `KelvinQuantumAuthenticated` | Hybrid cache+XOR + orbital reseed |
//!
//! Use `--auth` to append a version byte + 32-byte KMAC128 tag (NIST SP 800-185)
//! to defeat ciphertext malleability. The `secure` mode has built-in AEAD
//! authentication and ignores the `--auth` flag.
//!
//! Integration defaults to Verlet (energy-conserving). Use `--euler`
//! for numerically unstable integration (faster chaos amplification).

#![deny(unsafe_code)]

mod bench;
mod file_crypto;
mod in_memory;
mod keygen;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use kelvin::{
    Fixed, IntegrationMethod, Kelvin, OrbitalConfig, OrbitalKeyPair, CHAOS_DEFAULT_BYTES_PER_STEP,
    PHOTON_DEFAULT_BYTES_PER_STEP, QUANTUM_DEFAULT_BYTES_PER_STEP,
};
use ml_kem::KeyExport;
use std::fs;

use bench::run_benchmark;
use file_crypto::process_file_mode;
use in_memory::process_in_memory;
use keygen::generate_config;

/// Cryptographic mode for encrypt/decrypt operations.
#[derive(ValueEnum, Clone, Debug)]
enum CryptoMode {
    /// V1: ChaCha20Poly1305 AEAD (default)
    Secure,
    /// V2: Per-step SHAKE256 XOR streaming
    Chaos,
    /// V3: Fast HKDF→SHAKE256 XOR OTP
    Photon,
    /// H: Hybrid V3 bulk speed + V2 orbital entropy reseed
    Quantum,
}

#[derive(Parser)]
#[command(name = "kelvin", version, about = "Orbital Chaos KDF Cryptosystem")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new orbital configuration (shared secret)
    Keygen {
        /// Security level: standard, paranoid, or maximum
        #[arg(long, default_value = "standard")]
        level: String,
        /// Output file (default: stdout)
        #[arg(long)]
        output: Option<String>,
        /// Fast mode: use 110,000 simulation steps instead of the full step count
        /// (standard=1M, paranoid=10M, maximum=100M). Produces a valid config
        /// that passes the Lyapunov chaos check (~100k min_chaos_steps) while
        /// keeping simulation time under ~4s. Useful for benchmarking and testing.
        #[arg(long)]
        fast: bool,
    },
    /// Encrypt a file (or in-memory data with --in-memory)
    Encrypt {
        /// Cryptographic mode: secure, chaos, photon, or quantum
        #[arg(long, default_value = "secure")]
        mode: CryptoMode,
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
        /// Input file path (ignored if --in-memory is set)
        #[arg(long)]
        input: Option<String>,
        /// Output file path (ignored if --in-memory is set)
        #[arg(long)]
        output: Option<String>,
        /// Bytes of keystream per step/chunk (chaos/photon/quantum, mode-specific default)
        #[arg(long)]
        bytes_per_step: Option<u64>,
        /// Use Euler integration instead of default Verlet (numerically unstable, faster chaos)
        #[arg(long)]
        euler: bool,
        /// Append a version byte + 32-byte KMAC128 tag for authentication (chaos, photon, quantum modes)
        #[arg(long)]
        auth: bool,
        /// In-memory benchmark mode: process `size` bytes without file I/O
        #[arg(long)]
        in_memory: bool,
        /// Size in bytes for --in-memory mode (default: 1 GiB = 1073741824)
        #[arg(long, default_value = "1073741824")]
        size: u64,
    },
    /// Decrypt a file (or in-memory data with --in-memory)
    Decrypt {
        /// Cryptographic mode: secure, chaos, photon, or quantum
        #[arg(long, default_value = "secure")]
        mode: CryptoMode,
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
        /// Input file path (ignored if --in-memory is set)
        #[arg(long)]
        input: Option<String>,
        /// Output file path (ignored if --in-memory is set)
        #[arg(long)]
        output: Option<String>,
        /// Bytes of keystream per step/chunk (chaos/photon/quantum, mode-specific default)
        #[arg(long)]
        bytes_per_step: Option<u64>,
        /// Use Euler integration instead of default Verlet (numerically unstable, faster chaos)
        #[arg(long)]
        euler: bool,
        /// Verify and strip the version byte + 32-byte KMAC128 tag for authentication (chaos, photon, quantum modes)
        #[arg(long)]
        auth: bool,
        /// In-memory benchmark mode: process `size` bytes without file I/O
        #[arg(long)]
        in_memory: bool,
        /// Size in bytes for --in-memory mode (default: 1 GiB = 1073741824)
        #[arg(long, default_value = "1073741824")]
        size: u64,
    },
    /// Identify the Public Key associated with a configuration
    Identify {
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
        /// Show all keys (Classical and PQ)
        #[arg(long)]
        all: bool,
        /// Show Curve25519 Public Key
        #[arg(long)]
        ecc: bool,
        /// Show ML-KEM-768 Public Key
        #[arg(long)]
        kem: bool,
        /// Fast mode: use fewer simulation steps for quicker identification
        #[arg(long)]
        fast: bool,
    },
    /// Run performance benchmarks
    Benchmark,
    /// Analyze the cryptographic quality of a configuration
    Analyze {
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Keygen { level, output, fast } => {
            let config = generate_config(&level, fast)?;
            let json = config.to_json()?;
            if let Some(path) = output {
                fs::write(path, json).context("Failed to write config file")?;
                if fast {
                    println!("Generated {} config (fast mode, 110,000 steps).", level);
                } else {
                    println!("Generated {} config.", level);
                }
            } else {
                println!("{}", json);
            }
        },
        Commands::Encrypt {
            mode,
            config,
            input,
            output,
            bytes_per_step,
            euler,
            auth,
            in_memory,
            size,
        } => {
            let method = if euler { IntegrationMethod::Euler } else { IntegrationMethod::Verlet };
            let bps = resolve_bytes_per_step(&mode, bytes_per_step);
            if in_memory {
                process_in_memory(&mode, &config, true, bps, method, auth, size)?;
            } else {
                let input_path =
                    input.as_deref().context("--input is required (or use --in-memory)")?;
                let output_path =
                    output.as_deref().context("--output is required (or use --in-memory)")?;
                process_file_mode(
                    &mode,
                    &config,
                    input_path,
                    output_path,
                    true,
                    bps,
                    method,
                    auth,
                )?;
            }
        },
        Commands::Decrypt {
            mode,
            config,
            input,
            output,
            bytes_per_step,
            euler,
            auth,
            in_memory,
            size,
        } => {
            let method = if euler { IntegrationMethod::Euler } else { IntegrationMethod::Verlet };
            let bps = resolve_bytes_per_step(&mode, bytes_per_step);
            if in_memory {
                process_in_memory(&mode, &config, false, bps, method, auth, size)?;
            } else {
                let input_path =
                    input.as_deref().context("--input is required (or use --in-memory)")?;
                let output_path =
                    output.as_deref().context("--output is required (or use --in-memory)")?;
                process_file_mode(
                    &mode,
                    &config,
                    input_path,
                    output_path,
                    false,
                    bps,
                    method,
                    auth,
                )?;
            }
        },
        Commands::Identify { config, all, ecc, kem, fast } => {
            let config_json = fs::read_to_string(config).context("Failed to read config file")?;
            let config = OrbitalConfig::from_json(&config_json)?;

            if fast {
                println!("Warning: --fast flag is ignored for identification. Using full {} steps for deterministic key derivation.", config.total_steps);
            }

            println!("Deriving keys from orbital configuration (this may take a moment)...");
            let kp = OrbitalKeyPair::derive(&config)
                .map_err(|e| anyhow::anyhow!("Key derivation failed: {:?}", e))?;

            let mut shown = false;

            if all || ecc {
                println!("Curve25519 (Classical): {}", hex::encode(kp.curve_public.as_bytes()));
                shown = true;
            }
            if all || kem {
                println!("ML-KEM-768 (PQ-KEM):    {}", hex::encode(kp.kem_public.to_bytes()));
                shown = true;
            }

            if all || (!ecc && !kem) {
                println!("ML-DSA-65  (PQ-Sig):    {}", hex::encode(kp.dsa_public.to_bytes()));
                shown = true;
            }

            if !shown {
                println!("ML-DSA-65  (PQ-Sig):    {}", hex::encode(kp.dsa_public.to_bytes()));
            }
        },
        Commands::Analyze { config } => {
            let config_json = fs::read_to_string(config).context("Failed to read config file")?;
            let mut config = OrbitalConfig::from_json(&config_json)?;

            println!("Analyzing Cryptographic Quality...");

            println!("Deriving Base Key...");
            let k1 = Kelvin::new(config.clone()).context("Failed to init first instance")?;
            let kp1 = k1.asymmetric_keypair();
            let key1 = kp1.curve_public.as_bytes();

            println!("Flipping 1 bit in initial conditions...");
            config.bodies[0].mass = Fixed::from_raw(config.bodies[0].mass.to_raw() ^ 1);

            println!("Deriving Shadow Key...");
            let k2 = Kelvin::new(config).context("Failed to init shadow instance")?;
            let kp2 = k2.asymmetric_keypair();
            let key2 = kp2.curve_public.as_bytes();

            let mut diff_bits = 0;
            let mut set_bits = 0;
            for i in 0..32 {
                let diff = key1[i] ^ key2[i];
                diff_bits += diff.count_ones();
                set_bits += key1[i].count_ones();
            }

            println!("--- Analysis Results ---");
            println!("Target: Curve25519 Identity");
            println!("Total Bits: 256");
            println!("Bit Entropy (Base): {:.2} bits", set_bits as f32 / 256.0);
            println!(
                "Avalanche Effect: {} / 256 bits changed ({:.2}%)",
                diff_bits,
                (diff_bits as f32 / 256.0) * 100.0
            );

            if diff_bits > 110 && diff_bits < 146 {
                println!("Result: PASS (Strong Avalanche Effect)");
            } else {
                println!("Result: FAIL (Weak sensitivity to initial conditions)");
            }
        },
        Commands::Benchmark => {
            run_benchmark()?;
        },
    }
    Ok(())
}

/// Resolve the mode-specific default for `--bytes-per-step`.
///
/// If the user explicitly provided a value, use it. Otherwise, use the
/// mode-specific default from `parameters.rs`:
/// - Chaos:  `CHAOS_DEFAULT_BYTES_PER_STEP`   (1 MiB)
/// - Photon: `PHOTON_DEFAULT_BYTES_PER_STEP`  (64 MiB)
/// - Quantum:`QUANTUM_DEFAULT_BYTES_PER_STEP` (64 MiB)
/// - Secure: unused (ignored)
fn resolve_bytes_per_step(mode: &CryptoMode, user_value: Option<u64>) -> u64 {
    user_value.unwrap_or(match mode {
        CryptoMode::Chaos => CHAOS_DEFAULT_BYTES_PER_STEP,
        CryptoMode::Photon => PHOTON_DEFAULT_BYTES_PER_STEP,
        CryptoMode::Quantum => QUANTUM_DEFAULT_BYTES_PER_STEP,
        CryptoMode::Secure => CHAOS_DEFAULT_BYTES_PER_STEP, // unused, but keep consistent
    })
}
