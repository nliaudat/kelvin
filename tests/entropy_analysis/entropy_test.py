import json
import os
import subprocess
import math
import argparse
from collections import Counter

def run_keygen(index, output_dir, level):
    """Generates a single key using the Kelvin CLI."""
    output_path = os.path.join(output_dir, f"key_{index:05d}.json")
    cmd = [
        "cargo", "run", "--release", "-p", "kelvin-cli", "--",
        "keygen", "--level", level, "--output", output_path
    ]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path

def compute_shannon_entropy(data):
    """Compute Shannon entropy per byte of a byte sequence.
    
    H = -Σ p(x) × log₂(p(x)) for each byte value 0-255
    
    Returns entropy in bits per byte (max 8.0 for uniform distribution).
    """
    if len(data) == 0:
        return 0.0
    counter = Counter(data)
    total = len(data)
    entropy = 0.0
    for count in counter.values():
        p = count / total
        if p > 0:
            entropy -= p * math.log2(p)
    return entropy

def compute_correlation(data):
    """Compute Pearson correlation coefficient between adjacent bytes.
    
    For random data, r ≈ 0 (no correlation).
    Positive r means byte[i] tends to be similar to byte[i+1].
    Negative r means byte[i] tends to be opposite to byte[i+1].
    """
    if len(data) < 2:
        return 0.0
    n = len(data) - 1
    # Convert to floats
    x = [float(data[i]) for i in range(n)]
    y = [float(data[i + 1]) for i in range(n)]
    
    mean_x = sum(x) / n
    mean_y = sum(y) / n
    
    cov = sum((x[i] - mean_x) * (y[i] - mean_y) for i in range(n))
    var_x = sum((xi - mean_x) ** 2 for xi in x)
    var_y = sum((yi - mean_y) ** 2 for yi in y)
    
    if var_x == 0 or var_y == 0:
        return 0.0
    
    return cov / math.sqrt(var_x * var_y)

def analyze_entropy(output_dir, num_keys):
    """Performs statistical analysis on a set of generated keys."""
    print(f"\n{'='*60}")
    print(f" KELVIN ENTROPY ANALYSIS: {num_keys} KEYS")
    print(f"{'='*60}")
    
    sun_masses = []
    planet_masses = []
    positions = []
    velocities = []
    
    for i in range(num_keys):
        path = os.path.join(output_dir, f"key_{i:05d}.json")
        with open(path, 'r') as f:
            data = json.load(f)
            
        bodies = data['bodies']
        
        # Sun Analysis
        sun_masses.append(bodies[0]['mass'])
        
        # Planet Analysis
        for p in range(1, len(bodies)):
            planet_masses.append(bodies[p]['mass'])
            
            # Position Vector
            pos = bodies[p]['position']
            positions.append((pos['x'], pos['y'], pos['z']))
            
            # Velocity Vector
            vel = bodies[p]['velocity']
            velocities.append((vel['x'], vel['y'], vel['z']))

    def print_stat(label, values):
        unique_count = len(set(values))
        v_min = min(values) if not isinstance(values[0], tuple) else "N/A"
        v_max = max(values) if not isinstance(values[0], tuple) else "N/A"
        
        # Empirical Entropy: H = log2(unique_values)
        # This is the bits of entropy represented by the sample size
        empirical_bits = math.log2(unique_count) if unique_count > 0 else 0
        
        print(f"\n[ {label} ]")
        print(f"  Samples:    {len(values)}")
        print(f"  Unique:     {unique_count} ({unique_count/len(values)*100:.2f}%)")
        if v_min != "N/A":
            print(f"  Range:      {v_min} -> {v_max}")
        print(f"  Sample Entropy: ~{empirical_bits:.2f} bits")

    print_stat("Sun Mass", sun_masses)
    print_stat("Planet Masses", planet_masses)
    print_stat("Position Vectors (3D)", positions)
    print_stat("Velocity Vectors (3D)", velocities)

    # Collision Check
    duplicates = num_keys - len(set(sun_masses))
    print(f"\n{'='*60}")
    if duplicates == 0:
        print(" SUCCESS: No collisions detected in sun mass or configurations.")
    else:
        print(f" WARNING: {duplicates} collisions detected in sun mass!")
    print(f"{'='*60}\n")

