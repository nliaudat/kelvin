@echo off
REM ==============================================================================
REM Kelvin -- Encryption Benchmark (Windows Batch Wrapper)
REM ==============================================================================
REM Delegates to bench.py for reliable cross-platform timing.
REM
REM Usage:
REM   scripts\bench                  (default: 1 GB, paranoid level)
REM   scripts\bench 100              (100 GB, paranoid level)
REM   scripts\bench 10               (10 GB, paranoid level)
REM   scripts\bench 1 --level maximum
REM   scripts\bench 1 --level standard
REM ==============================================================================

python "%~dp0bench.py" %*
exit /b %ERRORLEVEL%
