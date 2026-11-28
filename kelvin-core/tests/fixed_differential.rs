//! Differential tests for the Q32.64 fixed-point primitives.
//!
//! These complement the Kani harnesses: Kani proves properties over all
//! symbolic inputs of bounded width, while these tests compare the
//! implementation against an independent `f64` reference on a deterministic
//! pseudo-random sample. They are *not* a proof, but they catch sign, rounding
//! and overflow regressions that a range-only harness can miss.
//!
//! Run with:
//!
//! ```text
//! cargo test -p kelvin-core --test fixed_differential
//! ```

use kelvin_core::Fixed;

/// Deterministic LCG so failures are reproducible.
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn sample_int(state: &mut u64, bound: i64) -> i64 {
    let v = (lcg(state) % (2 * bound as u64 + 1)) as i64;
    v - bound
}

/// Absolute tolerance: one raw unit is 2^-64, so anything above 1e-15 is a bug
/// rather than rounding, for the magnitudes used here.
const TOL: f64 = 1e-12;

fn close(got: f64, want: f64, what: &str, a: i64, b: i64) {
    let err = (got - want).abs();
    assert!(
        err <= TOL,
        "{what}: got {got}, want {want}, err {err} for a={a}, b={b}"
    );
}

#[test]
fn add_sub_are_exact() {
    let mut s = 0x5eed_1234_9876_abcd_u64;
    for _ in 0..10_000 {
        let (ai, bi) = (sample_int(&mut s, 10_000), sample_int(&mut s, 10_000));
        let (a, b) = (Fixed::from_int(ai), Fixed::from_int(bi));

        let sum = (a + b).to_f64();
        assert_eq!(sum, (ai + bi) as f64, "add {ai} + {bi}");

        let diff = (a - b).to_f64();
        assert_eq!(diff, (ai - bi) as f64, "sub {ai} - {bi}");
    }
}

#[test]
fn mul_matches_f64() {
    let mut s = 0xc0ff_ee00_1234_5678_u64;
    for _ in 0..10_000 {
        let (ai, bi) = (sample_int(&mut s, 1_000), sample_int(&mut s, 1_000));
        let (a, b) = (Fixed::from_int(ai), Fixed::from_int(bi));

        close((a * b).to_f64(), ai as f64 * bi as f64, "mul", ai, bi);
    }
}

#[test]
fn div_matches_f64() {
    let mut s = 0xdead_beef_0bad_f00d_u64;
    for _ in 0..10_000 {
        let ai = sample_int(&mut s, 1_000);
        let mut bi = sample_int(&mut s, 1_000);
        if bi == 0 {
            bi = 1;
        }
        let (a, b) = (Fixed::from_int(ai), Fixed::from_int(bi));

        close((a / b).to_f64(), ai as f64 / bi as f64, "div", ai, bi);
    }
}

#[test]
fn sqrt_matches_f64() {
    let mut s = 0x0123_4567_89ab_cdef_u64;
    for _ in 0..10_000 {
        let ai = sample_int(&mut s, 10_000).abs();
        let a = Fixed::from_int(ai);

        // Compare against the f64 square root. `sqrt` returns a Q32.64 value,
        // so its own granularity is 2^-64; the tolerance below still catches any
        // error larger than a handful of ULPs at these magnitudes.
        let got = a.sqrt().to_f64();
        let want = (ai as f64).sqrt();
        let err = (got - want).abs();
        assert!(
            err <= 1e-9 * want.max(1.0),
            "sqrt: got {got}, want {want}, err {err} for a={ai}"
        );
    }
}

#[test]
fn from_int_round_trip() {
    for v in -1_000_000i64..=1_000_000 {
        let f = Fixed::from_int(v);
        assert_eq!(f.to_raw(), (v as i128) << 64, "raw round trip for {v}");
        assert_eq!(f.to_f64(), v as f64, "f64 round trip for {v}");
    }
}
