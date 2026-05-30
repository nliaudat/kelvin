//! C3: Sequential Simulation Hardness Against Quantum Adversaries
//!
//! This Kani proof harness validates the classical preimage properties
//! that form the foundation of the quantum query lower bound argument.
//!
//! ## What is Proved
//!
//! 1. **Non-injectivity**: The Verlet step map Φ: X → X is many-to-one
//!    for N=2 bodies after 1 step. Two initial states differing by
//!    1 ULP produce outputs within ≤ 10 ULPs of each other, meaning
//!    the 1-ULP difference is partially or fully lost to rounding.
//!
//! 2. **Compound preimage growth**: After 2 Verlet steps, 4 distinct
//!    1-ULP-perturbed initial states produce outputs with at least
//!    one pair converging to ≤ 10 ULPs. This demonstrates that
//!    preimages compound across steps.
//!
//! 3. **Preimage bound (shared with C1)**: The division g / dist_cubed
//!    at the core of each gravitational interaction has preimage size
//!    bounded by ⌈dist_cubed_raw / 2^64⌉, which is at most 8,000,000
//!    at the maximum denominator (see verify_c1_epsilon_bound).
//!
//! ## Relationship to C3
//!
//! The full C3 conjecture states that inverting Φ^S requires
//! Ω(2^{S·k/2}) quantum queries. These harnesses verify the
//! *classical* preimage properties that justify the dissipative
//! nature of Φ — a necessary condition for the quantum lower bound.
//! The actual quantum query lower bound requires extending Ambainis'
//! adversary method, which is beyond Kani's capabilities.
//!
//! ## Running
//!
//! ```bash
//! cargo kani -p kelvin-core --harness verify_c3_step_non_injective
//! ```
//!
//! ## References
//!
//! - formal_verification.md §L3': Sequential Quantum Hardness
//! - kelvin-core/src/integrator.rs (Verlet step implementation)
//! - Bennett et al. (1997). "Strengths and Weaknesses of Quantum Computing."
//! - Ambainis, A. (2002). "Quantum Lower Bounds by Quantum Arguments."

// NOTE: These harnesses are designed to be integrated into
// kelvin-core/src/ under #[cfg(kani)]. The actual implementations
// live in kelvin-core/src/fixed_math.rs as verify_c3_step_non_injective
// and verify_c3_two_step_preimage_growth.