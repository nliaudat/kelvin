# ==============================================================================
# Kelvin — Build Release Artifacts
# ==============================================================================
# Builds all binaries, copies them to kelvin-demo/, and prepares the HTML
# visualizer and test scripts for distribution.
#
# Usage:
#   .\scripts\build-release.ps1
#
# Outputs to:
#   kelvin-demo/
#     kelvin.exe              — CLI binary (Windows)
#     kelvin-cli.exe          — CLI binary (alias)
#     kelvin-test-client.exe  — Test client
#     kelvin-test-server.exe  — Test server
#     keygen_identify.exe     — Keygen + Identify demo
#     simple_encrypt.exe      — Simple encrypt/decrypt demo
#     simple_streaming.exe    — Streaming demo
#     orbital_visualizer.html — 3D orbital simulation visualizer
#     sample_key.json         — Sample orbital config
#     test-all.ps1            — Windows test script
#     test-all.sh             — Linux test script
#     README.txt              — Quick-start guide
# ==============================================================================

$ErrorActionPreference = "Stop"
$Global:LASTEXITCODE = $null

$ROOT = Resolve-Path (Join-Path $PSScriptRoot "..")
$DEMO_DIR = Join-Path $ROOT "kelvin-demo"
if (-not (Test-Path $DEMO_DIR)) {
    New-Item -ItemType Directory -Path $DEMO_DIR -Force | Out-Null
}
$TARGET = Join-Path $ROOT "target"
$TARGET = Join-Path $TARGET "release"

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
    Write-Host "OK" -ForegroundColor Green
}

# ---------------------------------------------------------------------------
# 1. Build all workspace members (release mode)
# ---------------------------------------------------------------------------
Write-Step "1/5: Building workspace (release)"
cargo build --workspace --release
Check-Exit

# ---------------------------------------------------------------------------
# 2. Copy binaries to kelvin-demo/
# ---------------------------------------------------------------------------
Write-Step "2/5: Copying binaries to kelvin-demo/"

# CLI binary
$cliBin = Join-Path $TARGET "kelvin.exe"
if (Test-Path $cliBin) {
    Copy-Item $cliBin (Join-Path $DEMO_DIR "kelvin.exe") -Force
    Copy-Item $cliBin (Join-Path $DEMO_DIR "kelvin-cli.exe") -Force
    Write-Host "  [OK] kelvin.exe"
}

# Test client/server
$tcBin = Join-Path $TARGET "kelvin-test-client.exe"
if (Test-Path $tcBin) {
    Copy-Item $tcBin (Join-Path $DEMO_DIR "kelvin-test-client.exe") -Force
    Write-Host "  [OK] kelvin-test-client.exe"
}
$tsBin = Join-Path $TARGET "kelvin-test-server.exe"
if (Test-Path $tsBin) {
    Copy-Item $tsBin (Join-Path $DEMO_DIR "kelvin-test-server.exe") -Force
    Write-Host "  [OK] kelvin-test-server.exe"
}

# Demo binaries
$DEMO_BINS = @("keygen_identify", "simple_encrypt", "simple_streaming")
foreach ($bin in $DEMO_BINS) {
    $src = Join-Path $TARGET "$bin.exe"
    if (Test-Path $src) {
        Copy-Item $src (Join-Path $DEMO_DIR "$bin.exe") -Force
        Write-Host "  [OK] $bin.exe"
    }
}

# ---------------------------------------------------------------------------
# 3. Copy HTML visualizer
# ---------------------------------------------------------------------------
Write-Step "3/5: Copying HTML visualizer"

$HTML_SRC = Join-Path $ROOT "examples"
$HTML_SRC = Join-Path $HTML_SRC "orbital_visualizer.html"
if (Test-Path $HTML_SRC) {
    Copy-Item $HTML_SRC (Join-Path $DEMO_DIR "orbital_visualizer.html") -Force
    Write-Host "  [OK] orbital_visualizer.html"
}

