/-
# Q32.64 fixed-point arithmetic (legacy path)

Corrected 2026-11-23. The previous revision imported Mathlib, used a bare
slash-dash opener where a comment terminator was required (so the file never
parsed), and stated theorems that were false of its own definitions. This file
now builds on `KelvinProofs.FixedPoint` and restates the raw-integer facts under
the names the documentation cites.

## Scope

Only facts that are literally true of the raw `i128` payload are proved here.
There is no entropy statement in this file: bounding a remainder does not bound
information loss.
-/
import KelvinProofs.FixedPoint

namespace KelvinLegacy.FixedPoint

open KelvinProofs.FixedPoint

/-- Raw magnitude bound for a physical quantity with `|x| ≤ 100` (in AU). -/
def AU_MAX_RAW : Int := physBound

/-- Adding two values of magnitude at most `AU_MAX_RAW` stays in
`[-2*AU_MAX_RAW, 2*AU_MAX_RAW]`. -/
theorem no_overflow_addition (a b : Int)
    (ha : -AU_MAX_RAW ≤ a ∧ a ≤ AU_MAX_RAW)
    (hb : -AU_MAX_RAW ≤ b ∧ b ≤ AU_MAX_RAW) :
    -2 * AU_MAX_RAW ≤ a + b ∧ a + b ≤ 2 * AU_MAX_RAW := by
  omega

/-- Subtracting two values of magnitude at most `AU_MAX_RAW` stays in
`[-2*AU_MAX_RAW, 2*AU_MAX_RAW]`. -/
theorem no_overflow_subtraction (a b : Int)
    (ha : -AU_MAX_RAW ≤ a ∧ a ≤ AU_MAX_RAW)
    (hb : -AU_MAX_RAW ≤ b ∧ b ≤ AU_MAX_RAW) :
    -2 * AU_MAX_RAW ≤ a - b ∧ a - b ≤ 2 * AU_MAX_RAW := by
  omega

/-- Physical-range additions fit in `i128` (re-exported from the primary module). -/
theorem add_fits_i128 (a b : Int)
    (ha : -AU_MAX_RAW ≤ a ∧ a ≤ AU_MAX_RAW)
    (hb : -AU_MAX_RAW ≤ b ∧ b ≤ AU_MAX_RAW) :
    i128Min ≤ a + b ∧ a + b ≤ i128Max :=
  phys_add_fits_i128 a b ha hb

/-- Physical-range subtractions fit in `i128`. -/
theorem sub_fits_i128 (a b : Int)
    (ha : -AU_MAX_RAW ≤ a ∧ a ≤ AU_MAX_RAW)
    (hb : -AU_MAX_RAW ≤ b ∧ b ≤ AU_MAX_RAW) :
    i128Min ≤ a - b ∧ a - b ≤ i128Max :=
  phys_sub_fits_i128 a b ha hb

/-- Multiplication of two non-negative values bounded by `M` is bounded by
`M * M`. (The implementation splits into 64-bit halves; this is the range fact
that split relies on.) -/
theorem mul_boundedness (a b M : Int)
    (ha : 0 ≤ a) (haM : a ≤ M) (hb : 0 ≤ b) (hbM : b ≤ M) :
    a * b ≤ M * M := by
  have h1 : a * b ≤ M * b := Int.mul_le_mul_of_nonneg_right haM hb
  have h2 : M * b ≤ M * M := Int.mul_le_mul_of_nonneg_left hbM (by omega)
  omega

/-- Division and remainder reconstruct the dividend: `q*b + r = a`. -/
theorem div_correctness (a b : Int) : a / b * b + a % b = a :=
  div_rem_identity a b

/-- The remainder lies in `[0, b)` for positive divisors. -/
theorem remainder_bounds (a b : Int) (hb : 0 < b) : 0 ≤ a % b ∧ a % b < b :=
  rem_bounds a b hb

/-- Square-root postcondition. If `n = q*q + r` with `0 ≤ r ≤ 2q`, then
`q*q ≤ n < (q+1)*(q+1)`: the digit-by-digit routine's invariant. -/
theorem sqrt_correctness (n q s r : Int)
    (hs : s = q * q) (hn : n = s + r) (hr : 0 ≤ r) (hr2 : r ≤ 2 * q) :
    s ≤ n ∧ n < s + 2 * q + 1 := by
  omega

end KelvinLegacy.FixedPoint
