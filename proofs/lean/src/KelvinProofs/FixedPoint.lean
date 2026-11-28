/-
# Q32.64 fixed-point arithmetic (raw-integer model)

This module models the raw integer layer of `kelvin-core/src/fixed_math.rs`.
A `Fixed` value is an `Int` holding `round(x * 2^64)`; every operation in the
Rust implementation is ultimately integer arithmetic on that `i128` payload.

## Scope

Only arithmetic facts that are literally true of the Rust implementation are
proved here. Cryptographic properties (entropy loss per step, preimage
resistance, ...) are **not** established by this file; see
`documentation/formal_verification/` for the current status of each claim.
-/
import Init

namespace KelvinProofs.FixedPoint

/-- Raw Q32.64 representation: `raw = x * 2^64`. -/
abbrev Raw := Int

/-- Number of fractional bits. -/
def scaleBits : Nat := 64

/-- `1.0` in raw units. -/
def one : Int := 2 ^ scaleBits

/-- Largest representable `i128`. -/
def i128Max : Int := 2 ^ 127 - 1

/-- Smallest representable `i128`. -/
def i128Min : Int := -(2 ^ 127)

/-- Raw magnitude bound for a physical quantity with `|x| ≤ 100`. -/
def physBound : Int := 100 * one

theorem one_pos : 0 < one := by
  decide

/-- Adding two raw values whose magnitudes are bounded by `M` cannot overflow a
signed integer with bound `N`, provided `2*M ≤ N`. -/
theorem add_no_overflow (M N a b : Int) (hN : 2 * M ≤ N)
    (ha : -M ≤ a ∧ a ≤ M) (hb : -M ≤ b ∧ b ≤ M) :
    -N ≤ a + b ∧ a + b ≤ N := by
  omega

/-- Subtracting two bounded raw values cannot overflow. -/
theorem sub_no_overflow (M N a b : Int) (hN : 2 * M ≤ N)
    (ha : -M ≤ a ∧ a ≤ M) (hb : -M ≤ b ∧ b ≤ M) :
    -N ≤ a - b ∧ a - b ≤ N := by
  omega

/-- Physical-range additions stay inside `i128`. -/
theorem phys_add_fits_i128 (a b : Int)
    (ha : -physBound ≤ a ∧ a ≤ physBound)
    (hb : -physBound ≤ b ∧ b ≤ physBound) :
    i128Min ≤ a + b ∧ a + b ≤ i128Max := by
  have hN : 2 * physBound ≤ i128Max := by decide
  have h := add_no_overflow physBound i128Max a b hN ha hb
  have hmin : i128Min ≤ -i128Max := by decide
  omega

/-- Physical-range subtractions stay inside `i128`. -/
theorem phys_sub_fits_i128 (a b : Int)
    (ha : -physBound ≤ a ∧ a ≤ physBound)
    (hb : -physBound ≤ b ∧ b ≤ physBound) :
    i128Min ≤ a - b ∧ a - b ≤ i128Max := by
  have hN : 2 * physBound ≤ i128Max := by decide
  have h := sub_no_overflow physBound i128Max a b hN ha hb
  have hmin : i128Min ≤ -i128Max := by decide
  omega

/-- Euclidean division and remainder reconstruct the dividend exactly. This is
the arithmetic invariant the fixed-point `Div` implementation relies on. -/
theorem div_rem_identity (a b : Int) : a / b * b + a % b = a :=
  Int.ediv_mul_add_emod a b

/-- The remainder produced by division lies in `[0, b)` for positive divisors. -/
theorem rem_bounds (a b : Int) (hb : 0 < b) : 0 ≤ a % b ∧ a % b < b :=
  ⟨Int.emod_nonneg a (by omega), Int.emod_lt_of_pos a hb⟩

/-- Negation is an involution on raw values. -/
theorem neg_neg (a : Int) : -(-a) = a := by
  omega

end KelvinProofs.FixedPoint
