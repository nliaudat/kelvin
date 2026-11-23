//! Corrected Kani harnesses (2026-11-23).
//!
//! Each `include!` pulls in a harness file from `proofs/kani/`. Those files used
//! to sit outside every Cargo target, so `cargo kani` never saw them. Including
//! them here means they are compiled, type-checked and run by
//! `docker/run-kani.sh`.
//!
//! The harnesses were also rewritten: the originals asserted constants chosen
//! inside the harness, used `rand`/`Instant::now()` (unsupported by Kani), or
//! referenced modules that do not exist. Each file now states a property Kani
//! can actually check, and documents what it does *not* establish.
//!
//! This module is only compiled under `cfg(kani)`.

mod c1_information_loss {
    include!("../../proofs/kani/C1_verify_information_loss_total.rs");
}

mod c2_lyapunov {
    include!("../../proofs/kani/C2_verify_lyapunov_spectrum.rs");
}

mod c3_quantum {
    include!("../../proofs/kani/C3_verify_quantum_reduction.rs");
}

mod c4_keystream {
    include!("../../proofs/kani/C4_verify_security_bound.rs");
}

mod c5_side_channel {
    include!("../../proofs/kani/C5_verify_side_channel.rs");
}

mod c6_periodicity {
    include!("../../proofs/kani/C6_verify_periodicity.rs");
}

mod legacy_information_loss {
    include!("../../proofs/kani/verify_information_loss_total.rs");
}

mod legacy_lyapunov {
    include!("../../proofs/kani/verify_lyapunov_spectrum.rs");
}

mod legacy_periodicity {
    include!("../../proofs/kani/verify_periodicity.rs");
}

mod legacy_side_channel {
    include!("../../proofs/kani/verify_side_channel.rs");
}

mod legacy_c3 {
    include!("../../proofs/kani/verify_c3_quantum_reduction.rs");
}
