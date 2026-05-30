@echo off
REM ==============================================================================
REM Kelvin -- Comparative Benchmarks (Windows Batch)
REM ==============================================================================
REM Runs the criterion-based comparative benchmarks against industry-standard
REM cryptographic libraries (ring, dalek).
REM
REM Usage:
REM   scripts\comparative-bench               (run all benchmarks)
REM   scripts\comparative-bench quantum        (KelvinQuantum vs AES-256-GCM vs ChaCha20-Poly1305)
REM   scripts\comparative-bench streaming      (KelvinStreaming vs AES-256-CTR)
REM   scripts\comparative-bench keygen         (Orbital keygen vs X25519)
REM   scripts\comparative-bench signature      (ED25519 sign/verify)
REM   scripts\comparative-bench report         (open HTML report after running)
REM ==============================================================================

setlocal enabledelayedexpansion

cd /d "%~dp0.."

set BENCH_DIR=target\criterion

if /I "%1"=="report" goto :report
if /I "%1"=="quantum" goto :quantum
if /I "%1"=="streaming" goto :streaming
if /I "%1"=="keygen" goto :keygen
if /I "%1"=="signature" goto :signature
if "%1"=="" goto :all

echo Unknown target: %1
echo Usage: %~nx0 [quantum^|streaming^|keygen^|signature^|report]
exit /b 1

:all
echo ==============================================================================
echo   Kelvin -- Comparative Benchmarks
echo   Running all benchmarks...
echo ==============================================================================
echo.
cargo bench -p comparative-bench --bench throughput_quantum
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
cargo bench -p comparative-bench --bench throughput_streaming
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
cargo bench -p comparative-bench --bench keygen
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
cargo bench -p comparative-bench --bench signature
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
echo.
echo ==============================================================================
echo   All benchmarks complete.
echo   HTML reports: %BENCH_DIR%\reports\index.html
echo ==============================================================================
goto :eof

:quantum
cargo bench -p comparative-bench --bench throughput_quantum
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
echo.
echo   HTML report: %BENCH_DIR%\throughput_quantum\report\index.html
goto :eof

:streaming
cargo bench -p comparative-bench --bench throughput_streaming
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
echo.
echo   HTML report: %BENCH_DIR%\throughput_streaming\report\index.html
goto :eof

:keygen
cargo bench -p comparative-bench --bench keygen
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
echo.
echo   HTML report: %BENCH_DIR%\keygen\report\index.html
goto :eof

:signature
cargo bench -p comparative-bench --bench signature
if %ERRORLEVEL% NEQ 0 exit /b %ERRORLEVEL%
echo.
echo   HTML report: %BENCH_DIR%\signature\report\index.html
goto :eof

:report
if exist "%BENCH_DIR%\reports\index.html" (
    echo Opening HTML report...
    start "" "%BENCH_DIR%\reports\index.html"
) else (
    echo No report found. Run benchmarks first:
    echo   %~nx0
)
goto :eof