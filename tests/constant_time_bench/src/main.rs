//! # Constant-Time Side-Channel Benchmarking — Main Entry Point
//!
//! Runs all dudect benchmarks for the Kelvin cryptosystem's fixed-point
//! arithmetic operations.
//!
//! ## Usage
//!
//! ```bash
//! # Run all benchmarks
//! cargo run -p constant_time_bench
//!
//! # Run only sqrt-related benchmarks
//! cargo run -p constant_time_bench -- --filter sqrt
//!
//! # Run only multiplication benchmarks
//! cargo run -p constant_time_bench -- --filter mul
//!
//! # Run only division benchmarks
//! cargo run -p constant_time_bench -- --filter div
//!
//! # Run only acceleration benchmarks
//! cargo run -p constant_time_bench -- --filter acceleration
//!
//! # Continuous mode (runs indefinitely until Ctrl-C)
//! cargo run -p constant_time_bench -- --continuous mul
//! ```

use constant_time_bench::acceleration::bench_compute_accelerations;
use constant_time_bench::fixed_div::{bench_fixed_div_magnitude, bench_fixed_div_sign};
use constant_time_bench::fixed_mul::bench_fixed_mul;
use constant_time_bench::fixed_sqrt::bench_fixed_sqrt;
use constant_time_bench::fixed_sqrt_clamp::bench_fixed_sqrt_clamp;
use constant_time_bench::fixed_sqrt_edge::bench_fixed_sqrt_edge;

dudect_bencher::ctbench_main!(
    bench_fixed_mul,
    bench_fixed_div_magnitude,
    bench_fixed_div_sign,
    bench_fixed_sqrt,
    bench_fixed_sqrt_clamp,
    bench_fixed_sqrt_edge,
    bench_compute_accelerations
);
