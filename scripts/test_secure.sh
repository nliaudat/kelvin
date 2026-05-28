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
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

EXITCODE=0
FAILED_STEPS=()

step() {
    echo ""
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

run() {
    local name="$1"
    shift
    step "$name"
    if "$@"; then
        echo -e "${GREEN}PASSED${NC}"
    else
        echo -e "${RED}FAILED${NC}"
        EXITCODE=1
        FAILED_STEPS+=("$name")
    fi
}

# ---------------------------------------------------------------------------
# 1. Formatting check
# ---------------------------------------------------------------------------
run "1/9: Check formatting" cargo fmt --check

# ---------------------------------------------------------------------------
# 2. Unit tests (all workspace members)
# ---------------------------------------------------------------------------
run "2/9: Unit tests (all workspace members)" cargo test --release --lib --workspace
run "3/9: Unit tests - kelvin-stream (AES-NI feature)" cargo test --release --lib -p kelvin-stream --features aes-ni

# ---------------------------------------------------------------------------
# 3. Integration tests
# ---------------------------------------------------------------------------
run "4/9: Integration: Secure round-trip" cargo test --release -p kelvin --test v1_round_trip
run "5/9: Integration: Photon cipher" cargo test --release -p kelvin --test photon
run "6/9: Integration: Quantum cipher" cargo test --release -p kelvin --test quantum
run "7/9: Integration: Authenticated encryption" cargo test --release -p kelvin --test authenticated
run "8/9: Integration: Full pipeline" cargo test --release -p kelvin --test full_pipeline
run "9/9: Integration: Streaming API - Photon, Quantum, Chaos, Secure" cargo test --release -p kelvin --test streaming_api
run "Integration: Prism OTP key generator" cargo test --release -p kelvin --test prism
run "Integration: Flare FHE key generator" cargo test --release -p kelvin --test flare
run "Integration: Split secret sharing" cargo test --release -p kelvin --test split
run "Integration: Determinism (cross-platform golden hash)" cargo test --release -p kelvin-core --test determinism
run "Integration: Chaos test (Lyapunov estimation)" cargo test --release -p kelvin-kdf --test chaos_test
run "Integration: Client/Server self-test - Secure + Chaos Streaming" cargo run --release -p kelvin-test-client

# ---------------------------------------------------------------------------
# 4. Entropy analysis & statistical tests
# ---------------------------------------------------------------------------
run "Integration: Entropy analysis (SP 800-90B health tests)" cargo run --release -p entropy_analysis -- --keystream
run "Integration: NIST SP 800-22 statistical tests (all 6 variants)" cargo run --release -p nist_tests
run "Integration: NIST SP 800-90B keystream generation + analysis" bash -c "cargo run --release -p nist_800_90b -- generate --size 1048576 --output target/keystream_90b.bin && cargo run --release -p nist_800_90b -- analyze --input target/keystream_90b.bin"
run "Integration: NIST SP 800-90B Prism keystream generation + analysis" bash -c "cargo run --release -p nist_800_90b -- generate --size 1048576 --output target/keystream_90b_prism.bin --prism && cargo run --release -p nist_800_90b -- analyze --input target/keystream_90b_prism.bin"
run "Integration: NIST SP 800-90B non-IID entropy estimation (dj-on-github)" bash -c "python3 tests/sp800_90b_non_iid/sp800_90b_tests.py -t mcv target/keystream_90b.bin -s 10000 && python3 tests/sp800_90b_non_iid/sp800_90b_tests.py -t ttuple target/keystream_90b.bin -s 10000"

rm -f target/keystream_90b.bin target/keystream_90b_prism.bin

# ---------------------------------------------------------------------------
# 5. Constant-time benchmarks
# ---------------------------------------------------------------------------
run "Integration: Constant-time benchmarks (DudeCT)" cargo run --release -p constant_time_bench

# ---------------------------------------------------------------------------
# 6. Zeroize verification tests
# ---------------------------------------------------------------------------
run "Integration: Zeroize verification tests" cargo test --release -p zeroize_verify

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
if [ "$EXITCODE" -eq 0 ]; then
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  ALL SECURITY TESTS PASSED${NC}"
    echo -e "${GREEN}========================================${NC}"
    exit 0
else
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}  SOME SECURITY TESTS FAILED${NC}"
    echo -e "${RED}========================================${NC}"
    echo ""
    echo -e "${YELLOW}Failed steps:${NC}"
    for failed in "${FAILED_STEPS[@]}"; do
        echo "  - $failed"
    done
    echo ""
    exit 1
fi
