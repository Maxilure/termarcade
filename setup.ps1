<#
.SYNOPSIS
    TermArcade Windows Setup — one-click build & install.
.DESCRIPTION
    Checks for Rust, detects MSVC vs GNU toolchain, builds, and optionally
    installs TermArcade so you can run it from any terminal.
#>

$ErrorActionPreference = "Stop"

function Write-Step($msg) {
    Write-Host "==> $msg" -ForegroundColor Cyan
}

function Write-OK($msg) {
    Write-Host "    OK  $msg" -ForegroundColor Green
}

function Write-Warn($msg) {
    Write-Host "    WARN  $msg" -ForegroundColor Yellow
}

function Write-Fail($msg) {
    Write-Host "    FAIL $msg" -ForegroundColor Red
}

Write-Step "Checking for Rust..."
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $rustc -or -not $cargo) {
    Write-Fail "Rust or Cargo not found."
    Write-Host "Please install Rust from: https://rustup.rs" -ForegroundColor Yellow
    Write-Host "After installing, close and reopen PowerShell, then run this script again."
    exit 1
}
Write-OK "rustc $($rustc.Version) / cargo $($cargo.Version)"

Write-Step "Checking toolchain..."
$target = "x86_64-pc-windows-msvc"
$hasLink = Get-Command link.exe -ErrorAction SilentlyContinue
if (-not $hasLink) {
    Write-Warn "MSVC linker (link.exe) not found — this is normal if you don't have Visual Studio Build Tools."
    $target = "x86_64-pc-windows-gnu"
    Write-Warn "Switching to GNU toolchain: $target"

    $installed = rustup target list --installed
    if ($installed -notcontains $target) {
        Write-Step "Installing GNU toolchain..."
        rustup target install $target
        Write-OK "Installed $target"
    } else {
        Write-OK "Toolchain $target already installed"
    }
} else {
    Write-OK "MSVC linker found, using MSVC toolchain"
}

Write-Step "Building TermArcade (release)..."
if ($hasLink) {
    cargo build --release -p termarcade
} else {
    cargo build --release -p termarcade --target $target
}

if ($LASTEXITCODE -ne 0) {
    Write-Fail "Build failed. Check the errors above."
    exit 1
}
Write-OK "Build successful!"

$wantInstall = $false
Write-Step "Installing to Cargo bin directory..."
cargo install --path termarcade --target $target 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-OK "Installed! Run 'termarcade' from any terminal."
} else {
    $binPath = if ($hasLink) {
        "target\release\termarcade.exe"
    } else {
        "target\$target\release\termarcade.exe"
    }
    Write-Warn "Install via cargo failed. The binary is at: $binPath"
    Write-Host "You can copy it manually to any folder on your PATH, or"
    Write-Host "run it directly with: .\$binPath"
}

Write-Step "Done!"
Write-Host "To launch TermArcade, run: termarcade" -ForegroundColor Green
