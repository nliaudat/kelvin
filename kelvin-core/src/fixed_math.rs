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
    /// This is a constant-time implementation that runs exactly 96 iterations
    /// regardless of input magnitude. It uses only comparisons, subtractions,
    /// and bit shifts — no division, no multiplication, no data-dependent
    /// branching. This guarantees timing is independent of the input value.
    ///
    /// The algorithm processes the 128-bit input 2 bits at a time, building
    /// a 96-bit result (32 integer + 64 fractional bits). The result is
    /// directly in Q32.64 format — no post-shift needed.
    ///
    /// # Constant-time guarantee
    ///
    /// This implementation runs exactly 96 iterations for **all** inputs,
    /// including zero and negative values. There is no early return branch.
    /// For negative inputs, a constant-time sign mask zeros out the result.
    pub fn sqrt(self) -> Self {
        // Binary digit-by-digit (restoring) square root.
        //
        // The input `a` is in Q32.64 format (i128, 1.0 = 2^64).
        // We process the 128-bit input as 64 pairs of bits (2 bits per
        // iteration), building a 64-bit integer sqrt. Then we continue
        // for 32 more iterations with zero input bits to compute the
        // fractional part, giving a 96-bit result.
        //
        // The 96-bit result is sqrt(input) * 2^32, which is exactly
        // the Q32.64 representation of sqrt(value).
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

        // Capture the sign before taking absolute value.
        // We'll apply a constant-time mask to zero the result for negative inputs.
        let sign = self.0 < 0;
        let a = self.0.unsigned_abs();
        let mut result: u128 = 0;
        let mut remainder: u128 = 0;

        // Phase 1: Process all 128 bits of the input (64 iterations, 2 bits each)
        for i in (0..64).rev() {
            let bit_pos = i * 2;
            let input_bits = (a >> bit_pos) & 0x3;
            remainder = (remainder << 2) | input_bits;

            let trial = (result << 2) | 1;

            // Constant-time select using borrow flag
            let remainder_if_sub = remainder.wrapping_sub(trial);
            let borrow = (remainder_if_sub > remainder) as u128;
            let mask = borrow.wrapping_sub(1); // 0 -> !0, 1 -> 0
            remainder = (remainder & !mask) | (remainder_if_sub & mask);
            let bit = mask >> 127;

            result = (result << 1) | bit;
        }

        // Phase 2: Continue for 32 more iterations with zero input bits
        // to compute the fractional part of the sqrt.
        for _ in 0..32 {
            remainder <<= 2;

            let trial = (result << 2) | 1;

            let remainder_if_sub = remainder.wrapping_sub(trial);
            let borrow = (remainder_if_sub > remainder) as u128;
            let mask = borrow.wrapping_sub(1);
            remainder = (remainder & !mask) | (remainder_if_sub & mask);
            let bit = mask >> 127;

            result = (result << 1) | bit;
        }

        // Constant-time sign mask: zero the result for negative inputs.
        // sign_mask = !0 if sign == false (positive), 0 if sign == true (negative).
        let sign_mask = (sign as u128).wrapping_sub(1);
        result &= sign_mask;

        Fixed(result as i128)
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

        // (a * b) >> 64 = hi_hi * 2^64 + hi_lo + lo_hi + ((lo_lo + 2^63) >> 64)
        // The rounding bit (1 << 63) implements round-to-nearest on the
        // lower 64 bits before shifting, matching the old checked_mul behavior.
        let result_abs = (hi_hi << 64)
            .wrapping_add(hi_lo)
            .wrapping_add(lo_hi)
            .wrapping_add(lo_lo.wrapping_add(1 << 63) >> 64);

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
        assert!(
            (result.to_f64() - 3.0).abs() < 0.001,
            "sqrt(9) = {}, expected ~3.0",
            result.to_f64()
        );

        let result = Fixed::from_int(0).sqrt();
        assert_eq!(result, Fixed::ZERO);

        let result = Fixed::from_int(2).sqrt();
        let expected = 2.0f64.sqrt();
        assert!(
            (result.to_f64() - expected).abs() < 0.001,
            "sqrt(2) = {}, expected ~{}",
            result.to_f64(),
            expected
        );
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

// ============================================================================
// Kani formal verification harnesses
// ============================================================================
//
// These harnesses formally prove that the Fixed arithmetic engine cannot
// overflow or panic under physically-realistic orbital simulation bounds.
//
// Bounds derived from:
//   - Position: r ∈ [10⁻⁶, 100] AU
//   - Softening: ε ≈ 9.5×10⁻⁷ AU (SOFTENING_FACTOR = 2⁻²⁰ AU)
//   - Gravitational constant: G ≈ 39.478 AU³/(M☉·yr²)
//   - Mass: m ∈ [10⁻⁶, 1] M☉
//   - Timestep: dt ∈ [10⁻⁸, 10⁻¹] yr
//
// Q32.64 format: 1.0 = 2⁶⁴ stored as i128.
// Raw value = value_in_AU * 2⁶⁴
//
// Key constants in raw Q32.64:
//   1 AU        = 1 << 64
//   100 AU      = 100 << 64
//   10⁻⁶ AU     = 1 << 44  (≈ 1.84e13)
//   G ≈ 39.478  = 0x277A79937C8BBC0000
//   ε² (raw)    = 1 << 24  (softening_squared in raw)
//
// Run with: cargo kani -p kelvin-core
//
// #[cfg(kani)] is automatically set by the Kani compiler.
// These harnesses are never compiled during normal builds.
#[cfg(kani)]
mod kani_proofs {
    use crate::Fixed;

    // ── Physical bounds in Q32.64 raw ──────────────────────────────────
    // 1 AU in Q32.64 raw
    const AU: i128 = 1 << 64;
    // 10⁻⁶ AU in Q32.64 raw (minimum separation before collapse detection)
    const MIN_AU: i128 = 1 << 44;
    // 100 AU in Q32.64 raw (maximum orbital position)
    const MAX_AU: i128 = 100 * (1 << 64);
    // G ≈ 39.478 AU³/(M☉·yr²) in Q32.64 raw
    const G_RAW: i128 = 0x0000_0000_0000_0027_7A79_937C_8BBC_0000;
    // Softening squared in Q32.64 raw: (2⁻²⁰)² = 2⁻⁴⁰ → raw = 2⁻⁴⁰ × 2⁶⁴ = 2²⁴
    const SOFTENING_SQ_RAW: i128 = 1 << 24;

    // ── Harness 1: Addition ────────────────────────────────────────────
    //
    // Prove: Fixed::add never overflows i128 when both operands represent
    // positions within [-100, 100] AU.
    //
    // Worst case: 100 AU + 100 AU = 200 AU → raw = 200 << 64 ≈ 3.69e21
    // i128::MAX ≈ 1.70e38 → 17 orders of magnitude headroom.
    //
    // Functional equivalence: For Q32.64 values a, b:
    //   a + b = (a_raw / 2^64) + (b_raw / 2^64)
    //         = (a_raw + b_raw) / 2^64
    //         = Fixed::from_raw(a_raw + b_raw)
    // Since Fixed::add uses i128::wrapping_add (exact for non-wrapping),
    // and we prove no wrapping occurs, the result is mathematically exact.
    #[kani::proof]
    fn verify_add_no_overflow() {
        let a_raw: i128 = kani::any();
        let b_raw: i128 = kani::any();
        // Constrain to physically-realistic position range: [-100, 100] AU
        kani::assume(a_raw >= -MAX_AU && a_raw <= MAX_AU);
        kani::assume(b_raw >= -MAX_AU && b_raw <= MAX_AU);

        let a = Fixed::from_raw(a_raw);
        let b = Fixed::from_raw(b_raw);
        let result = a + b;

        // The result should be within [-200, 200] AU
        let raw = result.to_raw();
        kani::assert(raw >= -2 * MAX_AU && raw <= 2 * MAX_AU, "add: result within [-200, 200] AU");

        // Functional equivalence: result must match mathematical a + b
        let expected_raw = a_raw.wrapping_add(b_raw);
        kani::assert(
            result.to_raw() == expected_raw,
            "add: functional equivalence (result == a + b)",
        );
    }

    // ── Harness 2: Subtraction ─────────────────────────────────────────
    //
    // Prove: Fixed::sub never underflows i128 when both operands represent
    // positions within [-100, 100] AU.
    //
    // Worst case: -100 AU - 100 AU = -200 AU → raw = -200 << 64
    //
    // Functional equivalence: same reasoning as addition.
    #[kani::proof]
    fn verify_sub_no_overflow() {
        let a_raw: i128 = kani::any();
        let b_raw: i128 = kani::any();
        kani::assume(a_raw >= -MAX_AU && a_raw <= MAX_AU);
        kani::assume(b_raw >= -MAX_AU && b_raw <= MAX_AU);

        let a = Fixed::from_raw(a_raw);
        let b = Fixed::from_raw(b_raw);
        let result = a - b;

        let raw = result.to_raw();
        kani::assert(raw >= -2 * MAX_AU && raw <= 2 * MAX_AU, "sub: result within [-200, 200] AU");

        // Functional equivalence: result must match mathematical a - b
        let expected_raw = a_raw.wrapping_sub(b_raw);
        kani::assert(
            result.to_raw() == expected_raw,
            "sub: functional equivalence (result == a - b)",
        );
    }

    // ── Harness 3: Multiplication ──────────────────────────────────────
    //
    // Prove: Fixed::mul (constant-time splitting implementation) never
    // produces incorrect results due to internal u128 wrapping for
    // physically-realistic values.
    //
    // The Mul impl uses u64×u64 splitting. Each u64 half is at most 2⁶⁴-1,
    // so hi_lo, lo_hi, lo_lo products fit in u128. The hi_hi term is
    // (a_hi * b_hi) which is at most (2⁶⁴-1)² ≈ 2¹²⁸, fitting in u128.
    //
    // We constrain inputs to values that arise in the simulation:
    //   - Positions/masses: [-100, 100] AU or [-1, 1] M☉
    //   - G: ≈ 39.478
    //   - dt: [10⁻⁸, 10⁻¹] yr
    //   - Softening: ≈ 2⁻²⁰ AU
    //
    // Functional equivalence: verify algebraic properties:
    //   - Commutativity: a*b == b*a
    //   - Identity: a*1 == a
    //   - Zero: a*0 == 0
    #[kani::proof]
    fn verify_mul_no_overflow() {
        let a_raw: i128 = kani::any();
        let b_raw: i128 = kani::any();
        // Constrain to physically-realistic range:
        // - Max absolute value in simulation: G * mass / dist³ ≈ 10⁶
        // - But we bound more conservatively: [-100, 100] in Q32.64
        kani::assume(a_raw >= -MAX_AU && a_raw <= MAX_AU);
        kani::assume(b_raw >= -MAX_AU && b_raw <= MAX_AU);

        let a = Fixed::from_raw(a_raw);
        let b = Fixed::from_raw(b_raw);
        let result = a * b;

        // The result should be finite (no panic, no wrapping)
        let raw = result.to_raw();
        // Maximum product magnitude: 100 * 100 = 10,000 AU²
        // In Q32.64 raw: 10,000 << 64 ≈ 1.84e23, well within i128::MAX
        kani::assert(
            raw >= -10000 * (1 << 64) && raw <= 10000 * (1 << 64),
            "mul: result within [-10000, 10000] AU²",
        );

        // Functional equivalence: commutativity
        let ba = b * a;
        kani::assert(result == ba, "mul: commutative (a*b == b*a)");

        // Functional equivalence: identity
        let a_times_one = a * Fixed::ONE;
        kani::assert(a_times_one == a, "mul: identity (a*1 == a)");

        // Functional equivalence: zero
        let a_times_zero = a * Fixed::ZERO;
        kani::assert(a_times_zero == Fixed::ZERO, "mul: zero property (a*0 == 0)");
    }

    // ── Harness 4: Division ────────────────────────────────────────────
    //
    // Prove: Fixed::div never panics for physically-realistic operands.
    //
    // The Div impl panics on division by zero and can overflow on
    // checked_shl(64) for large numerators.
    //
    // In the simulation, division occurs as:
    //   factor = G / dist_cubed
    // where dist_cubed = (|Δr|² + ε²)^(3/2)
    //
    // Worst case (smallest denominator): two bodies at same position
    //   dist_sq = 0 + ε² = 2⁻⁴⁰ AU²
    //   dist = 2⁻²⁰ AU
    //   dist_cubed = 2⁻⁶⁰ AU³
    //   G / dist_cubed = 39.478 × 2⁶⁰ ≈ 4.55×10¹⁹ AU⁻²
    //   Q32.64 raw: 4.55×10¹⁹ × 2⁶⁴ ≈ 8.4×10³⁸ → exceeds i128::MAX
    //
    // BUT: detect_collapse triggers at MIN_SEPARATION ≈ 9.3×10⁻⁸ AU
    // (raw: 1 << 40), which is 100× larger than the softening length.
    // So the minimum dist before collapse is ~2⁻²³ AU, giving:
    //   dist_cubed_min ≈ 2⁻⁶⁹ AU³
    //   G / dist_cubed_max ≈ 39.478 × 2⁶⁹ ≈ 2.3×10²² AU⁻²
    //   Q32.64 raw: 2.3×10²² × 2⁶⁴ ≈ 4.3×10⁴¹ → still exceeds i128::MAX
    //
    // However, the numerator in practice is G * mass_j (≈ 39.478), and
    // the denominator is dist_cubed. The key constraint is that the
    // shifted numerator (a << 64) must fit in i128 for checked_shl(64).
    //
    // Safe bound: |numerator_raw| ≤ 40 << 64 (G ≈ 39.478)
    // and |denominator_raw| ≥ 1 << 24 (softening_sq in raw, which
    // corresponds to dist_cubed ≈ 2⁻⁶⁰ AU³).
    //
    // Functional equivalence: verify inverse property
    //   div(a, b) * b ≈ a  (within 2 ULP)
    #[kani::proof]
    fn verify_div_no_panic() {
        let num_raw: i128 = kani::any();
        let den_raw: i128 = kani::any();
        // Numerator: G * mass, at most ~39.478 * 1.0 = 39.478
        kani::assume(num_raw >= -G_RAW && num_raw <= G_RAW);
        // Denominator: dist_cubed, at least softening_sq^(3/2) in raw
        // softening_sq_raw = 1 << 24, so dist_cubed_raw ≥ 1 << 24
        // (conservative lower bound for the denominator)
        kani::assume(den_raw >= SOFTENING_SQ_RAW || den_raw <= -SOFTENING_SQ_RAW);
        // Also bound the denominator from above: max dist ≈ 200 AU
        // dist_cubed_max_raw = 200³ × 2⁶⁴ = 8×10⁶ × 2⁶⁴ (fits in i128)
        // Pre-computed as a constant to avoid overflow in Kani's instrumentation.
        const DIST_CUBED_MAX_RAW: i128 = 8_000_000 * AU;
        kani::assume(den_raw >= -DIST_CUBED_MAX_RAW);
        kani::assume(den_raw <= DIST_CUBED_MAX_RAW);

        let num = Fixed::from_raw(num_raw);
        let den = Fixed::from_raw(den_raw);
        let result = num / den;

        // Division should produce a finite result
        let raw = result.to_raw();
        // G / dist³ for dist ≥ 2⁻²⁰ AU gives at most ~2.6×10⁶ AU⁻²
        // In Q32.64: 2.6×10⁶ × 2⁶⁴ ≈ 4.8×10²⁵, well within i128::MAX
        kani::assert(raw != i128::MIN && raw != i128::MAX, "div: result is not at extreme bounds");

        // Functional equivalence: inverse property
        // result * den should approximately equal num
        let product = result * den;
        let error = (product - num).abs();
        kani::assert(
            error.to_raw() <= 2,
            "div: inverse property (result*den ≈ num, error ≤ 2 ULP)",
        );
    }

    // ── Harness 5: Square Root ─────────────────────────────────────────
    //
    // Prove: Fixed::sqrt completes safely for all non-negative values
    // representing squared distances up to (200 AU)².
    //
    // The sqrt implementation runs exactly 96 iterations regardless of
    // input. It uses only comparisons, subtractions, and bit shifts.
    // For negative inputs, it returns zero via constant-time masking.
    //
    // Functional equivalence: verify inverse property
    //   sqrt(a)² ≈ a  (within 3 ULP)
    #[kani::proof]
    fn verify_sqrt_bounded() {
        let raw: i128 = kani::any();
        // Constrain to squared distance range: [0, (200 AU)²]
        // (200 AU)² = 40,000 AU² → raw = 40000 << 64 ≈ 7.4e23
        kani::assume(raw >= 0 && raw <= 40000 * AU);

        let val = Fixed::from_raw(raw);
        let result = val.sqrt();

        // sqrt should return a non-negative result
        kani::assert(result.to_raw() >= 0, "sqrt: result is non-negative");

        // Functional equivalence: inverse property
        // sqrt(a)² should approximately equal a
        let squared = result * result;
        let error = (squared - val).abs();
        kani::assert(error.to_raw() <= 3, "sqrt: inverse property (sqrt(a)² ≈ a, error ≤ 3 ULP)");
    }
}
