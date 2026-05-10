//! Q32.64 fixed-point arithmetic.
//!
//! Format: Q32.64 stored as `i128` where `1.0 = 2^64`.
//!
//! - 32 integer bits (supports values up to ~4.29e9)
//! - 64 fractional bits (resolution ~5.4e-20)
//!
//! All operations are deterministic across platforms (pure integer math).
//!
//! ## References
//!
//! - Goldberg, D. (1991). "What Every Computer Scientist Should Know About
//!   Floating-Point Arithmetic." *ACM Computing Surveys*, 23(1), 5–48.
//!   doi:10.1145/103162.103163
//!   — Motivates the use of fixed-point arithmetic for cross-platform
//!     determinism in cryptographic applications.
//! - Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical
//!   Integration: Structure-Preserving Algorithms for Ordinary Differential
//!   Equations* (2nd ed.). Springer.
//!   — Theoretical foundation for structure-preserving integrators that
//!     require deterministic arithmetic.

use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use core::fmt;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// The scaling factor: 2^64
const SCALE: i128 = 1 << 64;

/// Q32.64 fixed-point number.
///
/// Stored as `i128` where `1.0 = 2^64`.
/// Range: approximately [-4.29e9, 4.29e9] with ~5.4e-20 precision.
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fixed(i128);

impl Fixed {
    /// Zero value.
    pub const ZERO: Fixed = Fixed(0);

    /// One value (1.0 in Q32.64).
    pub const ONE: Fixed = Fixed(SCALE);

    /// Maximum representable value.
    pub const MAX: Fixed = Fixed(i128::MAX);

    /// Minimum representable value.
    pub const MIN: Fixed = Fixed(i128::MIN);

    /// Create from raw i128 value (internal representation).
    #[inline]
    pub const fn from_raw(v: i128) -> Self {
        Fixed(v)
    }

    /// Create from integer (no fractional part).
    #[inline]
    pub const fn from_int(v: i64) -> Self {
        Fixed((v as i128) << 64)
    }

    /// Create from integer and fractional parts.
    ///
    /// `int` is the integer part, `frac` is the fractional part as a u64
    /// representing the fraction of 2^64.
    #[inline]
    pub fn from_parts(int: i64, frac: u64) -> Self {
        let int_part = (int as i128) << 64;
        let frac_part = frac as i128;
        Fixed(int_part.wrapping_add(frac_part))
    }

    /// Convert to f64 (for debugging/display only, NOT in core engine).
    #[inline]
    pub fn to_f64(self) -> f64 {
        let int_part = (self.0 >> 64) as f64;
        // Extract the lower 64 bits as the fractional part
        // Use wrapping to handle negative values correctly
        let frac_bits = self.0 as u64;
        let frac_part = (frac_bits as f64) / (18446744073709551616.0); // 2^64 as f64
        int_part + frac_part
    }

    /// Return raw i128 value.
    #[inline]
    pub const fn to_raw(self) -> i128 {
        self.0
    }

    /// Compute square root using Newton's method.
    ///
    /// Uses 20 iterations for constant-time operation.
    /// Returns the floor of the square root.
    pub fn sqrt(self) -> Self {
        if self.0 <= 0 {
            return Fixed::ZERO;
        }

        // For very small values, return early to avoid division by zero
        if self < Fixed::from_raw(1 << 8) {
            return Fixed::ZERO;
        }

        // Initial guess: use the value itself (good for values near 1.0)
        let mut x = self;

        // Newton's method: x_{n+1} = (x_n + a/x_n) / 2
        // 20 iterations is more than enough for convergence
        for _ in 0..20 {
            let div = self / x;
            x = (x + div) / Fixed::from_int(2);
        }

        x
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        Fixed(self.0.abs())
    }

    /// Check if value is zero.
    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Check if value is negative.
    #[inline]
    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Checked multiplication. Returns None on overflow.
    #[inline]
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let a = self.0;
        let b = rhs.0;
        // (a * b) >> 64 with rounding
        let product = a.checked_mul(b)?;
        let rounded = if product >= 0 {
            product + (SCALE >> 1)
        } else {
            product - (SCALE >> 1)
        };
        Some(Fixed(rounded >> 64))
    }

    /// Checked division. Returns None on overflow or division by zero.
    #[inline]
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        let a = self.0;
        let b = rhs.0;
        // (a << 64) / b
        let shifted = a.checked_shl(64)?;
        Some(Fixed(shifted / b))
    }
}

// Arithmetic operations

impl Add for Fixed {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Fixed(self.0 + rhs.0)
    }
}

impl AddAssign for Fixed {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Fixed {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Fixed(self.0 - rhs.0)
    }
}

