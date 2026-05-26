//! Named parameters for the Kelvin cryptosystem.
//!
//! All hardcoded values used across the `kelvin` crate are defined here
//! with documentation explaining what they control and the implications
//! of changing them.
//!
//! ## Conventions
//!
//! - **Seed sizes**: 2048 bytes for the main entropy pool (SHAKE256 XOF output).
//! - **XOF seed sizes**: 64 bytes for HKDF-SHA512 output seeding SHAKE256.
//! - **MAC key sizes**: 32 bytes for KMAC128 (NIST SP 800-185).
//! - **Chunk sizes**: 1 MiB (1,048,576 bytes) for keystream generation buffers.

use kelvin_core::IntegrationMethod;

// ============================================================================
// Seed & Key Sizes
// ============================================================================

/// Size of the orbital simulation seed in bytes.
///
/// Used by V1 (`Kelvin`) and `simulate_and_extract_seed_with_method` as the
/// primary seed material extracted from the orbital simulation via SHAKE256 XOF.
/// This seed is then passed to `KeySchedule` for key derivation.
///
/// **Why 2048?** SHAKE256 can produce arbitrary-length output. 2048 bytes
/// (16,384 bits) provides a large entropy pool for HKDF-SHA512 expansion,
/// allowing millions of key derivations without reseeding.
///
/// **Changing this** affects the maximum number of keys derivable from a single
/// orbital simulation. Larger values increase memory usage but allow more
/// key material before reseeding.
pub const ORBITAL_SEED_SIZE: usize = 2048;

/// Size of the V3 Photon base seed in bytes.
///
/// Used by `KelvinPhoton` and `KelvinPhotonAuthenticated` as the initial
/// entropy pool for HKDF-SHA512 expansion → SHAKE256 keystream generation.
/// Each reseed derives a fresh 2048-byte pool via BLAKE3 for forward secrecy.
///
/// **Why 2048?** Same rationale as `ORBITAL_SEED_SIZE`. The Photon mode
/// reuses the same seed size for consistency, but the seed is consumed
/// differently (HKDF expand → SHAKE256 XOF vs. KeySchedule key derivation).
///
/// **Changing this** would break compatibility with existing V3 ciphertexts.
pub const PHOTON_BASE_SEED_SIZE: usize = 2048;

/// Size of the H Quantum base seed in bytes.
///
/// Used by `KelvinQuantum` and `KelvinQuantumAuthenticated` as the base
/// entropy pool that is periodically refreshed with fresh orbital entropy
/// via `reseed_from_orbital_chaos`.
///
/// **Why 2048?** Same rationale as `ORBITAL_SEED_SIZE`. The Quantum mode
/// reuses the same seed size but the seed lifecycle is different — it is
/// XORed with fresh orbital entropy on each reseed rather than being
/// replaced via BLAKE3.
///
/// **Changing this** would break compatibility with existing H ciphertexts.
pub const QUANTUM_BASE_SEED_SIZE: usize = 2048;

/// Size of the KeySchedule seed in bytes.
///
/// Used by `KeySchedule` in `kelvin-kdf` for HKDF-SHA512-based key derivation
/// and BLAKE3-based reseeding. This is the V1 key schedule seed.
///
/// **Why 2048?** Same rationale as `ORBITAL_SEED_SIZE`. The KeySchedule
/// receives the orbital seed and uses it for HKDF key derivation with
/// forward secrecy via BLAKE3 reseeding.
///
/// **Changing this** would break compatibility with existing V1 key schedules.
pub const KEY_SCHEDULE_SEED_SIZE: usize = 2048;

/// Size of the XOF seed in bytes (HKDF-SHA512 output → SHAKE256 input).
///
/// Used by V3 (`KelvinPhoton`) and H (`KelvinQuantum`) to seed SHAKE256 XOF
/// for arbitrary-length keystream generation.
///
/// **Why 64?** HKDF-SHA512 can output up to 16,320 bytes per expand call.
/// 64 bytes is sufficient to seed SHAKE256's 256-bit security level while
/// keeping the HKDF call efficient.
///
/// **Changing this** affects the entropy available to seed SHAKE256. Must be
/// at least 32 bytes for 256-bit security.
pub const XOF_SEED_SIZE: usize = 64;

