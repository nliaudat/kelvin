# ==============================================================================
# Kelvin — Run All Tests (Windows PowerShell)
# ==============================================================================
# Runs the full test suite: build, lint, unit tests, integration tests.
# Exit code is 0 only if ALL steps pass.
#
# Usage:
#   .\scripts\test-all.ps1
# ==============================================================================

$ErrorActionPreference = "Stop"
$Global:LASTEXITCODE = $null

function Write-Step($msg) {
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  $msg" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

function Check-Exit {
    if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) {
        Write-Host "FAILED (exit code: $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "PASSED" -ForegroundColor Green
}

# ---------------------------------------------------------------------------
# 1. Build — verify all workspace members compile
# ---------------------------------------------------------------------------
Write-Step "1/8: Build workspace (default features)"
cargo build --workspace
Check-Exit

Write-Step "2/8: Build with AES-NI feature"
cargo build -p kelvin-stream --features aes-ni
Check-Exit
cargo build -p kelvin --features aes-ni
Check-Exit

# ---------------------------------------------------------------------------
# 2. Lint — clippy + rustfmt
# ---------------------------------------------------------------------------
Write-Step "3/8: Clippy (deny warnings)"
cargo clippy --workspace -- -D warnings
Check-Exit

Write-Step "4/8: Check formatting"
cargo fmt --check
Check-Exit

# ---------------------------------------------------------------------------
# 3. Unit tests
# ---------------------------------------------------------------------------
Write-Step "5/8: Unit tests — kelvin-core"
cargo test --lib -p kelvin-core
Check-Exit

Write-Step "6/8: Unit tests — kelvin-kdf"
cargo test --lib -p kelvin-kdf
Check-Exit

Write-Step "7/8: Unit tests — kelvin-stream (ChaCha + AES-GCM)"
cargo test --lib -p kelvin-stream
Check-Exit
cargo test --lib -p kelvin-stream --features aes-ni
Check-Exit

Write-Step "8/8: Unit tests — kelvin (top-level orchestrator)"
cargo test --lib -p kelvin
Check-Exit

# ---------------------------------------------------------------------------
# 4. Integration tests
# ---------------------------------------------------------------------------
Write-Step "Integration: Full pipeline"
cargo test -p kelvin --test full_pipeline
Check-Exit

Write-Step "Integration: Chaos test (Lyapunov estimation)"
cargo test -p kelvin-kdf --test chaos_test
Check-Exit

# ---------------------------------------------------------------------------
# All passed
# ---------------------------------------------------------------------------
Write-Host "`n========================================" -ForegroundColor Green
Write-Host "  ALL TESTS PASSED" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
exit 0
