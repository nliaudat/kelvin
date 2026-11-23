/-
# Floyd cycle detection (loop-invariant model)

Models the tortoise-and-hare loop used by `kelvin-core/src/period_detection.rs`.
The state itself is abstracted to its step index, so the theorems below are the
loop invariants that guarantee the loop makes progress and terminates within
the step budget.

## What is proved

* after `k` iterations the tortoise has advanced `k` steps;
* after `k` iterations the hare has advanced `2k` steps;
* once the hare's index exceeds `max`, the budget is exhausted.

## What is not proved

That a physical repeat of the simulation state is ever found (or not found) —
that depends on the dynamics and is an empirical question.
-/
import Init

namespace KelvinProofs.PeriodDetection

/-- One loop iteration: tortoise advances one state, hare advances two. -/
def advance (tortoise hare : Nat) : Nat × Nat := (tortoise + 1, hare + 2)

/-- Run `k` loop iterations from `(tortoise, hare)`. -/
def run : Nat → Nat × Nat → Nat × Nat
  | 0, s => s
  | k + 1, s => run k (advance s.1 s.2)

/-- After `k` iterations the tortoise has advanced exactly `k` states. -/
theorem run_tortoise (k t h : Nat) : (run k (t, h)).1 = t + k := by
  induction k generalizing t h with
  | zero => simp [run]
  | succ k ih =>
      simp [run, advance, ih]
      omega

/-- After `k` iterations the hare has advanced exactly `2k` states. -/
theorem run_hare (k t h : Nat) : (run k (t, h)).2 = h + 2 * k := by
  induction k generalizing t h with
  | zero => simp [run]
  | succ k ih =>
      simp [run, advance, ih]
      omega

/-- If the hare's index exceeds the step budget after `k` iterations, the loop
must stop. -/
theorem hare_exceeds_budget (k t h max : Nat) (hbudget : max < h + 2 * k) :
    max < (run k (t, h)).2 := by
  rw [run_hare]
  exact hbudget

end KelvinProofs.PeriodDetection
