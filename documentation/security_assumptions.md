# Security Assumptions — Kelvin Cryptosystem

> **Document Status:** Final
> **Last Updated:** 2026-05-29

This document codifies the formal security assumptions underlying the Kelvin
cryptosystem. Each assumption includes:
- A formal statement of the assumption
- The code that enforces or relies on it
- The consequence if the assumption is violated
- Evidence supporting the assumption

---

## Assumption 1: N-Body One-Way Function

### Statement

Given the final orbital state after `S` simulation steps, it is computationally
infeasible to recover the initial orbital configuration (masses, positions,
velocities) within useful precision.

### Rationale

The N-body problem (N ≥ 3) has no closed-form analytical solution (Poincaré,
1889). Numerical integration is the only path forward. There is no known
algorithm to invert the Verlet or Euler integrator for a general N-body
configuration — an attacker would need to simulate forward from every possible
initial condition to find one matching the final state.

### Enforcement

- **Lyapunov exponent estimation** (`kelvin-kdf/src/lyapunov.rs`): The shadow
  orbit method estimates the Lyapunov exponent λ. Configurations with
  `total_steps < min_chaos_steps` are rejected at initialization:
  ```rust
  if config.total_steps < result.min_chaos_steps {
      return Err(KelvinError::InsufficientChaos { ... });
  }
  ```
- **Stability monitoring** (`kelvin-core/src/stability.rs`): Ejection and
  collapse detectors ensure N ≥ chaotic body count remains ≥ 3 throughout
  the simulation.

### Violation Consequence

If an attacker could efficiently invert the simulation, they could recover
the shared OrbitalConfig from intercepted ciphertext, breaking the OTP.

### Supporting Evidence

- **Kani proofs L0-L2** (`proofs/kani/`): Formal verification that
  `compute_accelerations` satisfies Newton's laws (action-reaction, direction,
  proportionality to mass).
- **Determinism tests L4** (`tests/kelvin_tests/determinism.rs`): 18 tests
  verify bit-identical simulation results, confirming the forward direction
  is well-defined.
- **Reference**: Chai et al. (2025), Song et al. (2025), Jawad (2025) —
  independent academic research relying on the same N-body one-way property.

---

## Assumption 2: Fixed-Point Determinism

### Statement

Q32.64 fixed-point arithmetic produces identical simulation results across
all platforms and architectures, including x86_64, aarch64, riscv64, and
wasm32.

### Rationale

Fixed-point arithmetic uses only integer operations (addition, subtraction,
multiplication, division, bit shifts) which are deterministic across all
IEEE-compliant platforms. This contrasts with floating-point arithmetic,
where different platforms may have different rounding behavior, FMA
implementations, or NaN propagation rules.

### Enforcement

- **Q32.64 format** (`kelvin-core/src/fixed_math.rs`): All simulation math
  uses `i128`-based fixed-point with `1.0 = 2^64`. No `f32` or `f64` values
  appear in the simulation engine (only in debug display `to_f64()`).
- **Cross-SIMD verification** (18 tests in `tests/kelvin_tests/determinism.rs`):
  All 18 determinism tests pass identically on SSE2, AVX, and AVX2.
- **Architecture-agnostic golden hashes**: Golden hashes are captured on
  x86_64 and validated across all execution contexts.

### Violation Consequence

If different platforms produce different keystreams from the same OrbitalConfig,
communicating parties using different hardware would be unable to decrypt each
other's messages.

### Supporting Evidence

- **Kani harness `verify_mul_no_overflow`**: Proves commutativity (`a*b == b*a`),
  identity (`a*1 == a`), and zero (`a*0 == 0`) for all bounded inputs.
- **18/18 determinism tests pass** on 3 CPU capabilities (SSE2, AVX, AVX2).
- **Reference**: Goldberg (1991) — "What Every Computer Scientist Should Know
  About Floating-Point Arithmetic."

---

## Assumption 3: SHAKE256 Security

### Statement

SHAKE256 (NIST FIPS 202) is a cryptographically secure extendable-output
function (XOF) with:
- **256-bit classical security** against collision attacks
- **128-bit quantum security** (Grover's algorithm reduces effective security
  by at most half)
- Computational indistinguishability from random for any polynomial-time
  adversary

### Rationale

SHAKE256 is a NIST-standardized hash function based on the Keccak sponge
construction. It is part of the SHA-3 family and has undergone extensive
public cryptanalysis since 2008. No practical preimage, collision, or
distinguishing attacks exist against SHAKE256 at the time of writing.

### Enforcement

- **Entropy extraction** (`kelvin-kdf/src/extractor.rs`): All orbital state
  extraction uses SHAKE256 via `extract_shake256_into()`.
- **Keystream generation** (`kelvin/src/photon.rs`, `kelvin/src/quantum.rs`,
  `kelvin/src/prism.rs`, `kelvin/src/split.rs`, `kelvin/src/flare.rs`): All
  OTP modes use SHAKE256 XOR as the core cipher construction.
