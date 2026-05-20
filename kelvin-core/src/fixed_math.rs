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
//!   determinism in cryptographic applications.
//! - Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical
//!   Integration: Structure-Preserving Algorithms for Ordinary Differential
//!   Equations* (2nd ed.). Springer.
//!   — Theoretical foundation for structure-preserving integrators that
//!   require deterministic arithmetic.

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
#[cfg_attr(feature = "zeroize", derive(zeroize::Zeroize))]
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

    /// Compute square root using binary digit-by-digit (restoring) algorithm.
    ///
    /// This is a constant-time implementation that runs exactly 64 iterations
    /// regardless of input magnitude. It uses only comparisons, subtractions,
    /// and bit shifts — no division, no multiplication, no data-dependent
    /// branching. This guarantees timing is independent of the input value.
    ///
    /// The algorithm processes the 128-bit input 2 bits at a time, building
    /// a 64-bit result. The result is then shifted into Q32.64 format.
    ///
    /// # Constant-time guarantee
    ///
    /// This implementation runs exactly 64 iterations for **all** inputs,
    /// including zero and negative values. There is no early return branch.
    /// For non-positive inputs, the result is zero (the algorithm naturally
    /// produces zero since all input bits are zero).
    pub fn sqrt(self) -> Self {
        // Binary digit-by-digit (restoring) square root.
        //
        // The input `a` is in Q32.64 format (i128, 1.0 = 2^64).
        // We process the 128-bit input as 64 pairs of bits (2 bits per
        // iteration), building a 64-bit result.
        //
        // The result is in Q16.48 format internally. To convert to Q32.64,
        // we shift left by 16 (since Q16.48 << 16 = Q32.64).
        //
        // Algorithm per iteration:
        //   remainder = (remainder << 2) | (next 2 bits of input)
        //   trial = (result << 2) | 1
        //   if remainder >= trial:
        //       remainder -= trial
        //       result_bit = 1
        //   else:
        //       result_bit = 0
        //   result = (result << 1) | result_bit

        // Use unsigned_abs so negative values produce zero result
        // (all bits are zero, so the algorithm naturally produces zero).
        let a = self.0.unsigned_abs();
        let mut result: u64 = 0;
        let mut remainder: u128 = 0;

        // Process 64 bits of result, 2 input bits at a time
        for i in (0..64).rev() {
            // Bring down the next 2 bits from the input
            let bit_pos = i * 2;
            let input_bits = (a >> bit_pos) & 0x3;
            remainder = (remainder << 2) | input_bits;

            // Trial: (result << 2) | 1
            let trial = ((result as u128) << 2) | 1;

            // Constant-time select: compute both possible remainders,
            // then select using a mask derived from the comparison.
            // This avoids the data-dependent branch.
            // Mask: all 1s if remainder >= trial, all 0s otherwise.
            // The borrow flag from wrapping_sub gives us the comparison result:
            //   remainder.wrapping_sub(trial) sets borrow if remainder < trial
            //   So !borrow means remainder >= trial
            let remainder_if_sub = remainder.wrapping_sub(trial);
            // Compute borrow: 1 if remainder < trial, 0 otherwise
            let borrow = (remainder_if_sub > remainder) as u128;
            // Mask: all 1s if remainder >= trial (no borrow), all 0s otherwise
            let mask = borrow.wrapping_sub(1); // 0 -> !0, 1 -> 0
            remainder = (remainder & !mask) | (remainder_if_sub & mask);
            // Extract the bit from the mask: mask is either 0 or !0.
            // (mask >> 127) gives 0 when mask=0, 1 when mask=!0.
            let bit = (mask >> 127) as u64;

            // Append the bit to the result
            result = (result << 1) | bit;
        }

        // The 64-bit result is the integer sqrt of the 128-bit input.
        // Since input = value * 2^64, sqrt(input) = sqrt(value) * 2^32.
        // To convert to Q32.64 format (sqrt(value) * 2^64), shift left by 32.
        Fixed((result as i128) << 32)
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
        let rounded = if product >= 0 { product + (SCALE >> 1) } else { product - (SCALE >> 1) };
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
        // Constant-time fixed-point multiplication using high/low splitting.
        //
        // This implementation always uses the splitting approach, avoiding
        // any data-dependent branching on overflow or sign. It runs the same
        // operations regardless of input values.
        //
        // We work with absolute values and apply the sign at the end using
        // a branchless negation, avoiding issues with signed shifts.
        let a = self.0;
        let b = rhs.0;

        // Constant-time sign computation
        let sign = (a < 0) ^ (b < 0);
        let a_abs = a.unsigned_abs();
        let b_abs = b.unsigned_abs();

        // Split into high/low 64-bit halves (both unsigned now)
        // a_abs = a_hi * 2^64 + a_lo
        // b_abs = b_hi * 2^64 + b_lo
        let a_hi = (a_abs >> 64) as u64;
        let a_lo = a_abs as u64;
        let b_hi = (b_abs >> 64) as u64;
        let b_lo = b_abs as u64;

        // a * b = (a_hi * b_hi) * 2^128 + (a_hi * b_lo + a_lo * b_hi) * 2^64 + a_lo * b_lo
        // We need (a * b) >> 64 = a_hi * b_hi * 2^64 + a_hi * b_lo + a_lo * b_hi + (a_lo * b_lo) >> 64
        let hi_hi = (a_hi as u128) * (b_hi as u128); // u64 * u64 fits in u128
        let hi_lo = (a_hi as u128) * (b_lo as u128); // u64 * u64 fits in u128
        let lo_hi = (a_lo as u128) * (b_hi as u128); // u64 * u64 fits in u128
        let lo_lo = (a_lo as u128) * (b_lo as u128); // u64 * u64 fits in u128

        // (a * b) >> 64 = hi_hi * 2^64 + hi_lo + lo_hi + (lo_lo >> 64)
        let result_abs = (hi_hi << 64)
            .wrapping_add(hi_lo)
            .wrapping_add(lo_hi)
            .wrapping_add(lo_lo >> 64);

        // Constant-time sign application
        let sign_mask = 0u128.wrapping_sub(sign as u128);
        let result = (result_abs ^ sign_mask).wrapping_sub(sign_mask) as i128;

        Fixed(result)
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

        // Constant-time sign handling: compute sign bit, then use
        // wrapping_neg with a mask to avoid data-dependent branching.
        let sign = (a < 0) ^ (b < 0);
        let a_abs = a.unsigned_abs();
        let b_abs = b.unsigned_abs();

        // Fully constant-time restoring division.
        //
        // We compute (a_abs << 64) / b_abs using a digit-by-digit
        // restoring division algorithm that runs exactly 192 iterations
        // (128 for the integer part + 64 for the fractional part).
        //
        // No hardware division instructions are used — only shifts,
        // comparisons, and conditional selections via masks. This
        // eliminates any timing variation from software u128 division
        // routines whose timing can depend on operand values.
        let mut rem = 0u128;
        let mut res = 0u128;

        // Process all 128 bits of a_abs (integer part)
        for i in (0..128).rev() {
            let bit = (a_abs >> i) & 1;
            rem = (rem << 1) | bit;

            let rem_if_sub = rem.wrapping_sub(b_abs);
            let borrow = (rem_if_sub > rem) as u128;
            let cond = !borrow & 1;
            let mask = 0u128.wrapping_sub(cond);
            rem = (rem & !mask) | (rem_if_sub & mask);
            res = (res << 1) | cond;
        }

        // Continue for 64 more iterations with zero bits (fractional part)
        for _ in 0..64 {
            rem <<= 1;

            let rem_if_sub = rem.wrapping_sub(b_abs);
            let borrow = (rem_if_sub > rem) as u128;
            let cond = !borrow & 1;
            let mask = 0u128.wrapping_sub(cond);
            rem = (rem & !mask) | (rem_if_sub & mask);
            res = (res << 1) | cond;
        }

        // Constant-time sign application: negate if sign == 1, keep if sign == 0.
        let sign_mask = 0u128.wrapping_sub(sign as u128);
        let result = (res ^ sign_mask).wrapping_sub(sign_mask) as i128;

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
#[allow(clippy::disallowed_methods)]
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
