#!/usr/bin/env bash
# ==============================================================================
# Kelvin -- Comparative Benchmarks (Unix Shell)
# ==============================================================================
# Runs the criterion-based comparative benchmarks against industry-standard
# cryptographic libraries (ring, dalek).
#
# Usage:
#   scripts/comparative-bench.sh                (run all benchmarks)
#   scripts/comparative-bench.sh quantum         (KelvinQuantum vs AES-256-GCM vs ChaCha20-Poly1305)
#   scripts/comparative-bench.sh streaming       (KelvinStreaming vs AES-256-CTR)
#   scripts/comparative-bench.sh keygen          (Orbital keygen vs X25519)
#   scripts/comparative-bench.sh signature       (ED25519 sign/verify)
#   scripts/comparative-bench.sh report          (open HTML report after running)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BENCH_DIR="$PROJECT_DIR/target/criterion"

cd "$PROJECT_DIR"

usage() {
    echo "Usage: $(basename "$0") [quantum|streaming|keygen|signature|report]"
    exit 1
}

case "${1:-all}" in
    all)
        echo "=============================================================================="
        echo "  Kelvin -- Comparative Benchmarks"
        echo "  Running all benchmarks..."
        echo "=============================================================================="
        echo ""
        cargo bench -p comparative-bench --bench throughput_quantum
        cargo bench -p comparative-bench --bench throughput_streaming
        cargo bench -p comparative-bench --bench keygen
        cargo bench -p comparative-bench --bench signature
        echo ""
        echo "=============================================================================="
        echo "  All benchmarks complete."
        echo "  HTML reports: $BENCH_DIR/reports/index.html"
        echo "=============================================================================="
        ;;
    quantum)
        cargo bench -p comparative-bench --bench throughput_quantum
        echo ""
        echo "  HTML report: $BENCH_DIR/throughput_quantum/report/index.html"
        ;;
    streaming)
        cargo bench -p comparative-bench --bench throughput_streaming
        echo ""
        echo "  HTML report: $BENCH_DIR/throughput_streaming/report/index.html"
        ;;
    keygen)
        cargo bench -p comparative-bench --bench keygen
        echo ""
        echo "  HTML report: $BENCH_DIR/keygen/report/index.html"
        ;;
    signature)
        cargo bench -p comparative-bench --bench signature
        echo ""
        echo "  HTML report: $BENCH_DIR/signature/report/index.html"
        ;;
    report)
        if [ -f "$BENCH_DIR/reports/index.html" ]; then
            echo "Opening HTML report..."
            xdg-open "$BENCH_DIR/reports/index.html" 2>/dev/null || \
                open "$BENCH_DIR/reports/index.html" 2>/dev/null || \
                echo "Report at: $BENCH_DIR/reports/index.html"
        else
            echo "No report found. Run benchmarks first:"
            echo "  $(basename "$0")"
            exit 1
        fi
        ;;
    *)
        usage
        ;;
esac