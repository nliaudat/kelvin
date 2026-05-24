@echo off
REM ==============================================================================
REM Kelvin -- Encryption Benchmark (Windows Batch)
REM ==============================================================================
REM Benchmarks all modes (chaos, photon, quantum) on a file of configurable size
REM with alternating 0x00/0x01 pattern. Outputs results to documentation/bench_*.md
REM
REM Usage:
REM   scripts\bench              (default: 1 GB)
REM   scripts\bench 100          (100 GB)
REM   scripts\bench 10           (10 GB)
REM ==============================================================================

setlocal enabledelayedexpansion

set RED=[91m
set GREEN=[92m
set CYAN=[96m
set YELLOW=[93m
set NC=[0m

REM Default size = 1 GB
set SIZE_GB=%~1
if "%SIZE_GB%"=="" set SIZE_GB=1

REM Change to workspace root
cd /d "%~dp0.."

set TEMP_DIR=D:\temp\kelvin_temp
set KEY_FILE=%TEMP_DIR%\key.json
set INPUT_FILE=%TEMP_DIR%\test_%SIZE_GB%gb.bin
set ENC_FILE=%TEMP_DIR%\test_%SIZE_GB%gb.enc
set DEC_FILE=%TEMP_DIR%\test_%SIZE_GB%gb_dec.bin
set REPORT=documentation\bench_%SIZE_GB%gb.md

echo %CYAN%========================================%NC%
echo %CYAN%  Kelvin %SIZE_GB% GB Encryption Benchmark%NC%
echo %CYAN%========================================%NC%
echo.

REM ---------------------------------------------------------------------------
REM 1. Setup
REM ---------------------------------------------------------------------------
echo %CYAN%[1/6] Creating temp directory...%NC%
if not exist "%TEMP_DIR%" mkdir "%TEMP_DIR%"

echo %CYAN%[2/6] Building release CLI...%NC%
cargo build --release -p kelvin-cli
if errorlevel 1 (
    echo %RED%Build failed%NC%
    exit /b 1
)
echo %GREEN%  Done%NC%

echo %CYAN%[3/6] Generating orbital config...%NC%
target\release\kelvin.exe keygen --level standard --output "%KEY_FILE%"
if errorlevel 1 (
    echo %RED%Keygen failed%NC%
    exit /b 1
)
echo %GREEN%  Done%NC%

echo %CYAN%[4/6] Generating %SIZE_GB% GB alternating pattern file...%NC%
echo %YELLOW%  Creating zero file with fsutil...%NC%
set /a SIZE_BYTES=%SIZE_GB% * 1073741824
fsutil file createnew "%INPUT_FILE%" %SIZE_BYTES%
if errorlevel 1 (
    echo %RED%fsutil failed%NC%
    exit /b 1
)
echo %GREEN%  Zero file created%NC%

echo %YELLOW%  Writing alternating 0x00/0x01 pattern with Python...%NC%
REM Write Python script to temp file
>"%TEMP_DIR%\write_pattern.py" echo import sys
>>"%TEMP_DIR%\write_pattern.py" echo SIZE = %SIZE_GB% * 1024**3
>>"%TEMP_DIR%\write_pattern.py" echo CHUNK_SIZE = 16 * 1024**2
>>"%TEMP_DIR%\write_pattern.py" echo pattern = b'\x00\x01'
>>"%TEMP_DIR%\write_pattern.py" echo chunk = pattern * (CHUNK_SIZE // 2)
>>"%TEMP_DIR%\write_pattern.py" echo with open(r'%INPUT_FILE%', 'wb', buffering=1024*1024) as f:
>>"%TEMP_DIR%\write_pattern.py" echo     remaining = SIZE
>>"%TEMP_DIR%\write_pattern.py" echo     while remaining ^> 0:
>>"%TEMP_DIR%\write_pattern.py" echo         n = min(CHUNK_SIZE, remaining)
>>"%TEMP_DIR%\write_pattern.py" echo         f.write(chunk[:n])
>>"%TEMP_DIR%\write_pattern.py" echo         remaining -= n
>>"%TEMP_DIR%\write_pattern.py" echo         sys.stderr.write('.')
>>"%TEMP_DIR%\write_pattern.py" echo         sys.stderr.flush()
>>"%TEMP_DIR%\write_pattern.py" echo sys.stderr.write('\n')
python "%TEMP_DIR%\write_pattern.py"
del "%TEMP_DIR%\write_pattern.py"
if errorlevel 1 (
    echo %RED%Pattern write failed%NC%
    exit /b 1
)
echo %GREEN%  Done%NC%

echo %CYAN%[5/6] Computing input file hash...%NC%
for /f "skip=1 tokens=*" %%a in ('certutil -hashfile "%INPUT_FILE%" SHA256') do (
    if not defined INPUT_HASH set INPUT_HASH=%%a
)
echo %GREEN%  SHA256: %INPUT_HASH%%NC%

REM ---------------------------------------------------------------------------
REM 2. Benchmark
REM ---------------------------------------------------------------------------
echo.
echo %CYAN%========================================%NC%
echo %CYAN%  Running Benchmarks%NC%
echo %CYAN%========================================%NC%
echo.

REM Initialize report
echo # Kelvin %SIZE_GB% GB Encryption Benchmark > "%REPORT%"
echo. >> "%REPORT%"
echo **Date:** %DATE% %TIME% >> "%REPORT%"
echo **Platform:** Windows >> "%REPORT%"
echo **Test file:** %SIZE_GB% GB alternating 0x00/0x01 pattern >> "%REPORT%"
echo **Integration:** Verlet (default) >> "%REPORT%"
echo **Key level:** standard (5 bodies, 1M steps) >> "%REPORT%"
echo. >> "%REPORT%"
echo ^| Mode ^| Operation ^| Time ^(s^) ^| Throughput ^(GB/s^) ^| Verify ^| >> "%REPORT%"
echo ^|---^|-----------^|----------^|-------------------^|--------^| >> "%REPORT%"

set MODES=chaos photon quantum

for %%m in (%MODES%) do (
    echo.
    echo %CYAN%--- Mode: %%m ---%NC%

    REM Encrypt
    echo %YELLOW%  Encrypting...%NC%
    set ENC_START=%TIME%
    target\release\kelvin.exe encrypt --mode %%m --config "%KEY_FILE%" --input "%INPUT_FILE%" --output "%ENC_FILE%"
    set ENC_END=%TIME%
    if errorlevel 1 (
        echo %RED%  Encryption failed for %%m%NC%
        set ENC_TIME=ERROR
    ) else (
        call :duration ENC_TIME !ENC_START! !ENC_END!
        echo %GREEN%  Encrypt time: !ENC_TIME! s%NC%
    )

    REM Decrypt
    echo %YELLOW%  Decrypting...%NC%
    set DEC_START=%TIME%
    target\release\kelvin.exe decrypt --mode %%m --config "%KEY_FILE%" --input "%ENC_FILE%" --output "%DEC_FILE%"
    set DEC_END=%TIME%
    if errorlevel 1 (
        echo %RED%  Decryption failed for %%m%NC%
        set DEC_TIME=ERROR
    ) else (
        call :duration DEC_TIME !DEC_START! !DEC_END!
        echo %GREEN%  Decrypt time: !DEC_TIME! s%NC%
    )

    REM Verify
    set VERIFY=FAIL
    for /f "skip=1 tokens=*" %%a in ('certutil -hashfile "%DEC_FILE%" SHA256') do (
        if not defined DEC_HASH set DEC_HASH=%%a
    )
    if "!DEC_HASH!"=="%INPUT_HASH%" (
        set VERIFY=PASS
        echo %GREEN%  SHA256 match: PASS%NC%
    ) else (
        echo %RED%  SHA256 MISMATCH!%NC%
    )
    set DEC_HASH=

    REM Compute throughput
    if not "!ENC_TIME!"=="ERROR" (
        python -c "print('%%.3f' %% (%SIZE_GB% / !ENC_TIME!))" > %TEMP_DIR%\enc_gbps.txt
        set /p ENC_GBPS=<%TEMP_DIR%\enc_gbps.txt
    ) else (
        set ENC_GBPS=0
    )
    if not "!DEC_TIME!"=="ERROR" (
        python -c "print('%%.3f' %% (%SIZE_GB% / !DEC_TIME!))" > %TEMP_DIR%\dec_gbps.txt
        set /p DEC_GBPS=<%TEMP_DIR%\dec_gbps.txt
    ) else (
        set DEC_GBPS=0
    )

    REM Append to report
    echo ^| %%m ^| encrypt ^| !ENC_TIME! ^| !ENC_GBPS! ^| !VERIFY! ^| >> "%REPORT%"
    echo ^| %%m ^| decrypt ^| !DEC_TIME! ^| !DEC_GBPS! ^| !VERIFY! ^| >> "%REPORT%"

    REM Clean up intermediate files for this mode
    if exist "%ENC_FILE%" del "%ENC_FILE%"
    if exist "%DEC_FILE%" del "%DEC_FILE%"
)

REM Close report table
echo ^|---^|-----------^|----------^|-------------------^|--------^| >> "%REPORT%"
echo. >> "%REPORT%"
echo *Benchmark completed at %DATE% %TIME%* >> "%REPORT%"

REM ---------------------------------------------------------------------------
REM 3. Cleanup
REM ---------------------------------------------------------------------------
echo.
echo %CYAN%[6/6] Cleaning up...%NC%
if exist "%INPUT_FILE%" del "%INPUT_FILE%"
if exist "%KEY_FILE%" del "%KEY_FILE%"
if exist "%TEMP_DIR%\enc_gbps.txt" del "%TEMP_DIR%\enc_gbps.txt"
if exist "%TEMP_DIR%\dec_gbps.txt" del "%TEMP_DIR%\dec_gbps.txt"
rmdir "%TEMP_DIR%" 2>nul

echo.
echo %GREEN%========================================%NC%
echo %GREEN%  Benchmark Complete%NC%
echo %GREEN%========================================%NC%
echo.
echo Report written to: %REPORT%
type "%REPORT%"

exit /b 0

REM ---------------------------------------------------------------------------
REM Helper: compute duration between two %TIME% values in seconds
REM %1 = output variable name
REM %2 = start time (HH:MM:SS.CC)
REM %3 = end time (HH:MM:SS.CC)
REM ---------------------------------------------------------------------------
:duration
setlocal
for /f "tokens=1-4 delims=:.," %%a in ("%~2") do (
    set /a "sh=100%%a%%100", "sm=100%%b%%100", "ss=100%%c%%100", "sc=100%%d%%100"
)
for /f "tokens=1-4 delims=:.," %%a in ("%~3") do (
    set /a "eh=100%%a%%100", "em=100%%b%%100", "es=100%%c%%100", "ec=100%%d%%100"
)
set /a "start_cs=sh*360000+sm*6000+ss*100+sc"
set /a "end_cs=eh*360000+em*6000+es*100+ec"
if %end_cs% lss %start_cs% set /a "end_cs+=8640000"
set /a "diff_cs=end_cs-start_cs"
set /a "diff_s=diff_cs/100"
endlocal & set "%~1=%diff_s%"
exit /b 0
