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
//! | Mode | Engine | Cipher | Auth |
//! |------|--------|--------|------|
//! | `secure` (V1, default) | `Kelvin` | ChaCha20Poly1305 AEAD | ✅ |
//! | `chaos` (V2) | `KelvinStreaming` | SHAKE256 XOR per-step | ❌ |
//! | `photon` (V3) | `KelvinPhoton` | HKDF→SHAKE256 XOR | ❌ |
//! | `quantum` (H) | `KelvinQuantum` | Hybrid cache+XOR + orbital reseed | ❌ |

#![deny(unsafe_code)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use kelvin::{
    simulate_and_extract_seed_with_method, Fixed, IntegrationMethod, Kelvin, KelvinPhoton,
    KelvinQuantum, KelvinStreaming, OrbitalBody, OrbitalConfig, OrbitalKeyPair, Vec3,
};
use ml_kem::KeyExport;
use rand::Rng;
use std::fs;
use std::io::{Read, Write};

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
    },
    /// Encrypt a file
    Encrypt {
        /// Cryptographic mode: secure, chaos, photon, or quantum
        #[arg(long, default_value = "secure")]
        mode: CryptoMode,
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
        /// Input file path
        #[arg(long)]
        input: String,
        /// Output file path
        #[arg(long)]
        output: String,
        /// Bytes of keystream per simulation step (chaos mode only, default 1MB)
        #[arg(long, default_value = "1048576")]
        bytes_per_step: u64,
        /// Use Euler integration instead of Verlet for maximum chaos amplification
        #[arg(long)]
        euler: bool,
    },
    /// Decrypt a file
    Decrypt {
        /// Cryptographic mode: secure, chaos, photon, or quantum
        #[arg(long, default_value = "secure")]
        mode: CryptoMode,
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
        /// Input file path
        #[arg(long)]
        input: String,
        /// Output file path
        #[arg(long)]
        output: String,
        /// Bytes of keystream per simulation step (chaos mode only, default 1MB)
        #[arg(long, default_value = "1048576")]
        bytes_per_step: u64,
        /// Use Euler integration instead of Verlet for maximum chaos amplification
        #[arg(long)]
        euler: bool,
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
        Commands::Keygen { level, output } => {
            let config = generate_config(&level)?;
            let json = config.to_json()?;
            if let Some(path) = output {
                fs::write(path, json).context("Failed to write config file")?;
                println!("Generated {} config.", level);
            } else {
                println!("{}", json);
            }
        },
        Commands::Encrypt { mode, config, input, output, bytes_per_step, euler } => {
            let method = if euler { IntegrationMethod::Euler } else { IntegrationMethod::Verlet };
            process_file_mode(&mode, &config, &input, &output, true, bytes_per_step, method)?;
        },
        Commands::Decrypt { mode, config, input, output, bytes_per_step, euler } => {
            let method = if euler { IntegrationMethod::Euler } else { IntegrationMethod::Verlet };
            process_file_mode(&mode, &config, &input, &output, false, bytes_per_step, method)?;
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
                println!("ML-KEM-768 (PQ-KEM):    {}", hex::encode(&kp.kem_public.to_bytes()));
                shown = true;
            }

            if all || (!ecc && !kem) {
                println!("ML-DSA-65  (PQ-Sig):    {}", hex::encode(&kp.dsa_public.to_bytes()));
                shown = true;
            }

            if !shown {
                println!("ML-DSA-65  (PQ-Sig):    {}", hex::encode(&kp.dsa_public.to_bytes()));
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

#[allow(clippy::disallowed_methods)]
fn generate_config(level: &str) -> Result<OrbitalConfig> {
    let mut rng = rand::thread_rng();
    let (n_bodies, steps) = match level {
        "standard" => (5, 1_000_000),
        "paranoid" => (5, 10_000_000),
        "maximum" => {
            eprintln!("Warning: 'maximum' level uses 10 bodies and 100,000,000 simulation steps.");
            eprintln!("This will take significantly longer than 'standard' or 'paranoid'.");
            (10, 100_000_000)
        },
        _ => {
            anyhow::bail!("Unknown security level: {}. Use standard, paranoid, or maximum.", level)
        },
    };

    let mut bodies = Vec::with_capacity(n_bodies);

    let sun_mass = loop {
        let raw: i128 = (1 << 64) + rng.gen_range(-(1i128 << 62)..(1i128 << 62) + 1);
        let m = Fixed::from_raw(raw);
        if m >= Fixed::from_raw(3 << 62) && m <= Fixed::from_raw(5 << 62) {
            break m;
        }
    };
    bodies.push(OrbitalBody::new(
        sun_mass,
        Vec3::new(
            Fixed::from_raw(rng.gen_range(1 << 20..1 << 30)),
            Fixed::from_raw(rng.gen_range(1 << 20..1 << 30)),
            Fixed::from_raw(rng.gen_range(1 << 20..1 << 30)),
        ),
        Vec3::new(
            Fixed::from_raw(rng.gen_range(1 << 10..1 << 20)),
            Fixed::from_raw(rng.gen_range(1 << 10..1 << 20)),
            Fixed::from_raw(rng.gen_range(1 << 10..1 << 20)),
        ),
    ));

    for i in 1..n_bodies {
        let radius = (i as i64 + 1) * 50;
        let mass = Fixed::from_raw(rng.gen_range(1 << 30..1 << 35));

        let theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let phi = (rng.gen_range(-1.0..1.0f64)).acos();

        let x = (radius as f64) * phi.sin() * theta.cos();
        let y = (radius as f64) * phi.sin() * theta.sin();
        let z = (radius as f64) * phi.cos();

        let v_theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let v_phi = (rng.gen_range(-1.0..1.0f64)).acos();
        let v_mag = 1.0 / (radius as f64).sqrt() * 6.3;

        let vx = v_mag * v_phi.sin() * v_theta.cos();
        let vy = v_mag * v_phi.sin() * v_theta.sin();
        let vz = v_mag * v_phi.cos();

        bodies.push(OrbitalBody::new(
            mass,
            Vec3::new(
                Fixed::from_raw((x * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((y * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((z * (1u128 << 64) as f64) as i128),
            ),
            Vec3::new(
                Fixed::from_raw((vx * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((vy * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((vz * (1u128 << 64) as f64) as i128),
            ),
        ));
    }

    OrbitalConfig::new(
        bodies,
        steps,
        steps / 10,
        Fixed::from_raw(1 << 54),
        Fixed::from_raw(1 << 48),
        kelvin::DEFAULT_G,
    )
    .map_err(|e| anyhow::anyhow!(e))
}

/// Dispatch to the correct processing function based on mode.
fn process_file_mode(
    mode: &CryptoMode,
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    bytes_per_step: u64,
    method: IntegrationMethod,
) -> Result<()> {
    match mode {
        CryptoMode::Secure => process_file_secure(config_path, input_path, output_path, encrypt, method),
        CryptoMode::Chaos => process_file_chaos(config_path, input_path, output_path, encrypt, bytes_per_step, method),
        CryptoMode::Photon => process_file_photon(config_path, input_path, output_path, encrypt, method),
        CryptoMode::Quantum => process_file_quantum(config_path, input_path, output_path, encrypt, method),
    }
}

/// V1 Secure: ChaCha20Poly1305 AEAD (original Kelvin).
fn process_file_secure(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    method: IntegrationMethod,
) -> Result<()> {
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_name = if method == IntegrationMethod::Euler { "Euler" } else { "Verlet" };
    println!("Initializing Kelvin Secure (V1, {} integration, this may take a few seconds)...", method_name);
    let mut k = Kelvin::new_with_method(config, method).context("Failed to initialize Kelvin")?;

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

    println!("Processing (ChaCha20Poly1305 AEAD)...");
    let mut buffer = vec![0u8; 64 * 1024 + 16];
    let mut total_processed = 0u64;
    loop {
        let bytes_read = input_file.read(&mut buffer[..64 * 1024])?;
        if bytes_read == 0 {
            break;
        }

        if encrypt {
            k.encrypt(&mut buffer[..bytes_read])?;
        } else {
            k.decrypt(&mut buffer[..bytes_read])?;
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

/// V2 Chaos: Per-step SHAKE256 XOR streaming.
fn process_file_chaos(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    bytes_per_step: u64,
    method: IntegrationMethod,
) -> Result<()> {
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_name = if method == IntegrationMethod::Euler { "Euler" } else { "Verlet" };
    println!("Initializing Kelvin Chaos (V2, {} integration, instant setup)...", method_name);
    let mut ks = KelvinStreaming::new_with_method(config, bytes_per_step, method)
        .context("Failed to initialize KelvinStreaming")?;

    let rate = ks.benchmark(100);
    let file_size = fs::metadata(input_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let (steps_needed, est_secs) = ks.estimate_time(file_size, rate);
    if file_size > 0 {
        println!("Estimated: {} steps, ~{:.1}s ({:.0} steps/sec)", steps_needed, est_secs, rate);
    }

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

    println!("Processing (SHAKE256 XOR)...");
    let mut buffer = vec![0u8; 64 * 1024];
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
fn process_file_photon(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    method: IntegrationMethod,
) -> Result<()> {
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_name = if method == IntegrationMethod::Euler { "Euler" } else { "Verlet" };
    println!("Initializing Kelvin Photon (V3, {} integration, running orbital simulation)...", method_name);
    let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
        .context("Failed to run orbital simulation")?;

    let mut photon = KelvinPhoton::new(seed, 100_000);

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

    println!("Processing (HKDF→SHAKE256 XOR)...");
    let mut buffer = vec![0u8; 64 * 1024];
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
fn process_file_quantum(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    method: IntegrationMethod,
) -> Result<()> {
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_name = if method == IntegrationMethod::Euler { "Euler" } else { "Verlet" };
    println!("Initializing Kelvin Quantum (H, {} integration, running orbital simulation)...", method_name);
    let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
        .context("Failed to run orbital simulation")?;

    let mut quantum = KelvinQuantum::with_config(seed, 100_000, 1024 * 1024, 10_000, 10 * 1024 * 1024)
        .context("Failed to initialize KelvinQuantum")?;

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

    println!("Processing (Hybrid cache+XOR + orbital reseed)...");
    let mut buffer = vec![0u8; 64 * 1024];
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

fn run_benchmark() -> Result<()> {
    println!("Running Kelvin Benchmarks...");
    let levels = ["standard", "paranoid", "maximum"];

    for level in levels {
        println!("\nLevel: {}", level);
        let config = generate_config(level)?;
        let start = std::time::Instant::now();
        let _ = Kelvin::new(config)?;
        let duration = start.elapsed();
        println!("  Setup Time: {:?}", duration);

        let mut data = vec![0u8; 1024 * 1024];
        let mut k = Kelvin::new(generate_config(level)?)?;
        let start = std::time::Instant::now();
        k.encrypt(&mut data)?;
        let duration = start.elapsed();
        println!("  Encryption Throughput (ChaCha20): {:.2} MB/s", 1.0 / duration.as_secs_f64());
    }

    Ok(())
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
