#!/usr/bin/env bash
# ==============================================================================
# Kelvin — Run All Tests (Linux / macOS)
# ==============================================================================
# Runs the full test suite: build, lint, unit tests, integration tests.
# Exit code is 0 only if ALL steps pass.
#
# Usage:
#   ./scripts/test-all.sh
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
# 1. Build — verify all workspace members compile
# ---------------------------------------------------------------------------
step "1/8: Build workspace (default features)"
cargo build --workspace
pass

step "2/8: Build with AES-NI feature"
cargo build -p kelvin-stream --features aes-ni
cargo build -p kelvin --features aes-ni
pass

# ---------------------------------------------------------------------------
# 2. Lint — clippy + rustfmt
# ---------------------------------------------------------------------------
step "3/8: Clippy (deny warnings)"
cargo clippy --workspace -- -D warnings
pass

step "4/8: Check formatting"
cargo fmt --check
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

step "Integration: Chaos test (Lyapunov estimation)"
cargo test -p kelvin-kdf --test chaos_test
pass

# ---------------------------------------------------------------------------
# All passed
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  ALL TESTS PASSED${NC}"
echo -e "${GREEN}========================================${NC}"
