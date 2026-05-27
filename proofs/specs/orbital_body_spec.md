# OrbitalBody Specification

## Overview

An OrbitalBody represents a single massive body in the n-body simulation.
Each body has mass, position, and velocity, and can compute derived
physical quantities.

## Representation

```
struct OrbitalBody {
    mass: Fixed,     // Q32.64, in solar masses (M☉)
    position: Vec3,  // position vector in AU
    velocity: Vec3,  // velocity vector in AU/day
}
```

## Physical Quantities

### Kinetic Energy

```
KE = 0.5 × m × |v|²
```

Where:
- `m` = body mass
- `|v|²` = velocity length squared (dot product of velocity with itself)

### Momentum

```
p = m × v
```

Where:
- `m` = body mass
- `v` = velocity vector

Returns a Vec3 representing the linear momentum vector.

## Invariants

1. **Mass positive**: `mass > 0` for all valid bodies.
2. **Mass bounded**: `mass` is within the valid range for the simulation.
3. **Position bounded**: Position components are within the simulation
   domain (typically ±1000 AU).
4. **Velocity bounded**: Velocity components are within the simulation
   domain (typically ±100 AU/day).

## Physical Bounds

| Quantity | Minimum | Maximum | Unit |
|----------|---------|---------|------|
| Mass | 1e-10 | 100 | M☉ |
| Position | -1000 | 1000 | AU |
| Velocity | -100 | 100 | AU/day |
| Kinetic Energy | 0 | ~5e5 | (M☉·AU²/day²) |
| Momentum | -1e5 | 1e5 | (M☉·AU/day) |

## References

- Goldstein, H., Poole, C., & Safko, J. (2002). *Classical Mechanics*
  (3rd ed.). Addison-Wesley.
  — Standard reference for classical mechanics quantities.
