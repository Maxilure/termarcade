@echo off
REM TermArcade — double-click launcher for Windows
REM Opens PowerShell and runs TermArcade

powershell.exe -ExecutionPolicy Bypass -NoProfile -File "%~dp0termarcade.ps1" %*
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo TermArcade exited with error code %ERRORLEVEL%.
    pause
)
