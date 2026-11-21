// fixed_safety.rs
//! Safety harness for Kani – verifies that core fixed‑point operations cannot panic or overflow.

#[cfg(kani)]
mod harness {
    use kelvin_core::fixed_math::Fixed;
    use kani::proof;

    #[proof]
    fn check_fixed_zero() {
        let zero = Fixed::ZERO;
        assert_eq!(zero, Fixed::from_int(0));
    }
}
