//! C6: run exact-state Floyd on the real Verlet simulation.
//! `None` only means "no repeat within budget", never "period is long".
use kelvin_core::{
    detect_orbital_period, Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR,
    SOLAR_MASS,
};

fn config() -> [OrbitalBody; 3] {
    [
        OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO),
        OrbitalBody::new(
            SOLAR_MASS / Fixed::from_int(1047),
            Vec3::new(Fixed::from_int(5), Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(3), Fixed::ZERO),
        ),
        OrbitalBody::new(
            SOLAR_MASS / Fixed::from_int(10000),
            Vec3::new(Fixed::from_int(-3), Fixed::from_int(4), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-2), Fixed::from_int(-1), Fixed::ZERO),
        ),
    ]
}

#[test]
fn no_repeat_within_budget_is_reported_honestly() {
    let r = detect_orbital_period(&config(), DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G, 20_000);
    println!("observed (budget 20000): {r:?}");
    // Pins the observation recorded in TODO.md.
    assert_eq!(r, None);
}
