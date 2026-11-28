/-
# Kelvin formal verification — entry point

Corrected 2026-11-23. The previous revision of this file (a) imported Mathlib,
(b) used a bare slash-dash opener where a comment terminator was required, so
the file parsed as a single comment, and (c) documented a build command
(`lean --init mathlib`) that does not exist. It now builds with `lake build` and
no external dependencies.

The library is split in two layers:

* `KelvinProofs.*` — the primary, machine-checked modules.
* `fixed_point.*`, `n_body.*`, `dynamics.*`, `entropy_extraction.*`,
  `key_schedule.*` — the legacy paths kept for documentation links. They import
  the primary modules and restate the corresponding facts under their historical
  names. Nothing in them depends on Mathlib.

## What is proved

Integer arithmetic (overflow, division/remainder), the Verlet update schedule and
its reversibility, pairwise momentum cancellation, domain-framing injectivity,
conditional key/XOF uniqueness, and the Floyd loop invariants.

## What is not proved

Every cryptographic security claim (SHAKE256 preimage resistance, HMAC-SHA512 PRF
security, keystream indistinguishability), all entropy accounting, the Lyapunov
spectrum and the Kaplan–Yorke bound, and the quantum query lower bound. See
`proofs/lean/README.md` and `documentation/formal_verification/PROOF_INDEX.md`.
-/
import KelvinProofs
import Sanity
import fixed_point.FixedPoint
import n_body.NewtonianGravity
import dynamics.VerletIntegrator
import dynamics.LyapunovExponents
import entropy_extraction.Shake256Extraction
import key_schedule.HKDFDerivation

namespace KelvinFormalVerif

/-- The formalisation is well-formed: all imported modules elaborate. -/
theorem kelvin_system_well_defined : True := trivial

/-- The fixed-point layer is self-consistent: `0 < 2^64`. -/
theorem scale_positive : 0 < KelvinProofs.FixedPoint.one :=
  KelvinProofs.FixedPoint.one_pos

end KelvinFormalVerif
