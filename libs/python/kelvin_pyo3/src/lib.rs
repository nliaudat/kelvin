//! # kelvin-pyo3 — Python bindings for the Kelvin cryptosystem
//!
//! Provides native Python wrappers for all Kelvin encryption modes:
//! - `Kelvin` — V1 AEAD encrypt/decrypt (ChaCha20Poly1305)
//! - `KelvinPhoton` — V3 fast stream cipher (XOR-only)
//! - `KelvinQuantum` — H quantum-resistant stream cipher (XOR + reseeding)
//! - `KelvinPhotonAuthenticated` — V3 + BLAKE3-keyed MAC tag
//! - `KelvinQuantumAuthenticated` — H + BLAKE3-keyed MAC tag
//! - `KelvinStreaming` — V2 streaming mode (one step per chunk)
//! - `generate_config` — helper to create a fixed example orbital configuration
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**
//!
//! ## Example
//!
//! ```python
//! import kelvin_pyo3
//!
//! # Generate a config
//! config = kelvin_pyo3.generate_config()
//!
//! # V1 AEAD mode
//! k = kelvin_pyo3.Kelvin(config)
//! data = bytearray(b"Hello, world!" + b"\x00" * 16)  # 16 extra bytes for AEAD tag
//! k.encrypt(data)
//! k.decrypt(data)
//! print(data[:13])  # b"Hello, world!"
//! ```

// PyO3 requires unsafe for as_bytes_mut() and as_bytes() on Bound<'_, PyByteArray>
// (these were only made safe in PyO3 0.25+)
#![allow(unsafe_code)]

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyByteArray;

use kelvin::{
    FlareScheme, Kelvin as KelvinRust, KelvinError, KelvinFlare as KelvinFlareRust,
    KelvinPhoton as KelvinPhotonRust, KelvinPhotonAuthenticated as KelvinPhotonAuthRust,
    KelvinPrism as KelvinPrismRust, KelvinQuantum as KelvinQuantumRust,
    KelvinQuantumAuthenticated as KelvinQuantumAuthRust, KelvinSplit as KelvinSplitRust,
    KelvinStreaming as KelvinStreamingRust, OrbitalConfig, PHOTON_BASE_SEED_SIZE,
    QUANTUM_BASE_SEED_SIZE,
};

// ============================================================================
// Error conversion
// ============================================================================

fn map_error(e: KelvinError) -> PyErr {
    PyRuntimeError::new_err(format!("KelvinError: {}", e))
}

// ============================================================================
// Helper: generate a fixed example orbital configuration
// ============================================================================

/// Generate a fixed example orbital configuration as a JSON string.
///
/// Creates a 5-body system (1 central mass + 4 orbiting bodies) with
/// fixed masses and positions suitable for testing.
#[pyfunction]
fn generate_config() -> String {
    use kelvin::Fixed;
    use kelvin::OrbitalBody;
    use kelvin::Vec3;

    let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
    let planet1 = OrbitalBody::new(
        Fixed::from_raw(1 << 54),
        Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
        Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
    );
    let planet2 = OrbitalBody::new(
        Fixed::from_raw(1 << 53),
        Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
        Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
    );
    let planet3 = OrbitalBody::new(
        Fixed::from_raw(1 << 52),
        Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
        Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
    );
    let planet4 = OrbitalBody::new(
        Fixed::from_raw(1 << 51),
        Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
        Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
    );

    let config = OrbitalConfig::new(
        vec![sun, planet1, planet2, planet3, planet4],
        200,
        10,
        kelvin_core::DEFAULT_DT,
        kelvin_core::SOFTENING_FACTOR,
        kelvin::DEFAULT_G,
    )
    .expect("valid config");

    serde_json::to_string(&config).expect("JSON serialization")
}

