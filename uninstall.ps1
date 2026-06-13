<#
.SYNOPSIS
    Completely removes TermArcade — binary, build artifacts, saves, and preferences.
#>

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $PSCommandPath

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-OK($msg) { Write-Host "    OK  $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "    $msg" -ForegroundColor Yellow }

Write-Step "TermArcade Uninstaller"
Write-Step "This will permanently delete everything: binary, saved games, high scores, and build cache."

$confirm = Read-Host "Are you sure? (y/N)"
if ($confirm -ne "y" -and $confirm -ne "Y") {
    Write-Host "Cancelled." -ForegroundColor Yellow
    exit 0
}

# 1. Uninstall cargo binary
Write-Step "Removing installed binary..."
$binPath = (Get-Command termarcade -ErrorAction SilentlyContinue).Source
if ($binPath) {
    Remove-Item $binPath -Force -ErrorAction SilentlyContinue
    Write-OK "Removed: $binPath"
} else {
    cargo uninstall termarcade 2>$null
    Write-OK "Ran cargo uninstall (if it was installed via cargo)"
}

# 2. Delete build artifacts (target/)
$targetDir = Join-Path $ScriptDir "target"
if (Test-Path $targetDir) {
    Remove-Item $targetDir -Recurse -Force -ErrorAction SilentlyContinue
    Write-OK "Deleted: target/ (build artifacts)"
}

# 3. Delete save & pref files
Write-Step "Removing saved games and preferences..."
Get-ChildItem $ScriptDir -Filter "*.save" -File | ForEach-Object {
    Remove-Item $_.FullName -Force
    Write-OK "Deleted: $($_.Name)"
}
Get-ChildItem $ScriptDir -Filter "*.pref" -File | ForEach-Object {
    Remove-Item $_.FullName -Force
    Write-OK "Deleted: $($_.Name)"
}

# 4. Delete compiled launcher
$launcherExe = Join-Path $ScriptDir "TermArcade.exe"
if (Test-Path $launcherExe) {
    Remove-Item $launcherExe -Force
    Write-OK "Deleted: TermArcade.exe (compiled launcher)"
}

# 5. Optionally delete the whole project folder
Write-Step ""
Write-Warn "To fully remove everything, you can also delete this project folder:"
Write-Warn "    $ScriptDir"
$delFolder = Read-Host "Delete the project folder itself? (y/N)"
if ($delFolder -eq "y" -or $delFolder -eq "Y") {
    Write-Step "Deleting project folder..."
    Set-Location ..
    Remove-Item $ScriptDir -Recurse -Force
    Write-OK "Deleted: $ScriptDir"
}

Write-Step "Done. TermArcade has been completely removed."