/// Size of the KMAC128 MAC key in bytes.
///
/// Used by authenticated wrappers (`KelvinPhotonAuthenticated`,
/// `KelvinQuantumAuthenticated`, `KelvinStreamingAuthenticated`) for
/// NIST SP 800-185 KMAC128 authentication.
///
/// **Why 32?** KMAC128 provides 128-bit security against classical and quantum
/// adversaries. A 32-byte (256-bit) key is standard for HMAC/KMAC.
///
/// **Changing this** affects the security level of the MAC. Must be at least
/// 16 bytes for 128-bit security.
pub const MAC_KEY_SIZE: usize = 32;

/// Size of the entropy extraction buffer in bytes.
///
/// Used when extracting fresh entropy from the orbital state via SHAKE256
/// for reseeding (e.g., `KelvinQuantum::reseed_from_orbital_chaos`).
///
/// **Why 64?** Provides 512 bits of entropy per extraction, which is then
/// XORed into the 2048-byte base seed. 64 bytes is a standard XOF output
/// size for SHAKE256.
///
/// **Changing this** affects how much fresh entropy is mixed in per reseed.
pub const EXTRACT_BUF_SIZE: usize = 64;

// ============================================================================
// Lyapunov Estimation
// ============================================================================

/// Number of shadow steps used for Lyapunov time estimation.
///
/// Used by `simulate_and_extract_seed_with_method` and `Kelvin::init_with_method`
/// to estimate the Lyapunov exponent via the shadow orbit method.
///
/// **Why 100,000?** The previous value of 10,000 steps was insufficient for
/// bodies with ~1000-year orbital periods (wide orbits up to 500 AU at maximum
/// security level). 10,000 steps × 1e-3 yr/step = only 10 years of simulation,
/// which is barely a blink for a 500 AU orbit (~11,180 year period). 100,000
/// steps provides 100 years of simulation, enough to detect divergence even
/// in the widest orbits.
///
/// **Changing this** affects the accuracy of Lyapunov estimation:
/// - Higher values → more accurate but slower initialization
/// - Lower values → faster but may miss chaos in wide orbits
pub const LYAPUNOV_SHADOW_STEPS: u64 = 100_000;

/// Fast mode simulation steps for benchmarking/testing.
///
/// Set to `LYAPUNOV_SHADOW_STEPS + 10,000` (110,000), which is just above
/// the Lyapunov horizon. The `min_chaos_steps` from Lyapunov estimation is
/// at most `LYAPUNOV_SHADOW_STEPS + 1` (100,001) in the no-divergence case,
/// so 110,000 steps comfortably satisfies the chaos check while keeping
/// simulation time under ~4s (vs 336s for paranoid's 10M steps).
///
/// Used by `keygen --fast` and the internal `run_benchmark()`.
///
/// **Changing this** affects the minimum steps for fast mode:
/// - Must be >= `LYAPUNOV_SHADOW_STEPS + 1` to pass the chaos check
/// - Higher values → slower but more entropy
/// - Lower values → faster but may fail the chaos check
pub const FAST_STEPS: u64 = LYAPUNOV_SHADOW_STEPS + 10_000;

/// Fast mode reseed interval, proportionally scaled from `FAST_STEPS`.
///
/// Set to `FAST_STEPS / 10` (11,000), maintaining the same ratio as the
/// default `reseed_interval = total_steps / 10`.
pub const FAST_RESEED_INTERVAL: u64 = FAST_STEPS / 10;

// ============================================================================
// Keystream Generation
// ============================================================================

/// Maximum chunk size for keystream generation in bytes (1 MiB).
///
/// Used by V3 (`KelvinPhoton`) and H (`KelvinQuantum`) to process data in
/// fixed-size chunks, preventing OOM crashes when encrypting large (multi-GB)
/// inputs by avoiding a full-size keystream allocation.
///
/// **Why 1 MiB?** Balances memory usage (~1 MB per buffer) with throughput.
/// Larger chunks reduce loop overhead but increase peak memory. 1 MiB is a
/// common page cache-friendly size.
///
/// **Changing this** affects peak memory usage and throughput:
/// - Larger → fewer iterations, more memory
/// - Smaller → less memory, more iterations
pub const KEYSTREAM_CHUNK_SIZE: usize = 1024 * 1024;

