# Lyapunov Exponent Estimation Specification

## Overview

The Lyapunov exponent quantifies the rate of divergence of nearby trajectories
in the n-body phase space. It is the primary measure of chaos in the Kelvin
cryptosystem and determines how many simulation steps are needed to reach
the entropy threshold for cryptographic key generation.

## Definition

For a dynamical system with trajectory `x(t)`, the maximal Lyapunov exponent
`λ` is defined as:

```
λ = lim_{t→∞} lim_{δx(0)→0} (1/t) × ln(|δx(t)| / |δx(0)|)
```

where `δx(t)` is the deviation between two initially close trajectories.

## Numerical Estimation

Kelvin uses the **orbit separation method**:

### Algorithm

```
Input:  OrbitalState, perturbation_scale = 1e-10, sample_steps
Output: LyapunovResult { exponent, confidence }

1. Clone the orbital state → state_perturbed
2. Apply a small perturbation to all position components:
   for each body i:
       state_perturbed.bodies[i].position.x += perturbation_scale
       state_perturbed.bodies[i].position.y += perturbation_scale
       state_perturbed.bodies[i].position.z += perturbation_scale

3. Initialize divergence accumulator: sum_log_divergence = 0
4. For step = 0 to sample_steps:
   a. Step both states forward (verlet_step or euler_step)
   b. Compute phase space distance:
      δ = Σ_i |r_i - r'_i|² + |v_i - v'_i|²
   c. If δ > 0:
      sum_log_divergence += 0.5 × ln(δ)
   d. Renormalize: rescale perturbed state to keep δ small

5. λ = sum_log_divergence / (sample_steps × dt)
```

### Confidence Estimation

The confidence in the estimate increases with the number of sample steps:

| Sample Steps | Confidence |
|-------------|------------|
| < 100 | Low (transient) |
| 100–1000 | Medium |
| > 1000 | High (converged) |

## Properties

### Positive Exponent → Chaos

- `λ > 0`: System is chaotic (trajectories diverge exponentially)
- `λ ≈ 0`: System is periodic or quasi-periodic
- `λ < 0`: System is stable (trajectories converge)

### Kelvin's Chaotic Default

The default 5-body configuration produces `λ ≈ 0.1–0.5` per simulation step,
meaning trajectories diverge by a factor of `e^(0.1) ≈ 1.1` per step.

### Euler vs Verlet

The Euler integrator produces Lyapunov exponents approximately 10× larger
than Verlet for the same system, due to its numerical dissipation.

## Invariants

1. **Minimum bodies**: Requires at least 3 bodies (2-body systems are
   integrable and non-chaotic).
2. **Positive steps**: Requires at least 1 sample step.
3. **Scale invariance**: The exponent should be independent of the
   perturbation scale (for sufficiently small scales).

## References

- Lyapunov, A. M. (1892). *The General Problem of the Stability of Motion*.
- Benettin, G., et al. (1980). "Lyapunov characteristic exponents for
  smooth dynamical systems." *Meccanica*, 15(1), 9–20.
- Sprott, J. C. (2003). *Chaos and Time-Series Analysis*. Oxford University
  Press.
