@echo off
title StreamDeck Windows Host Engine
cd /d "%~dp0"

:: Auto-elevate to Administrator for input injection rights
net session >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [INFO] Requesting Administrator privileges to allow system hotkey simulation...
    powershell -NoProfile -Command "Start-Process cmd -ArgumentList '/c \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

echo ========================================================
echo   StreamDeck Clone: Starting Windows Host Engine (Admin)
echo ========================================================

:: 1. Forward USB port via ADB if a device is connected
echo [1/2] Setting up ADB USB reverse port forward (4455)...
adb reverse tcp:4455 tcp:4455 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [OK] USB port forward active: localhost:4455 ^<--^> Android
) else (
    echo [INFO] No ADB device connected via USB (Wi-Fi mode available)
)

:: 2. Launch Host Daemon
echo [2/2] Launching Windows Host Engine on ws://0.0.0.0:4455...
echo --------------------------------------------------------
if exist "host-desktop\target\release\host-desktop.exe" (
    "host-desktop\target\release\host-desktop.exe"
) else if exist "host-desktop\target\debug\host-desktop.exe" (
    "host-desktop\target\debug\host-desktop.exe"
) else (
    cd host-desktop && cargo run --release
)

pause
