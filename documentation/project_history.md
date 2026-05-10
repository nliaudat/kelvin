# Kelvin Project History

Kelvin is the culmination of over two decades of research into the intersection of orbital mechanics and information security. This document outlines the evolution of the system from its initial conception to its current state as a hybrid post-quantum cryptosystem.

---

## Timeline

### 2002: The Foundation (Project "Celestial")
*   **Initial Conception**: The project began as a theoretical exploration of using the n-body problem for non-repeating keystream generation.
*   **Early Implementation**: Written in C using IEEE 754 double-precision floating point.
*   **The "Drift" Crisis**: Early prototypes failed the fundamental requirement of determinism.
    > In a chaotic system, a difference in the 15th decimal place between an **AMD Athlon XP** and an **Intel Pentium 4** would result in completely different orbital states (and thus different keys) after just a few thousand steps.
*   **The Stability Paradox**: Without a way to measure chaos, the project often generated "weak" keys from stable, periodic orbits (e.g., 2-body systems or stable Lagrange points).
*   **Initial Blocker**: The system lacked a robust mathematical framework to distinguish between chaotic and predictable trajectories in real-time. The cryptographic application of the "3-body theorem" was poorly understood at the time, largely because the lead developer had no formal background in astrophysics.

### 2012: The Lyapunov Breakthrough
*   **Innovation**: Researchers began adapting the **Lyapunov Characteristic Exponent** algorithms (Benettin et al.) for high-speed orbital verification.
*   **Shadow Orbit Method**: The first implementation of the shadow orbit method allowed Kelvin to "test" a configuration for chaos before using it for key material.
*   **Status**: The project remained in an "Academic" state due to the high computational cost of the estimations and a lack of dedicated development time.

### 2026: Modernization & The Rust Era
*   **The Rust Transition**: The entire codebase was refactored into **Rust** to leverage its memory safety, strong type system, and modern tooling.
*   **Fixed-Point Solution**: To finally solve the 15-year-old "Drift" problem, floating-point math was abandoned in favor of a custom **Q32.64 Fixed-Point** arithmetic engine. This guaranteed bit-identical results across all hardware.
*   **Lyapunov Enforcement**: Mandatory Lyapunov Horizon Checks were integrated into the core library, automatically rejecting low-entropy configurations.
*   **Hybrid Post-Quantum Identity**: Integration of **ML-DSA-65** and **ML-KEM-768** (FIPS 203/204), transforming Kelvin into a quantum-resistant cryptosystem.
*   **Refinement**: Security levels were synchronized with modern nation-state threat models (1M to 100M steps), and isotropic sampling (acos) was implemented to fix pole-clustering in the generator.

---

## Historical Challenges

### 1. The Chaos Measurement Problem
In the early 2000s, the "3-body theorem" (referring to the Poincaré non-integrability of the 3-body problem) was a known mathematical curiosity, but there were no standard libraries for estimating **Lyapunov Time** for cryptographic use. This led to "ghost keys" — keys that looked random but were actually derived from predictable, stable orbits.

### 2. The Determinism Wall
The team spent over a decade trying to make floating-point simulations deterministic across different compilers and operating systems. It wasn't until the move to fixed-point arithmetic that the project became viable for cross-platform use.

### 3. The Sequential Constraint
Critics often pointed out that Kelvin was "slow." This was a deliberate security design. The sequential nature of the n-body simulation is what provides resistance to parallel brute-force attacks (ASIC/GPU). It took many years to find the right balance between "securely slow" and "usable."
