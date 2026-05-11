import json
import os
import subprocess
import math
from collections import Counter

# Configuration
NUM_KEYS = 1000
OUTPUT_DIR = "key_test_data"
LEVEL = "standard"

def run_keygen(index):
    output_path = os.path.join(OUTPUT_DIR, f"key_{index:04d}.json")
    # Using cargo run to ensure we use the latest code
    cmd = [
        "cargo", "run", "--release", "-p", "kelvin-cli", "--",
        "keygen", "--level", LEVEL, "--output", output_path
    ]
    subprocess.run(cmd, capture_output=True, check=True)
    return output_path

def analyze_keys():
    print(f"Analyzing {NUM_KEYS} keys...")
    
    sun_masses = []
    planet_masses = []
    initial_positions = [] # Store as tuples for uniqueness check
    
    for i in range(NUM_KEYS):
        path = os.path.join(OUTPUT_DIR, f"key_{i:04d}.json")
        with open(path, 'r') as f:
            data = json.load(f)
            
        bodies = data['bodies']
        
        # Sun analysis (body 0)
        sun_masses.append(bodies[0]['mass'])
        
        # Planet analysis (bodies 1-4)
        for p in range(1, 5):
            planet_masses.append(bodies[p]['mass'])
            pos = (bodies[p]['position']['x'], bodies[p]['position']['y'], bodies[p]['position']['z'])
            initial_positions.append(pos)

    # Statistics
    def stats(name, values):
        v_min = min(values)
        v_max = max(values)
        unique = len(set(values))
        print(f"\n--- {name} ---")
        print(f"  Range:  [{v_min}, {v_max}]")
        print(f"  Unique: {unique} / {len(values)} ({unique/len(values)*100:.2f}%)")
        
        # Estimate bits of entropy in the sample
        if unique > 1:
            entropy_sample = math.log2(unique)
            print(f"  Sample Entropy: ~{entropy_sample:.2f} bits")

    stats("Sun Mass", sun_masses)
    stats("Planet Masses", planet_masses)
    stats("Planet Initial Positions (Vectors)", initial_positions)

    # Check for collisions
    if len(set(sun_masses)) == NUM_KEYS:
        print("\n[PASS] No Sun Mass collisions found.")
    else:
        print("\n[WARN] Sun Mass collisions detected!")

if __name__ == "__main__":
    if not os.path.exists(OUTPUT_DIR):
        os.makedirs(OUTPUT_DIR)
    
    print(f"Generating {NUM_KEYS} keys in '{OUTPUT_DIR}'...")
    for i in range(NUM_KEYS):
        if i % 100 == 0:
            print(f"  Progress: {i}/{NUM_KEYS}")
        run_keygen(i)
    
    analyze_keys()
