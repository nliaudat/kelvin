# OrbitalState Specification

## Overview

OrbitalState is the high-level state machine that wraps the n-body
simulation, providing a safe API for stepping the simulation forward,
extracting entropy, and estimating chaos.

## State

```
struct OrbitalState {
    bodies: Vec<OrbitalBody>,  // Current body states
    step: u64,                 // Current simulation step
    config: OrbitalConfig,     // Simulation parameters
}
```

## Construction

### Chaotic Default

```
fn chaotic_default() -> Self
```

Creates a 5-body system with randomized initial conditions designed to
produce chaotic dynamics:

- 3 massive bodies in a figure-8 configuration
- 2 low-mass test particles in unstable orbits
- Standard gravitational constant (4π²)
- Default softening (0.001)
- Default time step (0.01)

## Stepping

### Verlet (Symplectic)

```
fn verlet_step(&mut self) -> Result<(), OrbitalError>
fn verlet_steps(&mut self, n: u64) -> Result<(), OrbitalError>
```

Advances the simulation using the Verlet integrator. The step counter
is incremented by 1 (or n) after each call.

### Euler (Non-Symplectic)

```
fn euler_step(&mut self) -> Result<(), OrbitalError>
fn euler_steps(&mut self, n: u64) -> Result<(), OrbitalError>
```

Advances the simulation using the Euler integrator. The step counter
is incremented by 1 (or n) after each call.

### Stability Check

```
fn check_stability(&self) -> Result<(), OrbitalError>
```

Checks for ejection or collapse. Returns `OrbitalError::Ejected` or
`OrbitalError::Collapsed` if the system has become unstable.

## Entropy Extraction

```
fn extract_entropy(&self, output: &mut [u8])
```

Extracts entropy from the current orbital state into the output buffer.
The extraction uses SHAKE256 with domain separation.

## Chaos Estimation

```
fn estimate_lyapunov(&mut self, sample_steps: u64) -> f64
```

Estimates the maximal Lyapunov exponent by running two parallel
simulations with a small perturbation and measuring divergence.

## Zeroization

```
fn zeroize(&mut self)
```

Securely zeroizes all body state and resets the step counter.
Called on drop to prevent key recovery from memory.

## Invariants

1. **Step monotonicity**: The step counter never decreases.
2. **Determinism**: The same initial state and same number of steps
   always produces the same final state.
3. **Overflow safety**: All operations use checked arithmetic.
4. **Zeroize on drop**: All sensitive state is cleared on drop.

## Error Types

```
enum OrbitalError {
    Ejected { body_index: usize, step: u64 },
    Collapsed { body_i: usize, body_j: usize, step: u64 },
    Overflow,
}
```

## References

- Sprott, J. C. (2003). *Chaos and Time-Series Analysis*. Oxford University
  Press.
  — Guidelines for chaotic system initialization and Lyapunov estimation.