// ============================================================================
// Domain Separators
// ============================================================================

/// Domain separator for V1 orbital state seed extraction.
///
/// Used in `simulate_and_extract_seed_with_method` and `Kelvin::init_with_method`
/// to domain-separate the SHAKE256 extraction of the initial 2048-byte seed
/// from the orbital simulation state.
///
/// **Why this value?** Ensures the V1 seed extraction is cryptographically
/// isolated from other extraction contexts (V2, V3, H, MAC keys).
///
/// **Changing this** would break compatibility with all existing keystreams.
pub const DOMSEP_ORBITAL_STATE_V1: &[u8] = b"kelvin-orbital-state-v1";

// Domain separator for V2 streaming keystream extraction.
// (Commented out: not currently used within the `kelvin` crate.
//  The V2 streaming mode uses a hardcoded domain separator in KelvinStreaming.)
// pub const DOMSEP_STREAMING_V2: &[u8] = b"kelvin-streaming-v2-v1-000000000";

/// Domain separator for V3 Photon keystream HKDF expansion.
///
/// Used in `KelvinPhoton::generate_keystream_into` to domain-separate the
/// HKDF-SHA512 expand step that derives the XOF seed.
///
/// **Changing this** would break compatibility with existing V3 ciphertexts.
pub const DOMSEP_PHOTON_KEYSTREAM_V1: &[u8] = b"kelvin-photon-keystream-v1";

/// Domain separator for V3 Photon reseed (BLAKE3).
///
/// Used in `KelvinPhoton::generate_keystream_into` to domain-separate the
/// BLAKE3 reseed that derives the next 2048-byte seed pool.
///
/// **Changing this** would break forward secrecy chain compatibility.
pub const DOMSEP_PHOTON_RESEED_V1: &[u8] = b"kelvin-photon-reseed-v1";

/// Domain separator for H Quantum seed perturbation.
///
/// Used in `KelvinQuantum::with_config` to domain-separate the BLAKE3 XOF
/// that generates perturbation material for the orbital state.
///
/// **Changing this** would change the chaotic trajectory for all instances.
pub const DOMSEP_QUANTUM_PERTURB_V1: &[u8] = b"kelvin-quantum-seed-perturb-v1";

/// Domain separator for H Quantum reseed (BLAKE3).
///
/// Used in `KelvinQuantum::reseed_from_orbital_chaos` to domain-separate the
/// BLAKE3 reseed that XORs fresh orbital entropy into the base seed.
///
/// **Changing this** would break forward secrecy chain compatibility.
pub const DOMSEP_QUANTUM_RESEED_V1: &[u8] = b"kelvin-quantum-reseed-v1";

/// Domain separator for H Quantum keystream cache (SHAKE256).
///
/// Used in `KelvinQuantum::refill_keystream_cache` to domain-separate the
/// SHAKE256 XOF that fills the keystream cache.
///
/// **Changing this** would break compatibility with existing H ciphertexts.
pub const DOMSEP_QUANTUM_CACHE_V1: &[u8] = b"kelvin-quantum-cache-v1";

/// Domain separator for H Quantum keystream (SHAKE256).
///
/// Used in `KelvinQuantum::refill_keystream_cache` as additional domain
/// separation within the SHAKE256 XOF input.
///
/// **Changing this** would break compatibility with existing H ciphertexts.
pub const DOMSEP_QUANTUM_KEYSTREAM: &[u8] = b"kelvin-quantum-keystream";

/// Domain separator for MAC key derivation (V3/H).
///
/// Used in `derive_mac_key` to domain-separate the HKDF-SHA512 expansion
/// that derives the 32-byte KMAC128 key from the 2048-byte seed.
///
/// **Changing this** would break authentication for all existing ciphertexts.
pub const DOMSEP_MAC_KEY_V1: &[u8] = b"kelvin-mac-key-v1";

