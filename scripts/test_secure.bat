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
set NC=[0m

set EXITCODE=0

REM Change to workspace root (parent of scripts/)
cd /d "%~dp0.."

REM ---------------------------------------------------------------------------
REM 1. Formatting check
REM ---------------------------------------------------------------------------
call :step "1/9: Check formatting"
cargo fmt --check
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 2. Unit tests (all workspace members)
REM ---------------------------------------------------------------------------
call :step "2/9: Unit tests (all workspace members)"
cargo test --release --lib --workspace
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "3/9: Unit tests - kelvin-stream (AES-NI feature)"
cargo test --release --lib -p kelvin-stream --features aes-ni
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM 3. Integration tests
REM ---------------------------------------------------------------------------
call :step "4/9: Integration: V1 round-trip"
cargo test --release -p kelvin --test v1_round_trip
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "5/9: Integration: Photon cipher"
cargo test --release -p kelvin --test photon
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "6/9: Integration: Quantum cipher"
cargo test --release -p kelvin --test quantum
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "7/9: Integration: Authenticated encryption"
cargo test --release -p kelvin --test authenticated
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "8/9: Integration: Full pipeline"
cargo test --release -p kelvin --test full_pipeline
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "9/9: Integration: Streaming API (Photon, Quantum, Chaos, Secure)"
cargo test --release -p kelvin --test streaming_api
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
REM 4. Entropy analysis & statistical tests
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
REM 5. Constant-time benchmarks
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
echo %GREEN%  ALL SECURITY TESTS PASSED%NC%
echo %GREEN%========================================%NC%
exit /b 0

:step
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %~1%NC%
echo %CYAN%========================================%NC%
exit /b 0
