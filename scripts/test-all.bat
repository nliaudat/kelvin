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
set NC=[0m

set EXITCODE=0

REM Change to workspace root (parent of scripts/)
cd /d "%~dp0.."

REM Initialize git submodules (NIST SP 800-90B tools)
if not exist .gitmodules goto :skip_submodules
git submodule update --init --recursive
:skip_submodules

call :step "1/8: Build workspace (release mode)"
cargo build --workspace --release
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "2/8: Build with AES-NI feature"
cargo build --release -p kelvin-stream --features aes-ni
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
cargo build --release -p kelvin --features aes-ni
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "2b/8: Build V2 Streaming example"
cargo build --release --example simple_streaming -p kelvin
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "2c/8: Build kelvin-ffi (C FFI bindings)"
cargo build --release -p kelvin-ffi
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 2. Lint -- clippy + rustfmt
REM ---------------------------------------------------------------------------

call :step "3/8: Clippy (deny warnings)"
cargo clippy --workspace -- -D warnings
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "4/8: Check formatting"
cargo fmt --check
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 3. Unit tests (all workspace members)
REM ---------------------------------------------------------------------------
call :step "5/8: Unit tests (all workspace members)"
cargo test --release --lib --workspace
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "6/8: Unit tests - kelvin-stream (AES-NI feature)"
cargo test --release --lib -p kelvin-stream --features aes-ni
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 4. Integration tests
REM ---------------------------------------------------------------------------
call :step "Integration: V1 round-trip"
cargo test --release -p kelvin --test v1_round_trip
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Photon cipher"
cargo test --release -p kelvin --test photon
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Quantum cipher"
cargo test --release -p kelvin --test quantum
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Authenticated encryption"
cargo test --release -p kelvin --test authenticated
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Full pipeline"
cargo test --release -p kelvin --test full_pipeline
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Streaming API (Photon, Quantum, Chaos, Secure)"
cargo test --release -p kelvin --test streaming_api
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Prism OTP key generator"
cargo test --release -p kelvin --test prism
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Chaos test (Lyapunov estimation)"
cargo test --release -p kelvin-kdf --test chaos_test
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: Client/Server self-test (V1 + V2 Streaming)"
cargo run --release -p kelvin-test-client
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 5. Entropy analysis & statistical tests
REM ---------------------------------------------------------------------------
call :step "Integration: Entropy analysis (SP 800-90B health tests)"
cargo run --release -p entropy_analysis -- --keystream
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: NIST SP 800-22 statistical tests (all 6 variants)"
cargo run --release -p nist_tests
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "Integration: NIST SP 800-90B keystream generation + analysis"
cargo run --release -p nist_800_90b -- generate --size 1048576 --output keystream_90b_test.bin
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
cargo run --release -p nist_800_90b -- analyze --input keystream_90b_test.bin
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!

call :step "Integration: NIST SP 800-90B Prism keystream generation + analysis"
cargo run --release -p nist_800_90b -- generate --size 1048576 --output keystream_90b_prism.bin --prism
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
cargo run --release -p nist_800_90b -- analyze --input keystream_90b_prism.bin
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!

call :step "Integration: NIST SP 800-90B non-IID entropy estimation (dj-on-github)"
python tests\sp800_90b_non_iid\sp800_90b_tests.py -t mcv keystream_90b_test.bin -s 10000
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
python tests\sp800_90b_non_iid\sp800_90b_tests.py -t ttuple keystream_90b_test.bin -s 10000
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!

del keystream_90b_test.bin
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 6. Constant-time benchmarks
REM ---------------------------------------------------------------------------
call :step "Integration: Constant-time benchmarks (DudeCT)"
cargo run --release -p constant_time_bench
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM All passed
REM ---------------------------------------------------------------------------
echo.
echo %GREEN%========================================%NC%
echo %GREEN%  ALL TESTS PASSED%NC%
echo %GREEN%========================================%NC%
exit /b 0

:step
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %~1%NC%
echo %CYAN%========================================%NC%
exit /b 0