/// Domain separator for V2 streaming MAC key derivation.
///
/// Used in `derive_mac_key_from_bodies` to domain-separate the SHAKE256
/// extraction and HKDF expansion that derives the KMAC128 key from the
/// initial orbital state.
///
/// **Changing this** would break authentication for existing V2 ciphertexts.
pub const DOMSEP_STREAMING_MAC_KEY_V1: &[u8] = b"kelvin-streaming-mac-key-v1";

/// Domain separator for Prism OTP key generation.
///
/// Used in `KelvinPrism::generate_keystream_into` to domain-separate the
/// HKDF-SHA512 expand step that derives the XOF seed for OTP key material.
/// This ensures Prism-generated OTP keys are cryptographically isolated
/// from normal V3 Photon keystream, preventing related-key attacks when
/// both are used in the same system.
///
/// **Changing this** would break compatibility with existing Prism OTP keys.
pub const DOMSEP_PRISM_KEYSTREAM_V1: &[u8] = b"kelvin-prism-keystream-v1";

/// Domain separator for Prism reseed (BLAKE3).
///
/// Used in `KelvinPrism::generate_keystream_into` to domain-separate the
/// BLAKE3 reseed that derives the next 2048-byte seed pool. This ensures
/// the forward secrecy chain is independent from V3 Photon's reseed chain.
///
/// **Changing this** would break forward secrecy chain compatibility.
pub const DOMSEP_PRISM_RESEED_V1: &[u8] = b"kelvin-prism-reseed-v1";

// ============================================================================
// Quantum Mode Defaults
// ============================================================================

/// Default perturbation scale for orbital state initialization.
///
/// Used in `KelvinQuantum::with_config` to scale the seed-derived perturbation
/// applied to body positions/velocities. The perturbation magnitude (~1e-12)
/// is small enough to stay within the chaotic regime but large enough to cause
/// rapid trajectory divergence between instances.
///
/// **Why 1e-12?** Small enough to not destabilize the simulation, large enough
/// to ensure unique trajectories. The Lyapunov exponent amplifies this to
/// macroscopic divergence within a few thousand steps.
///
/// **Changing this** affects how quickly instances diverge:
/// - Larger → faster divergence, risk of numerical instability
/// - Smaller → slower divergence, risk of synchronized instances
pub const QUANTUM_PERTURB_SCALE: f64 = 1e-12;

// ============================================================================
// Lyapunov Estimator Constants (kelvin-kdf)
// ============================================================================

// Number of shadow orbits for Lyapunov estimation (one per spatial axis).
// (Commented out: defined in `kelvin-kdf` crate directly, not imported here.)
// pub const SHADOW_ORBIT_COUNT: u32 = 3;

// Perturbation magnitude for shadow orbits (~6e-8 AU ≈ 9 km).
// (Commented out: defined in `kelvin-kdf` crate directly, not imported here.)
// pub const SHADOW_PERTURBATION_RAW: i128 = 1 << 40;

// Base margin factor for Lyapunov safety margin calculation.
// (Commented out: defined in `kelvin-kdf` crate directly, not imported here.)
// pub const LYAPUNOV_BASE_MARGIN: f64 = 10.0;

// Standard deviation scaling factor for dynamic margin adjustment.
// (Commented out: defined in `kelvin-kdf` crate directly, not imported here.)
// pub const LYAPUNOV_STD_DEV_SCALING: f64 = 50.0;

// Threshold for High confidence Lyapunov estimation (steps).
// (Commented out: defined in `kelvin-kdf` crate directly, not imported here.)
// pub const LYAPUNOV_HIGH_CONFIDENCE_THRESHOLD: u64 = 10_000;

// Threshold for Medium confidence Lyapunov estimation (steps).
// (Commented out: defined in `kelvin-kdf` crate directly, not imported here.)
// pub const LYAPUNOV_MEDIUM_CONFIDENCE_THRESHOLD: u64 = 1_000;

// ============================================================================
// CLI Keygen Defaults
// ============================================================================

