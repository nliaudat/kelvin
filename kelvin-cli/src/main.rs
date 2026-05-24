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
//! Use `--auth` to append a 32-byte KMAC128 tag (NIST SP 800-185) to defeat
//! ciphertext malleability. The `secure` mode has built-in AEAD authentication
//! and ignores the `--auth` flag.
//!
//! Integration defaults to Verlet (energy-conserving). Use `--euler`
//! for numerically unstable integration (faster chaos amplification).

#![deny(unsafe_code)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use kelvin::{
    simulate_and_extract_seed_with_method, Fixed, IntegrationMethod, Kelvin, KelvinPhoton,
    KelvinPhotonAuthenticated, KelvinQuantum, KelvinQuantumAuthenticated, KelvinStreaming,
    KelvinStreamingAuthenticated, OrbitalBody, OrbitalConfig, OrbitalKeyPair, Vec3,
};
use kelvin::{
    CHAOS_DEFAULT_BYTES_PER_STEP, DEFAULT_BYTES_PER_STEP, MAXIMUM_BODIES, MAXIMUM_STEPS,
    ORBITAL_VELOCITY_CONSTANT, PARANOID_BODIES, PARANOID_STEPS, PHOTON_DEFAULT_BYTES_PER_STEP,
    PHOTON_DEFAULT_MAX_RESEEDS, PLANET_MASS_MAX_RAW, PLANET_MASS_MIN_RAW, PLANET_RADIUS_MULTIPLIER,
    QUANTUM_DEFAULT_BYTES_PER_STEP, STANDARD_BODIES, STANDARD_STEPS, STREAMING_CHUNK_SIZE,
    SUN_MASS_CENTER, SUN_MASS_MAX_RAW, SUN_MASS_MIN_RAW, SUN_MASS_RANGE, SUN_POS_MAX_RAW,
    SUN_POS_MIN_RAW, SUN_VEL_MAX_RAW, SUN_VEL_MIN_RAW,
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
        /// Bytes of keystream per step/chunk (chaos/photon/quantum, mode-specific default)
        #[arg(long)]
        bytes_per_step: Option<u64>,
        /// Use Euler integration instead of default Verlet (numerically unstable, faster chaos)
        #[arg(long)]
        euler: bool,
        /// Append a 32-byte KMAC128 tag for authentication (chaos, photon, quantum modes)
        #[arg(long)]
        auth: bool,
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
        /// Bytes of keystream per step/chunk (chaos/photon/quantum, mode-specific default)
        #[arg(long)]
        bytes_per_step: Option<u64>,
        /// Use Euler integration instead of default Verlet (numerically unstable, faster chaos)
        #[arg(long)]
        euler: bool,
        /// Verify and strip the 32-byte KMAC128 tag for authentication (chaos, photon, quantum modes)
        #[arg(long)]
        auth: bool,
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
        Commands::Encrypt { mode, config, input, output, bytes_per_step, euler, auth } => {
            let method = if euler { IntegrationMethod::Euler } else { IntegrationMethod::Verlet };
            let bps = resolve_bytes_per_step(&mode, bytes_per_step);
            process_file_mode(&mode, &config, &input, &output, true, bps, method, auth)?;
        },
        Commands::Decrypt { mode, config, input, output, bytes_per_step, euler, auth } => {
            let method = if euler { IntegrationMethod::Euler } else { IntegrationMethod::Verlet };
            let bps = resolve_bytes_per_step(&mode, bytes_per_step);
            process_file_mode(&mode, &config, &input, &output, false, bps, method, auth)?;
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
        "standard" => (STANDARD_BODIES, STANDARD_STEPS),
        "paranoid" => (PARANOID_BODIES, PARANOID_STEPS),
        "maximum" => {
            eprintln!(
                "Warning: 'maximum' level uses {} bodies and {} simulation steps.",
                MAXIMUM_BODIES, MAXIMUM_STEPS
            );
            eprintln!("This will take significantly longer than 'standard' or 'paranoid'.");
            (MAXIMUM_BODIES, MAXIMUM_STEPS)
        },
        _ => {
            anyhow::bail!("Unknown security level: {}. Use standard, paranoid, or maximum.", level)
        },
    };

    let mut bodies = Vec::with_capacity(n_bodies);

    let sun_mass = loop {
        let raw: i128 = SUN_MASS_CENTER + rng.gen_range(-SUN_MASS_RANGE..SUN_MASS_RANGE + 1);
        let m = Fixed::from_raw(raw);
        if m >= Fixed::from_raw(SUN_MASS_MIN_RAW) && m <= Fixed::from_raw(SUN_MASS_MAX_RAW) {
            break m;
        }
    };
    bodies.push(OrbitalBody::new(
        sun_mass,
        Vec3::new(
            Fixed::from_raw(rng.gen_range(SUN_POS_MIN_RAW..SUN_POS_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_POS_MIN_RAW..SUN_POS_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_POS_MIN_RAW..SUN_POS_MAX_RAW)),
        ),
        Vec3::new(
            Fixed::from_raw(rng.gen_range(SUN_VEL_MIN_RAW..SUN_VEL_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_VEL_MIN_RAW..SUN_VEL_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_VEL_MIN_RAW..SUN_VEL_MAX_RAW)),
        ),
    ));

    for i in 1..n_bodies {
        let radius = (i as i64 + 1) * PLANET_RADIUS_MULTIPLIER;
        let mass = Fixed::from_raw(rng.gen_range(PLANET_MASS_MIN_RAW..PLANET_MASS_MAX_RAW));

        let theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let phi = (rng.gen_range(-1.0..1.0f64)).acos();

        let x = (radius as f64) * phi.sin() * theta.cos();
        let y = (radius as f64) * phi.sin() * theta.sin();
        let z = (radius as f64) * phi.cos();

        let v_theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let v_phi = (rng.gen_range(-1.0..1.0f64)).acos();
        let v_mag = 1.0 / (radius as f64).sqrt() * ORBITAL_VELOCITY_CONSTANT;

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
        kelvin::DEFAULT_DT,
        kelvin::SOFTENING_FACTOR,
        kelvin::DEFAULT_G,
    )
    .map_err(|e| anyhow::anyhow!(e))
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

/// Dispatch to the correct processing function based on mode.
#[allow(clippy::too_many_arguments)]
fn process_file_mode(
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
fn process_file_secure(
    config_path: &str,
    input_path: &str,
    output_path: &str,
    encrypt: bool,
    method: IntegrationMethod,
) -> Result<()> {
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    println!(
        "Initializing Kelvin Secure (V1, {} integration, this may take a few seconds)...",
        method_label
    );
    let mut k = Kelvin::new(config).context("Failed to initialize Kelvin")?;

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

    println!("Processing (ChaCha20Poly1305 AEAD)...");
    let mut buffer = vec![0u8; STREAMING_CHUNK_SIZE + 16];
    let mut total_processed = 0u64;
    loop {
        let bytes_read = input_file.read(&mut buffer[..STREAMING_CHUNK_SIZE])?;
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
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
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

        let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
        let mut output_file =
            fs::File::create(output_path).context("Failed to create output file")?;

        println!("Processing (SHAKE256 XOR + KMAC128)...");
        let chunk_size = STREAMING_CHUNK_SIZE;
        // During encryption, each plaintext chunk produces ciphertext + 32-byte tag.
        // During decryption, we need to read ciphertext + tag in one shot.
        let read_size = if encrypt { chunk_size } else { chunk_size + 32 };
        let mut buffer = vec![0u8; read_size];
        let mut chunk = Vec::with_capacity(chunk_size + 32);
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
    let file_size = fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);
    let (steps_needed, est_secs) = ks.estimate_time(file_size, rate);
    if file_size > 0 {
        println!("Estimated: {} steps, ~{:.1}s ({:.0} steps/sec)", steps_needed, est_secs, rate);
    }

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

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
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    let auth_label = if auth { " + KMAC128" } else { "" };
    println!(
        "Initializing Kelvin Photon (V3, {} integration, running orbital simulation{})...",
        method_label, auth_label
    );
    let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
        .context("Failed to run orbital simulation")?;

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

    let cipher_label = if auth { "HKDF→SHAKE256 XOR + KMAC128" } else { "HKDF→SHAKE256 XOR" };
    println!("Processing ({})...", cipher_label);

    if auth {
        let chunk_size = bytes_per_step as usize;
        let mut photon = KelvinPhotonAuthenticated::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
        // During encryption, each plaintext chunk produces ciphertext + 32-byte tag.
        // During decryption, we need to read ciphertext + tag in one shot.
        let read_size = if encrypt { chunk_size } else { chunk_size + 32 };
        let mut buffer = vec![0u8; read_size];
        let mut chunk = Vec::with_capacity(chunk_size + 32);
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
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    let method_label = if method == IntegrationMethod::Verlet { "Verlet" } else { "Euler" };
    let auth_label = if auth { " + KMAC128" } else { "" };
    println!(
        "Initializing Kelvin Quantum (H, {} integration, running orbital simulation{})...",
        method_label, auth_label
    );
    let (seed, _bodies) = simulate_and_extract_seed_with_method(&config, method)
        .context("Failed to run orbital simulation")?;

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;

    let cipher_label = if auth {
        "Hybrid cache+XOR + orbital reseed + KMAC128"
    } else {
        "Hybrid cache+XOR + orbital reseed"
    };
    println!("Processing ({})...", cipher_label);

    if auth {
        let chunk_size = bytes_per_step as usize;
        let mut quantum = KelvinQuantumAuthenticated::new(seed, PHOTON_DEFAULT_MAX_RESEEDS);
        // During encryption, each plaintext chunk produces ciphertext + 32-byte tag.
        // During decryption, we need to read ciphertext + tag in one shot.
        let read_size = if encrypt { chunk_size } else { chunk_size + 32 };
        let mut buffer = vec![0u8; read_size];
        let mut chunk = Vec::with_capacity(chunk_size + 32);
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

        let mut data = vec![0u8; DEFAULT_BYTES_PER_STEP as usize];
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
