/-
# Lyapunov exponents (legacy path)

Corrected 2026-11-23. The previous revision stated `kaplan_yorke_bound` as
`∃ D_KY, D_KY ≤ 4 ∧ D_KY > 3` and discharged it with `use 3.5`, which proves
only that some number lies in `(3, 4]`. It also defined `simulate` to return its
input and then "proved" sensitive dependence from it.

## What this file actually establishes

One arithmetic fact about the Kaplan–Yorke formula: if the leading exponents sum
to a non-negative value `S` and the next exponent has magnitude `g > 0`, then
`D_KY = j + S/g` is at least `j`. Nothing else.

## What is *not* established

`λ > 0`, `D_KY ∈ [3,4]`, sensitive dependence and chaos certification. These are
empirical claims (see `documentation/formal_verification/C2/`) and have no
machine-checked artifact. The exponent is an input here, not a theorem.
-/
import Init

namespace KelvinLegacy.Lyapunov

/-- Kaplan–Yorke arithmetic: with `j` positive exponents, non-negative partial
sum `S`, and `g > 0`, the dimension `j + S/g` is at least `j`. -/
theorem kaplan_yorke_at_least_j (j S g : Int) (hg : 0 < g) (hS : 0 ≤ S) :
    j ≤ j + S / g := by
  have hdiv : 0 ≤ S / g := Int.ediv_nonneg hS (by omega)
  omega

/-- The Kaplan–Yorke dimension is non-negative whenever its inputs are. -/
theorem kaplan_yorke_nonnegative (j S g : Int) (hj : 0 ≤ j) (hg : 0 < g) (hS : 0 ≤ S) :
    0 ≤ j + S / g := by
  have hdiv : 0 ≤ S / g := Int.ediv_nonneg hS (by omega)
  omega

end KelvinLegacy.Lyapunov