/// Number of bodies for "standard" security level.
///
/// **Why 8?** 8-body N-body systems are inherently chaotic with strong
/// sensitivity to initial conditions. The Lyapunov estimator reliably detects
/// chaos with 8 bodies, preventing `InsufficientChaos` errors.
///
/// **Changing this** affects the security/performance trade-off:
/// - More bodies → more chaotic, slower simulation
/// - Fewer bodies → faster simulation, may not reach chaotic regime
pub const STANDARD_BODIES: usize = 8;

/// Number of bodies for "paranoid" security level.
///
/// Same as standard (8 bodies) but with 10x more simulation steps.
pub const PARANOID_BODIES: usize = 8;

/// Number of bodies for "maximum" security level.
///
/// **Why 10?** 10 bodies with 100M steps provides maximum chaos amplification
/// at significant computational cost.
pub const MAXIMUM_BODIES: usize = 10;

/// Simulation steps for "standard" security level.
///
/// **Why 1,000,000?** Provides ~100x the Lyapunov horizon for typical 8-body
/// systems, ensuring deep chaotic mixing.
pub const STANDARD_STEPS: u64 = 1_000_000;

/// Simulation steps for "paranoid" security level.
///
/// **Why 10,000,000?** 10x more steps than standard for paranoid users.
pub const PARANOID_STEPS: u64 = 10_000_000;

/// Simulation steps for "maximum" security level.
///
/// **Why 100,000,000?** 100x more steps than standard. Takes significantly
/// longer but provides maximum entropy amplification.
pub const MAXIMUM_STEPS: u64 = 100_000_000;

// ============================================================================
// Keygen Sun Body Parameters
// ============================================================================

/// Center value for sun mass randomization (1 << 64 in Q32.64).
///
/// The sun's mass is randomized around this center value with ±(1 << 62) range.
/// The final mass is constrained to [3<<62, 5<<62] to ensure stability.
///
/// **Why 1<<64?** In Q32.64 fixed-point, this represents 1.0 solar mass.
/// The ±(1<<62) range provides ~25% variation while keeping the mass
/// physically reasonable.
pub const SUN_MASS_CENTER: i128 = 1 << 64;

/// Half-range for sun mass randomization (± this value).
pub const SUN_MASS_RANGE: i128 = 1 << 62;

/// Minimum acceptable sun mass (3/4 of center).
pub const SUN_MASS_MIN_RAW: i128 = 3 << 62;

/// Maximum acceptable sun mass (5/4 of center).
pub const SUN_MASS_MAX_RAW: i128 = 5 << 62;

/// Minimum random value for sun position components.
pub const SUN_POS_MIN_RAW: i128 = 1 << 20;

/// Maximum random value for sun position components.
pub const SUN_POS_MAX_RAW: i128 = 1 << 30;

/// Minimum random value for sun velocity components.
pub const SUN_VEL_MIN_RAW: i128 = 1 << 10;

/// Maximum random value for sun velocity components.
pub const SUN_VEL_MAX_RAW: i128 = 1 << 20;

// ============================================================================
// Keygen Planet Body Parameters
// ============================================================================

/// Radius multiplier for planet orbital distances.
///
/// Planet i is placed at radius = (i + 1) * PLANET_RADIUS_MULTIPLIER AU.
///
/// **Why 50?** Provides well-separated orbits from ~100 AU to ~500 AU for
/// 8 planets, avoiding gravitational collapse while keeping the system bound.
///
/// **Changing this** affects orbital spacing:
/// - Larger → wider orbits, longer periods, more steps needed for chaos
/// - Smaller → tighter orbits, risk of collapse
pub const PLANET_RADIUS_MULTIPLIER: i64 = 50;

/// Minimum random value for planet masses.
pub const PLANET_MASS_MIN_RAW: i128 = 1 << 30;

/// Maximum random value for planet masses.
pub const PLANET_MASS_MAX_RAW: i128 = 1 << 35;

/// Orbital velocity constant (2π, not 6.3).
///
/// Used to compute circular orbital velocity: v = 2π / sqrt(r).
/// The approximate value 6.3 was used historically; this constant
/// provides the exact 2π value.
///
/// **Why 2π?** For a Keplerian orbit, v = sqrt(GM/r). In AU-solar mass-year
/// units with G = 4π², this simplifies to v = 2π/√r for a circular orbit
/// around a solar-mass star.
pub const ORBITAL_VELOCITY_CONSTANT: f64 = std::f64::consts::TAU;

