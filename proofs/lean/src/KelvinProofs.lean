/-
# Kelvin formal verification — Lean 4 entry point

This library contains the machine-checked part of Kelvin's formal verification
effort. It is deliberately **self-contained** (no Mathlib dependency) and builds
with the toolchain pinned in `lean-toolchain`.

Build:

    lake build

See `proofs/lean/README.md` for what is and is not proved.
-/
import KelvinProofs.FixedPoint
import KelvinProofs.Integrator
import KelvinProofs.Gravity
import KelvinProofs.KeySchedule
import KelvinProofs.Entropy
import KelvinProofs.PeriodDetection
