//! 3D vector math and orbital body data structures.
//!
//! ## References
//!
//! - Murray, C. D., & Dermott, S. F. (1999). *Solar System Dynamics*.
//!   Cambridge University Press.
//!   — Reference for orbital mechanics, AU scaling, and gravitational
//!     constants used in Kelvin.

use crate::Fixed;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// 3D vector with fixed-point components.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "zeroize", derive(zeroize::Zeroize))]
pub struct Vec3 {
    /// X component.
    pub x: Fixed,
    /// Y component.
    pub y: Fixed,
    /// Z component.
    pub z: Fixed,
}

impl Vec3 {
    /// Zero vector.
    pub const ZERO: Vec3 = Vec3 {
        x: Fixed::ZERO,
        y: Fixed::ZERO,
        z: Fixed::ZERO,
    };

    /// Create a new 3D vector.
    #[inline]
    pub const fn new(x: Fixed, y: Fixed, z: Fixed) -> Self {
        Vec3 { x, y, z }
    }

    /// Dot product.
    #[inline]
    pub fn dot(self, other: Self) -> Fixed {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product.
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Squared length (x² + y² + z²).
    #[inline]
    pub fn length_squared(self) -> Fixed {
        self.dot(self)
    }

    /// Length (sqrt of squared length).
    #[inline]
    pub fn length(self) -> Fixed {
        self.length_squared().sqrt()
    }

    /// Scale vector by a factor.
    #[inline]
    pub fn scale(self, factor: Fixed) -> Self {
        Vec3 {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<Fixed> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Fixed) -> Self {
        self.scale(rhs)
    }
}

impl Div<Fixed> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Fixed) -> Self {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/// A celestial body in the orbital simulation.
///
/// Memory layout: 112 bytes = 7 × i128
/// - mass:      16 bytes
/// - position:  48 bytes (3 × i128)
/// - velocity:  48 bytes (3 × i128)
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "zeroize", derive(zeroize::Zeroize))]
pub struct OrbitalBody {
    /// Mass in solar masses.
    pub mass: Fixed,
    /// Position in AU.
    pub position: Vec3,
    /// Velocity in AU/year.
    pub velocity: Vec3,
}

impl OrbitalBody {
    /// Create a new orbital body.
    #[inline]
    pub const fn new(mass: Fixed, position: Vec3, velocity: Vec3) -> Self {
        OrbitalBody {
            mass,
            position,
            velocity,
        }
    }

    /// Kinetic energy: 0.5 * mass * v²
    #[inline]
    pub fn kinetic_energy(&self) -> Fixed {
        let v2 = self.velocity.length_squared();
        Fixed::from_parts(0, 1 << 63) * self.mass * v2 // 0.5 * mass * v²
    }

