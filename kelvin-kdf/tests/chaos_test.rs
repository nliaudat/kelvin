use kelvin_core::{Fixed, OrbitalBody, Vec3, simulate};
use kelvin_kdf::extract_seed;

fn test_system() -> Vec<OrbitalBody> {
    let sun = OrbitalBody::new(
        Fixed::ONE,
        Vec3::ZERO,
        Vec3::ZERO,
    );
    let planet1 = OrbitalBody::new(
        Fixed::from_raw(1 << 54), // small mass
        Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
        Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
    );
    let planet2 = OrbitalBody::new(
        Fixed::from_raw(1 << 53),
        Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
        Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
    );
    let planet3 = OrbitalBody::new(
        Fixed::from_raw(1 << 52),
        Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
        Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
    );
    let planet4 = OrbitalBody::new(
        Fixed::from_raw(1 << 51),
        Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
        Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
    );
    vec![sun, planet1, planet2, planet3, planet4]
}

/// Compute Hamming distance between two byte slices.
fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

#[test]
fn test_chaos_avalanche() {
    let mut bodies_orig = test_system();
    let mut bodies_pert = test_system();

    // Perturb the position of the first planet by ~10^-10 AU
    // 1.0 = 2^64, so 10^-10 ≈ 2^30.8 ≈ 1 << 31
    bodies_pert[1].position.x += Fixed::from_raw(1 << 31);

    let dt = kelvin_core::DEFAULT_DT;
    let softening = Fixed::from_raw(1 << 44); // Some softening
    let steps = 10_000;

    // Simulate both systems
    simulate(&mut bodies_orig, steps, dt, softening, kelvin_core::DEFAULT_G);
    simulate(&mut bodies_pert, steps, dt, softening, kelvin_core::DEFAULT_G);

    // Extract seeds
    let seed_orig = extract_seed(&bodies_orig, steps, b"avalanche-test");
    let seed_pert = extract_seed(&bodies_pert, steps, b"avalanche-test");

    // Calculate Hamming distance
    let dist = hamming_distance(&seed_orig, &seed_pert);

    // Seed is 64 bytes = 512 bits. Expected distance is ~256.
    // Standard deviation is sqrt(512 * 0.5 * 0.5) ≈ 11.3
    // We check within 4 standard deviations (~45 bits) -> 211 to 301
    assert!(
        dist >= 210 && dist <= 302,
        "Avalanche failed: expected ~256 flipped bits, got {}",
        dist
    );
}

#[test]
fn test_chaos_uniformity() {
    let mut ones_count = 0;
    let num_tests = 100;
    let dt = Fixed::from_raw(1 << 44);
    let softening = Fixed::from_raw(1 << 44);

    let mut bodies = test_system();
    let steps_per_run = 1000;

    for i in 0..num_tests {
        simulate(&mut bodies, steps_per_run, dt, softening, kelvin_core::DEFAULT_G);
        let seed = extract_seed(&bodies, (i + 1) * steps_per_run, b"uniformity-test");
        
        for byte in seed.iter() {
            ones_count += byte.count_ones();
        }
    }

    // 100 tests * 64 bytes * 8 bits = 51,200 bits
    // Expected ones: 25,600. Std dev: sqrt(51200 * 0.5 * 0.5) ≈ 113
    // We check within 4 standard deviations (~452)
    let total_bits = num_tests * 64 * 8;
    let expected = total_bits / 2;
    let tolerance = 500;

    assert!(
        (ones_count as u64) >= expected - tolerance && (ones_count as u64) <= expected + tolerance,
        "Uniformity failed: expected ~{} ones, got {}",
        expected,
        ones_count
    );
}
