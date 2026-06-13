<#
.SYNOPSIS
    Compiles TermArcade.cs into TermArcade.exe using the built-in .NET Framework C# compiler.
    No Visual Studio or extra tools required — works on any Windows machine.
#>

$ScriptDir = Split-Path -Parent $PSCommandPath
$csPath = Join-Path $ScriptDir "TermArcade.cs"
$outPath = Join-Path $ScriptDir "TermArcade.exe"

if (-not (Test-Path $csPath)) {
    Write-Host "TermArcade.cs not found at $csPath" -ForegroundColor Red
    exit 1
}

# Locate the built-in C# compiler (csc.exe)
$csc = Get-Command "csc.exe" -ErrorAction SilentlyContinue
if (-not $csc) {
    # Try the .NET Framework SDK path
    $csc = Get-ChildItem -Path "$env:windir\Microsoft.NET\Framework64" -Recurse -Filter "csc.exe" |
           Sort-Object Version |
           Select-Object -Last 1 -ExpandProperty FullName
}
if (-not $csc) {
    $csc = Get-ChildItem -Path "$env:windir\Microsoft.NET\Framework" -Recurse -Filter "csc.exe" |
           Sort-Object Version |
           Select-Object -Last 1 -ExpandProperty FullName
}

if (-not $csc) {
    Write-Host "C# compiler (csc.exe) not found. This should be part of any Windows installation." -ForegroundColor Red
    Write-Host "Try installing .NET Framework SDK or Visual Studio Build Tools." -ForegroundColor Yellow
    exit 1
}

Write-Host "Compiling TermArcade.exe ..." -ForegroundColor Cyan
Write-Host "Using: $csc" -ForegroundColor DarkGray

$refs = @(
    "System.dll"
    "System.Windows.Forms.dll"
    "Microsoft.VisualBasic.dll"
)

$argsList = @(
    "/target:winexe",
    "/out:$outPath",
    "/reference:" + ($refs -join ","),
    "/win32icon:" + (Join-Path $ScriptDir "termarcade.ico"),
    $csPath
)

# Remove icon reference if no icon file exists
$iconPath = Join-Path $ScriptDir "termarcade.ico"
if (-not (Test-Path $iconPath)) {
    $argsList = $argsList | Where-Object { $_ -notlike "/win32icon:*" }
}

& $csc $argsList

if ($LASTEXITCODE -eq 0 -and (Test-Path $outPath)) {
    Write-Host "Success! TermArcade.exe created." -ForegroundColor Green
    Write-Host "Double-click TermArcade.exe to launch the game." -ForegroundColor Green
} else {
    Write-Host "Compilation failed (exit code: $LASTEXITCODE)." -ForegroundColor Red
    exit 1
}
