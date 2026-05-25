#!/usr/bin/env bash
# ==============================================================================
# Kelvin — Run All Tests (Linux / macOS)
# ==============================================================================
# Runs the full test suite: build, lint, unit tests, integration tests,
# entropy analysis, NIST SP 800-22 statistical tests, and constant-time
# benchmarks.
#
# Exit code is 0 only if ALL steps pass.
#
# Usage:
#   ./scripts/test-all.sh
# ==============================================================================

set -euo pipefail

# Initialize git submodules (NIST SP 800-90B tools)
if [ -f .gitmodules ]; then
    git submodule update --init --recursive
fi

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
# 1. Build — verify all workspace members compile
# ---------------------------------------------------------------------------
step "1/8: Build workspace (default features)"
cargo build --workspace
pass

step "2/8: Build with AES-NI feature"
cargo build -p kelvin-stream --features aes-ni
cargo build -p kelvin --features aes-ni
pass

step "2b/8: Build V2 Streaming example"
cargo build --example simple_streaming -p kelvin
pass

# ---------------------------------------------------------------------------
# 2. Lint — clippy + rustfmt
# ---------------------------------------------------------------------------
step "3/8: Clippy (deny warnings)"
cargo clippy --workspace -- -D warnings
pass

step "4/8: Check formatting"
cargo fmt --all --check
pass

# ---------------------------------------------------------------------------
# 3. Unit tests
# ---------------------------------------------------------------------------
step "5/8: Unit tests — kelvin-core"
cargo test --lib -p kelvin-core
pass

step "6/8: Unit tests — kelvin-kdf"
cargo test --lib -p kelvin-kdf
pass

step "7/8: Unit tests — kelvin-stream (ChaCha + AES-GCM)"
cargo test --lib -p kelvin-stream
cargo test --lib -p kelvin-stream --features aes-ni
pass

step "8/8: Unit tests — kelvin (top-level orchestrator)"
cargo test --lib -p kelvin
pass

# ---------------------------------------------------------------------------
# 4. Integration tests
# ---------------------------------------------------------------------------
step "Integration: Full pipeline"
cargo test -p kelvin --test full_pipeline
pass

step "Integration: Streaming API (Photon, Quantum, Chaos, Secure)"
cargo test -p kelvin --test streaming_api
pass

step "Integration: Chaos test (Lyapunov estimation)"
cargo test -p kelvin-kdf --test chaos_test
pass

step "Integration: Client/Server self-test (V1 + V2 Streaming)"
cargo run -p kelvin-test-client
pass

# ---------------------------------------------------------------------------
# 5. Entropy analysis & statistical tests
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

step "Integration: NIST SP 800-90B non-IID entropy estimation (dj-on-github)"
python3 tests/sp800_90b_non_iid/sp800_90b_tests.py -t mcv target/keystream_90b.bin -s 10000
python3 tests/sp800_90b_non_iid/sp800_90b_tests.py -t ttuple target/keystream_90b.bin -s 10000

rm -f target/keystream_90b.bin
pass

# ---------------------------------------------------------------------------
# 6. Constant-time benchmarks
# ---------------------------------------------------------------------------
step "Integration: Constant-time benchmarks (DudeCT)"
cargo run --release -p constant_time_bench
pass

# ---------------------------------------------------------------------------
# All passed
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  ALL TESTS PASSED${NC}"
echo -e "${GREEN}========================================${NC}"
