//! # kelvin-core
//!
//! Fixed-point math and n-body simulation engine for the Kelvin cryptosystem.
//!
//! This crate provides:
//! - `Fixed` — Q32.64 fixed-point arithmetic (deterministic across platforms)
//! - `Vec3` — 3D vector math
//! - `OrbitalBody` — celestial body data structure
//! - Physical constants for AU-scale gravitational simulation
//! - Symplectic Verlet integrator (kick-drift-kick)
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.** This is an experimental
//! cryptosystem that has not undergone formal cryptanalysis.
//!
//! ## no-std support
//!
//! This crate is `no_std` compatible. It has no runtime dependencies.

#![no_std]
#![deny(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod fixed_math;
mod body;
mod constants;
mod integrator;

pub use fixed_math::Fixed;
pub use body::{Vec3, OrbitalBody};
pub use constants::{G, SOLAR_MASS, SOFTENING_FACTOR, DEFAULT_DT, MIN_BODIES, MAX_BODIES, DEFAULT_STEPS, DEFAULT_RESEED_INTERVAL};
pub use integrator::{compute_accelerations, verlet_step, simulate};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_basics() {
        assert_eq!(Fixed::ZERO, Fixed::from_int(0));
        assert_eq!(Fixed::ONE, Fixed::from_int(1));
    }

    #[test]
    fn test_vec3_basics() {
        let v = Vec3::ZERO;
        assert_eq!(v.x, Fixed::ZERO);
        assert_eq!(v.y, Fixed::ZERO);
        assert_eq!(v.z, Fixed::ZERO);
    }
}
