//! Kelvin CLI — Orbital Chaos KDF Cryptosystem
//!
//! Provides a command-line interface for:
//! - Generating orbital configurations (Keygen)
//! - Encrypting/Decrypting files
//! - Inspecting public keys (Identify)
//! - Benchmarking

#![deny(unsafe_code)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use kelvin::{Kelvin, OrbitalConfig, OrbitalBody, Vec3, Fixed};
use rand::Rng;
use std::fs;
use std::io::{Read, Write};
use ml_kem::KeyExport;

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
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
        /// Input file path
        #[arg(long)]
        input: String,
        /// Output file path
        #[arg(long)]
        output: String,
    },
    /// Decrypt a file
    Decrypt {
        /// Path to orbital config JSON
        #[arg(long)]
        config: String,
        /// Input file path
        #[arg(long)]
        input: String,
        /// Output file path
        #[arg(long)]
        output: String,
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
        }
        Commands::Encrypt { config, input, output } => {
            process_file(&config, &input, &output, true)?;
        }
        Commands::Decrypt { config, input, output } => {
            process_file(&config, &input, &output, false)?;
        }
        Commands::Identify { config, all, ecc, kem } => {
            let config_json = fs::read_to_string(config).context("Failed to read config file")?;
            let config = OrbitalConfig::from_json(&config_json)?;
            let k = Kelvin::new(config).context("Failed to initialize Kelvin")?;
            let kp = k.asymmetric_keypair();
            
            let mut shown = false;

            if all || ecc {
                println!("Curve25519 (Classical): {}", hex::encode(kp.curve_public.as_bytes()));
                shown = true;
            }
            if all || kem {
                println!("ML-KEM-768 (PQ-KEM):    {}", hex::encode(&kp.kem_public.to_bytes()));
                shown = true;
            }
            
            // Default: Show ML-DSA-65
            if all || (!ecc && !kem) {
                println!("ML-DSA-65  (PQ-Sig):    {}", hex::encode(&kp.dsa_public.to_bytes()));
                shown = true;
            }
            
            if !shown {
                 println!("ML-DSA-65  (PQ-Sig):    {}", hex::encode(&kp.dsa_public.to_bytes()));
            }
        }
        Commands::Analyze { config } => {
            let config_json = fs::read_to_string(config).context("Failed to read config file")?;
            let mut config = OrbitalConfig::from_json(&config_json)?;
            
            println!("Analyzing Cryptographic Quality...");
            
            // 1. Base Key
            println!("Deriving Base Key...");
            let k1 = Kelvin::new(config.clone()).context("Failed to init first instance")?;
            let kp1 = k1.asymmetric_keypair();
            let key1 = kp1.curve_public.as_bytes();

            // 2. Flip 1 bit in input (Sun mass)
            println!("Flipping 1 bit in initial conditions...");
            config.bodies[0].mass = Fixed::from_raw(config.bodies[0].mass.to_raw() ^ 1);
            
            println!("Deriving Shadow Key...");
            let k2 = Kelvin::new(config).context("Failed to init shadow instance")?;
            let kp2 = k2.asymmetric_keypair();
            let key2 = kp2.curve_public.as_bytes();

            // 3. Compare (Avalanche)
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
            println!("Avalanche Effect: {} / 256 bits changed ({:.2}%)", 
                diff_bits, (diff_bits as f32 / 256.0) * 100.0);
            
            if diff_bits > 110 && diff_bits < 146 {
                println!("Result: PASS (Strong Avalanche Effect)");
            } else {
                println!("Result: FAIL (Weak sensitivity to initial conditions)");
            }
        }
        Commands::Benchmark => {
            run_benchmark()?;
        }
    }
    Ok(())
}

fn generate_config(level: &str) -> Result<OrbitalConfig> {
    let mut rng = rand::thread_rng();
    let (n_bodies, steps) = match level {
        "standard" => (3, 1_000_000),
        "paranoid" => (5, 10_000_000),
        "maximum" => (10, 100_000_000),
        _ => anyhow::bail!("Unknown security level: {}. Use standard, paranoid, or maximum.", level),
    };

    let mut bodies = Vec::with_capacity(n_bodies);
    
    // Sun near center with non-zero jitter
    bodies.push(OrbitalBody::new(
        Fixed::ONE,
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

    // Add planets at stable orbits with full 3D randomization
    for i in 1..n_bodies {
        let radius = (i as i64 + 1) * 50;
        let mass = Fixed::from_raw(rng.gen_range(1 << 30..1 << 35));
        
        // Uniform spherical sampling for position
        let theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let phi = (rng.gen_range(-1.0..1.0f64)).acos();
        
        let x = (radius as f64) * phi.sin() * theta.cos();
        let y = (radius as f64) * phi.sin() * theta.sin();
        let z = (radius as f64) * phi.cos();

        // Velocity: uniform direction, fixed magnitude
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
        steps / 10, // reseed interval
        Fixed::from_raw(1 << 54), // dt = 1/1024
        Fixed::from_raw(1 << 48), // Large softening for stability
        kelvin::DEFAULT_G,
    ).map_err(|e| anyhow::anyhow!(e))
}

fn process_file(config_path: &str, input_path: &str, output_path: &str, encrypt: bool) -> Result<()> {
    let config_json = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config = OrbitalConfig::from_json(&config_json)?;
    
    println!("Initializing Kelvin (this may take a few seconds)...");
    let mut k = Kelvin::new(config).context("Failed to initialize Kelvin")?;

    let mut input_file = fs::File::open(input_path).context("Failed to open input file")?;
    let _metadata = input_file.metadata()?;
    
    let mut output_file = fs::File::create(output_path).context("Failed to create output file")?;
    
    println!("Processing...");
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
    let mut total_processed = 0u64;
    loop {
        let bytes_read = input_file.read(&mut buffer)?;
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
        if total_processed % (1024 * 1024) == 0 {
             print!(".");
             let _ = std::io::stdout().flush();
        }
    }

    println!("\n{} complete.", if encrypt { "Encryption" } else { "Decryption" });
    Ok(())
}

fn run_benchmark() -> Result<()> {
    println!("Running Kelvin Benchmarks...");
    let levels = ["standard", "paranoid"];
    
    for level in levels {
        println!("\nLevel: {}", level);
        let config = generate_config(level)?;
        let start = std::time::Instant::now();
        let _ = Kelvin::new(config)?;
        let duration = start.elapsed();
        println!("  Setup Time: {:?}", duration);
        
        // Throughput test
        let mut data = vec![0u8; 1024 * 1024]; // 1MB
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
