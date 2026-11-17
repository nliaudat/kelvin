// verify_information_loss_total.rs
//! Kani harness to verify the total information‑loss bound (C1).
//! It checks that the accumulated loss over a number of steps does not exceed the
//! expected per‑step bound multiplied by the step count.

#[cfg(kani)]
mod harness {
    use kani::cover;
    use rand::Rng;
    use kelvin_core::information_loss::{per_step_loss, total_loss};

    #[kani::proof]
    fn verify_information_loss_total() {
        // Choose a random number of steps (bounded for tractability).
        let mut rng = rand::thread_rng();
        let steps: usize = rng.gen_range(0..=1_000);

        // Compute the total loss using the placeholder implementation.
        let loss = total_loss(steps);

        // Expected bound: loss should be <= steps * per_step_loss().
        let bound = per_step_loss() * steps as f64;
        assert!(loss <= bound + 1e-6);
        cover!(true);
    }
}
