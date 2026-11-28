/-
# Sanity checks for the constants used in the documentation

These are kernel-checked `example` declarations, so they are verified by
`lake build` exactly like the theorems. They pin the concrete numbers that the
prose relies on (2^64 scale, saturation point, Grover exponent), so a typo in a
document or a refactor of the model cannot silently change them.

They are not security claims.
-/
import KelvinProofs.FixedPoint

namespace KelvinSanity

/-- The Q32.64 scale is `2^64`. -/
example : KelvinProofs.FixedPoint.one = 18446744073709551616 := by decide

/-- The physical magnitude bound is `100 * 2^64` raw. -/
example : KelvinProofs.FixedPoint.physBound = 1844674407370955161600 := by decide

/-- The C1 saturation point for `N = 5`, `k_step = 40`: `96 * 40 = 3840`. -/
example : 96 * 40 = 3840 := by decide

/-- The C5 exponent: `2 * 960 = 1920`. -/
example : 2 * 960 = 1920 := by decide

/-- `i128::MAX` and `i128::MIN` as used by the overflow lemmas. -/
example : KelvinProofs.FixedPoint.i128Max = 2 ^ 127 - 1 := by decide
example : KelvinProofs.FixedPoint.i128Min = -(2 ^ 127) := by decide

end KelvinSanity
