@echo off
REM ==============================================================================
REM Kelvin -- Security & Correctness Tests (Windows Batch)
REM ==============================================================================
REM Runs unit tests, integration tests, entropy analysis, NIST SP 800-22
REM statistical tests, and constant-time benchmarks.
REM
REM Exit code is 0 only if ALL steps pass.
REM
REM Usage:
REM   scripts\test_secure.bat
REM ==============================================================================

setlocal enabledelayedexpansion

set RED=[91m
set GREEN=[92m
set CYAN=[96m
set YELLOW=[93m
set NC=[0m

set EXITCODE=0
set FAILED_STEPS=

REM Change to workspace root (parent of scripts/)
cd /d "%~dp0.."

REM ---------------------------------------------------------------------------
REM 1. Formatting check
REM ---------------------------------------------------------------------------
set STEP_NAME=1/9: Check formatting
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo fmt --check
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 2. Unit tests (all workspace members)
REM ---------------------------------------------------------------------------
set STEP_NAME=2/9: Unit tests (all workspace members)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release --lib --workspace
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=3/9: Unit tests - kelvin-stream (AES-NI feature)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release --lib -p kelvin-stream --features aes-ni
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 3. Integration tests
REM ---------------------------------------------------------------------------
set STEP_NAME=4/9: Integration: Secure round-trip
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test v1_round_trip
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=5/9: Integration: Photon cipher
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test photon
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=6/9: Integration: Quantum cipher
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test quantum
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=7/9: Integration: Authenticated encryption
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test authenticated
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=8/9: Integration: Full pipeline
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test full_pipeline
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=9/9: Integration: Streaming API - Photon, Quantum, Chaos, Secure
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test streaming_api
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Prism OTP key generator
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test prism
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Flare FHE key generator
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test flare
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Split secret sharing
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test split
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Canonical test vectors (all modes)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test test_vectors
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Determinism (cross-platform golden hash)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin-core --test determinism
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Chaos test (Lyapunov estimation)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin-kdf --test chaos_test
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Client/Server self-test - Secure + Chaos Streaming
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo run --release -p kelvin-test-client
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 4. Entropy analysis & statistical tests
REM ---------------------------------------------------------------------------
set STEP_NAME=Integration: Entropy analysis (SP 800-90B health tests)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo run --release -p entropy_analysis -- --keystream
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: NIST SP 800-22 statistical tests (all 6 variants)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo run --release -p nist_tests
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: NIST SP 800-90B keystream generation + analysis
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cmd /c "cargo run --release -p nist_800_90b -- generate --size 1048576 --output keystream_90b_test.bin && cargo run --release -p nist_800_90b -- analyze --input keystream_90b_test.bin"
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: NIST SP 800-90B Prism keystream generation + analysis
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cmd /c "cargo run --release -p nist_800_90b -- generate --size 1048576 --output keystream_90b_prism.bin --prism && cargo run --release -p nist_800_90b -- analyze --input keystream_90b_prism.bin"
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: NIST SP 800-90B non-IID entropy estimation (dj-on-github)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cmd /c "python tests\sp800_90b_non_iid\sp800_90b_tests.py -t mcv keystream_90b_test.bin -s 10000 && python tests\sp800_90b_non_iid\sp800_90b_tests.py -t ttuple keystream_90b_test.bin -s 10000"
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

del keystream_90b_test.bin 2>nul

REM ---------------------------------------------------------------------------
REM 5. Constant-time benchmarks
REM ---------------------------------------------------------------------------
set STEP_NAME=Integration: Constant-time benchmarks (DudeCT)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo run --release -p constant_time_bench
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 6. Zeroize verification tests
REM ---------------------------------------------------------------------------
set STEP_NAME=Integration: Zeroize verification tests
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p zeroize_verify
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 7. L3 Pipeline Integrity Proofs (optional, requires Kani)
REM ---------------------------------------------------------------------------
set STEP_NAME=Optional: L3 Pipeline Integrity Kani proofs
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%C
echo This step is optional - requires `cargo kani` to be installed.
echo See scripts\run_proofs.bat for details.
echo %GREEN%SKIPPED (install Kani to enable)%NC%

REM ---------------------------------------------------------------------------
REM Summary
REM ---------------------------------------------------------------------------
echo.
if !EXITCODE! equ 0 (
    echo %GREEN%========================================%NC%
    echo %GREEN%  ALL SECURITY TESTS PASSED%NC%
    echo %GREEN%========================================%NC%
    exit /b 0
) else (
    echo %RED%========================================%NC%
    echo %RED%  SOME SECURITY TESTS FAILED%NC%
    echo %RED%========================================%NC%
    echo.
    echo %YELLOW%Failed steps:%NC%
    echo !FAILED_STEPS!
    echo.
    exit /b 1
)

:fail
set EXITCODE=1
set FAILED_STEPS=!FAILED_STEPS!  - !STEP_NAME!^|^
echo %RED%FAILED%NC%
exit /b 0

REM ==============================================================================
REM End of scripts\test_secure.bat
REM Updated: 2026-05-29 - Added test_vectors and L3 proof placeholders
REM ==============================================================================
