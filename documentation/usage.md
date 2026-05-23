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
# Default: Euler integration (faster chaos amplification, more entropy per step)
./kelvin encrypt --config my_secret.json --input database.tar --output database.tar.enc

# Verlet integration: symplectic, energy-conserving
./kelvin encrypt --config my_secret.json --input database.tar --output database.tar.enc --verlet
```

### Decrypt a File
Decryption is the exact inverse of encryption, using the same **Orbital Keystream**. The same config and integration method must be used.

```bash
# Default: Euler integration
./kelvin decrypt --config my_secret.json --input database.tar.enc --output database_restored.tar

# Verlet integration (must match encryption)
./kelvin decrypt --config my_secret.json --input database.tar.enc --output database_restored.tar --verlet
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

### Authenticated Encryption (`--auth`)
V2 (Chaos), V3 (Photon), and H (Quantum) modes are pure XOR stream ciphers with no built-in authentication. Append `--auth` to append a 32-byte KMAC128 tag (NIST SP 800-185) to the ciphertext, defeating malleability.

```bash
# Chaos mode with authentication
./kelvin encrypt --mode chaos --config my_secret.json --input file.txt --output file.enc --auth
./kelvin decrypt --mode chaos --config my_secret.json --input file.enc --output file.txt --auth

# Photon mode with authentication
./kelvin encrypt --mode photon --config my_secret.json --input file.txt --output file.enc --auth
./kelvin decrypt --mode photon --config my_secret.json --input file.enc --output file.txt --auth

# Quantum mode with authentication
./kelvin encrypt --mode quantum --config my_secret.json --input file.txt --output file.enc --auth
./kelvin decrypt --mode quantum --config my_secret.json --input file.enc --output file.txt --auth
```

> **Note:** The `--auth` flag is ignored in `secure` mode (V1 ChaCha20Poly1305 AEAD has built-in authentication).

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

### Integration Method: Euler vs Verlet

Kelvin supports two integration methods for the orbital simulation:

| Method | CLI Flag | Property | Best For |
|--------|----------|----------|----------|
| **Euler** (default) | *(none)* | 1st-order, numerically unstable, faster chaos amplification | Maximum entropy per step, shorter Lyapunov time |
| **Verlet** | `--verlet` | Symplectic, energy-conserving, physically realistic | Standard encryption, backward compatibility |

Euler's numerical instability amplifies chaos ~10x faster than Verlet, producing more entropy per CPU cycle. See [`Euler_vs_Verlet.md`](Euler_vs_Verlet.md) for the full theoretical analysis.

### Basic Encryption/Decryption
```rust
use kelvin::{IntegrationMethod, Kelvin, OrbitalConfig};
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

### Using Verlet Integration

To use Verlet integration instead of the default Euler, pass `IntegrationMethod::Verlet` to the constructor:

```rust
use kelvin::{IntegrationMethod, Kelvin, OrbitalConfig};

let config = OrbitalConfig::from_json(&config_json)?;

// Verlet integration: symplectic, energy-conserving
let mut k = Kelvin::new_with_method(config, IntegrationMethod::Verlet)?;

// Encrypt/decrypt works identically
let mut data = b"Hello Kelvin Chaos!".to_vec();
k.encrypt(&mut data)?;
```

The same applies to `KelvinStreaming`:

```rust
use kelvin::{IntegrationMethod, KelvinStreaming, OrbitalConfig};

let config = OrbitalConfig::from_json(&config_json)?;

// V2 streaming with Verlet integration
let mut ks = KelvinStreaming::new_with_method(config, 1024 * 1024, IntegrationMethod::Verlet)?;
ks.encrypt(&mut data)?;
```

### Authenticated Encryption (V2/V3/H + KMAC128)

For modes that lack built-in authentication (Chaos, Photon, Quantum), wrap the engine in its authenticated variant to append a 32-byte KMAC128 tag (NIST SP 800-185):

```rust
use kelvin::{
    KelvinPhotonAuthenticated, KelvinStreamingAuthenticated,
    KelvinQuantumAuthenticated, OrbitalConfig,
};

