//! Property-based fuzz test for `OrbitalConfig` deserialization.
//!
//! Generates arbitrary byte slices and attempts to parse them as:
//! - JSON (via `OrbitalConfig::from_json`)
//! - Binary (via `OrbitalConfig::from_binary`)
//!
//! Both paths are expected to return `Err` for most inputs.
//! The test detects panics (crashes), which indicate bugs in
//! the validation bodyguards or deserialization logic.
//!
//! Run with: `cargo test -p kelvin-fuzz`
//! Heavy run: `PROPTEST_CASES=100000 cargo test -p kelvin-fuzz`

use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_deserialize_config(data: Vec<u8>) {
        // ── JSON path ──
        // Try to interpret the input as a UTF-8 JSON string.
        // Most random bytes won't be valid UTF-8, and that's fine.
        if let Ok(json_str) = std::str::from_utf8(&data) {
            // `from_json` calls serde deserialization + validate().
            // Expected outcomes:
            //   - Err(ConfigError::Serialization) — malformed JSON
            //   - Err(ConfigError::TooFewBodies | ...) — valid JSON but fails validation
            //   - Ok(config) — valid config (rare, but acceptable)
            // Panic = bug.
            let _ = kelvin_kdf::OrbitalConfig::from_json(json_str);
        }

        // ── Binary path ──
        // `from_binary` parses the custom binary format + validate().
        // Expected outcomes:
        //   - Err(ConfigError::InvalidBinary) — malformed binary data
        //   - Err(ConfigError::TooFewBodies | ...) — valid binary but fails validation
        //   - Ok(config) — valid config (rare, but acceptable)
        // Panic = bug.
        let _ = kelvin_kdf::OrbitalConfig::from_binary(&data);
    }
}
