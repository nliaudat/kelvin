//! Lyapunov time estimation using the shadow orbit method.
//!
//! The Lyapunov time is the timescale over which nearby trajectories diverge
//! exponentially. It determines the maximum safe simulation time before
//! the orbital state becomes unpredictable (and thus useful for key derivation).
//!
//! ## References
//!
//! - Benettin, G., Galgani, L., Giorgilli, A., & Strelcyn, J.-M. (1980).
//!   "Lyapunov Characteristic Exponents for Smooth Dynamical Systems and
//!   for Hamiltonian Systems; A Method for Computing All of Them."
//!   *Meccanica*, 15(1), 9–20. doi:10.1007/BF02128236
//!   — Standard algorithm for computing Lyapunov exponents, adapted for
//!     Kelvin's shadow orbit method.
//! - Wolf, A., Swift, J. B., Swinney, H. L., & Vastano, J. A. (1985).
//!   "Determining Lyapunov Exponents from a Time Series." *Physica D:
//!   Nonlinear Phenomena*, 16(3), 285–317. doi:10.1016/0167-2789(85)90011-9
//!   — Shadow orbit method for Lyapunov estimation.
//! - Sano, M., & Sawada, Y. (1985). "Measurement of the Lyapunov Spectrum
//!   from a Chaotic Time Series." *Physical Review Letters*, 55(10),
//!   1082–1085. doi:10.1103/PhysRevLett.55.1082
//!   — Alternative Lyapunov estimation method.

use alloc::vec::Vec;
use core::fmt;

use kelvin_core::{Fixed, OrbitalBody, Vec3, verlet_step};

/// Confidence level for Lyapunov time estimation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LyapunovConfidence {
    /// Low confidence — use conservative estimate.
    Low,
    /// Medium confidence.
    Medium,
    /// High confidence — can use full estimate.
    High,
}

/// Result of Lyapunov time estimation.
#[derive(Clone, Debug)]
pub struct LyapunovResult {
    /// Estimated Lyapunov time in simulation steps.
    pub lyapunov_steps: u64,
    /// Maximum safe steps (typically Lyapunov_steps / 10).
    pub safe_steps: u64,
    /// Confidence level.
    pub confidence: LyapunovConfidence,
    /// Number of shadow orbits used.
    pub shadow_count: u32,
}

/// Lyapunov time estimator using the shadow orbit method.
///
/// Runs multiple perturbed copies of the orbital simulation and measures
/// the divergence rate.
#[derive(Debug)]
pub struct LyapunovEstimator<'a> {
    /// Reference bodies (initial conditions).
    reference: &'a [OrbitalBody],
    /// Time step.
    dt: Fixed,
    /// Softening factor.
    softening: Fixed,
}

impl<'a> LyapunovEstimator<'a> {
    /// Create a new Lyapunov estimator.
    pub fn new(
        reference: &'a [OrbitalBody],
        dt: Fixed,
        softening: Fixed,
    ) -> Self {
        LyapunovEstimator {
            reference,
            dt,
            softening,
        }
    }