// V2 Chaos + KMAC128
let config = OrbitalConfig::from_json(&config_json)?;
let mut ks = KelvinStreamingAuthenticated::new(config, 1024 * 1024)?;
let mut data = b"Hello authenticated streaming!".to_vec();
ks.encrypt(&mut data)?; // ciphertext + 32-byte tag appended

// V3 Photon + KMAC128
let (seed, _bodies) = kelvin::simulate_and_extract_seed_with_method(
    &config, kelvin::IntegrationMethod::Verlet
)?;
let mut photon = KelvinPhotonAuthenticated::new(seed, 100_000);
let mut data = b"Hello authenticated photon!".to_vec();
photon.encrypt(&mut data)?;

// H Quantum + KMAC128
let mut quantum = KelvinQuantumAuthenticated::with_config(
    seed, 100_000, 1024 * 1024, 10_000, 10 * 1024 * 1024
)?;
let mut data = b"Hello authenticated quantum!".to_vec();
quantum.encrypt(&mut data)?;
```

> **Important:** The authenticated wrappers append a 32-byte KMAC128 tag to the ciphertext. During decryption, the tag is verified in constant time using `subtle::ConstantTimeEq`. If the tag is missing or tampered, decryption returns an error.

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

### Security Level Recommendations

Kelvin provides three security levels that control the number of bodies and simulation steps:

| Level | Bodies | Steps | Use Case |
|-------|--------|-------|----------|
| `standard` | 5 | 1,000,000 | General purpose — fast setup, strong chaos |
| `paranoid` | 5 | 10,000,000 | Sensitive data — deeper simulation, higher entropy |
| `maximum` | 10 | 100,000,000 | **Highest security** — maximum bodies and steps for strongest entropy |

**Recommendation:** Use `standard` for everyday encryption, `paranoid` for sensitive data, and `maximum` for the strongest possible security (note: `maximum` takes significantly longer to generate).

### Chaotic Regime Enforcement
Kelvin will refuse to initialize if the `total_steps` requested in the configuration are less than the **Lyapunov Horizon**.
- **Always use `keygen`** to create configurations, as it selects parameters that reach the chaotic regime within the requested steps.
- If you manually edit a JSON config, ensure the simulation time is long enough to fully "scramble" the state.

### Large Files and Reseeding
Kelvin automatically reseeds the keystream by advancing through the **key schedule's virtual step counter** (not by re-running the simulation).
- Large files (GBs) will trigger multiple reseeding events.
- If the key schedule is exhausted, the library will return a `SeedExhausted` error.
- Each key can safely encrypt ~4 GiB of data. The total safe encryption capacity is `max_keys × 4 GiB`.

### Integrity & Authentication
> [!CAUTION]
> V2 (Chaos), V3 (Photon), and H (Quantum) modes are **pure XOR stream ciphers** — they do not provide built-in message authentication. Without authentication, an attacker can flip ciphertext bits and cause predictable plaintext changes (malleability).
>
> **Use `--auth`** to append a 32-byte KMAC128 tag (NIST SP 800-185) to the ciphertext, defeating malleability. The tag is verified in constant time during decryption.
>
> V1 (Secure) mode uses ChaCha20Poly1305 AEAD and has built-in authentication — the `--auth` flag is ignored for this mode.

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

---

## 6. V2 Streaming Mode (Real-Time Per-Step Simulation)

V2 Streaming (`KelvinStreaming`) replaces the virtual-time key schedule with a true per-step simulation. Each chunk of data advances the orbital simulation by one simulation step (Verlet or Euler), extracting keystream from the current chaotic state.

### Key Differences from V1

| Feature | V1 (`Kelvin`) | V2 (`KelvinStreaming`) |
|---------|---------------|------------------------|
| Simulation | Runs once upfront | One step per chunk |
| Keystream | Finite (key schedule) | Unlimited (keep simulating) |
| Setup time | Seconds to minutes | Instant |
| Cipher | ChaCha20Poly1305 AEAD | XOR with SHAKE256 XOF |
| Authentication | AEAD tag | None (XOR only) |
| Key rotation | Automatic via schedule | N/A (each step is unique) |

### Rust API

```rust
use kelvin::{KelvinStreaming, OrbitalConfig};

