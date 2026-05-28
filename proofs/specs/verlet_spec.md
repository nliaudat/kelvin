# Symplectic Verlet Integrator Specification

## Algorithm: Kick-Drift-Kick (Position Verlet)

The Verlet integrator evolves the n-body system through phase space using
a symplectic (energy-conserving) scheme:

### Step 1: Kick (Half Step)

For each body `i`:
```
v_i ← v_i + a_i × dt/2
```

where `a_i` is the gravitational acceleration computed from current positions.

### Step 2: Drift (Full Step)

For each body `i`:
```
x_i ← x_i + v_i × dt
```

### Step 3: Recompute Accelerations

Compute new accelerations `a'_i` from updated positions.

### Step 4: Kick (Half Step)

For each body `i`:
```
v_i ← v_i + a'_i × dt/2
```

## Gravitational Acceleration

For each pair `(i, j)`:
```
a_i += G × m_j × (r_j - r_i) / (|r_ij|² + ε²)^(3/2)
a_j -= G × m_i × (r_j - r_i) / (|r_ij|² + ε²)^(3/2)
```

where:
- `G` = gravitational constant (≈ 39.478 AU³/(M☉·yr²))
- `m_i, m_j` = masses in solar masses
- `r_ij = r_j - r_i` = separation vector
- `ε` = softening factor (≈ 9.5 × 10⁻⁷ AU)
- `|r_ij|² + ε²` = softened squared distance

## Invariants

### Linear Momentum Conservation

The Verlet integrator exactly conserves total linear momentum:
```
Σ m_i × v_i(t) = constant
```

This is because all forces are internal (action-reaction pairs cancel).

### Energy Stability

The Verlet integrator approximately conserves total energy over short timescales:
```
E(t) = KE(t) + PE(t) ≈ constant
```

Energy drift is bounded and does not grow secularly (symplectic property).

### Time Reversibility

The Verlet integrator is exactly time-reversible: applying the integrator
with `dt → -dt` returns the system to its initial state (within numerical
precision).

## Explicit Euler Integrator (Alternative)

For comparison, the explicit Euler integrator:

### Step 1: Compute accelerations `a_i` from current positions

### Step 2: Update positions using OLD velocity
```
x_i ← x_i + v_i × dt
```

### Step 3: Update velocities using current acceleration
```
v_i ← v_i + a_i × dt
```

### Properties

- **Not symplectic**: Energy drifts secularly (grows over time)
- **Not time-reversible**: Numerical dissipation creates one-way function
- **Chaos amplification**: Lyapunov time ~10× shorter than Verlet
- **Preferred for entropy generation**: Numerical instability is a feature

## References

- Verlet, L. (1967). "Computer 'Experiments' on Classical Fluids."
  *Physical Review*, 159(1), 98–103.
- Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical
  Integration* (2nd ed.). Springer.
- Wisdom, J., & Holman, M. (1991). "Symplectic Maps for the N-Body
  Problem." *The Astronomical Journal*, 102(4), 1528–1538.