// ============================================================================
// CLI Defaults
// ============================================================================

/// Default bytes per step for V2 Chaos streaming mode (1 MiB).
///
/// Controls how many bytes of keystream each simulation step produces.
/// Larger values mean fewer steps for a given file size, reducing the
/// number of Verlet/Euler integrations needed.
///
/// **Why 1 MiB?** Balances simulation cost (~0.3ms per Verlet step for
/// 10 bodies) with I/O efficiency. 1 MiB per step means ~1000 steps per GB.
///
/// **Changing this** affects the chaos mode throughput:
/// - Larger → fewer steps, faster processing, less frequent entropy refresh
/// - Smaller → more steps, slower processing, more frequent entropy refresh
pub const CHAOS_DEFAULT_BYTES_PER_STEP: u64 = 1024 * 1024;

/// Default bytes per step/chunk for V3 Photon mode (64 MiB).
///
/// Controls the I/O chunk size for photon mode. Photon generates keystream
/// from HKDF→SHAKE256 (no per-step simulation), so larger chunks reduce
/// loop overhead without any simulation cost.
///
/// **Why 64 MiB?** Photon is already I/O bound at ~30 GB/s. A 64 MiB chunk
/// reduces Python/CLI loop overhead while keeping memory usage reasonable.
///
/// **Changing this** affects photon mode throughput:
/// - Larger → fewer iterations, less overhead, more memory
/// - Smaller → more iterations, more overhead, less memory
pub const PHOTON_DEFAULT_BYTES_PER_STEP: u64 = 64 * 1024 * 1024;

/// Default bytes per step/chunk for H Quantum mode (64 MiB).
///
/// Controls the I/O chunk size for quantum mode. Like photon, quantum
/// generates keystream from a cache (SHAKE256 XOF) with periodic orbital
/// reseeding. Larger chunks reduce loop overhead.
///
/// **Why 64 MiB?** Same rationale as photon. Quantum's orbital reseeding
/// happens every 10 MiB regardless of chunk size, so chunk size only
/// affects I/O loop overhead.
///
/// **Changing this** affects quantum mode throughput:
/// - Larger → fewer iterations, less overhead, more memory
/// - Smaller → more iterations, more overhead, less memory
pub const QUANTUM_DEFAULT_BYTES_PER_STEP: u64 = 64 * 1024 * 1024;

/// Legacy alias for DEFAULT_BYTES_PER_STEP (kept for backward compatibility).
pub const DEFAULT_BYTES_PER_STEP: u64 = CHAOS_DEFAULT_BYTES_PER_STEP;

/// Default max reseeds for V3 Photon mode.
///
/// **Why 100,000?** Each reseed produces ~16 KB of HKDF output seeding
/// unlimited SHAKE256 keystream. 100,000 reseeds allows ~1.6 GB of key
/// material before exhaustion.
pub const PHOTON_DEFAULT_MAX_RESEEDS: u64 = 100_000;

/// Default max reseeds for H Quantum mode.
///
/// **Why 100,000?** Same rationale as Photon. Each reseed refreshes the
/// 2048-byte base seed with fresh orbital entropy.
pub const QUANTUM_DEFAULT_MAX_RESEEDS: u64 = 100_000;

/// Default cache size for H Quantum keystream (1 MiB).
pub const QUANTUM_DEFAULT_CACHE_SIZE: usize = 1024 * 1024;

/// Default integration method for H Quantum orbital reseeding.
///
/// Used in `KelvinQuantum::reseed_from_orbital_chaos` to advance the orbital
/// simulation when refreshing the base seed with fresh chaotic entropy.
///
/// **Why Verlet?** Verlet is energy-conserving and provides stable long-term
/// integration. Use Euler for faster chaos amplification (numerically unstable).
///
/// **Changing this** affects the chaotic trajectory of the reseed entropy.
pub const QUANTUM_DEFAULT_INTEGRATION_METHOD: IntegrationMethod = IntegrationMethod::Verlet;

