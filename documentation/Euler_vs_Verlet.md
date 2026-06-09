# Euler vs Verlet: Why Numerical Instability Is a Feature for Cryptographic Entropy

> **CLI Usage:** The `--euler` flag is available on both `encrypt` and `decrypt` subcommands for all modes (secure, chaos, photon, quantum). Use `--euler` to select Euler integration; omit it for the default Verlet. The integration method must match between encryption and decryption.

## The Critical Insight

For **physical simulation**, Verlet integration is superior — it conserves energy, preserves phase-space volume, and produces physically realistic trajectories. But for **cryptographic entropy generation**, Euler's numerical instability is a **feature, not a bug**.

| Property | Euler Method | Velocity Verlet |
|---|---|---|
| Order of accuracy | 1st order (O(Δt)) | 2nd order (O(Δt²)) |
| Energy drift | ❌ Severe (unstable) | ✅ Minimal (stable orbits) |
| Chaos amplification | ✅ Excessive (numerical noise dominates) | ⚠️ Moderate (physical chaos only) |
| Performance | ✅ ~20% faster | ✅ Still fast |
| Determinism | ✅ Deterministic | ✅ Deterministic |
| Physical realism | ❌ Poor (unphysical) | ✅ Good |
| Reversibility | ❌ Impossible (numerical loss) | ⚠️ Theoretically reversible |

## Why Euler Is Better for Cryptographic Entropy

### 1. Numerical Instability = More Entropy Per Step

Euler's explicit integration introduces systematic energy drift. Each step accumulates error, causing the trajectory to diverge from the "true" physical path. For entropy generation, this is ideal — the trajectory becomes unpredictable much faster than with Verlet.

```rust
// Euler: simple, numerically unstable (explicit: position before velocity)
let v_old = self.velocities[i][j];
self.positions[i][j] += v_old * DT;
self.velocities[i][j] += accel[i][j] * DT;

// Verlet: symplectic, energy-conserving
self.positions[i][j] += self.velocities[i][j] * DT + 0.5 * accel_current[i][j] * DT * DT;
// ... compute new accelerations ...
self.velocities[i][j] += 0.5 * (accel_current[i][j] + accel_new[i][j]) * DT;
```

### 2. Chaos Amplification = Shorter Lyapunov Time

The Lyapunov exponent measures how fast nearby trajectories diverge. Euler's numerical noise acts as a constant perturbation, effectively reducing the Lyapunov time by ~10x compared to Verlet.

| Metric¹ | Euler (dt=0.001) | Verlet (dt=0.01) |
|---|---|---|
| Lyapunov time | ~100 steps | ~1000 steps |
| Trajectory decorrelation per step² | ~0.1 decorrelation units | ~0.01 decorrelation units |
| Steps for trajectory decorrelation² | ~2,560 steps (estimate) | ~25,600 steps (estimate) |
| Energy drift | 1% per 1000 steps | 0.0001% per 1000 steps |

> ¹ These metrics measure **trajectory divergence** (how fast nearby trajectories separate from each other), not cryptographic entropy. A deterministic computation has zero bits of min-entropy regardless of how chaotic the dynamics appears.  

### 3. Harder to Reverse = Numerical Irreversibility

Verlet integration is symplectic — it preserves phase-space volume and is theoretically reversible. Given the full state, you can run Verlet backwards to recover previous states. Euler's numerical dissipation makes this practically impossible for a backward integrator: information is lost at each step through energy drift.

> ⚠️ **Important caveat**: Numerical irreversibility is **not** the same as a cryptographic one-way function. An attacker does not need to run the integrator backwards. The attacker runs the integrator **forwards** over candidate initial conditions — exactly the same operation as the legitimate party. Numerical dissipation makes the integrator non-invertible as a mathematical map, but this does **not** make the forward search problem any harder. The "one-way function" property Kelvin needs is that recovering initial conditions from observed output is computationally hard — this is a conjecture about forward search, not a consequence of backward irreversibility.

### 4. Faster Divergence = More Trajectory Divergence Per CPU Cycle

