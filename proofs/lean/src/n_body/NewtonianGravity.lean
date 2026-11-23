/-
# N-body gravity (legacy path)

Corrected 2026-11-23. The previous revision imported Mathlib and stated Newton's
third law over `Option Vec3` with an unclosed doc block and an ill-typed proof.
This file models the pairwise acceleration terms as integers and proves the
identities that actually hold at that level.

## What is proved

* the two momentum contributions of a pair cancel exactly, for arbitrary masses;
* a massless body exerts no acceleration;
* the dot product is symmetric, and scaling a vector scales the dot product.

## What is not proved

The inverse-square magnitude law, the force direction in a metric space, and
energy conservation require real analysis and are not formalised here.
-/
import KelvinProofs.Gravity

namespace KelvinLegacy.Gravity

open KelvinProofs.Gravity

/-- Integer 3-vector. -/
structure Vec3I where
  x : Int
  y : Int
  z : Int
deriving Repr, DecidableEq

/-- Dot product over the integers. -/
def dot (a b : Vec3I) : Int := a.x * b.x + a.y * b.y + a.z * b.z

/-- Scalar multiplication. -/
def scale (r : Int) (v : Vec3I) : Vec3I := ⟨r * v.x, r * v.y, r * v.z⟩

/-- The dot product is symmetric. -/
theorem dot_comm (a b : Vec3I) : dot a b = dot b a := by
  simp [dot, Int.mul_comm]

/-- Scaling the first argument scales the dot product. -/
theorem dot_scale_left (r : Int) (a b : Vec3I) :
    dot (scale r a) b = r * dot a b := by
  simp [dot, scale, Int.mul_add,
    Int.mul_comm, Int.mul_left_comm]

/-- Newton's third law at the momentum level: the two contributions to a pair's
momentum cancel exactly, for arbitrary masses. -/
theorem action_reaction_law (factor mi mj disp : Int) :
    mi * accel factor mj disp + mj * (-(accel factor mi disp)) = 0 :=
  pair_momentum_cancel factor mi mj disp

/-- A massless body exerts no acceleration. -/
theorem force_zero_of_zero_mass (factor disp : Int) :
    accel factor 0 disp = 0 :=
  accel_zero_mass factor disp

end KelvinLegacy.Gravity
