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

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Kelvin Entropy Test Suite")
    parser.add_argument("--keys", type=int, default=100, help="Number of keys to generate (default: 100)")
    parser.add_argument("--level", type=str, default="standard", help="Security level (standard/paranoid/maximum)")
    parser.add_argument("--dir", type=str, default="test_results", help="Directory to store keys")
    args = parser.parse_args()

    if not os.path.exists(args.dir):
        os.makedirs(args.dir)

    print(f"Starting generation of {args.keys} keys (Level: {args.level})...")
    for i in range(args.keys):
        if i > 0 and i % 50 == 0:
            print(f"  ... {i}/{args.keys} complete")
        run_keygen(i, args.dir, args.level)

    analyze_entropy(args.dir, args.keys)
