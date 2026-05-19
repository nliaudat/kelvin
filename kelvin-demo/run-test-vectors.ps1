# ==============================================================================
# Kelvin — Test Vector Generation & Verification (Standalone)
# ==============================================================================
# Uses pre-built binaries (kelvin-test-server.exe / kelvin-test-client.exe)
# to generate golden test vectors and verify cross-platform determinism.
#
# Usage:
#   .\run-test-vectors.ps1
#
# Steps:
#   1. Generate test vectors using kelvin-test-server.exe
#   2. Verify test vectors using kelvin-test-client.exe
#   3. Run built-in self-test (no external files needed)
# ==============================================================================

$ErrorActionPreference = "Stop"

$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$SERVER = Join-Path $SCRIPT_DIR "kelvin-test-server.exe"
$CLIENT = Join-Path $SCRIPT_DIR "kelvin-test-client.exe"
$VECTORS_DIR = Join-Path $SCRIPT_DIR "test-vectors"

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

# Check binaries exist
if (-not (Test-Path $SERVER)) {
    Write-Host "ERROR: kelvin-test-server.exe not found in $SCRIPT_DIR" -ForegroundColor Red
    Write-Host "Run .\scripts\build-release.ps1 first to build all binaries." -ForegroundColor Yellow
    exit 1
}
if (-not (Test-Path $CLIENT)) {
    Write-Host "ERROR: kelvin-test-client.exe not found in $SCRIPT_DIR" -ForegroundColor Red
    Write-Host "Run .\scripts\build-release.ps1 first to build all binaries." -ForegroundColor Yellow
    exit 1
}

# ---------------------------------------------------------------------------
# 1. Generate test vectors
# ---------------------------------------------------------------------------
Write-Step "1/3: Generating test vectors"
if (Test-Path $VECTORS_DIR) {
    Remove-Item -Recurse -Force $VECTORS_DIR
}
New-Item -ItemType Directory -Path $VECTORS_DIR -Force | Out-Null

& $SERVER --output $VECTORS_DIR
Check-Exit

# ---------------------------------------------------------------------------
# 2. Verify test vectors
# ---------------------------------------------------------------------------
Write-Step "2/3: Verifying test vectors"
& $CLIENT --vectors $VECTORS_DIR
Check-Exit

# ---------------------------------------------------------------------------
# 3. Run built-in self-test
# ---------------------------------------------------------------------------
Write-Step "3/3: Running built-in self-test"
& $CLIENT
Check-Exit

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
Write-Host "`n========================================" -ForegroundColor Green
Write-Host "  ALL TEST VECTOR CHECKS PASSED" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "`nTest vectors saved to: $VECTORS_DIR" -ForegroundColor Yellow
Write-Host "`nTo verify on another platform, copy the test-vectors/ directory" -ForegroundColor Yellow
Write-Host "and run: kelvin-test-client --vectors <path-to-test-vectors>" -ForegroundColor Yellow