/// Parse a JSON config string into an `OrbitalConfig`.
fn parse_config(config_json: &str) -> PyResult<OrbitalConfig> {
    serde_json::from_str(config_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid orbital config: {}", e)))
}

// ============================================================================
// Helper: get mutable bytes from a PyByteArray
// ============================================================================

fn get_mut_slice<'a>(data: &'a Bound<'a, PyByteArray>) -> &'a mut [u8] {
    // SAFETY: GIL is held (we're in a pyfunction/pymethod), and we don't
    // allow concurrent access to the PyByteArray from other threads.
    unsafe { data.as_bytes_mut() }
}

#[allow(dead_code)]
fn get_bytes<'a>(data: &'a Bound<'a, PyByteArray>) -> &'a [u8] {
    // SAFETY: GIL is held.
    unsafe { data.as_bytes() }
}

// ============================================================================
// V1 Kelvin — AEAD encrypt/decrypt (ChaCha20Poly1305)
// ============================================================================

/// V1 Kelvin AEAD encrypt/decrypt.
///
/// Uses ChaCha20Poly1305 for authenticated encryption.
/// The buffer must have 16 extra bytes after the plaintext for the AEAD tag.
///
/// ```python
/// k = kelvin_pyo3.Kelvin(config_json)
/// data = bytearray(b"secret" + b"\x00" * 16)
/// k.encrypt(data)
/// k.decrypt(data)
/// ```
#[pyclass(unsendable)]
struct Kelvin {
    inner: KelvinRust,
}

#[pymethods]
impl Kelvin {
    #[new]
    fn new(config_json: &str) -> PyResult<Self> {
        let config = parse_config(config_json)?;
        let inner = KelvinRust::new(config).map_err(map_error)?;
        Ok(Kelvin { inner })
    }

    /// Encrypt data in-place. Buffer must have 16 extra bytes for AEAD tag.
    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.encrypt(slice).map_err(map_error)
    }

    /// Decrypt data in-place. Buffer must contain ciphertext + 16-byte tag.
    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.decrypt(slice).map_err(map_error)
    }

    /// Total bytes processed since initialization.
    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    /// Remaining safe bytes before orbital time exhaustion.
    fn remaining_safe_bytes(&self) -> u64 {
        self.inner.remaining_safe_bytes()
    }
}

// ============================================================================
// V3 KelvinPhoton — Fast stream cipher (XOR-only)
// ============================================================================

/// V3 Photon fast stream cipher (XOR-only, no authentication).
///
/// ```python
/// p = kelvin_pyo3.KelvinPhoton(seed_bytes, max_reseeds=1000)
/// data = bytearray(b"secret")
/// p.encrypt(data)
/// p.decrypt(data)
/// ```
#[pyclass(unsendable)]
struct KelvinPhoton {
    inner: KelvinPhotonRust,
}

#[pymethods]
impl KelvinPhoton {
    #[new]
    fn new(seed: &[u8], max_reseeds: u64) -> PyResult<Self> {
        if seed.len() != PHOTON_BASE_SEED_SIZE {
            return Err(PyValueError::new_err(format!(
                "seed must be exactly {} bytes, got {}",
                PHOTON_BASE_SEED_SIZE,
                seed.len()
            )));
        }
        let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
        seed_arr.copy_from_slice(seed);
        let inner = KelvinPhotonRust::new(seed_arr, max_reseeds);
        Ok(KelvinPhoton { inner })
    }

    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.encrypt(slice).map_err(map_error)
    }

    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.decrypt(slice).map_err(map_error)
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }

    fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

// ============================================================================
// H KelvinQuantum — Quantum-resistant stream cipher
// ============================================================================

/// H Quantum stream cipher (XOR + orbital reseeding).
///
/// ```python
/// q = kelvin_pyo3.KelvinQuantum(seed_bytes, max_reseeds=1000)
/// data = bytearray(b"secret")
/// q.encrypt(data)
/// q.decrypt(data)
/// ```
#[pyclass(unsendable)]
struct KelvinQuantum {
    inner: KelvinQuantumRust,
}

#[pymethods]
impl KelvinQuantum {
    #[new]
    fn new(seed: &[u8], max_reseeds: u64) -> PyResult<Self> {
        if seed.len() != QUANTUM_BASE_SEED_SIZE {
            return Err(PyValueError::new_err(format!(
                "seed must be exactly {} bytes, got {}",
                QUANTUM_BASE_SEED_SIZE,
                seed.len()
            )));
        }
        let mut seed_arr = [0u8; QUANTUM_BASE_SEED_SIZE];
        seed_arr.copy_from_slice(seed);
        let inner = KelvinQuantumRust::new(seed_arr, max_reseeds);
        Ok(KelvinQuantum { inner })
    }

    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.encrypt(slice).map_err(map_error)
    }

    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.decrypt(slice).map_err(map_error)
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }

    fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

// ============================================================================
// V3 Photon Authenticated — + BLAKE3-keyed MAC
// ============================================================================

/// V3 Photon with BLAKE3-keyed MAC authentication.
///
/// Appends a 32-byte MAC tag to the ciphertext on encryption,
/// and verifies it in constant time before decryption.
///
/// ```python
/// a = kelvin_pyo3.KelvinPhotonAuthenticated(seed_bytes, max_reseeds=1000)
/// data = bytearray(b"secret")
/// a.encrypt(data)  # data is now ciphertext (6 bytes) + MAC tag (32 bytes)
/// a.decrypt(data)  # tag verified and stripped; data is back to b"secret"
/// ```
#[pyclass(unsendable)]
struct KelvinPhotonAuthenticated {
    inner: KelvinPhotonAuthRust,
}

