
@echo off
title Uninstall Snipster

echo.
echo ========================================
echo         SNIPSTER UNINSTALLER
echo ========================================
echo.

echo Requesting administrator permissions...

powershell.exe -NoProfile -Command ^
 "Start-Process powershell.exe -Verb RunAs -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File ""%~dp0Uninstall-Snipster.ps1""'"

exit /b