/// Default orbital steps per reseed for H Quantum (1,000).
///
/// **Why 1,000?** 1,000 Verlet steps (~0.05ms) provides sufficient trajectory
/// divergence to inject fresh entropy into the base seed. The previous value
/// of 10,000 was excessive — the Lyapunov exponent amplifies microscopic
/// perturbations to macroscopic divergence within a few hundred steps.
/// Reducing to 1,000 cuts orbital reseed cost by 10× while maintaining
/// security.
///
/// **Changing this** affects the orbital reseed cost:
/// - Larger → more entropy mixing, slower reseeding
/// - Smaller → faster reseeding, less entropy per reseed
pub const QUANTUM_DEFAULT_ORBITAL_STEPS: u64 = 1_000;

/// Default reseed interval for H Quantum in bytes (64 MiB).
///
/// **Why 64 MiB?** Matches Photon's reseed interval for consistency. At
/// 1,000 orbital steps per reseed, 64 MiB means ~16 reseeds per GB of data
/// (~16,000 orbital steps per GB). The previous value of 10 MiB caused
/// ~100 reseeds per GB (~1M orbital steps per GB), which was the dominant
/// performance bottleneck.
///
/// **Changing this** affects the orbital reseed frequency:
/// - Larger → fewer reseeds, faster processing, less frequent entropy refresh
/// - Smaller → more reseeds, slower processing, more frequent entropy refresh
pub const QUANTUM_DEFAULT_RESEED_INTERVAL: u64 = 64 * 1024 * 1024;

// Default dt raw value for keygen (1 << 54 in Q32.64 ≈ 1e-3 years).
// (Commented out: not currently used within the `kelvin` crate.
//  The CLI uses kelvin_core::DEFAULT_DT directly.)
// pub const DEFAULT_DT_RAW: i128 = 1 << 54;

// Default softening raw value for keygen (1 << 48 in Q32.64 ≈ 1e-5 AU).
// (Commented out: not currently used within the `kelvin` crate.
//  The CLI uses kelvin_core::SOFTENING_FACTOR directly.)
// pub const DEFAULT_SOFTENING_RAW: i128 = 1 << 48;

/// Buffer size for keystream cache in V2 streaming mode (64 KB).
///
/// **Why 64 KB?** Standard filesystem block size for efficient I/O.
pub const STREAMING_CHUNK_SIZE: usize = 64 * 1024;

/// Size of the KMAC128 authentication tag in bytes.
///
/// Used by authenticated wrappers (`KelvinPhotonAuthenticated`,
/// `KelvinQuantumAuthenticated`, `KelvinStreamingAuthenticated`) for
/// NIST SP 800-185 KMAC128 authentication.
///
/// **Why 32?** KMAC128 provides 128-bit security against forgery.
/// A 32-byte (256-bit) tag is standard for KMAC128.
///
/// **Changing this** would break compatibility with existing authenticated
/// ciphertexts.
pub const AUTH_TAG_LEN: usize = 32;

/// Wire format version for authenticated ciphertexts.
///
/// The version byte is prepended to the KMAC128 tag in the wire format:
///
/// ```text
/// ciphertext (N bytes) || version (1 byte) || KMAC128 tag (32 bytes)
/// ```
///
/// **Why 0x01?** This is the initial version. Future versions can be
/// detected and handled gracefully during decryption.
///
/// **Changing this** would break compatibility with existing authenticated
/// ciphertexts.
pub const AUTH_FORMAT_VERSION: u8 = 0x01;

/// Total overhead for authenticated ciphertexts in bytes.
///
/// This is the version byte (1) + the KMAC128 tag (32) = 33 bytes.
///
/// **Changing this** would break compatibility with existing authenticated
/// ciphertexts.
pub const AUTH_OVERHEAD: usize = 1 + AUTH_TAG_LEN;

// Extra bytes for AEAD authentication tag (ChaCha20Poly1305).
// (Commented out: not currently used within the `kelvin` crate.
//  The AEAD tag size is handled internally by ChaChaStream.)
// pub const AEAD_TAG_SIZE: usize = 16;
