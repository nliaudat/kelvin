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
Kelvin can derive a Curve25519 public key from the orbital state. This can be used to identify a peer or verify a configuration without sharing it.

```bash
./kelvin identify --config my_secret.json
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
println!("Public Key: {:x?}", kp.public_key.as_bytes());
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
