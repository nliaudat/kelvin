@echo off
REM ==============================================================================
REM Kelvin -- Run All Tests (Windows Batch)
REM ==============================================================================
REM Runs the full test suite: build, lint, unit tests, integration tests,
REM entropy analysis, NIST SP 800-22 statistical tests, and constant-time
REM benchmarks.
REM
REM Exit code is 0 only if ALL steps pass.
REM
REM Usage:
REM   scripts\test-all.bat
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

REM Initialize git submodules (NIST SP 800-90B tools)
if not exist .gitmodules goto :skip_submodules
git submodule update --init --recursive
:skip_submodules

set STEP_NAME=1/8: Build workspace (release mode)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo build --workspace --release
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=2/8: Build with AES-NI feature
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo build --release -p kelvin-stream --features aes-ni
if errorlevel 1 call :fail
if !EXITCODE! equ 0 (
    cargo build --release -p kelvin --features aes-ni
    if errorlevel 1 call :fail
)
echo %GREEN%PASSED%NC%

set STEP_NAME=2b/8: Build Chaos Streaming example
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo build --release --example simple_streaming -p kelvin
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=2c/8: Build kelvin-ffi (C FFI bindings)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo build --release -p kelvin-ffi
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 2. Lint -- clippy + rustfmt
REM ---------------------------------------------------------------------------

set STEP_NAME=3/8: Clippy (deny warnings)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo clippy --workspace -- -D warnings
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=4/8: Check formatting
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo fmt --check
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 3. Unit tests (all workspace members)
REM ---------------------------------------------------------------------------
set STEP_NAME=5/8: Unit tests (all workspace members)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release --lib --workspace
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=6/8: Unit tests - kelvin-stream (AES-NI feature)
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release --lib -p kelvin-stream --features aes-ni
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 4. Integration tests
REM ---------------------------------------------------------------------------
set STEP_NAME=Integration: Secure round-trip
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test v1_round_trip
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Photon cipher
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test photon
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Quantum cipher
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test quantum
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Authenticated encryption
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test authenticated
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Full pipeline
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %STEP_NAME%%NC%
echo %CYAN%========================================%NC%
cargo test --release -p kelvin --test full_pipeline
if errorlevel 1 call :fail
echo %GREEN%PASSED%NC%

set STEP_NAME=Integration: Streaming API - Photon, Quantum, Chaos, Secure
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
REM 5. Entropy analysis & statistical tests
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
REM 6. Constant-time benchmarks
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
REM 7. Zeroize verification tests
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
REM Summary
REM ---------------------------------------------------------------------------
echo.
if !EXITCODE! equ 0 (
    echo %GREEN%========================================%NC%
    echo %GREEN%  ALL TESTS PASSED%NC%
    echo %GREEN%========================================%NC%
    exit /b 0
) else (
    echo %RED%========================================%NC%
    echo %RED%  SOME TESTS FAILED%NC%
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
