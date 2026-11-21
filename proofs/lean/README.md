# Kelvin Cryptosystem Formal Verification (Lean)

Formal mathematical proofs for the Kelvin cryptosystem using the [Lean Theorem Prover](https://lean-lang.org/).

## Purpose

This directory provides **mathematical specifications** and **formal proofs** that complement the Kani-based code verification. While Kani verifies that the Rust implementation matches its specification, Lean proves that the specification itself is mathematically sound.

## Architecture

```
Kelvin Cryptosystem
├── Mathematical Specification (Lean proofs) ← This directory
│   ├── fixed_point/FixedPoint.lean       ← Q32.64 arithmetic
│   ├── n_body/NewtonianGravity.lean      ← Gravitational forces
│   ├── dynamics/VerletIntegrator.lean    ← Symplectic integration
│   ├── dynamics/LyapunovExponents.lean   ← Chaos certification
│   ├── entropy_extraction/Shake256Extraction.lean
│   └── key_schedule/HKDFDerivation.lean
│
├── Rust Implementation (kelvin-core)
│   ├── src/fixed_math.rs                  ← Q32.64 arithmetic
│   ├── src/body.rs                        ← Vec3, OrbitalBody
│   ├── src/integrator.rs                  ← Verlet, Euler
│   └── ...
│
└── Code Verification (Kani proofs)
    └── proofs/kani/*.rs                   ← Automated verification
```

## Installation

### Prerequisites

1. Install Lean 4:
```bash
lean --init mathlib
```

2. Navigate to the Lean proofs:
```bash
cd proofs/lean/src
```

## Building Proofs

### Build All Files

```bash
lean build KelvinFormalVerif.lean
```

### Build Specific Component

```bash
# Fixed-point arithmetic
lean build fixed_point/FixedPoint.lean

# Newtonian gravity
lean build n_body/NewtonianGravity.lean

# Verlet integrator
lean build dynamics/VerletIntegrator.lean

# Lyapunov exponents
lean build dynamics/LyapunovExponents.lean

# Entropy extraction
lean build entropy_extraction/Shake256Extraction.lean

# Key schedule
lean build key_schedule/HKDFDerivation.lean
```

## Proof Status

| Component | Mathematical Status | Corresponds to Kani |
|-----------|-------------------|---------------------|
| Fixed-Point Arithmetic | ✅ Fully Proved | L0, L1 |
| Newtonian Gravity | ✅ Fully Proved | L2 |
| Verlet Integrator | ✅ Fully Proved | L3 |
| Lyapunov Exponents | ⚠️ Justified (measured) | C2 |
| Entropy Extraction | ✅ Fully Proved | L3, C4 |
| Key Schedule | ✅ Fully Proved | L3 |

## Understanding "Justified" vs "Proved"

Some components are marked as **"justified"** rather than **"proved"** because they rely on:

1. **Empirical Measurement**: Lyapunov exponent λ ≈ 0.7 yr⁻¹ is measured, not derived
2. **Statistical Analysis**: NIST SP 800-90B test results are empirical
3. **Plausibility Arguments**: Grover search bounds are theoretical estimates

These components are still formally documented and verified through measurement, but the underlying values come from empirical data rather than pure mathematical derivation.

## Relationship to Kani Proofs

```
┌─────────────────────────────────────────────────────────┐
│  Mathematical Specification (Lean)                      │
│  - Pure mathematics                                     │
│  - Theoretical correctness                              │
│  - Interactive proof development                        │
└──────────────────────┬──────────────────────────────────┘
                       ↓ equivalence (proven)
┌─────────────────────────────────────────────────────────┐
│  Rust Implementation (kelvin-core)                       │
│  - Q32.64 arithmetic                                   │
│  - N-body simulation                                    │
│  - Entropy extraction                                   │
└──────────────────────┬──────────────────────────────────┘
                       ↓ equivalence (Kani verified)
┌─────────────────────────────────────────────────────────┐
│  Code Verification (Kani proofs)                         │
│  - Automated model checking                             │
│  - Implementation bug detection                         │
│  - Golden hash verification                             │
└─────────────────────────────────────────────────────────┘
```

## How to Write Proofs in Lean

### Basic Structure

```lean
/-- A theorem statement /-
theorem my_theorem (args : Type) : conclusion := by
  -- Proof goes here
  intro h  -- Introduce hypotheses
  rw [h]   -- Rewrite using hypothesis
  exact ...  -- Provide the term
```

### Common Tactics

```lean
-- Introduction tactics
intro h        -- Introduce hypothesis
intros h1 h2   -- Introduce multiple hypotheses

-- Rewriting tactics
rw [h]        -- Rewrite using equality
simp [h]      -- Simplify using lemmas
cases h       -- Case analysis on inductive type

-- Proof completion
exact term    -- Provide the exact term
trivial       -- For trivial goals
apply lemma   -- Apply a lemma to reduce goal
```

### Mathematical Notation

```lean
-- Real numbers and operations
def x : ℝ := 3.14
theorem add_comm (a b : ℝ) : a + b = b + a := by
  ring        -- Algebraic simplification

-- Quantifiers
theorem forall_example : ∀ n : ℕ, n + 0 = n := by
  intro n     -- Introduce universal quantifier
  rfl         -- Reflexivity

-- Existential quantifiers
theorem exists_example : ∃ x : ℝ, x > 0 := by
  use 1       -- Witness for existential
  norm_num    -- Prove property of witness
```

## File Organization

```
proofs/lean/
├── README.md                      ← This file
└── src/
    ├── KelvinFormalVerif.lean     ← Main entry point
    ├── fixed_point/
    │   └── FixedPoint.lean        ← Q32.64 arithmetic proofs
    ├── n_body/
    │   └── NewtonianGravity.lean  ← Gravitational force proofs
    ├── dynamics/
    │   ├── VerletIntegrator.lean   ← Integration algorithm proofs
    │   └── LyapunovExponents.lean  ← Chaos certification
    ├── entropy_extraction/
    │   └── Shake256Extraction.lean ← Entropy pool extraction
    └── key_schedule/
        └── HKDFDerivation.lean     ← Key derivation proofs
```

## Contributing

To add a new proof:

1. Create a `.lean` file in the appropriate subdirectory
2. Import necessary Mathlib modules at the top
3. Define types and structures matching Kelvin's Rust implementation
4. State theorems with clear documentation comments
5. Provide proofs (complete or use `sorry` as placeholder)
6. Add the new theorem to `KelvinFormalVerif.lean`

## License

This formal verification code is licensed under the same terms as Kelvin:
- Apache License, Version 2.0
- MIT license

At your option.

## References

- [Lean Theorem Prover Documentation](https://leanprover-community.github.io/)
- [Mathlib Library](https://github.com/leanprover-community/mathlib)
- [Apple Corecrypto Formal Verification](https://security.apple.com/blog/formal-verification-corecrypto/)
- Kelvin Cryptosystem: `documentation/formal_verification.md`

## Additional Resources

- [Lean 4 Tutorial](https://leanprover-community.github.io/tutorial/)
- [Mathlib Cheat Sheet](https://github.com/leanprover-community/mathlib/blob/master/docs/CheatSheet.md)
- [Lean Prover Zulip Chat](https://zulip.chat.leanprover-community.org/)
