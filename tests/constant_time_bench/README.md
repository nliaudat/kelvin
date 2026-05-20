# Constant-Time Side-Channel Benchmarking

This crate uses the [dudect-bencher](https://crates.io/crates/dudect-bencher) framework
(Welch's t-test) to empirically verify that the Q32.64 fixed-point arithmetic operations
in `kelvin-core` execute in constant time regardless of secret input values.

**Full results report:** See [Section 8 of the Proof of Concept](../documentation/proof_of_concept.md#8-constant-time-side-channel-benchmarking).

## Quick Start

```bash
# Run all benchmarks
cargo run -p constant_time_bench

# Run specific benchmarks
cargo run -p constant_time_bench -- --filter sqrt
cargo run -p constant_time_bench -- --filter mul
cargo run -p constant_time_bench -- --filter div
cargo run -p constant_time_bench -- --filter acceleration

# Continuous mode (runs until Ctrl-C)
cargo run -p constant_time_bench -- --continuous mul
```

## Benchmarks

| Benchmark | What it tests |
|-----------|---------------|
| `bench_fixed_mul` | Direct path vs. high/low splitting fallback |
| `bench_fixed_div_magnitude` | Small vs. large dividend/divisor magnitudes |
| `bench_fixed_div_sign` | Positive vs. negative signs |
| `bench_fixed_sqrt` | Small vs. large positive values (realistic range) |
| `bench_fixed_sqrt_clamp` | Values below vs. above clamping threshold |
| `bench_fixed_sqrt_edge` | Zero/negative (early return) vs. positive (validation) |
| `bench_compute_accelerations` | Small vs. large masses (same position magnitudes) |

## References

- Reparaz, O., Balasch, J., & Verbauwhede, I. (2017). "Dude, is my code constant time?"
  *Design, Automation & Test in Europe Conference (DATE)*. doi:10.23919/DATE.2017.7927267
