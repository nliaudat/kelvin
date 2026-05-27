# Vec3 Vector Operations Specification

## Overview

Vec3 is a 3-dimensional vector type using Q32.64 fixed-point arithmetic for
all components. It represents positions, velocities, accelerations, forces,
and momenta in the n-body simulation.

## Representation

```
struct Vec3 {
    x: Fixed,  // Q32.64 fixed-point
    y: Fixed,  // Q32.64 fixed-point
    z: Fixed,  // Q32.64 fixed-point
}
```

## Operations

### Addition

```
a + b = Vec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
```

Properties:
- Commutative: `a + b = b + a`
- Associative: `(a + b) + c = a + (b + c)`
- Identity: `a + zero = a`

### Subtraction

```
a - b = Vec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
```

### Negation

```
-a = Vec3 { x: -a.x, y: -a.y, z: -a.z }
```

### Scalar Multiplication

```
a * s = Vec3 { x: a.x * s, y: a.y * s, z: a.z * s }
```

### Scalar Division

```
a / s = Vec3 { x: a.x / s, y: a.y / s, z: a.z / s }
```

### Dot Product

```
a · b = a.x * b.x + a.y * b.y + a.z * b.z
```

Properties:
- Commutative: `a · b = b · a`
- Bilinear: `(a + b) · c = a · c + b · c`
- Zero: `a · zero = 0`

### Cross Product

```
a × b = Vec3 {
    x: a.y * b.z - a.z * b.y,
    y: a.z * b.x - a.x * b.z,
    z: a.x * b.y - a.y * b.x,
}
```

Properties:
- Anti-commutative: `a × b = -(b × a)`
- Orthogonal: `(a × b) · a = 0` and `(a × b) · b = 0`
- Zero for parallel: `a × a = zero`

### Length

```
|a| = sqrt(a.x² + a.y² + a.z²)
```

### Length Squared

```
|a|² = a.x² + a.y² + a.z²
```

### Scale (alias for scalar multiplication)

```
a.scale(s) = a * s
```

## Invariants

1. **Component-wise**: All Vec3 operations are component-wise applications
   of the corresponding Fixed operations.
2. **No overflow**: All operations use checked arithmetic internally.
3. **No panic**: Division by zero is handled by Fixed's checked_div.

## Physical Interpretation

| Quantity | Vec3 Represents |
|----------|----------------|
| Position | `r = (x, y, z)` in AU |
| Velocity | `v = (vx, vy, vz)` in AU/day |
| Acceleration | `a = (ax, ay, az)` in AU/day² |
| Momentum | `p = m × v` |
| Force | `F = m × a` |

## References

- Arfken, G. B., & Weber, H. J. (2005). *Mathematical Methods for
  Physicists* (6th ed.). Elsevier.
  — Standard reference for vector algebra.
