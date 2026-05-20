//! # Constant-Time Side-Channel Benchmarking for Kelvin Cryptosystem
//!
//! This crate uses the dudect-bencher framework (Welch's t-test) to empirically
//! verify that the Q32.64 fixed-point arithmetic operations in `kelvin-core` and
//! the KDF extraction in `kelvin-kdf` execute in constant time regardless of
//! secret input values.
//!
//! ## Methodology
//!
//! Each benchmark generates ~100,000 random test vectors, randomly assigning each
//! to one of two classes (`Class::Left` vs `Class::Right`). The two classes
//! represent different input regimes (e.g., small vs. large values). Each
//! operation is timed in nanoseconds via `Instant::now()` + `black_box()`.
//!
//! Welch's t-test is then computed comparing the two timing distributions.
//! A `|t| < 5` threshold indicates no statistically significant timing
//! difference — i.e., the operation is constant-time with respect to the
//! input class.
//!
//! ## References
//!
//! - Reparaz, O., Balasch, J., & Verbauwhede, I. (2017). "Dude, is my code
//!   constant time?" *Design, Automation & Test in Europe Conference (DATE)*.
//!   doi:10.23919/DATE.2017.7927267
//!   — Introduces the dudect (DudeCT) methodology for testing constant-time
//!   execution using statistical hypothesis testing.
//! - Bernstein, D. J. (2005). "Cache-timing attacks on AES."
//!   — Seminal work demonstrating the practical threat of timing side channels.
//! - Kocher, P. (1996). "Timing attacks on implementations of Diffie-Hellman,
//!   RSA, DSS, and other systems." *CRYPTO '96*.
//!   — Foundational paper on timing side-channel attacks.

pub mod fixed_mul;
pub mod fixed_div;
pub mod fixed_sqrt;
pub mod fixed_sqrt_clamp;
pub mod fixed_sqrt_edge;
pub mod acceleration;
