# OrbitalConfig Specification

## Overview

OrbitalConfig defines the parameters for an n-body simulation used for
cryptographic key generation. It specifies the number of bodies, their
initial conditions, integration parameters, and extraction settings.

## Parameters

```
struct OrbitalConfig {
    num_bodies: u32,        // Number of bodies (3–12)
    seed_bodies: Vec<OrbitalBody>,  // Initial body configuration
    steps: u64,             // Simulation steps
    dt: Fixed,              // Time step (AU/day)
    softening: Fixed,       // Softening parameter (AU)
    g: Fixed,               // Gravitational constant (AU³/M☉·day²)
    method: IntegrationMethod,  // Verlet or Euler
    domain: String,         // Domain separation string
}
```

## Integration Method

```
enum IntegrationMethod {
    Verlet,  // Symplectic, 2nd order (default)
    Euler,   // Non-symplectic, 1st order
}
```

## Default Values

| Parameter | Default | Rationale |
|-----------|---------|-----------|
| num_bodies | 5 | Minimum for chaotic 3-body + 2 test particles |
| steps | 1000 | Sufficient for entropy threshold |
| dt | 0.01 | Balances accuracy vs. chaos amplification |
| softening | 0.001 | Prevents singularities at close approaches |
| g | 4π² | Standard gravitational constant in AU units |
| method | Verlet | Symplectic, energy-conserving |

## Validation Rules

1. **num_bodies**: Must be between `MIN_BODIES` (3) and `MAX_BODIES` (12).
2. **seed_bodies**: Must match `num_bodies` in length.
3. **steps**: Must be > 0.
4. **dt**: Must be > 0.
5. **softening**: Must be > 0.
6. **g**: Must be > 0.
7. **domain**: Must be non-empty.

## Binary Serialization

The config can be serialized to/from a binary format for storage:

```
[u32: num_bodies]
[u64: steps]
[i128: dt_raw]
[i128: softening_raw]
[i128: g_raw]
[u8: method (0=Verlet, 1=Euler)]
[u32: domain_len]
[bytes: domain]
[repeated num_bodies × OrbitalBody]
```

Each OrbitalBody is serialized as:
```
[i128: mass_raw]
[i128: pos_x_raw]
[i128: pos_y_raw]
[i128: pos_z_raw]
[i128: vel_x_raw]
[i128: vel_y_raw]
[i128: vel_z_raw]
```

## Invariants

1. **Determinism**: The same config always produces the same simulation.
2. **Round-trip**: `from_binary(to_binary(config)) == config`.
3. **Validation**: Invalid configs are rejected at construction time.

## References

- Sprott, J. C. (2003). *Chaos and Time-Series Analysis*. Oxford University
  Press.
  — Guidelines for chaotic system configuration.
