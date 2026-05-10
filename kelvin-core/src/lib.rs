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
//!
//! ## References
//!
//! - Goldberg, D. (1991). "What Every Computer Scientist Should Know About
//!   Floating-Point Arithmetic." *ACM Computing Surveys*, 23(1), 5–48.
//! - Verlet, L. (1967). "Computer 'Experiments' on Classical Fluids."
//!   *Physical Review*, 159(1), 98–103.
//! - Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical
//!   Integration* (2nd ed.). Springer.
//! - Wisdom, J., & Holman, M. (1991). "Symplectic Maps for the N-Body
//!   Problem." *The Astronomical Journal*, 102(4), 1528–1538.
//! - Murray, C. D., & Dermott, S. F. (1999). *Solar System Dynamics*.
//!   Cambridge University Press.

#![no_std]
#![deny(unsafe_code)]
#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod fixed_math;
mod body;
mod constants;
mod integrator;
mod stability;

pub use fixed_math::Fixed;
pub use body::{Vec3, OrbitalBody};
pub use constants::{DEFAULT_G, SOLAR_MASS, SOFTENING_FACTOR, DEFAULT_DT, MIN_DT, MAX_DT, MIN_BODIES, MAX_BODIES, DEFAULT_STEPS, DEFAULT_RESEED_INTERVAL, MIN_SEPARATION, MONITOR_INTERVAL, EJECTION_ENERGY_THRESHOLD};
pub use integrator::{compute_accelerations, verlet_step, simulate};
pub use stability::{StabilityError, is_body_ejected, detect_collapse, simulate_with_monitoring};

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
