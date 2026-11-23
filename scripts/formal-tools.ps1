# Build and smoke-test the formal/analysis Docker images.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\formal-tools.ps1 -Build
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\formal-tools.ps1 -Smoke
#   powershell ... -Build -Smoke -Tools proverif,tamarin
#
# Docker writes warnings to stderr; Windows PowerShell 5.1 turns those into
# terminating errors under `Stop`, so native calls run with `Continue` and their
# merged output is inspected explicitly.
param(
    [switch]$Build,
    [switch]$Smoke,
    [string[]]$Tools = @()
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$compose = Join-Path $repo 'docker\docker-compose.formal.yml'

# `powershell -File script.ps1 -Tools a,b` passes "a,b" as one string; split it
# so both `-Tools a,b` and `-Tools @('a','b')` work.
$Tools = @($Tools | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' })

if (-not (Test-Path $compose)) { throw "compose file not found: $compose" }

$allTools = @('proverif', 'tamarin', 'cryptoverif', 'easycrypt', 'smt', 'rust-tools')
if ($Tools.Count -gt 0) {
    $unknown = $Tools | Where-Object { $allTools -notcontains $_ }
    if ($unknown) { throw "unknown tool(s): $($unknown -join ', '). Valid: $($allTools -join ', ')" }
    $selected = $Tools
} else {
    $selected = $allTools
}

function Invoke-Compose {
    param([string[]]$ComposeArgs)
    $old = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $out = & docker compose -f $compose @ComposeArgs 2>&1
    $code = $LASTEXITCODE
    $ErrorActionPreference = $old
    return @{ Output = @($out); Code = $code }
}

if (-not $Build -and -not $Smoke) {
    Write-Host 'Nothing to do. Pass -Build and/or -Smoke.'
    Write-Host "Tools: $($allTools -join ', ')"
    exit 0
}

if ($Build) {
    foreach ($tool in $selected) {
        Write-Host ''
        Write-Host "== building $tool" -ForegroundColor Cyan
        $r = Invoke-Compose @('build', $tool)
        if ($r.Code -ne 0) {
            $r.Output | Select-Object -Last 20 | ForEach-Object { Write-Host $_ }
            Write-Error "build failed for $tool"
            exit 1
        }
        Write-Host "   built $tool" -ForegroundColor Green
    }
}

if ($Smoke) {
    # Each entry runs one short command that exercises the tool.
    $checks = @(
        @{ Tool = 'proverif';    Args = @('run', '--rm', '-T', 'proverif', 'formal/proverif/kelvin_smoke.pv'); Expect = 'RESULT not attacker\(key\[\]\) is true' },
        @{ Tool = 'tamarin';     Args = @('run', '--rm', '-T', 'tamarin', '--prove', 'formal/tamarin/kelvin_smoke.spthy'); Expect = 'key_secret \(all-traces\): verified' },
        @{ Tool = 'easycrypt';   Args = @('run', '--rm', '-T', 'easycrypt', 'formal/easycrypt/smoke.ec') },
        @{ Tool = 'easycrypt';   Args = @('run', '--rm', '-T', '--entrypoint', 'bash', 'easycrypt', '-c', 'easycrypt why3config') },
        @{ Tool = 'cryptoverif'; Args = @('run', '--rm', '-T', '--entrypoint', 'cryptoverif', 'cryptoverif', '-help') },
        @{ Tool = 'smt';         Args = @('run', '--rm', '-T', '--entrypoint', 'z3', 'smt', '--version') },
        @{ Tool = 'smt';         Args = @('run', '--rm', '-T', '--entrypoint', 'cvc5', 'smt', '--version') },
        @{ Tool = 'rust-tools';  Args = @('run', '--rm', '-T', '--entrypoint', 'cargo', 'rust-tools', 'fuzz', '--version') },
        @{ Tool = 'rust-tools';  Args = @('run', '--rm', '-T', '--entrypoint', 'cargo', 'rust-tools', 'audit', '--version') },
        @{ Tool = 'rust-tools';  Args = @('run', '--rm', '-T', '--entrypoint', 'cargo', 'rust-tools', 'deny', '--version') },
        @{ Tool = 'rust-tools';  Args = @('run', '--rm', '-T', '--entrypoint', 'cargo', 'rust-tools', 'mutants', '--version') },
        @{ Tool = 'rust-tools';  Args = @('run', '--rm', '-T', '--entrypoint', 'cargo', 'rust-tools', 'miri', '--version') }
    )

    $failed = 0
    foreach ($check in $checks) {
        if ($selected -notcontains $check.Tool) { continue }

        $label = ($check.Args -join ' ')
        Write-Host ''
        Write-Host "== $label" -ForegroundColor Cyan
        $r = Invoke-Compose $check.Args
        $ok = ($r.Code -eq 0)
        if ($ok -and $check.ContainsKey('Expect') -and $check.Expect) {
            $text = ($r.Output -join "`n")
            if ($text -notmatch $check.Expect) { $ok = $false }
        }
        if ($ok) {
            Write-Host '   PASS' -ForegroundColor Green
        } else {
            $failed++
            Write-Host "   FAIL (exit $($r.Code))" -ForegroundColor Red
            $r.Output | Select-Object -Last 15 | ForEach-Object { Write-Host "   $_" }
        }
    }

    Write-Host ''
    if ($failed -gt 0) {
        Write-Host "$failed smoke check(s) failed" -ForegroundColor Red
        exit 1
    }
    Write-Host 'all selected smoke checks passed' -ForegroundColor Green
}