impl SubAssign for Fixed {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for Fixed {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        // (a * b) >> 64 with rounding
        // Use checked multiplication; if it overflows i128, fall back to
        // splitting into high/low parts for correct fixed-point arithmetic.
        let a = self.0;
        let b = rhs.0;
        match a.checked_mul(b) {
            Some(product) => {
                let rounded = if product >= 0 {
                    product + (SCALE >> 1)
                } else {
                    product - (SCALE >> 1)
                };
                Fixed(rounded >> 64)
            }
            None => {
                // Fallback: split into high/low 64-bit halves
                // a = a_hi * 2^64 + a_lo
                // b = b_hi * 2^64 + b_lo
                // a * b = (a_hi * b_hi) * 2^128 + (a_hi * b_lo + a_lo * b_hi) * 2^64 + a_lo * b_lo
                // We need (a * b) >> 64 = a_hi * b_hi * 2^64 + a_hi * b_lo + a_lo * b_hi + (a_lo * b_lo) >> 64
                let a_hi = a >> 64;
                let a_lo = a as u64;
                let b_hi = b >> 64;
                let b_lo = b as u64;

                let hi_hi = a_hi * b_hi; // i64 * i64 fits in i128
                let hi_lo = a_hi * (b_lo as i128); // i64 * u64 fits in i128
                let lo_hi = (a_lo as i128) * b_hi; // u64 * i64 fits in i128
                let lo_lo = (a_lo as u128) * (b_lo as u128); // u64 * u64 fits in u128

                // (a * b) >> 64 = hi_hi * 2^64 + hi_lo + lo_hi + (lo_lo >> 64)
                // This result is already in Q32.64 format, no extra shift needed.
                // Rounding is not applied here since the result is already at the
                // target precision (the lo_lo term provides fractional rounding).
                let result = (hi_hi << 64)
                    .wrapping_add(hi_lo)
                    .wrapping_add(lo_hi)
                    .wrapping_add((lo_lo >> 64) as i128);
                Fixed(result)
            }
        }
    }
}

impl MulAssign for Fixed {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for Fixed {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        if rhs.0 == 0 {
            panic!("Fixed::div: division by zero");
        }
        let a = self.0;
        let b = rhs.0;
        // Compute (a << 64) / b using long division to avoid overflow.
        //
        // We split a into 32-bit chunks: a = a2 * 2^64 + a1 * 2^32 + a0
        // Then (a << 64) / b = (a2 * 2^128 + a1 * 2^96 + a0 * 2^64) / b
        //
        // Using long division:
        //   remainder = 0
        //   for each chunk from high to low:
        //     remainder = remainder * 2^32 + chunk
        //     result_chunk = remainder / b
        //     remainder = remainder % b
        //
        // We process 3 chunks (a2, a1, a0) to get the full (a << 64) / b result.
        // a2 = upper 32 bits of a (bits 95..64)
        // a1 = bits 63..32 of a
        // a0 = bits 31..0 of a

        // Extract 32-bit chunks from a (which is in Q32.64 format)
        // a = a2 * 2^64 + a1 * 2^32 + a0
        let a2 = (a >> 64) as u32 as i128; // bits 95..64 (sign-extended)
        let a1 = ((a >> 32) as u32) as i128; // bits 63..32
        let a0 = (a as u32) as i128; // bits 31..0

        // Long division: compute (a2 * 2^64 + a1 * 2^32 + a0) * 2^64 / b
        // = (a2 * 2^128 + a1 * 2^96 + a0 * 2^64) / b
        
        // Start with remainder = 0
        // Process a2 (which is at position 2^64 in the original a, but we're computing a << 64)
        // Actually, (a << 64) = a2 * 2^128 + a1 * 2^96 + a0 * 2^64
        // We process from high to low: a2, a1, a0, then 0 (for the fractional part)
        
        let mut rem = a2;
        let mut result;
        
        // Process a2 (at position 2^128)
        let q2 = rem / b;
        rem %= b;
        result = q2; // q2 is at position 2^64 in the final result
        
        // Process a1 (at position 2^96, which is 2^32 in the result)
        rem = (rem << 32) + a1;
        let q1 = rem / b;
        rem %= b;
        result = (result << 32) + q1;
        
        // Process a0 (at position 2^64, which is 2^0 in the result)
        rem = (rem << 32) + a0;
        let q0 = rem / b;
        rem %= b;
        result = (result << 32) + q0;
        
        // Process the fractional part (we need 64 bits of fraction)
        // rem is the remainder after processing all of a's bits
        // We need to continue dividing to get 64 fractional bits
        // Each iteration gives us 32 bits of fraction
        for _ in 0..2 {
            rem <<= 32;
            let q = rem / b;
            rem %= b;
            result = (result << 32) + q;
        }
        
        Fixed(result)
    }
}

impl DivAssign for Fixed {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Neg for Fixed {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Fixed(-self.0)
    }
}

// Comparison

impl PartialEq for Fixed {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Fixed {}

impl PartialOrd for Fixed {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fixed {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

// Display

impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.to_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(Fixed::ZERO, Fixed::from_int(0));
        assert_eq!(Fixed::ONE, Fixed::from_int(1));
        assert!(Fixed::MAX > Fixed::ZERO);
        assert!(Fixed::MIN < Fixed::ZERO);
    }

