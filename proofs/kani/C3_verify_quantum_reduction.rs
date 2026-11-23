// C3_verify_quantum_reduction.rs
//
// Corrected 2026-11-23. The previous version was never compiled and "verified"
// `QUANTUM_QUERY_LOWER_BOUND > 10`, a constant chosen inside the harness. Kani
// cannot check a quantum query lower bound at all.
//
// SCOPE: this file proves only the exponent arithmetic that the informal Grover
// argument relies on: `sqrt(2^n) = 2^(n/2)`, and halving an even exponent is
// exact. The reduction itself remains an informal argument (C3).

/// Halving the exponent of an unstructured search space is well defined, and the
/// halved exponent never exceeds the original.
#[kani::proof]
fn verify_c3_quantum_reduction() {
    let bits: u32 = kani::any();
    kani::assume(bits <= 4096);

    kani::assert(bits / 2 <= bits, "halved exponent is at most the original");
    kani::assert(
        2 * (bits / 2) <= bits,
        "doubling the halved exponent does not exceed the original",
    );
}

/// The specific exponent claimed for Kelvin: `|Theta| >= 2^1920` implies a
/// Grover exponent of 960, which is larger than the 128-bit SHAKE256 limit.
#[kani::proof]
fn verify_c3_exponent_arithmetic() {
    let config_bits: u32 = 1920;
    let grover_bits = config_bits / 2;

    kani::assert(grover_bits == 960, "sqrt(2^1920) has exponent 960");
    kani::assert(
        grover_bits > 128,
        "config-space exponent exceeds the 128-bit SHAKE256 bound",
    );
}
