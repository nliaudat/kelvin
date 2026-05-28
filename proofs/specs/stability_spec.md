# Stability Monitoring Specification

## Overview

The stability monitoring system detects when an n-body simulation has
become physically unrealistic (ejection or collapse), allowing the
simulation to be restarted with a fresh configuration.

## Ejection Detection

A body is considered ejected when its total energy (kinetic + potential)
exceeds zero, meaning it is no longer gravitationally bound to the system.

### Algorithm

```
fn is_body_ejected(body: &OrbitalBody, bodies: &[OrbitalBody], g: Fixed) -> bool {
    // Compute gravitational potential at body's position
    let potential = gravitational_potential(body, bodies, g);

    // Total specific energy = KE/m + PE/m
    let specific_ke = 0.5 * |body.velocity|²;
    let specific_pe = potential;

    specific_ke + specific_pe > 0
}
```

### Properties

- A body with `KE + PE > 0` has escape velocity and will leave the system.
- Ejection is irreversible in the absence of external forces.

## Collapse Detection

A collapse is detected when any two bodies are closer than a threshold
distance, indicating a potential singularity or numerical instability.

### Algorithm

```
fn detect_collapse(bodies: &[OrbitalBody], threshold: Fixed) -> bool {
    for each pair (i, j) with i < j:
        let distance = |bodies[i].position - bodies[j].position|
        if distance < threshold:
            return true
    return false
}
```

Default threshold: `1e-6` AU.

## Gravitational Potential

The gravitational potential at a body's position due to all other bodies:

```
fn gravitational_potential(body: &OrbitalBody, bodies: &[OrbitalBody], g: Fixed) -> Fixed {
    let mut potential = Fixed::from_int(0);
    for each other body j where j != body:
        let r = |body.position - bodies[j].position|
        potential += -g * bodies[j].mass / sqrt(r² + ε²)
    potential
}
```

Where `ε` is the softening parameter.

## Simulation with Monitoring

### Verlet

```
fn simulate_with_monitoring(
    bodies: &mut [OrbitalBody],
    steps: u64,
    dt: Fixed,
    softening: Fixed,
    g: Fixed,
    stability_interval: u64,
    collapse_threshold: Fixed,
) -> Result<(), StabilityError>
```

### Euler

```
fn simulate_with_monitoring_euler(
    bodies: &mut [OrbitalBody],
    steps: u64,
    dt: Fixed,
    softening: Fixed,
    g: Fixed,
    stability_interval: u64,
    collapse_threshold: Fixed,
) -> Result<(), StabilityError>
```

### Stability Check Interval

The stability check is performed every `stability_interval` steps.
If `stability_interval == 0`, no stability checks are performed.

### Error Types

```
enum StabilityError {
    Ejected { body_index: usize, step: u64 },
    Collapsed { body_i: usize, body_j: usize, step: u64 },
}
```

## Invariants

1. **Ejection**: A body with `KE + PE > 0` is always detected as ejected.
2. **Collapse**: Two bodies closer than threshold are always detected.
3. **No false positive**: A stable system never triggers an error.
4. **Zero interval**: `stability_interval == 0` skips all checks.

## References

- Binney, J., & Tremaine, S. (2008). *Galactic Dynamics* (2nd ed.).
  Princeton University Press.
  — Standard reference for gravitational potential and escape velocity.