    /// Estimate the Lyapunov time.
    ///
    /// `shadow_steps` is the number of steps to run each shadow orbit.
    /// `max_steps` is the maximum number of steps to consider.
    ///
    /// Returns the estimated Lyapunov time and safe steps.
    pub fn estimate(&self, shadow_steps: u64, max_steps: u64) -> Result<LyapunovResult, LyapunovError> {
        if self.reference.len() < 2 {
            return Err(LyapunovError::TooFewBodies);
        }
        if shadow_steps == 0 {
            return Err(LyapunovError::ZeroSteps);
        }

        // Run reference simulation
        let mut ref_bodies = self.reference.to_vec();
        for _ in 0..shadow_steps {
            verlet_step(&mut ref_bodies, self.dt, self.softening);
        }

        // Create shadow orbits with small perturbations
        let perturbation = Fixed::from_raw(1 << 40); // ~2^-24 ≈ 6e-8
        let mut divergences: Vec<Fixed> = Vec::new();

        // Run 3 shadow orbits (one per axis perturbation)
        for axis in 0..3 {
            let mut shadow = self.reference.to_vec();

            // Perturb position of first body
            let delta = match axis {
                0 => Vec3::new(perturbation, Fixed::ZERO, Fixed::ZERO),
                1 => Vec3::new(Fixed::ZERO, perturbation, Fixed::ZERO),
                _ => Vec3::new(Fixed::ZERO, Fixed::ZERO, perturbation),
            };
            shadow[0].position = shadow[0].position + delta;

            // Run shadow simulation
            for _ in 0..shadow_steps {
                verlet_step(&mut shadow, self.dt, self.softening);
            }

            // Measure divergence
            let mut total_div = Fixed::ZERO;
            for (r, s) in ref_bodies.iter().zip(shadow.iter()) {
                let diff = s.position - r.position;
                total_div += diff.length();
            }
            divergences.push(total_div / Fixed::from_int(self.reference.len() as i64));
        }

        // Average divergence
        let avg_divergence = divergences.iter().fold(Fixed::ZERO, |a, b| a + *b)
            / Fixed::from_int(divergences.len() as i64);

        // Estimate Lyapunov exponent λ ≈ ln(d/ε) / t
        // where d = divergence, ε = perturbation, t = time
        let initial_perturbation = perturbation;
        let time = Fixed::from_int(shadow_steps as i64) * self.dt;

        if avg_divergence <= initial_perturbation || time <= Fixed::ZERO {
            // No detectable divergence — system is stable
            return Ok(LyapunovResult {
                lyapunov_steps: max_steps,
                safe_steps: max_steps,
                confidence: LyapunovConfidence::Low,
                shadow_count: divergences.len() as u32,
            });
        }

        // λ = ln(d/ε) / t
        // We approximate ln using the fact that ln(x) ≈ 2 * (x-1)/(x+1) for x near 1
        // For larger values, we use a simpler approximation
        let ratio = avg_divergence / initial_perturbation;
        let ln_ratio = if ratio > Fixed::from_int(100) {
            // ln(100) ≈ 4.6, use a fixed cap
            Fixed::from_parts(4, 0x9999_9999_9999_9999) // ~4.6
        } else if ratio > Fixed::ONE {
            // Use approximation: ln(x) ≈ 2*(x-1)/(x+1)
            let two = Fixed::from_int(2);
            let num = ratio - Fixed::ONE;
            let den = ratio + Fixed::ONE;
            two * num / den
        } else {
            Fixed::ZERO
        };

        let lyapunov_exponent = if time > Fixed::ZERO {
            ln_ratio / time
        } else {
            Fixed::ZERO
        };

        // Lyapunov time = 1/λ (in steps)
        let lyapunov_time_steps = if lyapunov_exponent > Fixed::ZERO {
            let one_over_lambda = Fixed::ONE / lyapunov_exponent;
            // Convert to steps
            let steps_fixed = one_over_lambda / self.dt;
            // Convert to u64
            let raw = steps_fixed.to_raw() >> 64;
            if raw > 0 {
                raw as u64
            } else {
                1
            }
        } else {
            max_steps
        };

        // Safe steps = Lyapunov time / 10 (conservative)
        let safe_steps = (lyapunov_time_steps / 10).max(1).min(max_steps);

        // Determine confidence
        let confidence = if shadow_steps >= 10000 {
            LyapunovConfidence::High
        } else if shadow_steps >= 1000 {
            LyapunovConfidence::Medium
        } else {
            LyapunovConfidence::Low
        };

        Ok(LyapunovResult {
            lyapunov_steps: lyapunov_time_steps,
            safe_steps,
            confidence,
            shadow_count: divergences.len() as u32,
        })
    }
}

/// Errors from Lyapunov estimation.
#[derive(Clone, Debug)]
pub enum LyapunovError {
    /// Need at least 2 bodies for meaningful chaos.
    TooFewBodies,
    /// Shadow steps must be > 0.
    ZeroSteps,
}

impl fmt::Display for LyapunovError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LyapunovError::TooFewBodies => write!(f, "need at least 2 bodies for Lyapunov estimation"),
            LyapunovError::ZeroSteps => write!(f, "shadow steps must be > 0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::Fixed;

    fn two_body_system() -> Vec<OrbitalBody> {
        let sun = OrbitalBody::new(
            Fixed::ONE,
            Vec3::ZERO,
            Vec3::ZERO,
        );
        let planet = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        );
        vec![sun, planet]
    }

    #[test]
    fn test_lyapunov_estimate() {
        let bodies = two_body_system();
        let estimator = LyapunovEstimator::new(
            &bodies,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
        );
        let result = estimator.estimate(100, 10000).unwrap();
        assert!(result.lyapunov_steps > 0);
        assert!(result.safe_steps > 0);
        assert!(result.safe_steps <= result.lyapunov_steps);
        assert!(result.shadow_count > 0);
    }

    #[test]
    fn test_lyapunov_too_few_bodies() {
        let body = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let bodies = [body];
        let estimator = LyapunovEstimator::new(
            &bodies,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
        );
        let result = estimator.estimate(100, 10000);
        assert!(matches!(result, Err(LyapunovError::TooFewBodies)));
    }

    #[test]
    fn test_lyapunov_zero_steps() {
        let bodies = two_body_system();
        let estimator = LyapunovEstimator::new(
            &bodies,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
        );
        let result = estimator.estimate(0, 10000);
        assert!(matches!(result, Err(LyapunovError::ZeroSteps)));
    }

    #[test]
    fn test_lyapunov_safe_steps_bounded() {
        let bodies = two_body_system();
        let estimator = LyapunovEstimator::new(
            &bodies,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
        );
        let result = estimator.estimate(100, 500).unwrap();
        assert!(result.safe_steps <= 500);
    }

    #[test]
    fn test_lyapunov_confidence_levels() {
        let bodies = two_body_system();
        let estimator = LyapunovEstimator::new(
            &bodies,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
        );
        // Short run → Low confidence
        let result = estimator.estimate(10, 1000).unwrap();
        assert_eq!(result.confidence, LyapunovConfidence::Low);
    }
}
