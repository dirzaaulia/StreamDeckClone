@echo off
title StreamDeck Clone - Tailscale Remote Launcher
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-tailscale.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Launcher exited with error code %ERRORLEVEL%
    pause
)
