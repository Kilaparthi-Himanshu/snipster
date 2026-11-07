
@echo off
title Install Snipster

cd /d "%~dp0"

echo.
echo ========================================
echo          SNIPSTER INSTALLER
echo ========================================
echo.

if not exist "Snipster.msix" (
    echo ERROR: Snipster.msix not found.
    pause
    exit /b 1
)

if not exist "Snipster.cer" (
    echo ERROR: Snipster.cer not found.
    pause
    exit /b 1
)

echo Requesting administrator permissions...

powershell.exe -NoProfile -Command ^
 "Start-Process powershell.exe -Verb RunAs -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File ""%~dp0Install-Snipster.ps1""'"

exit /b
