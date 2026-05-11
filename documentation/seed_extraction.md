# Seed Extraction & Entropy Derivation

This document explains how the Kelvin system extracts cryptographic seeds from chaotic orbital simulations. This process is the bridge between physical dynamics and digital security.

## Overview

Seed extraction is the process of taking the raw physical state of the N-body simulation and mapping it into a uniformly distributed, fixed-length bitstring. In Kelvin, this is performed after the simulation has run for a sufficient number of steps to ensure that the [Lyapunov divergence](proof_of_concept.md) has fully randomized the state.

## Orbital Parameters Used

The extraction process hashes the complete state of the orbital system to ensure that any variation in any physical parameter results in a different cryptographic key. The following parameters are used as input to the entropy extractor:

### 1. Per-Body State
For every body in the simulation, seven parameters are hashed:
- **Mass ($M$)**: Represented as a 128-bit fixed-point value in solar masses ($M_{\odot}$).
- **Position ($x, y, z$)**: 3D coordinates in Astronomical Units (AU).
- **Velocity ($v_x, v_y, v_z$)**: 3D velocity vectors in AU/year.
- **Gravitational Force ($F_x, F_y, F_z$)**: The instantaneous acceleration vector acting on the body from all other planets.

### 2. Temporal Context
- **Step Counter**: The 64-bit integer representing the current simulation step. This ensures that the system can derive unique keys at different points in its evolution without needing to change the initial conditions.

### 3. System Configuration
- **Body Count**: The total number of bodies in the system.
- **Domain Separator**: A cryptographic personalization string used to ensure domain separation between different instances or protocols using the same chaotic seed.

## Cryptographic Mechanism

Kelvin uses **SHAKE256** (the Keccak Extendable-Output Function) for entropy extraction. This allows the system to derive large entropy pools (defaulting to **2048 bytes**) from a single simulation state.

### Input Serialization
All parameters are serialized in **little-endian** format to ensure deterministic behavior across different CPU architectures. The hash input is structured as follows:

1. `domain_separator` (Variable length)
2. `g` (Gravitational Constant, 16 bytes)
3. `softening` (Softening Factor, 16 bytes)
4. `step_counter` (8 bytes)
5. `body_count` (4 bytes)
6. For each body:
   - `mass` (16 bytes)
   - `position.x` (16 bytes)
   - `position.y` (16 bytes)
   - `position.z` (16 bytes)
   - `velocity.x` (16 bytes)
   - `velocity.y` (16 bytes)
   - `velocity.z` (16 bytes)
   - `force_vector.x` (Gravitational Force, 16 bytes)
   - `force_vector.y` (Gravitational Force, 16 bytes)
   - `force_vector.z` (Gravitational Force, 16 bytes)

### Rationale
- **SHAKE256 (XOF)**: Chosen for its efficiency and ability to produce arbitrary length output (XOF). This allows the system to initialize a massive 2048-byte master seed pool.
- **Keccak Sponge**: SHAKE256 is built on the Keccak sponge, providing inherent post-quantum resistance and immunity to length-extension attacks.
- **Force Vector Binding**: Including the instantaneous gravitational forces ensures that the seed is bound not just to coordinates, but to the physical laws ($G$, softening) and interactions of the system.
- **Domain Separation** prevents "key leakage" where a seed used for one purpose (e.g., signing) might inadvertently be identical to a seed used for another (e.g., encryption).
- **Full State Hashing**: Including mass, position, and velocity ensures that the entire physical degree of freedom of the system is captured.

## Empirical Verification

The robustness of this extraction process has been verified through large-scale statistical testing:
- **Collision Resistance**: Analysis of 1,000 unique configurations showed zero collisions in the derived seeds.
- **Sensitivity**: Even minimal changes to the Sun mass ($\pm 25\%$) result in complete avalanche across the 2048-byte entropy pool.
- **Detailed Findings**: See the [Entropy Analysis Report](entropy_report.md) for full statistical data.

---
*Last Updated: 2026-05-11*

## Avalanche Effect

Due to the chaotic nature of the underlying N-body problem, the extraction process exhibits a "Double Avalanche":
1. **Physical Avalanche**: Microscopic changes in initial conditions grow exponentially over time due to positive Lyapunov exponents.
2. **Cryptographic Avalanche**: The SHAKE256 hash ensures that even a single bit change in the final orbital coordinates or forces flips approximately 50% of the bits in the resulting seed.

This combination makes it computationally infeasible to reverse the seed to find the original orbital state or to predict future seeds without running the full simulation.