def analyze_keystream():
    """Generate a keystream and analyze its Shannon entropy and correlation.
    
    This function generates raw keystream bytes by running the NIST test binary
    (which exports keystream.bin), then analyzes the raw binary data directly.
    """
    print(f"\n{'='*60}")
    print(" KEYSTREAM STATISTICAL ANALYSIS")
    print(f"{'='*60}")
    
    # Generate a 1MB keystream by running the NIST test binary
    # which exports keystream.bin with raw keystream bytes
    print("\nGenerating 1MB keystream via NIST test binary...")
    result = subprocess.run(
        ["cargo", "run", "--release", "-p", "nist_tests", "--"],
        capture_output=True, text=False
    )
    # Decode with error handling for Windows CP1252 compatibility
    stdout = result.stdout.decode('utf-8', errors='replace')
    stderr = result.stderr.decode('utf-8', errors='replace')
    print(stdout)
    if stderr.strip():
        print(stderr)
    
    # Read the raw keystream bytes from the exported file
    keystream_path = "keystream.bin"
    if os.path.exists(keystream_path):
        with open(keystream_path, 'rb') as f:
            keystream_bytes = f.read()
        
        print(f"\n  Keystream size: {len(keystream_bytes)} bytes")
        
        # Shannon Entropy of raw bytes
        shannon = compute_shannon_entropy(keystream_bytes)
        print(f"  Shannon Entropy: {shannon:.4f} bits/byte (max 8.0)")
        if shannon > 7.5:
            print("  [PASS] Near-maximal entropy (good randomness)")
        elif shannon > 6.0:
            print("  [WARN] Moderate entropy")
        else:
            print("  [FAIL] Low entropy")
        
        # Correlation Coefficient of raw bytes
        corr = compute_correlation(keystream_bytes)
        print(f"  Adjacent-byte Correlation: {corr:.6f} (expected ~0)")
        if abs(corr) < 0.01:
            print("  [PASS] No significant correlation detected")
        elif abs(corr) < 0.05:
            print("  [WARN] Weak correlation detected")
        else:
            print("  [FAIL] Strong correlation detected")
        
        # Byte value distribution analysis
        counter = Counter(keystream_bytes)
        min_count = min(counter.values())
        max_count = max(counter.values())
        expected = len(keystream_bytes) / 256
        print(f"  Byte value distribution: min={min_count}, max={max_count}, expected={expected:.0f}")
        if max_count - min_count < 2.0 * math.sqrt(expected):
            print("  [PASS] Byte distribution is uniform")
        else:
            print("  [WARN] Byte distribution shows some variation")
        
        # Cleanup
        os.remove(keystream_path)
    else:
        print("  [WARN] Could not generate keystream for analysis")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Kelvin Entropy Test Suite")
    parser.add_argument("--keys", type=int, default=100, help="Number of keys to generate (default: 100)")
    parser.add_argument("--level", type=str, default="standard", help="Security level (standard/paranoid/maximum)")
    parser.add_argument("--dir", type=str, default="test_results", help="Directory to store keys")
    parser.add_argument("--keystream", action="store_true", help="Run keystream Shannon entropy and correlation analysis")
    args = parser.parse_args()

    if not os.path.exists(args.dir):
        os.makedirs(args.dir)

    print(f"Starting generation of {args.keys} keys (Level: {args.level})...")
    for i in range(args.keys):
        if i > 0 and i % 50 == 0:
            print(f"  ... {i}/{args.keys} complete")
        run_keygen(i, args.dir, args.level)

    analyze_entropy(args.dir, args.keys)
    
    if args.keystream:
        analyze_keystream()
