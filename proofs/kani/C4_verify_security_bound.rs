// C4_verify_security_bound.rs
//
// Corrected 2026-11-23. The previous version "proved" `Adv <= negl(n) + 2^-128`
// by asserting `128 == 128`; the arithmetic of an exponent is not an advantage
// bound. Kani cannot check a cryptographic reduction.
//
// SCOPE: this file proves the structural facts that the C4 argument uses:
// domain-separated framing is injective, and the operational bound is the
// minimum of the SHAKE256 and configuration-space exponents. The advantage
// bound itself remains open (C4).

const C4_TAG_LEN: usize = 1;

/// Domain-separated framing: one tag byte followed by the big-endian payload.
fn c4_frame(tag: u8, payload: u64) -> [u8; 9] {
    let mut out = [0u8; 9];
    out[0] = tag;
    out[1..].copy_from_slice(&payload.to_be_bytes());
    out
}

/// Distinct domain tags produce distinct framed inputs, for every payload.
#[kani::proof]
fn verify_domain_separation() {
    let t1: u8 = kani::any();
    let t2: u8 = kani::any();
    let payload: u64 = kani::any();

    kani::assume(t1 != t2);

    kani::assert(
        c4_frame(t1, payload) != c4_frame(t2, payload),
        "distinct domain tags imply distinct framed inputs",
    );
}

/// The same tag and payload frame identically (determinism of framing).
#[kani::proof]
fn verify_framing_deterministic() {
    let tag: u8 = kani::any();
    let payload: u64 = kani::any();

    kani::assert(
        c4_frame(tag, payload) == c4_frame(tag, payload),
        "framing is deterministic",
    );
    kani::assert(C4_TAG_LEN == 1, "framing uses a one-byte tag prefix");
}

/// Grover halves the exponent of a 256-bit symmetric key, leaving 128 bits.
#[kani::proof]
fn verify_shake256_grover_arithmetic() {
    let classical_bits: u32 = 256;
    let quantum_bits = classical_bits / 2;

    kani::assert(quantum_bits == 128, "Grover on 256 bits leaves 128 bits");
}

/// The operational bound is the smaller of the two exponents.
#[kani::proof]
fn verify_bound_is_min_of_two() {
    let config_bits: u32 = 960;
    let shake_bits: u32 = 128;

    let effective = if config_bits < shake_bits {
        config_bits
    } else {
        shake_bits
    };

    kani::assert(effective == 128, "SHAKE256 is the limiting exponent");
    kani::assert(effective <= config_bits, "operational bound <= config-space bound");
    kani::assert(effective <= shake_bits, "operational bound <= SHAKE256 bound");
}