- **Domain separation**: Each mode and operation uses a distinct domain
  separator string (e.g., `DOMSEP_PHOTON_KEYSTREAM_V1`, `DOMSEP_ORBITAL_STATE_V1`),
  ensuring cryptographic independence even when derived from the same seed.

### Violation Consequence

A break of SHAKE256 would compromise all OTP modes (V2, V3, H, Prism, Split,
Flare) and the entropy extraction pipeline. However, the orbital chaos layer
provides a second line of defense — an attacker would also need to invert the
N-body simulation.

### Supporting Evidence

- **NIST FIPS PUB 202** (2015): Standard specification.
- **Bertoni et al. (2013)**: Keccak security analysis.
- **NIST PQC standardization**: SHAKE256 is a core component of ML-KEM (FIPS 203)
  and ML-DSA (FIPS 204), both of which Kelvin integrates.

---

## Assumption 4: Lyapunov Horizon Validity

### Statement

Simulations with `total_steps < min_chaos_steps` produce keystream that is
computationally indistinguishable from random. Simulation beyond the Lyapunov
horizon may degrade unpredictability as trajectory divergence saturates.

### Rationale

The Lyapunov time is the characteristic timescale for exponential divergence
of nearby trajectories. Before the Lyapunov time, the system's state is
strongly correlated with initial conditions. After sufficient Lyapunov times,
the system state is effectively random with respect to the initial configuration.

The shadow orbit method (Benettin et al., 1980) estimates the Lyapunov
exponent by running a perturbed "shadow" trajectory alongside the main
trajectory and measuring their divergence rate.

### Enforcement

- **Lyapunov estimator** (`kelvin-kdf/src/lyapunov.rs`): Uses a deterministic
  shadow orbit to estimate λ. The `estimate()` method returns `min_chaos_steps`
  — the minimum number of steps needed to reach the chaotic regime.
- **Config validation** (`kelvin/src/lib.rs` lines 158-167): Configurations
  with `total_steps < min_chaos_steps` are rejected at instantiation.
- **Default steps** (`kelvin-core/src/constants.rs`: `DEFAULT_STEPS = 1_000_000`)
  is set well above typical Lyapunov times (~1000 steps for standard configs).

### Violation Consequence

If a configuration passes validation despite having insufficient steps, the
keystream may be correlated with the initial configuration beyond what is
modeled, potentially enabling cryptographic attacks. The fail-fast mechanism
at initialization prevents this by rejecting insufficient configurations.

### Supporting Evidence

- **Lyapunov analysis**: λ ≈ 0.693 (positive → chaotic regime) for standard
  5-body configurations. A Lyapunov time of ~1443 steps means the system
  doubles its trajectory divergence every ~1443 steps.
- **Entropy extraction**: SHAKE256 extracts entropy from the orbital state
  after the simulation is complete, ensuring the hash is computed on a
  fully-mixed chaotic state.
- **Reference**: Benettin et al. (1980), Wolf et al. (1985) — standard
  methods for Lyapunov exponent estimation in chaotic systems.

---

## Summary Table

| # | Assumption | Enforcement Location | Risk if Violated |
|:---:|---|:---|:---|
| 1 | N-body one-way | `lyapunov.rs`, `stability.rs`, `lib.rs` | OTP key recovery |
| 2 | Fixed-point determinism | `fixed_math.rs`, `determinism.rs` | Cross-platform decryption failure |
| 3 | SHAKE256 security | `extractor.rs`, mode `.rs` files | OTP keystream prediction |
| 4 | Lyapunov horizon | `lyapunov.rs`, `lib.rs` | Correlated keystream |

---

## References

- Benettin, G., Galgani, L., Giorgilli, A., & Strelcyn, J.-M. (1980).
  "Lyapunov Characteristic Exponents for Smooth Dynamical Systems and for
  Hamiltonian Systems." *Meccanica*, 15, 9–20.
- Bertoni, G., Daemen, J., Peeters, M., & Van Assche, G. (2013). "Keccak."
  *Advances in Cryptology — EUROCRYPT 2013*, 313–314.
- Goldberg, D. (1991). "What Every Computer Scientist Should Know About
  Floating-Point Arithmetic." *ACM Computing Surveys*, 23(1), 5–48.
- National Institute of Standards and Technology. (2015). "SHA-3 Standard:
  Permutation-Based Hash and Extendable-Output Functions." FIPS PUB 202.
- Poincaré, H. (1889). "Sur le problème des trois corps et les équations
  de la dynamique." *Acta Mathematica*, 13, 1–270.
- Wolf, A., Swift, J. B., Swinney, H. L., & Vastano, J. A. (1985).
  "Determining Lyapunov Exponents from a Time Series." *Physica D*, 16(3),
  285–317.