    /// Linear momentum: mass * velocity
    #[inline]
    pub fn momentum(&self) -> Vec3 {
        self.velocity.scale(self.mass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_zero() {
        let v = Vec3::ZERO;
        assert_eq!(v.x, Fixed::ZERO);
        assert_eq!(v.y, Fixed::ZERO);
        assert_eq!(v.z, Fixed::ZERO);
    }

    #[test]
    fn test_vec3_add() {
        let a = Vec3::new(Fixed::from_int(1), Fixed::from_int(2), Fixed::from_int(3));
        let b = Vec3::new(Fixed::from_int(4), Fixed::from_int(5), Fixed::from_int(6));
        let c = a + b;
        assert_eq!(c.x, Fixed::from_int(5));
        assert_eq!(c.y, Fixed::from_int(7));
        assert_eq!(c.z, Fixed::from_int(9));
    }

    #[test]
    fn test_vec3_sub() {
        let a = Vec3::new(Fixed::from_int(5), Fixed::from_int(7), Fixed::from_int(9));
        let b = Vec3::new(Fixed::from_int(1), Fixed::from_int(2), Fixed::from_int(3));
        let c = a - b;
        assert_eq!(c.x, Fixed::from_int(4));
        assert_eq!(c.y, Fixed::from_int(5));
        assert_eq!(c.z, Fixed::from_int(6));
    }

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::new(Fixed::from_int(1), Fixed::from_int(0), Fixed::from_int(0));
        let b = Vec3::new(Fixed::from_int(0), Fixed::from_int(1), Fixed::from_int(0));
        assert_eq!(a.dot(b), Fixed::ZERO); // orthogonal

        let c = Vec3::new(Fixed::from_int(2), Fixed::from_int(3), Fixed::from_int(4));
        let d = Vec3::new(Fixed::from_int(5), Fixed::from_int(6), Fixed::from_int(7));
        assert_eq!(c.dot(d), Fixed::from_int(2 * 5 + 3 * 6 + 4 * 7));
    }

    #[test]
    fn test_vec3_cross() {
        let a = Vec3::new(Fixed::from_int(1), Fixed::from_int(0), Fixed::from_int(0));
        let b = Vec3::new(Fixed::from_int(0), Fixed::from_int(1), Fixed::from_int(0));
        let c = a.cross(b);
        assert_eq!(c.x, Fixed::ZERO);
        assert_eq!(c.y, Fixed::ZERO);
        assert_eq!(c.z, Fixed::from_int(1));
    }

    #[test]
    fn test_vec3_length() {
        let v = Vec3::new(Fixed::from_int(3), Fixed::from_int(4), Fixed::ZERO);
        assert_eq!(v.length_squared(), Fixed::from_int(25));
        assert!((v.length().to_f64() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_vec3_scale() {
        let v = Vec3::new(Fixed::from_int(2), Fixed::from_int(3), Fixed::from_int(4));
        let s = v.scale(Fixed::from_int(3));
        assert_eq!(s.x, Fixed::from_int(6));
        assert_eq!(s.y, Fixed::from_int(9));
        assert_eq!(s.z, Fixed::from_int(12));
    }

    #[test]
    fn test_vec3_neg() {
        let v = Vec3::new(Fixed::from_int(1), Fixed::from_int(-2), Fixed::from_int(3));
        let n = -v;
        assert_eq!(n.x, Fixed::from_int(-1));
        assert_eq!(n.y, Fixed::from_int(2));
        assert_eq!(n.z, Fixed::from_int(-3));
    }

    #[test]
    fn test_vec3_add_assign() {
        let mut a = Vec3::new(Fixed::from_int(1), Fixed::from_int(2), Fixed::from_int(3));
        a += Vec3::new(Fixed::from_int(4), Fixed::from_int(5), Fixed::from_int(6));
        assert_eq!(a.x, Fixed::from_int(5));
        assert_eq!(a.y, Fixed::from_int(7));
        assert_eq!(a.z, Fixed::from_int(9));
    }

    #[test]
    fn test_vec3_sub_assign() {
        let mut a = Vec3::new(Fixed::from_int(5), Fixed::from_int(7), Fixed::from_int(9));
        a -= Vec3::new(Fixed::from_int(1), Fixed::from_int(2), Fixed::from_int(3));
        assert_eq!(a.x, Fixed::from_int(4));
        assert_eq!(a.y, Fixed::from_int(5));
        assert_eq!(a.z, Fixed::from_int(6));
    }

    #[test]
    fn test_orbital_body_kinetic_energy() {
        let body = OrbitalBody::new(
            Fixed::from_int(2),
            Vec3::ZERO,
            Vec3::new(Fixed::from_int(3), Fixed::ZERO, Fixed::ZERO),
        );
        // KE = 0.5 * 2 * 9 = 9
        assert!((body.kinetic_energy().to_f64() - 9.0).abs() < 0.001);
    }

    #[test]
    fn test_orbital_body_momentum() {
        let body = OrbitalBody::new(
            Fixed::from_int(3),
            Vec3::ZERO,
            Vec3::new(Fixed::from_int(4), Fixed::from_int(5), Fixed::from_int(6)),
        );
        let p = body.momentum();
        assert_eq!(p.x, Fixed::from_int(12));
        assert_eq!(p.y, Fixed::from_int(15));
        assert_eq!(p.z, Fixed::from_int(18));
    }

    #[test]
    fn test_orbital_body_zero_velocity() {
        let body = OrbitalBody::new(
            Fixed::from_int(5),
            Vec3::ZERO,
            Vec3::ZERO,
        );
        assert_eq!(body.kinetic_energy(), Fixed::ZERO);
        assert_eq!(body.momentum(), Vec3::ZERO);
    }
}
