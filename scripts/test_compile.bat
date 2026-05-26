@echo off
REM ==============================================================================
REM Kelvin -- Compile & Lint Tests (Windows Batch)
REM ==============================================================================
REM Verifies that all workspace members compile and pass lint checks.
REM
REM Exit code is 0 only if ALL steps pass.
REM
REM Usage:
REM   scripts\test_compile.bat
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

call :step "1/5: Build workspace (default features)"
cargo build --workspace
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "2/5: Build with AES-NI feature"
cargo build -p kelvin-stream --features aes-ni
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
cargo build -p kelvin --features aes-ni
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "3/5: Build V2 Streaming example"
cargo build --example simple_streaming -p kelvin
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "4/5: Build kelvin-ffi (C FFI bindings)"
cargo build -p kelvin-ffi
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

call :step "5/5: Clippy (deny warnings)"
cargo clippy --workspace -- -D warnings
if errorlevel 1 set EXITCODE=1
if !EXITCODE! neq 0 exit /b !EXITCODE!
echo %GREEN%PASSED%NC%

REM ---------------------------------------------------------------------------
REM All passed
REM ---------------------------------------------------------------------------
echo.
echo %GREEN%========================================%NC%
echo %GREEN%  ALL COMPILE TESTS PASSED%NC%
echo %GREEN%========================================%NC%
exit /b 0

:step
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  %~1%NC%
echo %CYAN%========================================%NC%
exit /b 0
