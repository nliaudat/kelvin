# Mutation testing for the Kani harnesses.
#
# Copies the repository to a temporary directory, introduces one small defect in
# the implementation, rebuilds, and runs the relevant harness. The harness is
# supposed to FAIL on a mutated implementation.
#
#   CAUGHT      harness failed on the mutant  -> the harness constrains the code
#   NOT CAUGHT  harness passed on the mutant  -> the harness is too weak
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\mutation-check.ps1
#   powershell ... -Mutation add-plus-one
#   powershell ... -Mutation all -Keep
param(
    [ValidateSet('all', 'add-plus-one', 'mul-plus-one', 'period-plus-one')]
    [string]$Mutation = 'all',
    [switch]$Keep
)

$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$work = Join-Path $env:TEMP ('kelvin-mut-' + [guid]::NewGuid().ToString('N').Substring(0, 8))

Write-Host "repo:  $repo"
Write-Host "work:  $work"

# robocopy exit codes 0-7 are success; >= 8 is an error.
$null = robocopy $repo $work /E /XD .git target .lake results node_modules /XF *.log /NFL /NDL /NJH /NJS
if ($LASTEXITCODE -ge 8) { throw "robocopy failed with code $LASTEXITCODE" }

function Get-WorkFile([string]$relative) {
    return (Join-Path $work $relative)
}

function Set-Text([string]$path, [string]$old, [string]$new) {
    $text = [System.IO.File]::ReadAllText($path)
    if (-not $text.Contains($old)) {
        throw "mutation anchor not found in $path"
    }
    [System.IO.File]::WriteAllText($path, $text.Replace($old, $new), (New-Object System.Text.UTF8Encoding($false)))
}

function Set-Regex([string]$path, [string]$pattern, [string]$replacement) {
    $text = [System.IO.File]::ReadAllText($path)
    $new = [regex]::Replace($text, $pattern, $replacement)
    if ($new -eq $text) {
        throw "mutation regex did not match in $path"
    }
    [System.IO.File]::WriteAllText($path, $new, (New-Object System.Text.UTF8Encoding($false)))
}

function Invoke-Harness([string]$name) {
    # Invoke `cargo kani` directly as the container entrypoint. No shell script,
    # no bind-mount dependency, no quoting: the container exit code is Kani's.
    $compose = Join-Path $work 'docker\docker-compose.yml'
    $oldEap = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $null = & docker compose -f $compose run -T --rm --entrypoint cargo kani kani -p kelvin-core --harness $name 2>$null
    $code = $LASTEXITCODE
    $ErrorActionPreference = $oldEap
    if ($code -eq 0) { return 'PASS' }
    if ($code -eq 1) { return 'FAIL' }
    return "ERROR($code)"
}

$mutations = @(
    @{
        Name    = 'add-plus-one'
        Harness = 'verify_add_no_overflow'
        What    = 'Fixed::add returns a + b + 1 raw unit'
        Apply   = {
            Set-Text (Get-WorkFile 'kelvin-core\src\fixed_math.rs') `
                'Fixed(self.0 + rhs.0)' 'Fixed(self.0 + rhs.0 + 1)'
        }
    },
    @{
        Name    = 'mul-plus-one'
        Harness = 'verify_mul_range'
        What    = 'Fixed::mul returns the product plus one raw unit'
        Apply   = {
            Set-Regex (Get-WorkFile 'kelvin-core\src\fixed_math.rs') `
                'let result = \(result_abs \^ sign_mask\)\.wrapping_sub\(sign_mask\) as i128;\s*Fixed\(result\)' `
                'let result = (result_abs ^ sign_mask).wrapping_sub(sign_mask) as i128; Fixed(result + 1)'
        }
    },
    @{
        Name    = 'period-plus-one'
        Harness = 'verify_find_cycle_is_complete_and_correct'
        What    = 'find_cycle reports lambda + 1 instead of lambda'
        Apply   = {
            Set-Text (Get-WorkFile 'kelvin-core\src\period_detection.rs') `
                'Some(Cycle { mu, lambda })' 'Some(Cycle { mu, lambda: lambda + 1 })'
        }
    }
)

$rows = @()
foreach ($m in $mutations) {
    if ($Mutation -ne 'all' -and $Mutation -ne $m.Name) { continue }

    Write-Host ''
    Write-Host "=== mutation $($m.Name): $($m.What)"
    Write-Host "    harness: $($m.Harness)"

    # Re-copy the two source files so each mutation starts from the pristine tree.
    Copy-Item (Get-WorkFile 'kelvin-core\src\fixed_math.rs') (Get-WorkFile 'kelvin-core\src\fixed_math.rs.bak') -Force
    Copy-Item (Get-WorkFile 'kelvin-core\src\period_detection.rs') (Get-WorkFile 'kelvin-core\src\period_detection.rs.bak') -Force

    & $m.Apply
    $result = Invoke-Harness $m.Harness

    Remove-Item (Get-WorkFile 'kelvin-core\src\fixed_math.rs') -Force
    Remove-Item (Get-WorkFile 'kelvin-core\src\period_detection.rs') -Force
    Move-Item (Get-WorkFile 'kelvin-core\src\fixed_math.rs.bak') (Get-WorkFile 'kelvin-core\src\fixed_math.rs')
    Move-Item (Get-WorkFile 'kelvin-core\src\period_detection.rs.bak') (Get-WorkFile 'kelvin-core\src\period_detection.rs')

    $verdict = if ($result -eq 'FAIL') { 'CAUGHT' } else { 'NOT CAUGHT' }
    Write-Host "    harness verdict: $result  => $verdict"
    $rows += [pscustomobject]@{
        Mutation = $m.Name
        Harness  = $m.Harness
        Defect   = $m.What
        Result   = $result
        Verdict  = $verdict
    }
}

Write-Host ''
$rows | Format-Table -AutoSize | Out-String | Write-Host

if (-not $Keep) {
    Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
}

# Exit 0 only when every mutant was caught.
if ($rows | Where-Object { $_.Verdict -ne 'CAUGHT' }) { exit 1 }
exit 0
