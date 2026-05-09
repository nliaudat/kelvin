================================================================================
KELVIN: ORBITAL CHAOS KDF CRYPTOSYSTEM — IMPLEMENTATION PLAN
================================================================================

1. PROJECT OVERVIEW
================================================================================

Kelvin (backronym: KDF from n-body Lyapunov Instability Naturally) is a
cryptographic system that uses chaotic 3D n-body gravitational simulation
as a key derivation function (KDF). The orbital simulation generates
unpredictable seed material, which feeds a standard stream cipher (ChaCha20)
for high-speed keystream generation capable of encrypting terabytes of data.

  Property              Value
  ────────────────────  ──────────────────────────────────
  Language              Rust (with C FFI for mobile/IoT)
  Minimum Rust          1.75+ (stable)
  License               TBD
  Target Platforms      Linux, macOS, Windows, iOS, Android, WASM
  Security Target       256-bit equivalent

2. ARCHITECTURE
================================================================================

  Shared Secret (Orbital Parameters)
      │
      ▼
  ┌──────────────────┐     Lyapunov Time Calculator
  │  Validate Config  │────▶ estimates safe duration
  └──────────────────┘
      │
      ▼
  ┌──────────────────┐
  │  N-Body Engine   │  Fixed-point, deterministic
  │  (Phase 1 KDF)   │  Symplectic integrator
  └──────────────────┘  Sequential, non-parallelizable
      │
      ▼
  ┌──────────────────┐
  │  Master Seed     │  256-bit from hashed state
  └──────────────────┘
      │
      ▼
  ┌──────────────────┐
  │  Stream Cipher   │  ChaCha20 or AES-256-CTR
  │  (Phase 2 PRNG)  │  Hardware-accelerated
  └──────────────────┘  Terabytes output
      │
      ▼
  Keystream

3. CRATE STRUCTURE
================================================================================

kelvin/
├── kelvin-core/          # no-std, no-alloc, pure simulation
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── fixed_math.rs    # Fixed-point arithmetic (Q32.64)
│       ├── body.rs          # OrbitalBody struct
│       ├── integrator.rs    # Symplectic Verlet
│       ├── state.rs         # Simulation state machine
│       └── constants.rs     # Physical constants, scales
│
├── kelvin-kdf/           # Key derivation layer
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── config.rs        # OrbitalConfig parsing/validation
│       ├── lyapunov.rs      # Lyapunov time estimation
│       ├── extractor.rs     # Entropy extraction (SHA3-512)
│       └── schedule.rs      # Key schedule management
│
├── kelvin-stream/        # Stream cipher integration
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── chacha.rs        # ChaCha20 wrapper
│       ├── aes_ctr.rs       # AES-256-CTR wrapper (optional)
│       └── traits.rs        # StreamCipher trait
│
├── kelvin/               # Top-level orchestrator
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Kelvin struct, public API
│       ├── encrypt.rs       # Encryption interface
│       ├── decrypt.rs       # Decryption interface
│       └── error.rs         # Error types
│
├── kelvin-cli/           # CLI tool
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── keygen.rs        # Key generation
│       └── benchmark.rs     # Performance testing
│
├── kelvin-ffi/           # C/Android/iOS bindings
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── c_api.rs
│
├── tests/
│   ├── integration/
│   └── vectors/            # Known-answer test vectors
│
└── examples/
    ├── simple_encrypt.rs
    └── large_file.rs

4. FIXED-POINT MATH SPECIFICATION
================================================================================

Format: Q32.64 (96 bits total, stored as i128)
- 32 integer bits (supports values up to ~4 billion)
- 64 fractional bits (resolution ~5.4e-20)
- Stored as: i128 where 1.0 = 2^64

Why Q32.64:
- Enough range for AU-scale positions
- Enough precision for chaotic divergence
- 128-bit intermediates fit in CPU registers
- Deterministic across platforms (integer only)

