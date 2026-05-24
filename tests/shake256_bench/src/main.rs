/// SHAKE256 throughput benchmark — with and without asm acceleration.
///
/// Measures MB/s for producing keystream from a seeded SHAKE256 XOF,
/// using the same patterns as Kelvin Photon and Quantum modes.
///
/// Run with:
///     cargo run -p shake256-bench --release
///
/// To test with asm acceleration, edit this Cargo.toml to add features = ["asm"]:
///     sha3 = { version = "0.10", features = ["asm"] }
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::Shake256;

fn main() {
    let chunk_size = 1024 * 1024; // 1 MiB chunks
    let chunks = 256; // 256 MiB total (faster to run, still representative)
    let total_mib = chunks;

    // ── Test 1: Clone-per-chunk pattern ──
    println!("=== Test 1: Clone-per-chunk (re-absorb seed each time) ===");
    println!("  Producing {} MiB in {} × 1 MiB chunks...", total_mib, chunks);

    let mut hasher = Shake256::default();
    hasher.update(&[42u8; 32]);

    let start = std::time::Instant::now();
    let mut output = vec![0u8; chunk_size];
    for _ in 0..chunks {
        let mut reader = hasher.clone().finalize_xof();
        reader.read(&mut output);
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!("  Total: {} MiB in {:.3}s", total_mib, elapsed);
    println!("  Throughput: {:.1} MB/s", total_mib as f64 / elapsed);

    // ── Test 2: Persistent reader (no clone) — matches Photon/Quantum ──
    println!();
    println!("=== Test 2: Persistent reader (no clone, matches Photon/Quantum) ===");
    println!("  Producing {} MiB in {} × 1 MiB chunks...", total_mib, chunks);

    let mut hasher = Shake256::default();
    hasher.update(&[42u8; 32]);
    let mut reader = hasher.finalize_xof();

    let start = std::time::Instant::now();
    for _ in 0..chunks {
        XofReader::read(&mut reader, &mut output);
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!("  Total: {} MiB in {:.3}s", total_mib, elapsed);
    println!("  Throughput: {:.1} MB/s", total_mib as f64 / elapsed);

    // ── Test 3: Persistent reader + XOR (matches actual encrypt/decrypt) ──
    println!();
    println!("=== Test 3: Persistent reader + XOR (matches actual encrypt/decrypt) ===");
    println!("  Producing {} MiB in {} × 1 MiB chunks...", total_mib, chunks);

    let mut hasher = Shake256::default();
    hasher.update(&[42u8; 32]);
    let mut reader = hasher.finalize_xof();
    let mut data = vec![0xABu8; chunk_size];

    let start = std::time::Instant::now();
    for _ in 0..chunks {
        XofReader::read(&mut reader, &mut output);
        for (d, k) in data.iter_mut().zip(output.iter()) {
            *d ^= k;
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!("  Total: {} MiB in {:.3}s", total_mib, elapsed);
    println!("  Throughput: {:.1} MB/s", total_mib as f64 / elapsed);

    // ── Summary ──
    println!();
    println!("=== Summary ===");
    println!("  Build: pure Rust (no asm)");
    println!("  CPU: AMD Ryzen 5 5600");
}
