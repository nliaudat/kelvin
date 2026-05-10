# Kelvin Usage Guide

Kelvin is an orbital-chaos-based Key Derivation Function (KDF) and stream cipher. This guide covers how to use the CLI tool and how to integrate the library into your Rust projects.

---

## 1. Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (1.70 or later)
- Git

### Build the CLI
```bash
git clone https://github.com/nliaudat/kelvin
cd kelvin
cargo build --release -p kelvin-cli
```
The binary will be located at `target/release/kelvin`.

---

## 2. CLI Usage

### Generate an Orbital Configuration
The configuration is your **Shared Secret**. It contains the planetary parameters and simulation settings.

```bash
# Security Levels: standard, paranoid, maximum
./kelvin keygen --level standard --output my_secret.json
```

### Encrypt a File
Kelvin uses the orbital simulation to generate a chaotic keystream for encryption.

```bash
./kelvin encrypt --config my_secret.json --input database.tar --output database.tar.enc
```

### Decrypt a File
Decryption is the exact inverse of encryption. The same config must be used.

```bash
./kelvin decrypt --config my_secret.json --input database.tar.enc --output database_restored.tar
```

### Identity (Asymmetric Keys)
Kelvin derives multiple Post-Quantum (PQ) and classical asymmetric identities from the orbital state. This allows for quantum-safe identification of peers.

```bash
# Default: Show ML-DSA-65 (Post-Quantum Signature Identity)
./kelvin identify --config my_secret.json

# Show all identities (ML-DSA-65, ML-KEM-768, and Curve25519)
./kelvin identify --config my_secret.json --all

# Show specific identities
./kelvin identify --config my_secret.json --ecc --kem
```

---

## 3. Rust API Usage

Add `kelvin` to your `Cargo.toml`:
```toml
[dependencies]
kelvin = { git = "https://github.com/nliaudat/kelvin", features = ["serde"] }
```

### Basic Encryption/Decryption
```rust
use kelvin::{Kelvin, OrbitalConfig};
use std::fs;

fn main() -> anyhow::Result<()> {
    // 1. Load configuration
    let config_json = fs::read_to_string("my_secret.json")?;
    let config = OrbitalConfig::from_json(&config_json)?;

    // 2. Initialize Kelvin (Runs Lyapunov estimation and initial simulation)
    let mut k = Kelvin::new(config)?;

    // 3. Encrypt data in-place
    let mut data = b"Hello Kelvin Chaos!".to_vec();
    k.encrypt(&mut data)?;
    
    println!("Encrypted: {:02x?}", data);

    // 4. Decrypt (requires a fresh instance with the same config)
    let config = OrbitalConfig::from_json(&config_json)?;
    let mut k2 = Kelvin::new(config)?;
    k2.decrypt(&mut data)?;

    assert_eq!(&data, b"Hello Kelvin Chaos!");
    Ok(())
}
```

### Accessing Asymmetric Keys
```rust
let kp = k.asymmetric_keypair();
// Classical
println!("Curve25519: {:x?}", kp.curve_public.as_bytes());
// Post-Quantum
println!("ML-KEM-768: {:x?}", kp.kem_public.to_bytes());
println!("ML-DSA-65:  {:x?}", kp.dsa_public.to_bytes());
```

---

## 4. Security Recommendations

### Chaotic Regime Enforcement
Kelvin will refuse to initialize if the `total_steps` requested in the configuration are less than the **Lyapunov Horizon**.
- **Always use `keygen`** to create configurations, as it selects parameters that reach the chaotic regime within the requested steps.
- If you manually edit a JSON config, ensure the simulation time is long enough to fully "scramble" the state.

### Large Files and Reseeding
Kelvin automatically reseeds the keystream by advancing the orbital simulation. 
- Large files (GBs) will trigger multiple reseeding events.
- If the simulation time is exhausted, the library will return a `SeedExhausted` error.

### Integrity & AEAD
> [!CAUTION]
> Kelvin is a **stream cipher**, not an AEAD. It does not provide built-in message authentication. For production use, wrap the output in a MAC (like HMAC-SHA256) to prevent tampering.