// Create streaming instance (instant — no upfront simulation)
let config = OrbitalConfig::from_json(&config_json)?;
let mut ks = KelvinStreaming::new(config, 1024 * 1024)?; // 1 MiB per step

// Encrypt (advances simulation by 1 step per bytes_per_step chunk)
let mut data = b"Hello, V2 streaming!".to_vec();
ks.encrypt(&mut data)?;

// Decrypt with a new instance (same config = same keystream)
let config = OrbitalConfig::from_json(&config_json)?;
let mut ks2 = KelvinStreaming::new(config, 1024 * 1024)?;
ks2.decrypt(&mut data)?;
assert_eq!(&data, b"Hello, V2 streaming!");
```

### How `bytes_per_step` Works

The `bytes_per_step` parameter controls how many keystream bytes are produced per simulation step (Verlet or Euler). This is critical for understanding the streaming API's behavior:

- **Fixed-size chunking**: `process_chunk()` internally processes data in fixed-size chunks of `bytes_per_step` bytes, advancing the simulation by one step per chunk. This ensures the keystream is **deterministic regardless of caller chunking** — a 100-byte call and two 50-byte calls produce the same ciphertext for the same total bytes.
- **Partial chunks**: A final partial chunk still advances the simulation by one step, but only XORs the needed bytes. The remaining keystream bytes are discarded.
- **Consistency with `estimate_time`**: Both `process_chunk()` and `estimate_time()` use the same `bytes_per_step` field, so time estimates are always accurate regardless of chunk size.

```rust
// Example: bytes_per_step = 1024
let mut ks = KelvinStreaming::new(config, 1024)?;

// A 100-byte call → 1 simulation step (100 of 1024 keystream bytes used)
let mut small = vec![0u8; 100];
ks.encrypt(&mut small)?;

// A 2000-byte call → 2 simulation steps (1024 + 976 bytes)
let mut large = vec![0u8; 2000];
ks.encrypt(&mut large)?;

// Total: 3 steps, 2100 bytes processed
assert_eq!(ks.step(), 3);
assert_eq!(ks.bytes_processed(), 2100);
```

### Benchmarking and ETA

```rust
// Benchmark simulation speed
let rate = ks.benchmark(1000); // steps/sec

// Estimate time for a file
let (steps, seconds) = ks.estimate_time(file_size, rate);
println!("Need {steps} steps, ~{seconds:.1}s");
```

The `estimate_time` function divides the file size by `bytes_per_step` to compute the number of steps needed, then divides by the benchmark rate for the time estimate. This is always consistent with actual processing because both use the same `bytes_per_step` value.

### Example

```bash
cargo run --example simple_streaming -p kelvin
```

### 3D Orbital Visualizer

The demo kit includes a 3D orbital visualizer (`kelvin-demo/orbital_visualizer.html`) that provides real-time visualization of the n-body simulation:

- **Open in browser**: No server required — just open the HTML file.
- **Load config**: Use the file picker to load a `key.json` file and visualize its orbital dynamics.
- **Controls**: Drag to rotate, scroll to zoom, Space/P to pause, R to reset.
- **KDF pipeline display**: Hover over each stage (Extract, Expand, Encrypt, Sign) for detailed information about the cryptographic pipeline.
- **Performance**: Key material display is throttled to once per second to avoid redundant SHAKE256 calculations during high-speed simulation.

### Security Notes

- **V2 is a pure XOR stream cipher** — no authentication. Use a MAC for integrity.
- Each step produces `bytes_per_step` bytes of keystream from SHAKE256 XOF.
- The domain separator `b"kelvin-streaming-v2-v1-000000000"` ensures domain separation from V1.
- **Deterministic**: same config + same step count = same keystream, regardless of how the caller chunks the data (as long as total bytes processed is the same).
- **Unlimited keystream**: Unlike V1's finite key schedule, V2 can keep simulating indefinitely — there is no `SeedExhausted` error.
