/// Simple example: encrypt and decrypt a message using Kelvin.
///
/// Run with: cargo run --example simple_encrypt
use kelvin::{Kelvin, OrbitalConfig, Fixed, Vec3, OrbitalBody};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple 2-body orbital configuration
    let sun = OrbitalBody::new(
        Fixed::ONE, // 1 solar mass
        Vec3::ZERO,
        Vec3::ZERO,
    );
    let planet = OrbitalBody::new(
        Fixed::from_raw(1 << 54), // ~1e-6 solar masses
        Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
        Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
    );

    let config = OrbitalConfig::new(
        vec![sun, planet],
        10000,       // total steps
        1000,        // reseed interval
        Fixed::from_raw(1 << 44), // dt ≈ 1e-6
        Fixed::from_raw(1 << 44), // softening ≈ 1e-6
    )?;

    // Initialize Kelvin
    let mut k = Kelvin::new(config)?;

    // Encrypt a message
    let mut message = b"Hello, Kelvin! This is a secret message.".to_vec();
    println!("Original:  {:?}", std::str::from_utf8(&message)?);

    k.encrypt(&mut message)?;
    println!("Encrypted: {:02x?}", &message[..20]);

    k.decrypt(&mut message)?;
    println!("Decrypted: {:?}", std::str::from_utf8(&message)?);

    assert_eq!(&message, b"Hello, Kelvin! This is a secret message.");
    println!("\n✓ Round-trip encryption/decryption successful!");
    Ok(())
}