Euler uses a smaller timestep (dt=0.001 vs Verlet's dt=0.01) for stability, but the per-step cost is ~20% lower (one acceleration computation vs two). Combined with the 10x faster chaos amplification, Euler produces usable trajectory divergence ~13x faster than Verlet.

> ⚠️ **Important caveat**: The metrics below measure **trajectory divergence** (how fast nearby trajectories separate), not cryptographic entropy. A deterministic computation, no matter how divergent, has zero bits of min-entropy — the output is fully determined by the input. Trajectory divergence complicates search but does not add entropy to the system. The "Steps for 256-bit security" figures are estimates of how many simulation steps are needed before small initial differences produce completely different states — they are NOT bits of cryptographic min-entropy.

| Metric¹ | Euler (dt=0.001) | Verlet (dt=0.01) | Winner |
|---|---|---|---|
| Steps for trajectory decorrelation | ~2,560 (estimate) | ~25,600 (estimate) | Euler (10x faster) |
| Time per step | ~80ns | ~100ns | Euler (20% faster) |
| Time to trajectory decorrelation | ~0.2ms | ~2.6ms | Euler (13x faster) |
| Numerical reversibility | ❌ Impossible | ⚠️ Possible | Euler |
| Determinism | ✅ Yes | ✅ Yes | Draw |

> ¹ Note: Euler uses a 10× smaller timestep than Verlet for numerical stability. When comparing at equal physical simulation time, Euler requires 10× more steps than Verlet. The per-step comparisons above reflect cost per step, not per unit of simulated physical time. "Trajectory decorrelation" means initial-condition divergence reaches a threshold where the state is decorrelated from initial values — this is a physical chaos property, not cryptographic entropy.

## Implementation

### Euler Step (Default for Kelvin-Quantum)

```rust
/// Euler integration optimized for maximum chaos amplification.
///
/// Uses dt=0.001 (10x smaller than Verlet) for stability,
/// but energy drift still creates ~10x more trajectory divergence.
///
/// NOTE: This is pseudocode for illustrative purposes. The production
/// implementation uses Q32.64 fixed-point arithmetic (i128-based)
/// in kelvin-core/src/fixed_math.rs.
pub fn euler_step(&mut self) -> Result<(), OrbitalError> {
    const DT: f64 = 0.001;
    const G: f64 = 1.0;
    const EPSILON: f64 = 1e-6;

    if self.step >= MAX_EULER_STEPS {
        return Err(OrbitalError::OrbitalOverflow { ... });
    }

    let mut accel = [[0.0; 3]; 5];
    Self::compute_accelerations(
        &self.positions, &self.masses, &mut accel, G, EPSILON,
    );

    for i in 0..5 {
        for j in 0..3 {
            // True explicit Euler: position before velocity
            let v_old = self.velocities[i][j];
            self.positions[i][j] += v_old * DT;
            self.velocities[i][j] += accel[i][j] * DT;
        }
    }

    self.step += 1;
    Ok(())
}
```

### Verlet Step (Available for Verification)

```rust
/// Velocity Verlet for symplectic integration.
///
/// Preserves phase-space volume for long-term stability.
/// Available for verification and backward compatibility.
pub fn verlet_step(&mut self) -> Result<(), OrbitalError> {
    const DT: f64 = 0.01;
    // ... two acceleration computations per step ...
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│           KELVIN-QUANTUM (Euler-Optimized)                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  INITIALIZATION:                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ OrbitalState::chaotic_default()                      │   │
│  │ ├── masses: [1.0, 0.001, 3e-6, 3.7e-8, 1e-11]       │   │
│  │ └── asymmetric positions + velocities                │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ BULK KEYSTREAM (V3 fast path):                        │   │
│  │ base_seed → HKDF → SHAKE256 → 1MB cache               │   │
│  │ XOR from cache → ... → cache empty                    │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ RESEED (Euler chaos injection):                       │   │
│  │ for _ in 0..10_000 { euler_step() }                   │   │
│  │ fresh_entropy = SHAKE256(positions + velocities)      │   │
│  │ base_seed[i] ^= fresh_entropy[i]                      │   │
│  │ → Repeat bulk cache refill with fresh seed            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Safety Considerations

### Overflow Protection
Euler is less stable than Verlet, so `MAX_EULER_STEPS` is set to 100M (vs Verlet's 1B). At dt=0.001, this represents 100,000 time units — more than enough for entropy generation.

### Energy Drift Monitoring
The `test_euler_energy_drift` test verifies that Euler has significantly more energy drift than Verlet after 10,000 steps. This drift is the mechanism for chaos amplification.

### Trajectory Divergence
The `test_euler_diverges_from_verlet` test verifies that Euler and Verlet trajectories diverge by >1.0 position units after 1000 steps, confirming that Euler produces fundamentally different (more chaotic) dynamics.

## References

- Benettin et al. (1980). "Lyapunov Characteristic Exponents for Smooth Dynamical Systems." *Meccanica*, 15, 9–20.
- Wolf et al. (1985). "Determining Lyapunov Exponents from a Time Series." *Physica D*, 16(3), 285–317.
- Hairer, E., Lubich, C., & Wanner, G. (2006). "Geometric Numerical Integration." Springer.
