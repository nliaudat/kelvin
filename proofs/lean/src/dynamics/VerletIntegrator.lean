/-
# Velocity Verlet integrator (legacy path)

Corrected 2026-11-23. The previous revision imported Mathlib, contained unclosed
doc blocks and malformed terms, and asserted symplectic/energy theorems it could
not prove. This file imports `KelvinProofs.Integrator` and restates the schedule
facts under the historical names.

## What is proved

* the drift uses the velocity after the first half-kick;
* both kicks use the half timestep;
* the kick-drift-kick map is exactly reversible;
* running zero steps is the identity.

## What is not proved

Symplecticity in the Hamiltonian sense and `O(dt²)` energy conservation are not
formalised here; they require a real-valued phase-space model.
-/
import KelvinProofs.Integrator

namespace KelvinLegacy.Verlet

open KelvinProofs.Integrator

/-- The drift uses the velocity *after* the first kick. -/
theorem verlet_drift_uses_post_kick_velocity (pos vel a a' halfDt dt : Int) :
    (step pos vel a a' halfDt dt).1 = pos + (vel + a * halfDt) * dt :=
  step_fst pos vel a a' halfDt dt

/-- Both kicks use the half timestep. -/
theorem verlet_uses_half_steps (pos vel a a' halfDt dt : Int) :
    (step pos vel a a' halfDt dt).2 = (vel + a * halfDt) + a' * halfDt :=
  step_snd pos vel a a' halfDt dt

/-- Time-reversibility: undoing the second kick, the drift and the first kick
recovers the initial state exactly. -/
theorem verlet_time_reversible (pos vel a a' halfDt dt : Int) :
    (let p := step pos vel a a' halfDt dt
     let v1 := p.2 - a' * halfDt
     let x0 := p.1 - v1 * dt
     let v0 := v1 - a * halfDt
     x0 = pos ∧ v0 = vel) :=
  step_reversible pos vel a a' halfDt dt

/-- Running zero simulation steps is the identity. -/
theorem simulate_zero_steps {α : Type} (f : α → α) (x : α) :
    iterate f 0 x = x :=
  simulate_zero f x

/-- `n + 1` steps are one step followed by `n` more. -/
theorem simulate_succ {α : Type} (f : α → α) (n : Nat) (x : α) :
    iterate f (n + 1) x = iterate f n (f x) :=
  iterate_succ f n x

end KelvinLegacy.Verlet
