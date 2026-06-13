<#
.SYNOPSIS
    Launch TermArcade.
.DESCRIPTION
    Finds the TermArcade binary (release, then debug) and runs it with any
    arguments you pass.  If the binary hasn't been built yet, runs setup.ps1 first.
#>

$ScriptDir = Split-Path -Parent $PSCommandPath

# Candidates: release first, then debug
$candidates = @(
    if (Get-Command termarcade -ErrorAction SilentlyContinue) { (Get-Command termarcade).Source }
    Join-Path $ScriptDir "target\release\termarcade.exe"
    Join-Path $ScriptDir "target\x86_64-pc-windows-gnu\release\termarcade.exe"
    Join-Path $ScriptDir "target\debug\termarcade.exe"
    Join-Path $ScriptDir "target\x86_64-pc-windows-gnu\debug\termarcade.exe"
) | Where-Object { $_ -and (Test-Path $_) }

if (-not $candidates) {
    Write-Host "TermArcade binary not found. Running setup first..." -ForegroundColor Yellow
    & (Join-Path $ScriptDir "setup.ps1")
    $candidates = @(
        Join-Path $ScriptDir "target\release\termarcade.exe"
        Join-Path $ScriptDir "target\x86_64-pc-windows-gnu\release\termarcade.exe"
    ) | Where-Object { Test-Path $_ }
    if (-not $candidates) {
        Write-Host "Build failed or binary still not found." -ForegroundColor Red
        exit 1
    }
}

$binary = $candidates[0]
Write-Host "Launching TermArcade..." -ForegroundColor Cyan
& $binary $args
