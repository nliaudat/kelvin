// information_loss.rs
//! Simple placeholder implementations for per‑step and total information‑loss calculations.
//! In a full formalisation these would be derived from the simulation's
//! arithmetic properties. Here we provide deterministic stubs sufficient for
//! the Kani harnesses.

/// Per‑step information‑loss bound (bits). For the placeholder we assume
/// a conservative 1 bit loss per simulation step.
pub fn per_step_loss() -> f64 {
    1.0
}

/// Total information‑loss over `steps` simulation steps.
/// Returns `steps * per_step_loss()`.
pub fn total_loss(steps: usize) -> f64 {
    per_step_loss() * steps as f64
}
