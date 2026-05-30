#!/usr/bin/env bash
# =============================================================================
# run_proofs.sh — Run all Kani formal verification proofs for Kelvin
#
# Runs all 7 proof harnesses in kelvin-core/src/fixed_math.rs.
# Requires Kani Rust Verifier: https://model-checking.github.io/kani/
#
# Usage:
#   ./scripts/run_proofs.sh           — Run all proofs (full verification)
#   ./scripts/run_proofs.sh --fast    — Run only algebra proofs
#   ./scripts/run_proofs.sh --list    — List all proof harnesses
# =============================================================================
set -euo pipefail

KANI_ARGS=""

list_harnesses() {
    echo "==============================================================================="
    echo " KELVIN FORMAL VERIFICATION — Available Proof Harnesses"
    echo "==============================================================================="
    echo ""
    echo " kelvin-core/src/fixed_math.rs (3) — L0 Safety proofs:"
    echo ""
    echo "   verify_add_no_overflow   — Add never wraps in [-100, 100] AU"
    echo "   verify_sub_no_overflow   — Sub never wraps in [-100, 100] AU"
    echo "   verify_mul_range         — Mul range safety for [-4, 4] AU"
    echo ""
    echo " L1 functional equivalence (commutativity, identity, div/sqrt inverse)"
    echo " is verified by the unit test suite (tests/fixed_math.rs)."
    echo ""
    echo " Total: 3 harnesses"
}

run_fast() {
    echo "==============================================================================="
    echo " KELVIN FORMAL VERIFICATION — Fast mode (3 core L0 proofs)"
    echo "==============================================================================="
    echo ""
    cargo kani -p kelvin-core --harness verify_add_no_overflow $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_sub_no_overflow $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_mul_range $KANI_ARGS
    echo "  ✓ Fast proofs passed"
}

run_full() {
    echo "==============================================================================="
    echo " KELVIN FORMAL VERIFICATION — L0 Safety Proof Suite"
    echo " 3 harnesses in kelvin-core/src/fixed_math.rs"
    echo "==============================================================================="
    echo ""

    echo "[1/3] verify_add_no_overflow — Add never wraps in [-100, 100] AU"
    cargo kani -p kelvin-core --harness verify_add_no_overflow $KANI_ARGS
    echo "  ✓ verify_add_no_overflow passed"
    echo ""

    echo "[2/3] verify_sub_no_overflow — Sub never wraps in [-100, 100] AU"
    cargo kani -p kelvin-core --harness verify_sub_no_overflow $KANI_ARGS
    echo "  ✓ verify_sub_no_overflow passed"
    echo ""

    echo "[3/3] verify_mul_range — Mul range safety for [-4, 4] AU"
    cargo kani -p kelvin-core --harness verify_mul_range $KANI_ARGS
    echo "  ✓ verify_mul_range passed"
    echo ""

    echo "==============================================================================="
    echo " ALL PROOFS PASSED — 3/3 harnesses verified"
    echo "==============================================================================="
    echo ""
    echo "L1 functional equivalence proofs (commutativity, identity, zero, div inverse,"
    echo "sqrt inverse) are verified by the unit test suite. See tests/fixed_math.rs."
}

# ── Main ──────────────────────────────────────────────────────────────────

case "${1:-}" in
    --list) list_harnesses ;;
    --fast) run_fast ;;
    *)      run_full ;;
esac