@echo off
REM ==============================================================================
REM build-ea-iid.bat — Build NIST SP 800-90B Entropy Assessment Tool (Windows)
REM ==============================================================================
REM Compiles the official NIST entropy assessment tool (ea_iid) from the
REM tests/ea_iid/ git submodule using Docker.
REM
REM Requires: Docker Desktop (or Docker Engine)
REM
REM Usage:
REM   tests\build-ea-iid.bat
REM
REM Exit code: 0 on success, 1 on failure
REM ==============================================================================

setlocal enabledelayedexpansion

set RED=[91m
set GREEN=[92m
set CYAN=[96m
set YELLOW=[93m
set NC=[0m

echo.
echo %CYAN%========================================%NC%
echo %CYAN%  NIST SP 800-90B ea_iid Build Script%NC%
echo %CYAN%========================================%NC%
echo.

REM ---------------------------------------------------------------------------
REM 1. Initialize submodule
REM ---------------------------------------------------------------------------
echo [1/4] Initializing git submodule...
git submodule update --init tests\ea_iid
if errorlevel 1 (
    echo %RED%FAILED: Could not initialize submodule%NC%
    exit /b 1
)
echo %GREEN%OK%NC%

REM ---------------------------------------------------------------------------
REM 2. Check for Docker
REM ---------------------------------------------------------------------------
echo [2/4] Checking for Docker...
docker --version >nul 2>&1
if errorlevel 1 (
    echo %RED%FAILED: Docker not found. Install Docker Desktop from:%NC%
    echo   https://www.docker.com/products/docker-desktop/
    exit /b 1
)
echo %GREEN%Found Docker%NC%

REM ---------------------------------------------------------------------------
REM 3. Build with Docker
REM ---------------------------------------------------------------------------
echo [3/4] Building ea_iid in Docker...
echo This will download an Ubuntu base image (~80 MB) on first run.

docker build -t ea_iid-builder -f tests\ea_iid\Dockerfile tests\ea_iid
if errorlevel 1 (
    echo %RED%FAILED: Docker build error%NC%
    exit /b 1
)
echo %GREEN%Docker build successful%NC%

REM ---------------------------------------------------------------------------
REM 4. Extract binaries
REM ---------------------------------------------------------------------------
echo [4/4] Extracting binaries...

REM Create a temporary container and copy binaries out
docker create --name ea_iid_extract ea_iid-builder /bin/sh >nul 2>&1
if errorlevel 1 (
    echo %RED%FAILED: Could not create extraction container%NC%
    exit /b 1
)

docker cp ea_iid_extract:/ea_iid tests\ea_iid\ea_iid >nul 2>&1
docker cp ea_iid_extract:/ea_non_iid tests\ea_iid\ea_non_iid >nul 2>&1
docker cp ea_iid_extract:/ea_restart tests\ea_iid\ea_restart >nul 2>&1
docker cp ea_iid_extract:/ea_conditioning tests\ea_iid\ea_conditioning >nul 2>&1
docker cp ea_iid_extract:/ea_transpose tests\ea_iid\ea_transpose >nul 2>&1

docker rm ea_iid_extract >nul 2>&1

echo %GREEN%Binaries extracted to tests/ea_iid/%NC%

REM ---------------------------------------------------------------------------
REM Done
REM ---------------------------------------------------------------------------
echo.
echo %GREEN%========================================%NC%
echo %GREEN%  BUILD SUCCESSFUL%NC%
echo %GREEN%========================================%NC%
echo.
echo Binaries:
echo   tests\ea_iid\ea_iid              — IID tests
echo   tests\ea_iid\ea_non_iid          — Non-IID tests
echo   tests\ea_iid\ea_restart          — Restart tests
echo   tests\ea_iid\ea_conditioning     — Conditioning calculator
echo   tests\ea_iid\ea_transpose        — Data transpose utility
echo.
echo To run on Kelvin keystream:
echo   python tests\ea_iid\ea_iid.py -i keystream_1gb.bin -o results.txt
echo.

exit /b 0
