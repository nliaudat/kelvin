/-
# Gravitational force pair (integer model)

Models the pairwise acceleration terms computed by
`kelvin-core/src/integrator.rs::compute_accelerations`:

    a_i += (G / (|r|² + ε²)^{3/2}) * m_j * (r_j - r_i)
    a_j -= (G / (|r|² + ε²)^{3/2}) * m_i * (r_j - r_i)

The scalar factor `G / (|r|² + ε²)^{3/2}` is kept opaque; the theorem below is
the momentum-conservation identity that follows from the pairing alone.
-/
import Init

namespace KelvinProofs.Gravity

/-- Acceleration contribution along one axis: `factor * mass * displacement`. -/
def accel (factor mass disp : Int) : Int := factor * mass * disp

/-- The two contributions to a pair's momentum cancel exactly (Newton's third
law at the momentum level), for arbitrary masses. -/
theorem pair_momentum_cancel (factor mi mj disp : Int) :
    mi * accel factor mj disp + mj * (-(accel factor mi disp)) = 0 := by
  simp [accel, Int.mul_neg, Int.mul_comm, Int.mul_left_comm]
  omega

/-- A massless body exerts no acceleration. -/
theorem accel_zero_mass (factor disp : Int) : accel factor 0 disp = 0 := by
  simp [accel]

end KelvinProofs.Gravity