Operations required:
  ✓ Addition / Subtraction
  ✓ Multiplication (with rounding)
  ✓ Division (with rounding)
  ✓ Integer square root (Newton's method, deterministic)
  ✓ Comparison (Eq, Ord)

Softening factor (ϵ):
  - Prevents singularities when bodies pass close to each other
  - Added to squared distance: a = G * m_j / (r² + ϵ²)^(3/2)
  - Recommended value: ϵ = 1e-6 AU (in Q32.64: 2^64 * 1e-6 ≈ 1.84e13)
  - Must be large enough to prevent overflow in 1/r² computation
  - Must be small enough to preserve chaotic dynamics
  - **Review note**: The softening factor directly affects both numerical
    stability and entropy quality. Too large → suppresses chaos. Too small
    → overflow risk. Must be validated with property-based tests across
    the full range of orbital configurations.

Range verification:
  - Q32.64 supports values up to ~4.29e9 (2^32 - 1)
  - Solar system scale: Oort cloud extends ~100,000 AU from Sun
  - Worst-case: body at 100,000 AU with velocity up to ~100 AU/yr
  - Position + velocity * dt must not overflow 32-bit integer part
  - **Review note**: Explicit overflow analysis needed for worst-case
    orbital configurations (high eccentricity, close encounters).
    Property-based tests should verify no overflow path exists.

Anti-features (excluded):
  ✗ Trig functions (not needed for Verlet integration)
  ✗ Transcendental functions
  ✗ Floating-point conversion paths in core engine

5. ORBITAL BODY DATA STRUCTURE
================================================================================

  struct OrbitalBody {
      mass: Fixed,                    // 16 bytes (i128)
      position: Vec3 { x, y, z },     // 48 bytes (3 × i128)
      velocity: Vec3 { x, y, z },     // 48 bytes (3 × i128)
  }                                   // Total: 112 bytes per body

  Memory budget (5 bodies): 560 bytes
  Memory budget (7 bodies): 784 bytes

6. SYMPLECTIC INTEGRATOR
================================================================================

Algorithm: Kick-Drift-Kick (Leapfrog Verlet)

  1. Compute accelerations for all bodies at current positions
  2. Kick: v += a * (dt / 2)
  3. Drift: x += v * dt
  4. Recompute accelerations at new positions
  5. Kick: v += a_new * (dt / 2)

Properties:
  - Symplectic (preserves phase space volume)
  - Time-reversible (for validation)
  - Energy error bounded (no secular drift)
  - Second-order accurate
  - Requires only one acceleration computation per step (after
    initial step, accelerations can be reused)

Acceleration computation:
  - All pairs O(n^2) for n <= 7 bodies
  - No tree codes (unnecessary for small n)
  - Softening factor to prevent singularities
  - Newton's third law applied (compute force once per pair)

7. LYAPUNOV TIME ESTIMATION
================================================================================

Purpose: Determine safe simulation duration before chaotic divergence
         makes the system physically meaningless.

Method: Shadow Orbit
  1. Clone initial state
  2. Perturb one body's position by ε = 1e-10 (minimum detectable)
  3. Run both simulations in parallel
  4. Measure phase-space separation at each step
  5. Fit exponential: separation(t) = ε0 * exp(λt)
  6. Lyapunov time τ = 1/λ
  7. Safe steps = τ / (10 * dt)  [10% safety margin]

Exponential fitting methodology:
  - Collect separation measurements at regular intervals (every 1000 steps)
  - Compute log(separation) for each measurement point
  - Apply linear least-squares regression: log(s) = log(ε0) + λt
  - λ = slope of the fitted line
  - τ = 1/λ
  - **Review note**: The Lyapunov exponent is a statistical quantity with
    inherent uncertainty. The 10% safety margin is a heuristic — it should
    be validated against known chaotic systems (e.g., the Hénon-Heiles
    system or a 3-body problem with known Lyapunov time). Consider
    reporting a confidence interval rather than a single value.

Output:
  - lyapunov_time: Fixed
  - safe_steps: u64
  - divergence_history: Vec<Fixed> (optional, for analysis)

Validation requirement:
  - Config must have total_steps <= safe_steps
  - Reject config if insufficient Lyapunov time
  - Return clear error with requested vs safe values
  - **Review note**: For small-n systems (3-7 bodies) in specific
    configurations, Lyapunov time estimation has significant variance.
    The estimator should be validated against known reference systems
    during development.

8. ENTROPY EXTRACTION
================================================================================

Hash function: SHA3-512 (Keccak)

Why SHA3-512:
  - Sponge construction (different from SHA2 family)
  - Hardware acceleration on ARMv8.4+
  - No length-extension attacks
  - 512-bit output matches maximum security target

Extraction algorithm:
  1. Hash step counter (u64, little-endian)
  2. Hash domain separator: b"kelvin-orbital-state-v1"
  3. For each body:
     a. Hash mass.to_le_bytes()
     b. Hash position.x.to_le_bytes()
     c. Hash position.y.to_le_bytes()
     d. Hash position.z.to_le_bytes()
     e. Hash velocity.x.to_le_bytes()
     f. Hash velocity.y.to_le_bytes()
     g. Hash velocity.z.to_le_bytes()
  4. Finalize hash -> [u8; 64] full seed
  5. Master key = full_seed[..32] (first 256 bits)

Entropy quality considerations:
  - SHA3-512's sponge construction provides good diffusion, so
    neighboring orbital states produce uncorrelated hash outputs
  - **Review note**: The effective entropy per extraction depends on
    how much the system has diverged between extractions. At small
    step intervals, consecutive orbital states may be correlated.
    The domain separator and step counter in the hash input mitigate
    this, but the reseed interval should be large enough that the
    orbital state has evolved significantly between extractions.
  - Recommended minimum reseed interval: at least 1000 steps to
    ensure sufficient phase-space divergence between extractions.

9. KEY SCHEDULE
================================================================================

Reseeding strategy:
  - Maximum 2^32 ChaCha20 blocks per seed (~256 GB)
  - Before limit: run orbital simulation forward by reseed_interval steps
  - Extract new 256-bit seed from evolved orbital state
  - Reinitialize stream cipher with new seed and incremented nonce
  - Continue encryption seamlessly

State machine:
  INITIALIZED -> SEEDING -> STREAMING -> RESEEDING -> STREAMING -> ...
                                              │
                                              ▼
                                        EXHAUSTED (safe steps reached)

Exhaustion behavior:
  - Return KelvinError::OrbitalTimeExhausted
  - Provide remaining safe bytes count
  - Do not attempt to continue (security boundary)

Reseeding rationale:
  - The 256 GB ChaCha20 limit per key/nonce pair is the binding constraint
  - Reseeding before this limit provides forward secrecy: compromising
    the current seed does not reveal past keystream
  - **Review note**: Reseeding is primarily a forward secrecy measure
    rather than an entropy requirement. Each extraction produces a full
    256-bit key from SHA3-512, which is sufficient for the full 256 GB
    ChaCha20 window. The reseed interval can be tuned based on the
    desired forward secrecy granularity.

10. STREAM CIPHER (PHASE 2)
================================================================================

Primary: ChaCha20
  - 256-bit key
  - 96-bit nonce (64-bit counter + 32-bit zero padding)
  - 64-byte blocks
  - Safe up to 256 GB per key/nonce pair

Implementation:
  - Use chacha20 crate (audited, widely deployed)
  - Buffer partial blocks for efficiency
  - Zeroize buffers after use

Fallback option: AES-256-CTR
  - Requires hardware acceleration (AES-NI or ARM Crypto Extensions)
  - Feature-gated behind "aes-ni" feature flag

11. PUBLIC API DESIGN
================================================================================

  /// Main entry point
  impl Kelvin {
      /// Create from validated configuration
      pub fn new(config: OrbitalConfig) -> Result<Self, KelvinError>;

      /// Encrypt data in-place (XOR with keystream)
      pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError>;

      /// Decrypt data in-place (identical to encrypt)
      pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError>;

      /// Bytes encrypted since initialization
      pub fn bytes_processed(&self) -> u64;

      /// Remaining safe bytes before exhaustion
      pub fn remaining_safe_bytes(&self) -> u64;
  }

  /// Configuration from shared secret
  impl OrbitalConfig {
      /// Parse from JSON
      pub fn from_json(json: &str) -> Result<Self, KelvinError>;

      /// Parse from binary format
      pub fn from_bytes(bytes: &[u8]) -> Result<Self, KelvinError>;

      /// Serialize to JSON
      pub fn to_json(&self) -> String;

      /// Serialize to compact binary
      pub fn to_bytes(&self) -> Vec<u8>;

      /// Estimate setup time on current hardware
      pub fn estimate_setup_time(&self) -> std::time::Duration;
  }

12. SECURITY LEVELS
================================================================================

  Level 1: STANDARD (messaging, < 1 GB)
  ─────────────────────────────────────
  Bodies:             3 (central + 2 planets)
  Simulation steps:   1,000,000
  Extract every:      10,000 steps
  Setup time:         ~100-500 ms
  Key material:       672 bits (21 params × 32-bit)
  Lyapunov margin:    10% (100K safe steps)
  Max keystream:      ~2.5 TB (100 extracts × 256 GB)

  Level 2: PARANOID (files, < 1 TB)
  ─────────────────────────────────────
  Bodies:             5 (central + 4 planets)
  Simulation steps:   10,000,000
  Extract every:      100,000 steps
  Setup time:         ~1-5 seconds
  Key material:       1120 bits (35 params × 32-bit)
  Lyapunov margin:    10% (1M safe steps)
  Max keystream:      ~25 TB (100 extracts × 256 GB)

  Level 3: MAXIMUM (long-term secrets, multi-TB)
  ─────────────────────────────────────
  Bodies:             7 (central + 6 planets)
  Simulation steps:   100,000,000
  Extract every:      1,000,000 steps
  Setup time:         ~30-120 seconds
  Key material:       1568 bits (49 params × 32-bit)
  Lyapunov margin:    10% (10M safe steps)
  Max keystream:      ~250 TB (100 extracts × 256 GB)

13. KEY GENERATION ALGORITHM
================================================================================

Random planet generator constraints:
  - Semi-major axis: 0.1 to 100 AU
  - Eccentricity: 0.0 to 0.9
  - Inclination: 0 to 180 degrees (full 3D)
  - Mass: 0.001 to 10 Jupiter masses
  - Position derived from orbital elements
  - Central body: 1 solar mass at origin

Procedure:
  1. Seed CSPRNG from OS entropy (getrandom)
  2. Generate n-1 planets with bounded random parameters
  3. Convert orbital elements to Cartesian state vectors
  4. Validate system is not immediately unstable
  5. Run Lyapunov estimator
  6. Reject and retry if safe_steps < minimum threshold
  7. Package into OrbitalConfig with all simulation parameters
  8. Output as JSON or compact binary

Stability criteria:
  - "Immediately unstable" defined as: any body reaching escape
    velocity (> sqrt(2 * G * M_central / r)) within the first
    10,000 simulation steps
  - Minimum safe_steps threshold: at least 2× the requested
    simulation steps (to allow for the 10% Lyapunov margin)
  - **Review note**: The rejection rate for randomly generated
    configs should be estimated during development. If >50% of
    random configs are rejected, the parameter bounds may need
    adjustment or the retry logic should be optimized.

OrbitalConfig wire format (compact binary):
  - All multi-byte values are little-endian (matching SHA3 extraction)
  - No padding bytes between fields (packed struct layout)
  - Version byte at offset 0 for forward compatibility

  Offset  Size  Field
  ──────  ────  ──────────────────────────────────────────────
  0       1     Version (0x01)
  1       1     Number of bodies (n, 3-7)
  2       8     Total simulation steps (u64)
  10      8     Reseed interval steps (u64)
  18      8     Time step dt in Q32.64 (Fixed, 16 bytes)
  26      8     Softening factor ϵ in Q32.64 (Fixed, 16 bytes)
  34      16    ┐
  50      16    │ Per-body data, repeated n times:
  66      16    │   mass (Fixed, 16 bytes)
  82      16    │   position.x (Fixed, 16 bytes)
  98      16    │   position.y (Fixed, 16 bytes)
  114     16    │   position.z (Fixed, 16 bytes)
  130     16    │   velocity.x (Fixed, 16 bytes)
  146     16    │   velocity.y (Fixed, 16 bytes)
  162     16    ┘   velocity.z (Fixed, 16 bytes)
  ──────  ────
  34+112n Total binary size (n bodies)

  Example sizes:
    n=3:  34 + 336 = 370 bytes
    n=5:  34 + 560 = 594 bytes
    n=7:  34 + 784 = 818 bytes

  JSON format:
    - Same fields, human-readable
    - Fixed values serialized as hex strings (32 hex chars)
    - Includes version field for schema migration

14. DEPENDENCIES
================================================================================

  kelvin-core:
    (none — pure integer math)

  kelvin-kdf:
    kelvin-core   { path = "../kelvin-core" }
    sha3          { version = "0.10", default-features = false }
    zeroize       { version = "1.6", features = ["zeroize_derive"] }

  kelvin-stream:
    chacha20      { version = "0.9" }
    zeroize       { version = "1.6" }
    aes + ctr     { optional, for aes-ni feature }

  kelvin:
    kelvin-core   { path = "../kelvin-core" }
    kelvin-kdf    { path = "../kelvin-kdf" }
    kelvin-stream { path = "../kelvin-stream" }
    thiserror     { version = "1.0" }
    serde         { version = "1.0", features = ["derive"] }
    serde_json    { version = "1.0", optional = true }
    zeroize       { version = "1.6" }

  kelvin-cli:
    kelvin        { path = "../kelvin" }
    clap          { version = "4.0", features = ["derive"] }
    serde_json    { version = "1.0" }
    indicatif     { version = "0.17" }  // Progress bars for long ops

  kelvin-ffi:
    kelvin        { path = "../kelvin" }
    (no additional — pure C ABI)

15. ERROR TYPES
================================================================================

  #[derive(Debug, thiserror::Error)]
  pub enum KelvinError {
      /// Configuration failed validation
      #[error("invalid configuration: {0}")]
      InvalidConfig(String),

      /// Lyapunov time insufficient for requested steps
      #[error("insufficient Lyapunov time: requested {requested} steps, safe {safe}")]
      InsufficientLyapunovTime { requested: u64, safe: u64 },

      /// Orbital simulation time exhausted
      #[error("orbital simulation time exhausted at step {step}")]
      OrbitalTimeExhausted { step: u64 },

      /// Seed material exhausted (should not happen before orbital exhaustion)
      #[error("seed material exhausted")]
      SeedExhausted,

      /// I/O error during encryption/decryption
      #[error("I/O error: {0}")]
      Io(#[from] std::io::Error),

      /// Serialization error
      #[error("serialization error: {0}")]
      Serialization(String),
  }

16. SECURITY PROPERTIES (REQUIRED)
================================================================================

  ✓ Deterministic across all supported platforms
    - Test vectors verified on x86_64, aarch64, wasm32
    - Fixed-point math audited for platform-specific behavior
    - No undefined behavior in integer operations

  ✓ No simulation shortcut
    - Sequential dependency: step N+1 requires all previous steps
    - Chaotic: cannot predict state without full simulation
    - Lyapunov time estimation is conservative (10% margin)

  ✓ Side-channel resistance
    - Constant-time fixed-point operations where possible
    - No branching on secret-dependent values
    - Memory access patterns independent of orbital state
    - All comparisons use constant-time equality

  ✓ Forward secrecy (within orbital simulation)
    - State overwritten after each reseed operation
    - Old seeds not recoverable from current orbital state
    - All buffers zeroized on drop (Zeroize trait)

  ✓ Resistance to known-plaintext attacks
    - Stream cipher (ChaCha20) provides this property
    - Orbital parameters unrecoverable from stream cipher output
    - Seed extraction uses cryptographic hash (one-way function)

  ✓ Key material protection
    - OrbitalConfig stored only in memory, zeroized after use
    - No logging of key material
    - Serde skip for sensitive fields

17. CROSS-PLATFORM STRATEGY
================================================================================

  Desktop (Linux, macOS, Windows):
    - Native Rust compilation
    - ChaCha20 with software fallback
    - Optional AES-NI for Intel/AMD

  Mobile (iOS, Android):
    - Rust cross-compilation via cargo-ndk / cargo-lipo
    - C FFI exposed via kelvin-ffi crate
    - iOS: Swift wrapper around C API
    - Android: JNI wrapper via jni crate
    - Hardware ChaCha20 if available (ARMv8.4+)

  Web (WASM):
    - wasm-pack build for kelvin (core + kdf + stream)
    - wasm-bindgen for JavaScript interop
    - Run in Web Workers for non-blocking setup

  IoT (future):
    - kelvin-core compatible with no-std, no-alloc
    - Fixed-point math compiles on any 32-bit+ architecture
    - C FFI allows integration with embedded C codebases

18. TEST PLAN
================================================================================

  Unit Tests (per crate):
    □ Fixed-point arithmetic: all operations, edge cases, overflow
    □ Vec3 operations: dot product, length, normalization
    □ Integrator: energy conservation, momentum conservation
    □ Lyapunov estimator: known chaotic systems (double pendulum analog)
    □ Extractor: deterministic output, domain separation
    □ Key schedule: correct step counting, exhaustion detection
    □ Stream cipher: matches RFC 8439 test vectors

  Integration Tests:
    □ Encrypt/decrypt round-trip for 1 KB, 1 MB, 100 MB
    □ Deterministic across platforms (CI: x86_64, aarch64, wasm)
    □ Reseeding produces different keystreams
    □ Key separation: different configs produce uncorrelated output
    □ Known-answer test vectors (golden files)

  Property-Based Tests (proptest or quickcheck):
    □ Random OrbitalConfig always validates or gives clear error
    □ Encrypt(X) ^ Encrypt(X) = 0 (idempotent)
    □ Two Kelvin instances with same config produce identical keystream

  Benchmarks (criterion):
    □ Fixed-point multiply/divide throughput
    □ Acceleration computation for n={3,5,7}
    □ Single simulation step latency
    □ Full KDF setup time at each security level
    □ Stream cipher throughput (GB/s)
    □ End-to-end encryption throughput

  Fuzz Testing (cargo-fuzz):
    □ OrbitalConfig::from_bytes with random input (no crashes)
    □ Kelvin::encrypt with random data (no panics)
    □ Lyapunov estimator with edge-case orbital configurations

19. DOCUMENTATION PLAN
================================================================================

  API Documentation (rustdoc):
    □ All public types and functions documented
    □ Examples for common use cases
    □ Safety notes for cryptographic usage

  Book / Guide (mdBook):
    □ Introduction to orbital KDF concept
    □ Installation and quick start
    □ Key generation guide
    □ Usage examples (file encryption, network protocol)
    □ Security considerations and threat model
    □ Performance characteristics
    □ FAQ

  Specification (separate document):
    □ Fixed-point format specification
    □ Integration algorithm (pseudocode)
    □ Lyapunov estimation method
    □ Seed extraction algorithm
    □ Wire format for OrbitalConfig

20. IMPLEMENTATION PHASES (EFFORT ESTIMATE)
================================================================================

  These are effort estimates assuming one developer working full-time
  (~40 hours per week). Calendar time will be longer if part-time.

  Phase 1: Core Fixed-Point Math + Performance Benchmark
  ───────────────────────────────────────────────────────
  Effort:     3-4 effort-weeks
  Deliverable: kelvin-core crate with Fixed, Vec3, OrbitalBody
  Key files:   fixed_math.rs, body.rs
  Tests:       All arithmetic operations, edge cases, property tests
  Benchmark:   After implementing Fixed and Vec3, immediately benchmark
               multiply/divide throughput (ns/op). After implementing the
               Verlet integrator with n=3, benchmark single-step latency.
               Extrapolate to estimate full setup times for all three
               security levels. If Level 3 (100M steps × 7 bodies) exceeds
               5 minutes on a modern CPU, revisit the architecture before
               proceeding to Phase 2.

  Phase 2: Symplectic Integrator
  ─────────────────────────────────
  Effort:     2 effort-weeks
  Deliverable: Working 3D Verlet integration
  Key files:   integrator.rs, constants.rs
  Tests:       Energy conservation, momentum conservation, known orbits

  Phase 3: Lyapunov Time Estimator
  ─────────────────────────────────
  Effort:     3 effort-weeks
  Deliverable: LyapunovEstimator with shadow orbit method
  Key files:   lyapunov.rs
  Tests:       Chaotic systems validation, conservative estimation

  Phase 4: Entropy Extraction
  ─────────────────────────────────
  Effort:     1 effort-week
  Deliverable: StateExtractor with SHA3-512
  Key files:   extractor.rs
  Tests:       Deterministic output, domain separation

  Phase 5: Key Schedule
  ─────────────────────────────────
  Effort:     2 effort-weeks
  Deliverable: KeySchedule with reseeding
  Key files:   schedule.rs, config.rs
  Tests:       Step counting, exhaustion, seamless reseeding

  Phase 6: Stream Cipher Integration
  ─────────────────────────────────
  Effort:     1 effort-week
  Deliverable: ChaChaStream wrapper
  Key files:   chacha.rs, traits.rs
  Tests:       RFC 8439 test vectors, multi-GB generation

  Phase 7: Top-Level API
  ─────────────────────────────────
  Effort:     2 effort-weeks
  Deliverable: Kelvin struct with encrypt/decrypt
  Key files:   lib.rs, encrypt.rs, decrypt.rs, error.rs
  Tests:       Round-trip, large files, error handling

  Phase 8: Key Generation CLI
  ─────────────────────────────────
  Effort:     1 effort-week
  Deliverable: kelvin keygen command
  Key files:   keygen.rs, main.rs
  Tests:       Generated configs always validate

  Phase 9: Testing and Validation
  ─────────────────────────────────
  Effort:     3 effort-weeks
  Deliverable: Complete test suite, known-answer vectors
  Key files:   All tests/, vectors/
  Activities:  Cross-platform CI, fuzzing, benchmarks

  Phase 10: FFI and Mobile Support
  ─────────────────────────────────
  Effort:     2-3 effort-weeks
  Deliverable: C API, iOS/Android wrappers
  Key files:   kelvin-ffi/src/, platform wrappers

  Phase 11: Documentation
  ─────────────────────────────────
  Effort:     2 effort-weeks
  Deliverable: API docs, user guide, specification
  Activities:  rustdoc, mdBook, specification document

  ─────────────────────────────────────────────────────────────
  Total effort:       20-22 effort-weeks
  Solo calendar:      5-6 months (full-time), 8-12 months (part-time)
  Team of 2 calendar: 3-4 months (full-time)
  Team of 3 calendar: 2-3 months (full-time)

21. KNOWN RISKS AND MITIGATIONS
================================================================================

  Risk 1: Floating-point non-determinism
  ──────────────────────────────────────
  Severity: Critical
  Mitigation: No floating-point anywhere. Pure integer fixed-point math.
              Verified by cross-platform test vectors in CI.

  Risk 2: Lyapunov time estimation accuracy
  ──────────────────────────────────────
  Severity: High
  Mitigation: Conservative 10% margin. Shadow orbit with two independent
              runs. Rejection of configs that diverge too quickly.
              Peer review of estimation methodology.

  Risk 3: Side-channel leakage via timing
  ──────────────────────────────────────
  Severity: Medium
  Mitigation: Constant-time operations for comparisons. Fixed iteration
              count for sqrt. No early exits based on secret data.

  Risk 4: ChaCha20 implementation vulnerability
  ──────────────────────────────────────
  Severity: Low
  Mitigation: Use audited chacha20 crate. Verify against RFC 8439 test
              vectors. Feature-gate alternative stream ciphers.

  Risk 5: Integer overflow in fixed-point
  ──────────────────────────────────────
  Severity: Medium
  Mitigation: Use i128 for all intermediates. Saturation arithmetic
              where appropriate. Extensive property-based testing.

  Risk 6: WASM performance
  ──────────────────────────────────────
  Severity: Low
  Mitigation: WASM has good i128 support. ChaCha20 is efficient in WASM.
              Acceptable setup time for web use cases.

  Risk 7: Unproven cryptographic novelty (REVIEW FINDING)
  ────────────────────────────────────────────────────────
  Severity: Critical
  Description: The core security claim — that n-body simulation is
               "computationally irreducible" and provides a unique KDF
               property — is plausible but unproven. An attacker with
               mathematical sophistication might find shortcuts for
               specific orbital configurations. The orbital parameter
               space could have hidden structure that reduces effective
               key space.
  Mitigation:
    External:
      - Formal cryptanalysis and peer-reviewed publication before
        production use
      - Prominent "EXPERIMENTAL — NOT FOR PRODUCTION USE" warning
        on all entry points
      - Conservative parameter selection (n >= 5 for any real use)
    Internal validation (implement during Phase 3-4):
      - Reduced-round attack simulation: run a simplified KDF with
        lower precision (Q16.32), fewer bodies (n=3), and fewer steps
        (10K). Attempt to predict/extract from it using linear
        regression on orbital state. Measure the gap between actual
        and predicted state to quantify the "irreducibility" margin.
      - Key avalanche test: generate 1000 random OrbitalConfigs,
        flip one bit in each, and verify that the resulting keystream
        differs in ~50% of bits (avalanche criterion). This validates
        that the SHA3 extraction + orbital chaos together provide
        full diffusion.
      - Statistical randomness testing: run keystream output from
        diverse OrbitalConfigs through NIST SP 800-22 or PractRand
        to verify no statistical biases emerge from the orbital KDF
        stage (as opposed to the ChaCha20 stage, which is already
        known to pass).
      - Correlation test: verify that keystreams from configs with
        similar (but not identical) orbital parameters are uncorrelated.
        This tests whether the orbital parameter space has "smooth
        regions" that produce related outputs.

  Risk 8: Performance estimate optimism (REVIEW FINDING)
  ────────────────────────────────────────────────────────
  Severity: Medium
  Description: Estimated setup times (0.5s - 2min) may be optimistic.
               100M steps × 7 bodies × O(n²) = ~4.9B pair calculations,
               each involving i128 multiply, Newton sqrt iterations, and
               softened acceleration.
  Mitigation: Prototype kelvin-core first to validate performance
              estimates before committing to the full architecture.
              Profile on target hardware early.

  Risk 9: i128 portability on embedded targets (REVIEW FINDING)
  ──────────────────────────────────────────────────────────────
  Severity: Medium
  Description: i128 is not available on all embedded targets. Some
               no-std environments lack compiler support for 128-bit
               integer operations.
  Mitigation: Clarify Tier 1 vs experimental targets. Consider a
              software i128 fallback for targets without hardware
              support. Gate i128-dependent code behind a feature flag.

22. THREAT MODEL (REVIEW ADDITION)
================================================================================

  This section defines the attacker model and security boundaries
  for the Kelvin cryptosystem.

  Attacker capabilities:
    - Full access to ciphertext (known-ciphertext attack)
    - May have access to some plaintext-ciphertext pairs (known-plaintext)
    - Can run the Kelvin simulation on arbitrary hardware
    - Has knowledge of the algorithm (Kerckhoffs's principle)
    - Does NOT have access to the OrbitalConfig shared secret
    - Computational resources: up to nation-state level (GPU clusters, ASICs)

  What Kelvin defends against:
    - Brute-force search of orbital parameter space: each guess requires
      a full simulation run (seconds to minutes), inherently sequential
    - Parallelization: Verlet integrator is sequential, cannot be sped up
      with more hardware beyond a single simulation instance
    - Side-channel recovery of orbital state from timing/power analysis

  What Kelvin does NOT defend against:
    - Mathematical breakthroughs in n-body problem analysis
    - Quantum attacks on the stream cipher (ChaCha20 has 256-bit keys,
      providing post-quantum 128-bit security)
    - Compromise of the OrbitalConfig shared secret (this is the key)
    - Implementation bugs (buffer overflows, use-after-free, etc.)

  Comparison with existing KDFs:
    - Argon2: Memory-hard, parallelizable across CPUs/GPUs for verification
    - Kelvin: Computationally-hard, inherently sequential, not memory-hard
    - Kelvin's advantage: attacker cannot verify a key guess without
      running the full simulation AND attempting decryption
    - Kelvin's disadvantage: higher setup cost, unproven cryptanalysis

23. DESIGN REVIEW RECOMMENDATIONS (REVIEW ADDITION)

  The following recommendations emerged from a design review of the
  implementation plan. These should be addressed before or during
  implementation.

  Priority 1 (Before implementation):
    □ Add a prominent "EXPERIMENTAL — NOT FOR PRODUCTION USE" warning
      to all public API entry points, CLI help text, and documentation
    □ Create a THREAT_MODEL.md document (outlined in section 22 above)
    □ Publish a white paper describing the cryptographic rationale for
      peer review before any production deployment

  Priority 2 (During Phase 1-2):
    □ Prototype kelvin-core (fixed-point math + integrator) first to
      validate performance estimates against real hardware
    □ Verify Q32.64 range is sufficient for worst-case orbital
      configurations (high eccentricity, close encounters)
    □ Validate softening factor (ϵ) choice with property-based tests
    □ Test i128 availability on all target platforms

  Priority 3 (During Phase 3):
    □ Validate Lyapunov estimator against known chaotic systems
      (e.g., Hénon-Heiles system, restricted 3-body problem)
    □ Report confidence intervals rather than single Lyapunov time values:
      run 5 shadow orbits per configuration, each with a different
      perturbed body or perturbation direction. Compute mean (μ) and
      standard deviation (σ) across the 5 samples. If σ/μ > 0.5 (high
      variance), reject the configuration as too unstable to estimate
      reliably. Report τ = μ ± 2σ as the confidence interval.
    □ Estimate rejection rate for randomly generated orbital configs

  Priority 4 (During Phase 4-5):
    □ Analyze entropy quality: verify that consecutive extractions
      produce uncorrelated output (NIST SP 800-90B tests)
    □ Document reseeding rationale clearly (forward secrecy vs entropy)

  Priority 5 (Pre-release):
    □ Third-party security audit by a qualified cryptography firm
    □ Cross-platform test vector verification (x86_64, aarch64, wasm32)
    □ Fuzz testing with cargo-fuzz for all deserialization paths

24. FUTURE ENHANCEMENTS (POST-V1)

  □ General relativistic corrections (post-Newtonian)
  □ Stellar evolution effects (mass loss, radiation pressure)
  □ GPU-accelerated simulation (CUDA/WebGPU) for Level 3 keys
  □ Multi-body resonance tracking for additional entropy
  □ Integration with Signal Double Ratchet for messaging
  □ Hardware security module (HSM) integration
  □ Post-quantum hybrid mode (combine with Kyber/ML-KEM)
  □ Formal verification of fixed-point arithmetic
  □ Third-party security audit

25. KEY EXCHANGE PROTOCOL (REVIEW ADDITION)

  Kelvin does not define a key exchange protocol — the OrbitalConfig is
  a shared secret that must be established between communicating parties
  through an out-of-band mechanism. This section documents the recommended
  approaches and their security properties.

  Threat model for key exchange:
    - The OrbitalConfig is the cryptographic key. Its compromise during
      exchange is equivalent to key compromise.
    - The exchange channel must provide: confidentiality, integrity, and
      authentication of the OrbitalConfig.
    - Kelvin does not provide authentication — the exchange mechanism
      must ensure that both parties have the same OrbitalConfig and that
      it was not tampered with in transit.

  Method 1: Direct out-of-band exchange (RECOMMENDED)
  ────────────────────────────────────────────────────
  Suitable for: Air-gapped scenarios, in-person key exchange,
                pre-shared keys in organizational settings.

  Procedure:
    1. Party A generates an OrbitalConfig using kelvin keygen
    2. Party A encodes the config as a QR code (compact binary, ~370-818
       bytes, fits in a single QR code at version 20+ with error correction)
    3. Party B scans the QR code and decodes the OrbitalConfig
    4. Both parties independently run the simulation to verify the config
       is valid (Lyapunov time check, stability check)
    5. Encryption begins

  Security properties:
    - Confidentiality: Physical control of the QR code (visual inspection,
      no electronic interception)
    - Integrity: QR code error correction detects corruption
    - Authentication: Visual identification of the exchanging party
    - Limitations: Not suitable for remote exchange; requires physical
      proximity

  Method 2: Encrypted file transfer
  ────────────────────────────────────
  Suitable for: Remote exchange over an existing encrypted channel
                (Signal, PGP-encrypted email, HTTPS file drop).

  Procedure:
    1. Party A generates an OrbitalConfig
    2. Party A encrypts the config with an existing key (e.g., PGP public
       key of Party B, Signal message, TLS session)
    3. Party A transmits the encrypted config over the existing channel
    4. Party B decrypts and validates the OrbitalConfig

  Security properties:
    - Confidentiality: Inherited from the existing encrypted channel
    - Integrity: Inherited from the existing channel (MACs, signatures)
    - Authentication: Inherited from the existing channel (PKI, Signal
      safety numbers, etc.)
    - Limitations: Security is bounded by the existing channel — if the
      channel is compromised, the OrbitalConfig is compromised. This
      defeats the purpose of using Kelvin for additional security.

  Method 3: Split-config exchange (EXPERIMENTAL)
  ────────────────────────────────────────────────
  Suitable for: Scenarios where no single trusted channel exists, but
                multiple independent channels are available.

  Procedure:
    1. Party A generates an OrbitalConfig with n bodies
    2. Party A splits the config into k shares (k >= 2):
       - Share 1: body parameters for bodies 1..floor(n/2)
       - Share 2: body parameters for bodies floor(n/2)+1..n
       - Share 3 (optional): simulation parameters (steps, dt, ϵ)
    3. Each share is transmitted over a different channel (e.g., Share 1
       via email, Share 2 via SMS, Share 3 via postal mail)
    4. Party B reassembles the shares and validates the full config

  Security properties:
    - Confidentiality: An attacker must compromise ALL channels to recover
      the full OrbitalConfig
    - Integrity: Each share can be validated independently (checksums)
    - Authentication: Each channel provides its own authentication
    - Limitations: More complex; requires coordination across channels;
      the split must not create exploitable structure (e.g., knowing half
      the bodies should not allow predicting the other half — this is
      guaranteed by the chaotic nature of the system, but is unproven)

  Method 4: Diffie-Hellman-style joint generation (FUTURE RESEARCH)
  ───────────────────────────────────────────────────────────────────
  Suitable for: Protocol-level key agreement without pre-shared secrets.

  Concept (not yet designed):
    - Both parties contribute random planetary parameters
    - The combined parameter set is used as the OrbitalConfig
    - Neither party controls the full config unilaterally
    - Requires careful design to prevent one party from choosing
      parameters that weaken the system (e.g., choosing all bodies
      in a stable resonance that reduces chaos)

  Security properties:
    - Forward secrecy: Possible if the joint config is ephemeral
    - Authentication: Requires an additional authentication layer
    - Limitations: NOT RECOMMENDED for v1. The interaction between
      independently-chosen orbital parameters and the resulting chaos
      properties is not well-understood. A malicious party could
      choose parameters that produce a non-chaotic (or weakly chaotic)
      system, reducing the effective security.

  Recommendation for v1:
    - Use Method 1 (QR code) for in-person exchange
    - Use Method 2 (encrypted file transfer) for remote exchange,
      with the understanding that security is bounded by the existing
      channel
    - Method 3 (split-config) is acceptable for high-value scenarios
      where multiple independent channels exist
    - Method 4 (joint generation) is deferred to post-v1 research

END OF IMPLEMENTATION PLAN
