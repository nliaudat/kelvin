//! Physical constants for the orbital simulation.
//!
//! All values are in AU-solar mass-year units.
//!
//! ## References
//!
//! - Murray, C. D., & Dermott, S. F. (1999). *Solar System Dynamics*.
//!   Cambridge University Press.
//!   — Reference for gravitational constant G = 4π² in AU³/(M☉·yr²).

use crate::Fixed;

/// Gravitational constant G in AU³/(M☉·yr²).
///
/// G = 4π² AU³/(M☉·yr²) when using AU, solar masses, and years.
/// This is exact by definition of the astronomical unit.
pub const DEFAULT_G: Fixed = Fixed::from_raw(0x0000_0000_0000_0027_7A79_937C_8BBC_0000);
// DEFAULT_G ≈ 39.47841760435743... (4π²)

/// Solar mass in solar masses (1.0 by definition).
pub const SOLAR_MASS: Fixed = Fixed::ONE;

/// Default softening factor to prevent singularities.
///
/// ~1e-6 AU in Q32.64 = 2^64 / 1e6 ≈ 1.84e13
pub const SOFTENING_FACTOR: Fixed = Fixed::from_raw(1 << 44);

/// Default time step in years.
///
/// ~1e-6 years ≈ 31.5 seconds
pub const DEFAULT_DT: Fixed = Fixed::from_raw(1 << 44);

/// Minimum number of bodies required for meaningful chaos.
pub const MIN_BODIES: usize = 3;

/// Maximum number of bodies supported.
pub const MAX_BODIES: usize = 100;

/// Default number of simulation steps.
pub const DEFAULT_STEPS: u64 = 1_000_000;

/// Default reseed interval (steps between key schedule reseeds).
pub const DEFAULT_RESEED_INTERVAL: u64 = 10_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g_positive() {
        assert!(DEFAULT_G > Fixed::ZERO);
    }

    #[test]
    fn test_g_approx_4pi2() {
        // G should be approximately 39.478...
        let g_f64 = DEFAULT_G.to_f64();
        assert!((g_f64 - 39.478).abs() < 0.01);
    }

    #[test]
    fn test_softening_positive() {
        assert!(SOFTENING_FACTOR > Fixed::ZERO);
    }

    #[test]
    fn test_dt_positive() {
        assert!(DEFAULT_DT > Fixed::ZERO);
    }

    #[test]
    fn test_min_bodies() {
        assert!(MIN_BODIES >= 2);
    }

    #[test]
    fn test_max_bodies() {
        assert!(MAX_BODIES >= MIN_BODIES);
    }
}