#[pymethods]
impl KelvinPhotonAuthenticated {
    #[new]
    fn new(seed: &[u8], max_reseeds: u64) -> PyResult<Self> {
        if seed.len() != PHOTON_BASE_SEED_SIZE {
            return Err(PyValueError::new_err(format!(
                "seed must be exactly {} bytes, got {}",
                PHOTON_BASE_SEED_SIZE,
                seed.len()
            )));
        }
        let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
        seed_arr.copy_from_slice(seed);
        let inner = KelvinPhotonAuthRust::new(seed_arr, max_reseeds);
        Ok(KelvinPhotonAuthenticated { inner })
    }

    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let mut buf = data.to_vec();
        self.inner.encrypt(&mut buf).map_err(map_error)?;
        data.resize(buf.len())?;
        get_mut_slice(data).copy_from_slice(&buf);
        Ok(())
    }

    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let mut buf = data.to_vec();
        self.inner.decrypt(&mut buf).map_err(map_error)?;
        data.resize(buf.len())?;
        get_mut_slice(data).copy_from_slice(&buf);
        Ok(())
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }

    fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

// ============================================================================
// H Quantum Authenticated — + BLAKE3-keyed MAC
// ============================================================================

/// H Quantum with BLAKE3-keyed MAC authentication.
///
/// Appends a 32-byte MAC tag to the ciphertext on encryption,
/// and verifies it in constant time before decryption.
///
/// ```python
/// a = kelvin_pyo3.KelvinQuantumAuthenticated(seed_bytes, max_reseeds=1000)
/// data = bytearray(b"secret")
/// a.encrypt(data)  # data is now ciphertext (6 bytes) + MAC tag (32 bytes)
/// a.decrypt(data)  # tag verified and stripped; data is back to b"secret"
/// ```
#[pyclass(unsendable)]
struct KelvinQuantumAuthenticated {
    inner: KelvinQuantumAuthRust,
}

#[pymethods]
impl KelvinQuantumAuthenticated {
    #[new]
    fn new(seed: &[u8], max_reseeds: u64) -> PyResult<Self> {
        if seed.len() != QUANTUM_BASE_SEED_SIZE {
            return Err(PyValueError::new_err(format!(
                "seed must be exactly {} bytes, got {}",
                QUANTUM_BASE_SEED_SIZE,
                seed.len()
            )));
        }
        let mut seed_arr = [0u8; QUANTUM_BASE_SEED_SIZE];
        seed_arr.copy_from_slice(seed);
        let inner = KelvinQuantumAuthRust::new(seed_arr, max_reseeds);
        Ok(KelvinQuantumAuthenticated { inner })
    }

    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let mut buf = data.to_vec();
        self.inner.encrypt(&mut buf).map_err(map_error)?;
        data.resize(buf.len())?;
        get_mut_slice(data).copy_from_slice(&buf);
        Ok(())
    }

    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let mut buf = data.to_vec();
        self.inner.decrypt(&mut buf).map_err(map_error)?;
        data.resize(buf.len())?;
        get_mut_slice(data).copy_from_slice(&buf);
        Ok(())
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }

    fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

// ============================================================================
// V2 KelvinStreaming — One step per chunk
// ============================================================================

/// V2 Streaming mode: one simulation step per data chunk.
///
/// Unlike V1 which runs the entire simulation upfront, `KelvinStreaming`
/// advances the simulation by one step for each chunk of data processed.
/// This provides unlimited keystream.
///
/// ```python
/// ks = kelvin_pyo3.KelvinStreaming(config_json, bytes_per_step=1048576)
/// data = bytearray(b"large file contents")
/// ks.encrypt(data)
/// ks.decrypt(data)
/// ```
#[pyclass(unsendable)]
struct KelvinStreaming {
    inner: KelvinStreamingRust,
}

#[pymethods]
impl KelvinStreaming {
    #[new]
    fn new(config_json: &str, bytes_per_step: u64) -> PyResult<Self> {
        let config = parse_config(config_json)?;
        let inner = KelvinStreamingRust::new(config, bytes_per_step).map_err(map_error)?;
        Ok(KelvinStreaming { inner })
    }

    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.encrypt(slice).map_err(map_error)
    }

    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.decrypt(slice).map_err(map_error)
    }

    fn step(&self) -> u64 {
        self.inner.step()
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    fn bytes_per_step(&self) -> u64 {
        self.inner.bytes_per_step()
    }

    /// Benchmark simulation speed. Returns steps/second.
    fn benchmark(&self, sample_steps: u64) -> f64 {
        self.inner.benchmark(sample_steps)
    }

    /// Estimate time to process a file.
    /// Returns (steps_needed, estimated_seconds).
    fn estimate_time(&self, file_size: u64, steps_per_sec: f64) -> (u64, f64) {
        self.inner.estimate_time(file_size, steps_per_sec)
    }
}

// ============================================================================
// KelvinPrism — OTP Key Generator for Homomorphic Encryption
// ============================================================================