    #[test]
    fn test_from_int() {
        assert_eq!(Fixed::from_int(0).to_f64(), 0.0);
        assert_eq!(Fixed::from_int(1).to_f64(), 1.0);
        assert_eq!(Fixed::from_int(-1).to_f64(), -1.0);
        assert_eq!(Fixed::from_int(42).to_f64(), 42.0);
    }

    #[test]
    fn test_from_parts() {
        let v = Fixed::from_parts(3, 1 << 63); // 3.5
        assert!((v.to_f64() - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_add() {
        assert_eq!(Fixed::from_int(2) + Fixed::from_int(3), Fixed::from_int(5));
        assert_eq!(Fixed::from_int(-1) + Fixed::from_int(1), Fixed::ZERO);
    }

    #[test]
    fn test_sub() {
        assert_eq!(Fixed::from_int(5) - Fixed::from_int(3), Fixed::from_int(2));
        assert_eq!(Fixed::from_int(3) - Fixed::from_int(5), Fixed::from_int(-2));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Fixed::from_int(3) * Fixed::from_int(4), Fixed::from_int(12));
        assert_eq!(Fixed::from_int(-2) * Fixed::from_int(3), Fixed::from_int(-6));
        assert_eq!(Fixed::from_int(0) * Fixed::from_int(5), Fixed::ZERO);
    }

    #[test]
    fn test_div() {
        let result = Fixed::from_int(10) / Fixed::from_int(3);
        let expected = 10.0 / 3.0;
        assert!((result.to_f64() - expected).abs() < 0.001);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_div_by_zero() {
        let _ = Fixed::from_int(1) / Fixed::ZERO;
    }

    #[test]
    fn test_neg() {
        assert_eq!(-Fixed::from_int(5), Fixed::from_int(-5));
        assert_eq!(-Fixed::ZERO, Fixed::ZERO);
    }

    #[test]
    fn test_sqrt() {
        let result = Fixed::from_int(9).sqrt();
        assert!((result.to_f64() - 3.0).abs() < 0.001);

        let result = Fixed::from_int(0).sqrt();
        assert_eq!(result, Fixed::ZERO);

        let result = Fixed::from_int(2).sqrt();
        let expected = 2.0f64.sqrt();
        assert!((result.to_f64() - expected).abs() < 0.001);
    }

    #[test]
    fn test_abs() {
        assert_eq!(Fixed::from_int(5).abs(), Fixed::from_int(5));
        assert_eq!(Fixed::from_int(-5).abs(), Fixed::from_int(5));
        assert_eq!(Fixed::ZERO.abs(), Fixed::ZERO);
    }

    #[test]
    fn test_comparison() {
        assert!(Fixed::from_int(3) > Fixed::from_int(2));
        assert!(Fixed::from_int(2) < Fixed::from_int(3));
        assert_eq!(Fixed::from_int(2), Fixed::from_int(2));
        assert!(Fixed::from_int(-1) < Fixed::ZERO);
    }

    #[test]
    fn test_commutative() {
        let a = Fixed::from_int(7);
        let b = Fixed::from_int(11);
        assert_eq!(a + b, b + a);
        assert_eq!(a * b, b * a);
    }

    #[test]
    fn test_associative() {
        let a = Fixed::from_int(2);
        let b = Fixed::from_int(3);
        let c = Fixed::from_int(5);
        assert_eq!((a + b) + c, a + (b + c));
        assert_eq!((a * b) * c, a * (b * c));
    }

    #[test]
    fn test_checked_mul_overflow() {
        // 1000 * 1000 = 1,000,000 which fits in Q32.64 but overflows i128
        // during the intermediate (a * b) computation, so checked_mul returns None.
        let result = Fixed::from_int(1000).checked_mul(Fixed::from_int(1000));
        assert!(result.is_none());
        // The regular mul handles this via the fallback path
        let result2 = Fixed::from_int(1000) * Fixed::from_int(1000);
        assert_eq!(result2, Fixed::from_int(1_000_000));
    }

    #[test]
    fn test_checked_div_by_zero() {
        assert!(Fixed::from_int(1).checked_div(Fixed::ZERO).is_none());
    }

    #[test]
    fn test_raw_roundtrip() {
        let v = Fixed::from_int(42);
        assert_eq!(Fixed::from_raw(v.to_raw()), v);
    }

    #[test]
    fn test_add_assign() {
        let mut a = Fixed::from_int(5);
        a += Fixed::from_int(3);
        assert_eq!(a, Fixed::from_int(8));
    }

    #[test]
    fn test_mul_assign() {
        let mut a = Fixed::from_int(5);
        a *= Fixed::from_int(3);
        assert_eq!(a, Fixed::from_int(15));
    }

    #[test]
    fn test_div_assign() {
        let mut a = Fixed::from_int(15);
        a /= Fixed::from_int(3);
        assert_eq!(a, Fixed::from_int(5));
    }

    #[test]
    fn test_sub_assign() {
        let mut a = Fixed::from_int(5);
        a -= Fixed::from_int(3);
        assert_eq!(a, Fixed::from_int(2));
    }
}
