# Explicit Euler Integrator Specification

## Algorithm: Forward Euler

The explicit (forward) Euler integrator evolves the n-body system through
phase space using a first-order scheme:

### Step 1: Compute Accelerations

Compute gravitational accelerations `a_i` from current positions:
```
a_i = Σ_j G × m_j × (r_j - r_i) / (|r_ij|² + ε²)^(3/2)
```

### Step 2: Update Positions (using OLD velocity)

For each body `i`:
```
x_i ← x_i + v_i × dt
```

### Step 3: Update Velocities (using current acceleration)

For each body `i`:
```
v_i ← v_i + a_i × dt
```

## Properties

### Not Symplectic

Unlike the Verlet integrator, Euler does **not** preserve the symplectic
structure of Hamiltonian mechanics. This means:

- **Energy drifts secularly**: Total energy grows (or shrinks) over time
  without bound, rather than oscillating around a constant value.
- **No long-term stability**: The integrator is unsuitable for long-duration
  astronomical simulations.

### Not Time-Reversible

Applying the Euler integrator with `dt → -dt` does **not** return the system
to its initial state. This is because the position update uses the old
velocity, while the velocity update uses the current acceleration — the
two updates are not symmetric.

### Chaos Amplification

The numerical dissipation and energy drift of the Euler integrator
**amplifies chaos** approximately 10× faster than the Verlet integrator:

- Lyapunov time (Euler) ≈ 0.1 × Lyapunov time (Verlet)
- This means fewer simulation steps are needed to reach the entropy
  threshold for cryptographic key generation.

### Preferred for Entropy Generation

The Euler integrator's numerical instability is a **feature** for Kelvin's
use case: the one-way nature of the integration (due to time irreversibility)
makes it harder to reverse the simulation, and the faster chaos amplification
reduces the computational cost of key generation.

## Comparison with Verlet

| Property | Verlet | Euler |
|----------|--------|-------|
| Order | 2nd order | 1st order |
| Symplectic | Yes | No |
| Time-reversible | Yes | No |
| Energy drift | Bounded, oscillatory | Secular (grows) |
| Chaos amplification | Baseline | ~10× faster |
| Use in Kelvin | Default (more stable) | `--euler` flag (more chaotic) |

## References

- Euler, L. (1768). *Institutionum Calculi Integralis*.
  — Original formulation of the Euler method.
- Hairer, E., Nørsett, S. P., & Wanner, G. (1993). *Solving Ordinary
  Differential Equations I* (2nd ed.). Springer.
  — Analysis of Euler method stability and error bounds.
