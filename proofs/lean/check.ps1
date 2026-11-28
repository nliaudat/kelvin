# Verify the Lean suite the way CI would (Windows PowerShell version).
#
# 1. Build every module (the kernel checks every proof during elaboration).
# 2. Fail if any proof escape hatch is present.
# 3. Print the theorem count.
#
# Usage: .\check.ps1
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

Write-Host '== lake build =='
lake build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host '== scanning for proof escape hatches =='
$hits = Select-String -Path (Get-ChildItem src -Recurse -Filter *.lean).FullName `
    -Pattern '\b(sorry|admit|axiom|native_decide)\b'
if ($hits) {
    $hits | ForEach-Object { Write-Host "$($_.Path):$($_.LineNumber): $($_.Line.Trim())" }
    Write-Error 'forbidden proof escape hatch found'
    exit 1
}
Write-Host 'OK: no sorry/admit/axiom/native_decide'

$count = (Select-String -Path (Get-ChildItem src -Recurse -Filter *.lean).FullName `
    -Pattern '^\s*(theorem|lemma)\s').Count
Write-Host "== $count theorems/lemmas checked =="