/// Prism OTP key generator for homomorphic encryption.
#[pyclass(unsendable)]
struct KelvinPrism {
    inner: KelvinPrismRust,
}

#[pymethods]
impl KelvinPrism {
    #[new]
    fn new(seed: &[u8], max_reseeds: u64) -> PyResult<Self> {
        if seed.len() != PHOTON_BASE_SEED_SIZE {
            return Err(PyValueError::new_err(format!(
                "seed must be exactly {} bytes, got {}",
                PHOTON_BASE_SEED_SIZE,
                seed.len()
            )));
        }
        let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
        seed_arr.copy_from_slice(seed);
        Ok(KelvinPrism { inner: KelvinPrismRust::new(seed_arr, max_reseeds) })
    }

    fn generate_otp_key(&mut self, len: usize) -> Vec<u8> {
        self.inner.generate_otp_key(len).expect("generate_otp_key").to_vec()
    }

    fn split_key(&mut self, len: usize) -> (Vec<u8>, Vec<u8>) {
        let (a, b) = self.inner.split_key(len).expect("split_key");
        (a.to_vec(), b.to_vec())
    }

    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.encrypt(slice).map_err(map_error)
    }

    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.decrypt(slice).map_err(map_error)
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }
    fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }
    fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

/// Split XOR key-splitter for homomorphic encryption.
#[pyclass(unsendable)]
struct KelvinSplit {
    inner: KelvinSplitRust,
}

#[pymethods]
impl KelvinSplit {
    #[new]
    fn new(seed: &[u8], max_reseeds: u64) -> PyResult<Self> {
        if seed.len() != PHOTON_BASE_SEED_SIZE {
            return Err(PyValueError::new_err(format!(
                "seed must be exactly {} bytes, got {}",
                PHOTON_BASE_SEED_SIZE,
                seed.len()
            )));
        }
        let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
        seed_arr.copy_from_slice(seed);
        Ok(KelvinSplit { inner: KelvinSplitRust::new(seed_arr, max_reseeds) })
    }

    fn generate_master_key(&mut self, len: usize) -> Vec<u8> {
        self.inner.generate_master_key(len).expect("generate_master_key").to_vec()
    }

    fn split_key(&mut self, len: usize) -> (Vec<u8>, Vec<u8>) {
        let (a, b) = self.inner.split_key(len).expect("split_key");
        (a.to_vec(), b.to_vec())
    }

    fn encrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.encrypt(slice).map_err(map_error)
    }

    fn decrypt(&mut self, data: &Bound<'_, PyByteArray>) -> PyResult<()> {
        let slice = get_mut_slice(data);
        self.inner.decrypt(slice).map_err(map_error)
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }
    fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }
    fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

/// Flare chaotic FHE secret key generator.
#[pyclass(unsendable)]
struct KelvinFlare {
    inner: KelvinFlareRust,
}

#[pymethods]
impl KelvinFlare {
    #[new]
    fn new(seed: &[u8], max_reseeds: u64) -> PyResult<Self> {
        if seed.len() != PHOTON_BASE_SEED_SIZE {
            return Err(PyValueError::new_err(format!(
                "seed must be exactly {} bytes, got {}",
                PHOTON_BASE_SEED_SIZE,
                seed.len()
            )));
        }
        let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
        seed_arr.copy_from_slice(seed);
        Ok(KelvinFlare { inner: KelvinFlareRust::new(seed_arr, max_reseeds) })
    }

    fn generate_secret_key(&mut self, len: usize) -> Vec<u8> {
        self.inner.generate_secret_key(len).expect("generate_secret_key").to_vec()
    }

    fn generate_fhe_key(&mut self, scheme: i32, len: usize) -> PyResult<Vec<u8>> {
        let fs = match scheme {
            0 => FlareScheme::Bfv,
            1 => FlareScheme::Ckks,
            2 => FlareScheme::Tfhe,
            _ => return Err(PyValueError::new_err("invalid scheme (0=BFV, 1=CKKS, 2=TFHE)")),
        };
        Ok(self.inner.generate_fhe_key(fs, len).expect("generate_fhe_key").key().to_vec())
    }

    fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }
    fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }
    fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

/// Python bindings for the Kelvin Orbital Chaos KDF Cryptosystem.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_config, m)?)?;

    m.add_class::<Kelvin>()?;
    m.add_class::<KelvinPhoton>()?;
    m.add_class::<KelvinQuantum>()?;
    m.add_class::<KelvinPhotonAuthenticated>()?;
    m.add_class::<KelvinQuantumAuthenticated>()?;
    m.add_class::<KelvinStreaming>()?;
    m.add_class::<KelvinPrism>()?;
    m.add_class::<KelvinSplit>()?;
    m.add_class::<KelvinFlare>()?;

    Ok(())
}
