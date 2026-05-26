#!/usr/bin/env bash
# ==============================================================================
# Kelvin — Security & Correctness Tests (Linux / macOS)
# ==============================================================================
# Runs unit tests, integration tests, entropy analysis, NIST SP 800-22
# statistical tests, and constant-time benchmarks.
#
# Exit code is 0 only if ALL steps pass.
#
# Usage:
#   ./scripts/test_secure.sh
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

step() {
    echo ""
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

pass() {
    echo -e "${GREEN}PASSED${NC}"
}

# ---------------------------------------------------------------------------
# 1. Formatting check
# ---------------------------------------------------------------------------
step "1/9: Check formatting"
cargo fmt --check
pass

# ---------------------------------------------------------------------------
# 2. Unit tests (all workspace members)
# ---------------------------------------------------------------------------
step "2/9: Unit tests (all workspace members)"
cargo test --release --lib --workspace
pass

step "3/9: Unit tests - kelvin-stream (AES-NI feature)"
cargo test --release --lib -p kelvin-stream --features aes-ni
pass

# ---------------------------------------------------------------------------
# 3. Integration tests
# ---------------------------------------------------------------------------
step "4/9: Integration: V1 round-trip"
cargo test --release -p kelvin --test v1_round_trip
pass

step "5/9: Integration: Photon cipher"
cargo test --release -p kelvin --test photon
pass

step "6/9: Integration: Quantum cipher"
cargo test --release -p kelvin --test quantum
pass

step "7/9: Integration: Authenticated encryption"
cargo test --release -p kelvin --test authenticated
pass

step "8/9: Integration: Full pipeline"
cargo test --release -p kelvin --test full_pipeline
pass

step "9/9: Integration: Streaming API (Photon, Quantum, Chaos, Secure)"
cargo test --release -p kelvin --test streaming_api
pass

step "Integration: Chaos test (Lyapunov estimation)"
cargo test --release -p kelvin-kdf --test chaos_test
pass

step "Integration: Client/Server self-test (V1 + V2 Streaming)"
cargo run --release -p kelvin-test-client
pass

# ---------------------------------------------------------------------------
# 4. Entropy analysis & statistical tests
# ---------------------------------------------------------------------------
step "Integration: Entropy analysis (SP 800-90B health tests)"
cargo run --release -p entropy_analysis -- --keystream
pass

step "Integration: NIST SP 800-22 statistical tests (all 6 variants)"
cargo run --release -p nist_tests
pass

step "Integration: NIST SP 800-90B keystream generation + analysis"
cargo run --release -p nist_800_90b -- generate --size 1048576 --output target/keystream_90b.bin
cargo run --release -p nist_800_90b -- analyze --input target/keystream_90b.bin

step "Integration: NIST SP 800-90B Prism keystream generation + analysis"
cargo run --release -p nist_800_90b -- generate --size 1048576 --output target/keystream_90b_prism.bin --prism
cargo run --release -p nist_800_90b -- analyze --input target/keystream_90b_prism.bin

step "Integration: NIST SP 800-90B non-IID entropy estimation (dj-on-github)"
python3 tests/sp800_90b_non_iid/sp800_90b_tests.py -t mcv target/keystream_90b.bin -s 10000
python3 tests/sp800_90b_non_iid/sp800_90b_tests.py -t ttuple target/keystream_90b.bin -s 10000

rm -f target/keystream_90b.bin target/keystream_90b_prism.bin
pass

# ---------------------------------------------------------------------------
# 5. Constant-time benchmarks
# ---------------------------------------------------------------------------
step "Integration: Constant-time benchmarks (DudeCT)"
cargo run --release -p constant_time_bench
pass

# ---------------------------------------------------------------------------
# All passed
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  ALL SECURITY TESTS PASSED${NC}"
echo -e "${GREEN}========================================${NC}"
