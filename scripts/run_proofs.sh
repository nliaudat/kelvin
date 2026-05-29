#!/usr/bin/env bash
# =============================================================================
# run_proofs.sh — Run all Kani formal verification proofs for Kelvin
#
# Runs all 56 proof harnesses across 7 proof files in kelvin-core.
# Requires Kani Rust Verifier: https://model-checking.github.io/kani/
#
# Usage:
#   ./scripts/run_proofs.sh           — Run all proofs (full verification)
#   ./scripts/run_proofs.sh --fast    — Run only fixed_equivalence proofs
#   ./scripts/run_proofs.sh --list    — List all proof harnesses
# =============================================================================
set -euo pipefail

KANI_ARGS="--enable-unstable --restrict-vtable"
PROOF_DIR="$(cd "$(dirname "$0")/../proofs/kani" && pwd)"

list_harnesses() {
    echo "==============================================================================="
    echo " KELVIN FORMAL VERIFICATION — Available Proof Harnesses"
    echo "==============================================================================="
    echo ""
    echo " pipeline_proofs.rs (3) — L3 Pipeline Integrity:"
    echo "   verify_pipeline_invariants"
    echo "   verify_extract_domain_sep_symbolic"
    echo "   verify_simulate_loop_equivalence"
    echo ""
    echo " fixed_equivalence.rs (5):"
    echo "   verify_fixed_add_commutative"
    echo "   verify_fixed_add_associative"
    echo "   verify_fixed_mul_commutative"
    echo "   verify_fixed_mul_associative"
    echo "   verify_fixed_sub_self_zero"
    echo ""
    echo " acceleration_proofs.rs (3):"
    echo "   verify_acceleration_single_body"
    echo "   verify_acceleration_two_body"
    echo "   verify_acceleration_newton_third"
    echo ""
    echo " vec3_proofs.rs (18):"
    echo "   verify_vec3_add_commutative"
    echo "   verify_vec3_add_associative"
    echo "   verify_vec3_add_identity"
    echo "   verify_vec3_sub_self_zero"
    echo "   verify_vec3_neg_add_self_zero"
    echo "   verify_vec3_dot_commutative"
    echo "   verify_vec3_dot_zero"
    echo "   verify_vec3_cross_anticommutative"
    echo "   verify_vec3_cross_self_zero"
    echo "   verify_vec3_cross_orthogonal_dot"
    echo "   verify_vec3_length_nonnegative"
    echo "   verify_vec3_length_squared_nonnegative"
    echo "   verify_vec3_length_squared_eq_dot"
    echo "   verify_vec3_scale_zero"
    echo "   verify_vec3_scale_one"
    echo "   verify_orbital_body_kinetic_energy_nonnegative"
    echo "   verify_orbital_body_zero_velocity_zero_ke"
    echo "   verify_orbital_body_momentum_proportional_to_velocity"
    echo ""
    echo " integrator_proofs.rs (9):"
    echo "   verify_compute_accelerations_single_body_zero"
    echo "   verify_compute_accelerations_two_body_equal_mass"
    echo "   verify_compute_accelerations_newton_third_law"
    echo "   verify_verlet_step_conserves_momentum"
    echo "   verify_verlet_step_single_body_no_change"
    echo "   verify_euler_step_conserves_momentum"
    echo "   verify_euler_step_single_body_no_change"
    echo "   verify_total_energy_negative_for_bound_system"
    echo "   verify_center_of_mass_two_equal_bodies"
    echo ""
    echo " stability_proofs.rs (6):"
    echo "   verify_gravitational_potential_negative"
    echo "   verify_gravitational_potential_zero_for_single_body"
    echo "   verify_is_body_ejected_no_false_positive_for_bound"
    echo "   verify_is_body_ejected_detects_escape_velocity"
    echo "   verify_detect_collapse_no_false_positive"
    echo "   verify_detect_collapse_detects_close_bodies"
    echo ""
    echo " extraction_proofs.rs (7):"
    echo "   verify_feed_orbital_state_empty_bodies"
    echo "   verify_feed_orbital_state_single_body"
    echo "   verify_feed_orbital_state_buffer_size"
    echo "   verify_extract_seed_length"
    echo "   verify_extract_seed_extended_length"
    echo "   verify_domain_separator_non_empty"
    echo "   verify_domain_separators_unique"
    echo ""
    echo " orbital_state_proofs.rs (8):"
    echo "   verify_chaotic_default_body_count"
    echo "   verify_step_counter_type"
    echo "   verify_verlet_step_no_panic_single_body"
    echo "   verify_euler_step_no_panic_single_body"
    echo "   verify_extract_entropy_output_size"
    echo "   verify_lyapunov_minimum_bodies"
    echo "   verify_lyapunov_positive_steps"
    echo "   verify_zeroize_clears_state"
    echo ""
    echo ""
    echo " Total: 59 harnesses (56 previous + 3 L3 pipeline proofs)"
}