# ---------------------------------------------------------------------------
# 4. Copy test scripts and sample config
# ---------------------------------------------------------------------------
Write-Step "4/5: Copying test scripts and sample config"

# Test scripts
Copy-Item (Join-Path $PSScriptRoot "test-all.ps1") (Join-Path $DEMO_DIR "test-all.ps1") -Force
Write-Host "  [OK] test-all.ps1"
Copy-Item (Join-Path $PSScriptRoot "test-all.sh") (Join-Path $DEMO_DIR "test-all.sh") -Force
Write-Host "  [OK] test-all.sh"

# Sample config
$SAMPLE_JSON = Join-Path $ROOT "sample_key.json"
if (Test-Path $SAMPLE_JSON) {
    Copy-Item $SAMPLE_JSON (Join-Path $DEMO_DIR "sample_key.json") -Force
    Write-Host "  [OK] sample_key.json"
}

# ---------------------------------------------------------------------------
# 5. Write README
# ---------------------------------------------------------------------------
Write-Step "5/5: Writing README.txt"

$buildDate = Get-Date -Format "yyyy-MM-dd HH:mm"
$readmeContent = @"
========================================
  KELVIN CRYPTOSYSTEM - DEMO KIT
  Orbital Chaos KDF - Post-Quantum Cryptography
========================================

WHAT'S INCLUDED
---------------
  kelvin.exe              - CLI: keygen, encrypt, decrypt, identify
  kelvin-test-client.exe  - Integration test client
  kelvin-test-server.exe  - Integration test server
  keygen_identify.exe     - Key generation + identity demo
  simple_encrypt.exe      - Simple encrypt/decrypt demo
  simple_streaming.exe    - Streaming encrypt/decrypt demo
  orbital_visualizer.html - 3D orbital simulation (open in browser)
  sample_key.json         - Sample orbital configuration
  test-all.ps1            - Windows test suite
  test-all.sh             - Linux test suite

QUICK START
-----------
  1. Generate a key:
     .\kelvin.exe keygen --output mykey.json

  2. Identify public keys:
     .\kelvin.exe identify --config mykey.json --fast

  3. Encrypt a file:
     .\kelvin.exe encrypt --config mykey.json --input secret.txt --output secret.enc

  4. Decrypt a file:
     .\kelvin.exe decrypt --config mykey.json --input secret.enc --output secret.txt

  5. Open orbital_visualizer.html in a browser for 3D visualization.

  6. Run all tests:
     .\test-all.ps1

DEMO BINARIES
-------------
  keygen_identify.exe:
     Generates a random orbital config and derives hybrid PQ key pairs.
     Usage: .\keygen_identify.exe

  simple_encrypt.exe:
     Encrypts/decrypts a hardcoded message using V1 (ChaCha20).
     Usage: .\simple_encrypt.exe

  simple_streaming.exe:
     Demonstrates V2 streaming encrypt/decrypt.
     Usage: .\simple_streaming.exe

NOTES
-----
  - Use --fast with identify for quick key display (caps at 10K steps).
  - Without --fast, identify runs the full simulation (may be slow).
  - All binaries are 64-bit Windows executables.
  - Requires no external dependencies.

BUILD DATE: $buildDate
"@

Set-Content -Path (Join-Path $DEMO_DIR "README.txt") -Value $readmeContent
Write-Host "  [OK] README.txt"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
Write-Host "`n========================================" -ForegroundColor Green
Write-Host "  BUILD COMPLETE" -ForegroundColor Green
Write-Host "  Output: $DEMO_DIR" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

# List output files
Write-Host "`nOutput files:" -ForegroundColor Yellow
Get-ChildItem $DEMO_DIR | ForEach-Object {
    $fsize = "{0:N0}" -f $_.Length
    Write-Host ("  " + $_.Name + " (" + $fsize + " bytes)")
}
