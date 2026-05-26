//! Key generation — orbital configuration generation.
//!
//! Generates random orbital configurations at the requested security level.
//! Supports standard, paranoid, and maximum levels with optional fast mode
//! (110,000 steps) for benchmarking and testing.

use anyhow::Result;
use kelvin::{
    Fixed, OrbitalBody, OrbitalConfig, Vec3, FAST_RESEED_INTERVAL, FAST_STEPS, MAXIMUM_BODIES,
    MAXIMUM_STEPS, ORBITAL_VELOCITY_CONSTANT, PARANOID_BODIES, PARANOID_STEPS, PLANET_MASS_MAX_RAW,
    PLANET_MASS_MIN_RAW, PLANET_RADIUS_MULTIPLIER, STANDARD_BODIES, STANDARD_STEPS,
    SUN_MASS_CENTER, SUN_MASS_MAX_RAW, SUN_MASS_MIN_RAW, SUN_MASS_RANGE, SUN_POS_MAX_RAW,
    SUN_POS_MIN_RAW, SUN_VEL_MAX_RAW, SUN_VEL_MIN_RAW,
};
use rand::Rng;

/// Generate an orbital configuration at the given security level.
///
/// When `fast` is true, uses 110,000 simulation steps instead of the full
/// step count (standard=1M, paranoid=10M, maximum=100M). This produces a
/// valid config that passes the Lyapunov chaos check (~100k min_chaos_steps)
/// while keeping simulation time under ~4s.
#[allow(clippy::disallowed_methods)]
pub fn generate_config(level: &str, fast: bool) -> Result<OrbitalConfig> {
    let mut rng = rand::thread_rng();
    let (n_bodies, steps) = match level {
        "standard" => (STANDARD_BODIES, STANDARD_STEPS),
        "paranoid" => (PARANOID_BODIES, PARANOID_STEPS),
        "maximum" => {
            eprintln!(
                "Warning: 'maximum' level uses {} bodies and {} simulation steps.",
                MAXIMUM_BODIES, MAXIMUM_STEPS
            );
            eprintln!("This will take significantly longer than 'standard' or 'paranoid'.");
            (MAXIMUM_BODIES, MAXIMUM_STEPS)
        },
        _ => {
            anyhow::bail!("Unknown security level: {}. Use standard, paranoid, or maximum.", level)
        },
    };

    // When fast mode is enabled, override the step count to 110,000
    // (just above the Lyapunov horizon) and scale reseed_interval
    // proportionally. The body state (positions/velocities/masses) is
    // unchanged — it's still generated with the full body count for the
    // requested security level.
    let (use_steps, use_reseed) =
        if fast { (FAST_STEPS, FAST_RESEED_INTERVAL) } else { (steps, steps / 10) };

    let mut bodies = Vec::with_capacity(n_bodies);

    let sun_mass = loop {
        let raw: i128 = SUN_MASS_CENTER + rng.gen_range(-SUN_MASS_RANGE..SUN_MASS_RANGE + 1);
        let m = Fixed::from_raw(raw);
        if m >= Fixed::from_raw(SUN_MASS_MIN_RAW) && m <= Fixed::from_raw(SUN_MASS_MAX_RAW) {
            break m;
        }
    };
    bodies.push(OrbitalBody::new(
        sun_mass,
        Vec3::new(
            Fixed::from_raw(rng.gen_range(SUN_POS_MIN_RAW..SUN_POS_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_POS_MIN_RAW..SUN_POS_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_POS_MIN_RAW..SUN_POS_MAX_RAW)),
        ),
        Vec3::new(
            Fixed::from_raw(rng.gen_range(SUN_VEL_MIN_RAW..SUN_VEL_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_VEL_MIN_RAW..SUN_VEL_MAX_RAW)),
            Fixed::from_raw(rng.gen_range(SUN_VEL_MIN_RAW..SUN_VEL_MAX_RAW)),
        ),
    ));

    for i in 1..n_bodies {
        let radius = (i as i64 + 1) * PLANET_RADIUS_MULTIPLIER;
        let mass = Fixed::from_raw(rng.gen_range(PLANET_MASS_MIN_RAW..PLANET_MASS_MAX_RAW));

        let theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let phi = (rng.gen_range(-1.0..1.0f64)).acos();

        let x = (radius as f64) * phi.sin() * theta.cos();
        let y = (radius as f64) * phi.sin() * theta.sin();
        let z = (radius as f64) * phi.cos();

        let v_theta = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        let v_phi = (rng.gen_range(-1.0..1.0f64)).acos();
        let v_mag = 1.0 / (radius as f64).sqrt() * ORBITAL_VELOCITY_CONSTANT;

        let vx = v_mag * v_phi.sin() * v_theta.cos();
        let vy = v_mag * v_phi.sin() * v_theta.sin();
        let vz = v_mag * v_phi.cos();

        bodies.push(OrbitalBody::new(
            mass,
            Vec3::new(
                Fixed::from_raw((x * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((y * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((z * (1u128 << 64) as f64) as i128),
            ),
            Vec3::new(
                Fixed::from_raw((vx * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((vy * (1u128 << 64) as f64) as i128),
                Fixed::from_raw((vz * (1u128 << 64) as f64) as i128),
            ),
        ));
    }

    OrbitalConfig::new(
        bodies,
        use_steps,
        use_reseed,
        kelvin::DEFAULT_DT,
        kelvin::SOFTENING_FACTOR,
        kelvin::DEFAULT_G,
    )
    .map_err(|e| anyhow::anyhow!(e))
}
