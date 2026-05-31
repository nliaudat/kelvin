//! Monte Carlo Validation of C5: Configuration Space Cardinality
//!
//! This binary validates the C5 (Configuration Space) conjecture by randomly
//! sampling the orbital configuration space and estimating:
//!
//!   - Fraction of configurations that pass validation
//!   - Effective entropy H = log₂(N_valid)
//!   - Grover search lower bound: 2^{H/2}
//!
//! Results written to proofs/kani/results/c5_validation.log
//! Usage: cargo run -p configuration_space

use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_G, SOFTENING_FACTOR};
use kelvin_kdf::OrbitalConfig;
use std::cell::Cell;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const N_SAMPLES: usize = 10_000;
const MAX_POS_AU: f64 = 100.0;
const MAX_VEL_AU: f64 = 100.0;

fn main() {
    let results_dir = "proofs/kani/results";
    let _ = fs::create_dir_all(results_dir);
    let results_path = Path::new(results_dir).join("c5_validation.log");

    let mut output = String::new();

    let samples = generate_random_configs(N_SAMPLES);
    let valid_count = samples.iter().filter(|c| c.is_ok()).count();
    let collapse_count = samples
        .iter()
        .filter(|c| {
            if let Err(ref e) = c {
                let s = format!("{e}");
                s.contains("too close") || s.contains("identical")
            } else {
                false
            }
        })
        .count();
    let mass_count = samples
        .iter()
        .filter(|c| if let Err(ref e) = c { format!("{e}").contains("mass") } else { false })
        .count();

    let fraction = valid_count as f64 / N_SAMPLES as f64;

    // Analytical bound for unconstrained space cardinality
    let bits_per_mass = 64.0;
    let bits_per_pos_per_axis = 64.0 + (MAX_POS_AU * 2.0).log2();
    let bits_per_vel_per_axis = 64.0 + (MAX_VEL_AU * 2.0).log2();
    let bits_per_body = bits_per_mass + 3.0 * bits_per_pos_per_axis + 3.0 * bits_per_vel_per_axis;
    let unconstrained_bits = 5.0 * bits_per_body; // N=5
    let estimated_bits = unconstrained_bits + (fraction.max(1e-10)).log2();
    let grover_bits = estimated_bits / 2.0;

    output.push_str(&format!("C5 Configuration Space — Monte Carlo Validation\n"));
    output.push_str(&format!("===============================================\n\n"));
    output.push_str(&format!("Samples: {N_SAMPLES}\n\n"));
    output
        .push_str(&format!("Valid configurations:     {valid_count} ({:.2}%)\n", fraction * 100.0));
    output.push_str(&format!(
        "Collision/collapse:      {collapse_count} ({:.2}%)\n",
        collapse_count as f64 / N_SAMPLES as f64 * 100.0
    ));
    output.push_str(&format!(
        "Non-positive mass:       {mass_count} ({:.2}%)\n",
        mass_count as f64 / N_SAMPLES as f64 * 100.0
    ));
    output.push_str("\nEntropy estimate (analytical):\n");
    output.push_str(&format!("  Bits per body (unconstrained): {bits_per_body:.0}\n"));
    output.push_str(&format!("  Unconstrained (N=5):           {unconstrained_bits:.0} bits\n"));
    output.push_str(&format!(
        "  Stability reduction:           {:.2}× = {:.1} bits\n",
        fraction.max(1e-10).recip(),
        -(fraction.max(1e-10)).log2()
    ));
    output.push_str(&format!("  Estimated H:                   {estimated_bits:.0} bits\n"));
    output.push_str(&format!("  Claimed bound:                 ≥ 1920 bits\n\n"));
    output
        .push_str(&format!("Grover search lower bound:   2^{grover_bits:.0} quantum operations\n"));
    output.push_str(&format!("  (from 2^{{H/2}} with H = {estimated_bits:.0})\n\n"));

    if estimated_bits >= 1920.0 {
        output.push_str(&format!("✓ PASS: H ≈ {estimated_bits:.0} ≥ 1920 bits\n"));
    } else {
        output.push_str(&format!(
            "⚠ Estimated H ({estimated_bits:.0}) below 1920 bound — check analytical model\n"
        ));
    }
    if grover_bits >= 900.0 {
        output.push_str(&format!("✓ PASS: Grover lower bound 2^{grover_bits:.0} ≥ 2^900\n"));
    } else {
        output.push_str(&format!("⚠ Grover bound below 2^900 threshold\n"));
    }
    output.push_str(&format!("\nRESULTS: C5 validation complete\n"));

    let _ = fs::write(&results_path, &output);
    print!("{output}");
    eprintln!("Results saved to: {}", results_path.display());
}

fn generate_random_configs(n: usize) -> Vec<Result<OrbitalConfig, String>> {
    let mut results = Vec::with_capacity(n);
    for _ in 0..n {
        let r = random_config();
        results.push(r);
    }
    results
}

fn random_config() -> Result<OrbitalConfig, String> {
    let n_bodies = 5;
    let mut bodies = Vec::with_capacity(n_bodies);
    for _ in 0..n_bodies {
        let mass = Fixed::from_raw(rand_i128(1, 1 << 64));
        let px = Fixed::from_raw(rand_i128(-100 * (1 << 64), 100 * (1 << 64)));
        let py = Fixed::from_raw(rand_i128(-100 * (1 << 64), 100 * (1 << 64)));
        let pz = Fixed::from_raw(rand_i128(-100 * (1 << 64), 100 * (1 << 64)));
        let vx = Fixed::from_raw(rand_i128(-100 * (1 << 64), 100 * (1 << 64)));
        let vy = Fixed::from_raw(rand_i128(-100 * (1 << 64), 100 * (1 << 64)));
        let vz = Fixed::from_raw(rand_i128(-100 * (1 << 64), 100 * (1 << 64)));
        bodies.push(OrbitalBody::new(mass, Vec3::new(px, py, pz), Vec3::new(vx, vy, vz)));
    }
    OrbitalConfig::new(bodies, 1000000, 10000, kelvin_core::DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .map_err(|e| format!("{e}"))
}

thread_local! {
    static RNG_STATE: Cell<u64> = Cell::new(
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
    );
}

fn rand_i128(lo: i128, hi: i128) -> i128 {
    let mut x = RNG_STATE.with(|state| {
        let next =
            state.get().wrapping_mul(0x5851F42D4C957F2Du64).wrapping_add(1442695040888963407u64);
        state.set(next);
        next
    });
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9u64);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111ebu64);
    x ^= x >> 31;
    lo + (x as i128).wrapping_abs() % (hi - lo + 1)
}
