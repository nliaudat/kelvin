/-
# Velocity Verlet integrator (integer model)

Models the kick-drift-kick scheme of `kelvin-core/src/integrator.rs::verlet_step`:

1. kick:  `v ← v + a * dt/2`
2. drift: `x ← x + v * dt`   (using the velocity *after* the first kick)
3. kick:  `v ← v + a' * dt/2`

Accelerations are kept as opaque integer values, so the theorems below concern
the *scheduling* of the update (half-steps, ordering, reversibility) rather than
the gravitational force law, which is modelled in `KelvinProofs.Gravity`.
-/
import Init

namespace KelvinProofs.Integrator

/-- One Cartesian coordinate of a body, in raw Q32.64 units. -/
structure Body where
  pos : Int
  vel : Int
deriving Repr, DecidableEq

/-- Velocity kick over an interval `halfDt`. -/
def kick (v a halfDt : Int) : Int := v + a * halfDt

/-- Position drift over a full step. -/
def drift (pos v dt : Int) : Int := pos + v * dt

/-- One kick-drift-kick step, matching `verlet_step`. -/
def step (pos vel a a' halfDt dt : Int) : Int × Int :=
  let v1 := kick vel a halfDt
  let x1 := drift pos v1 dt
  let v2 := kick v1 a' halfDt
  (x1, v2)

/-- The drift uses the velocity *after* the first (half) kick. -/
theorem step_fst (pos vel a a' halfDt dt : Int) :
    (step pos vel a a' halfDt dt).1 = pos + (vel + a * halfDt) * dt := by
  simp [step, kick, drift]

/-- Both kicks use the half timestep. -/
theorem step_snd (pos vel a a' halfDt dt : Int) :
    (step pos vel a a' halfDt dt).2 = (vel + a * halfDt) + a' * halfDt := by
  simp [step, kick, drift]

/-- Time-reversibility of the scheme: undoing the second kick, the drift, and
the first kick (with negated half-steps) recovers the original state exactly. -/
theorem step_reversible (pos vel a a' halfDt dt : Int) :
    (let p := step pos vel a a' halfDt dt
     let v1 := p.2 - a' * halfDt
     let x0 := p.1 - v1 * dt
     let v0 := v1 - a * halfDt
     x0 = pos ∧ v0 = vel) := by
  simp [step, kick, drift]

/-- Iterate a state map `n` times. -/
def iterate {α : Type} (f : α → α) : Nat → α → α
  | 0, x => x
  | n + 1, x => iterate f n (f x)

theorem iterate_zero {α : Type} (f : α → α) (x : α) : iterate f 0 x = x := rfl

theorem iterate_succ {α : Type} (f : α → α) (n : Nat) (x : α) :
    iterate f (n + 1) x = iterate f n (f x) := rfl

/-- Running zero simulation steps is the identity, matching `simulate(.., 0, ..)`. -/
theorem simulate_zero {α : Type} (f : α → α) (x : α) : iterate f 0 x = x :=
  iterate_zero f x

end KelvinProofs.Integrator
