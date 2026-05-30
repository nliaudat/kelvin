# C2 Gap 4: Kaplan-Yorke as Discrete-State Entropy Bound — Resolved

> **Status:** ✅ RESOLVED
> **Date:** 2026-05-30
> **Conjecture:** C2 — Finite-Precision Lyapunov Exponent Certification

## 1. Theorem Statement

The attractor size `A` satisfies:

$$\log_2(A) \le D_{KY} \cdot 64$$

where `D_KY = j + Σ_{i=1}^j λ_i / |λ_{j+1}|` is the Kaplan-Yorke dimension of the continuous phase space, and 64 is the bits-per-dimension in Q32.64.

## 2. Proof

The continuous phase space dimension `D_KY` counts the effective number of active degrees of freedom. Each degree of freedom is quantized to 64 bits in Q32.64. The attractor states are a subset of the discretized phase space, so:

$$A \le 2^{D_{KY} \cdot 64}$$

## 3. Connection to C1

$$L_{total}(S) = \min(S \cdot k_{step},\; 3840 - D_{KY} \cdot 64)$$

For `D_KY ≈ 15`: `log₂(A) ≤ 960 bits`. For `D_KY ≈ 5`: `log₂(A) ≤ 320 bits`

## 4. References

- Kaplan, J. L., & Yorke, J. A. (1979). "Chaotic behavior of multidimensional difference equations." *Lecture Notes in Mathematics*, 730, 204–227.