run_fast() {
    echo "==============================================================================="
    echo " KELVIN FORMAL VERIFICATION — Fast mode (fixed_equivalence only)"
    echo "==============================================================================="
    echo ""
    cargo kani -p kelvin-core --harness verify_fixed_add_commutative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_add_associative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_mul_commutative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_mul_associative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_sub_self_zero $KANI_ARGS
    echo "  ✓ Fast proofs passed"
}

run_full() {
    echo "==============================================================================="
    echo " KELVIN FORMAL VERIFICATION — Full Proof Suite"
    echo " 56 harnesses across 7 files"
    echo "==============================================================================="
    echo ""

    echo "[1/7] fixed_equivalence.rs — Q32.64 arithmetic functional equivalence"
    cargo kani -p kelvin-core --harness verify_fixed_add_commutative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_add_associative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_mul_commutative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_mul_associative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_fixed_sub_self_zero $KANI_ARGS
    echo "  ✓ fixed_equivalence.rs passed"
    echo ""

    echo "[2/7] acceleration_proofs.rs — compute_accelerations composite proofs"
    cargo kani -p kelvin-core --harness verify_acceleration_single_body $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_acceleration_two_body $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_acceleration_newton_third $KANI_ARGS
    echo "  ✓ acceleration_proofs.rs passed"
    echo ""

    echo "[3/7] vec3_proofs.rs — Vec3 vector operation proofs (18 harnesses)"
    cargo kani -p kelvin-core --harness verify_vec3_add_commutative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_add_associative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_add_identity $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_sub_self_zero $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_neg_add_self_zero $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_dot_commutative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_dot_zero $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_cross_anticommutative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_cross_self_zero $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_cross_orthogonal_dot $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_length_nonnegative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_length_squared_nonnegative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_length_squared_eq_dot $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_scale_zero $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_vec3_scale_one $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_orbital_body_kinetic_energy_nonnegative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_orbital_body_zero_velocity_zero_ke $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_orbital_body_momentum_proportional_to_velocity $KANI_ARGS
    echo "  ✓ vec3_proofs.rs passed"
    echo ""

    echo "[4/7] integrator_proofs.rs — Verlet + Euler integrator proofs (9 harnesses)"
    cargo kani -p kelvin-core --harness verify_compute_accelerations_single_body_zero $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_compute_accelerations_two_body_equal_mass $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_compute_accelerations_newton_third_law $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_verlet_step_conserves_momentum $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_verlet_step_single_body_no_change $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_euler_step_conserves_momentum $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_euler_step_single_body_no_change $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_total_energy_negative_for_bound_system $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_center_of_mass_two_equal_bodies $KANI_ARGS
    echo "  ✓ integrator_proofs.rs passed"
    echo ""

    echo "[5/7] stability_proofs.rs — Ejection + collapse detection proofs (6 harnesses)"
    cargo kani -p kelvin-core --harness verify_gravitational_potential_negative $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_gravitational_potential_zero_for_single_body $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_is_body_ejected_no_false_positive_for_bound $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_is_body_ejected_detects_escape_velocity $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_detect_collapse_no_false_positive $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_detect_collapse_detects_close_bodies $KANI_ARGS
    echo "  ✓ stability_proofs.rs passed"
    echo ""

    echo "[6/7] extraction_proofs.rs — Entropy extraction safety proofs (7 harnesses)"
    cargo kani -p kelvin-core --harness verify_feed_orbital_state_empty_bodies $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_feed_orbital_state_single_body $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_feed_orbital_state_buffer_size $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_extract_seed_length $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_extract_seed_extended_length $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_domain_separator_non_empty $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_domain_separators_unique $KANI_ARGS
    echo "  ✓ extraction_proofs.rs passed"
    echo ""

    echo "[7/7] orbital_state_proofs.rs — OrbitalState safety proofs (8 harnesses)"
    cargo kani -p kelvin-core --harness verify_chaotic_default_body_count $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_step_counter_type $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_verlet_step_no_panic_single_body $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_euler_step_no_panic_single_body $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_extract_entropy_output_size $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_lyapunov_minimum_bodies $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_lyapunov_positive_steps $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_zeroize_clears_state $KANI_ARGS
    echo "  ✓ orbital_state_proofs.rs passed"
    echo ""

    echo "[8/8] pipeline_proofs.rs — L3 Pipeline Integrity proofs (3 harnesses)"
    cargo kani -p kelvin-core --harness verify_pipeline_invariants $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_extract_domain_sep_symbolic $KANI_ARGS
    cargo kani -p kelvin-core --harness verify_simulate_loop_equivalence $KANI_ARGS
    echo "  ✓ pipeline_proofs.rs passed"
    echo ""

    echo "==============================================================================="
    echo " ALL PROOFS PASSED — 59/59 harnesses verified"
    echo "==============================================================================="
}

# ── Main ──────────────────────────────────────────────────────────────────

case "${1:-}" in
    --list) list_harnesses ;;
    --fast) run_fast ;;
    *)      run_full ;;
esac
