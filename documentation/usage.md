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
Kelvin uses the orbital simulation to generate a chaotic **Orbital Keystream** for encryption.

```bash
./kelvin encrypt --config my_secret.json --input database.tar --output database.tar.enc
```

### Decrypt a File
Decryption is the exact inverse of encryption, using the same **Orbital Keystream**. The same config must be used.

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

### Analyze Configuration Quality
Verify that a configuration has sufficient chaos and sensitivity to initial conditions.
```bash
./kelvin analyze --config my_secret.json
```
This performs a bit-flip on the Sun mass and measures the **Avalanche Effect** on the resulting cryptographic identity.
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
Kelvin automatically reseeds the keystream by advancing through the **key schedule's virtual step counter** (not by re-running the simulation).
- Large files (GBs) will trigger multiple reseeding events.
- If the key schedule is exhausted, the library will return a `SeedExhausted` error.
- Each key can safely encrypt ~4 GiB of data. The total safe encryption capacity is `max_keys × 4 GiB`.

### Integrity & AEAD
> [!CAUTION]
> Kelvin is a **stream cipher**, not an AEAD. It does not provide built-in message authentication. For production use, wrap the output in a MAC (like HMAC-SHA256) to prevent tampering.

---

## 5. Understanding Time Evolution

Kelvin's KDF pipeline operates in two distinct time domains. Understanding the difference is critical for correct usage.

### 5.1 Real Time (Orbital Simulation)

The **real simulation** runs once during `Kelvin::new()`:

```
OrbitalConfig → simulate_with_monitoring(bodies, total_steps)
                ↓
                Final orbital state (positions, velocities, forces)
                ↓
                extract_seed(state) → 2048-byte entropy pool
```

- Runs for `config.total_steps` iterations of Verlet integration
- Monitored for stability (ejections, collapses)
- The Lyapunov estimator verifies the system has entered the chaotic regime
- **This simulation runs exactly once** — it is never re-run during encryption

### 5.2 Virtual Time (Key Schedule)

The **key schedule** manages a virtual step counter that tracks how much of the orbital "time budget" has been consumed:

```
2048-byte seed → KeySchedule::new(seed, total_steps, reseed_interval, safe_steps)
                 ↓
                 max_keys = safe_steps / reseed_interval
                 ↓
                 Each next_key() call:
                   1. Derives key+nonce via HKDF-SHA512 from current seed
                   2. Advances virtual step: step += reseed_interval
                   3. Reseeds entropy pool via BLAKE3
                   4. Checks exhaustion: step >= safe_steps?
```

- **Virtual steps** are not real simulation iterations — they are a counter
- The schedule is **exhausted** when `step >= min(safe_steps, total_steps)`
- Once exhausted, no more keys can be derived (returns `None`)

### 5.3 The Dual Role of `min_chaos_steps`

The Lyapunov estimator computes `min_chaos_steps` — the minimum simulation steps needed to reach the chaotic regime. This value serves **two roles**:

| Role | Context | Meaning |
|------|---------|---------|
| **Lower bound** | Initialization check | `total_steps >= min_chaos_steps` — the simulation must run long enough to enter chaos |
| **Upper bound** | Key schedule (`safe_steps`) | Virtual steps cannot exceed `min_chaos_steps` — you cannot derive keys beyond the reliable horizon |

### 5.4 Practical Example

```
Configuration:
  total_steps = 200        (real simulation runs 200 steps)
  reseed_interval = 10     (each key consumes 10 virtual steps)
  min_chaos_steps = 73     (from Lyapunov estimation)

Key Schedule:
  max_keys = 73 / 10 = 7   (integer division)
  Key 1:  step=10,  keys=1,  ~28 GiB remaining
  Key 2:  step=20,  keys=2,  ~24 GiB remaining
  ...
  Key 7:  step=70,  keys=7,  ~0  GiB remaining → EXHAUSTED
```

### 5.5 Checking Remaining Capacity

```rust
let remaining = k.remaining_safe_bytes();  // bytes before exhaustion
```

This returns `remaining_keys × 4 GiB` (conservative estimate). When it reaches 0, the `Kelvin` instance can no longer encrypt or decrypt — you must create a new instance with a different configuration.
