# ========================================================
# StreamDeck Clone - 1-Command All-in-One Launcher
# Builds & starts both Host Engine (Admin) and Desktop GUI
# ========================================================

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $ScriptDir

# 1. Elevate to Administrator if not already elevated
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[INFO] Requesting Administrator privileges for native Windows input injection..." -ForegroundColor Yellow
    Start-Process powershell -Verb RunAs -ArgumentList "-NoExit", "-ExecutionPolicy", "Bypass", "-File", "`"$PSCommandPath`""
    exit
}

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "   StreamDeck Clone: Building & Launching Environment   " -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

# 2. Setup ADB USB Reverse (Port 4455)
Write-Host "[1/4] Setting up ADB USB reverse port forward..." -ForegroundColor Green
try {
    adb reverse tcp:4455 tcp:4455 2>$null
    Write-Host "      [OK] USB port forward active: localhost:4455 <-> Android" -ForegroundColor Gray
} catch {
    Write-Host "      [INFO] No ADB device connected via USB (Wi-Fi mode active)" -ForegroundColor Gray
}

# 3. Build & verify Host Engine
Write-Host "[2/4] Verifying Host Engine release binary..." -ForegroundColor Green
$HostExe = Join-Path $ScriptDir "host-desktop\target\release\host-desktop.exe"
if (-not (Test-Path $HostExe)) {
    Write-Host "      Building host-desktop in release mode..." -ForegroundColor Yellow
    cargo build --release --manifest-path (Join-Path $ScriptDir "host-desktop\Cargo.toml")
}

# 4. Build & verify Desktop GUI (Vite + Tauri)
Write-Host "[3/4] Verifying Desktop Configurator GUI..." -ForegroundColor Green
$GuiExe = Join-Path $ScriptDir "desktop-gui\src-tauri\target\release\streamdeck-gui.exe"
$DistHtml = Join-Path $ScriptDir "desktop-gui\dist\index.html"

if (-not (Test-Path $DistHtml)) {
    Write-Host "      Building desktop-gui frontend (npm run build)..." -ForegroundColor Yellow
    Push-Location (Join-Path $ScriptDir "desktop-gui")
    npm run build
    Pop-Location
}

if (-not (Test-Path $GuiExe)) {
    Write-Host "      Building desktop-gui native binary..." -ForegroundColor Yellow
    cargo build --release --manifest-path (Join-Path $ScriptDir "desktop-gui\src-tauri\Cargo.toml")
}

# 5. Launch Host Engine
Write-Host "[4/4] Launching StreamDeck Host Engine and GUI..." -ForegroundColor Green

# Kill existing host-desktop if already running to prevent port conflict
Get-Process -Name "host-desktop" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300

# Start host-desktop in its own window so logs are visible
Start-Process -FilePath $HostExe -WorkingDirectory (Join-Path $ScriptDir "host-desktop")

# Start desktop GUI
Start-Process -FilePath $GuiExe -WorkingDirectory (Join-Path $ScriptDir "desktop-gui")

Write-Host ""
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "  StreamDeck is RUNNING!" -ForegroundColor Green
Write-Host "  - Host Engine: ws://0.0.0.0:4455 (Admin mode)" -ForegroundColor White
Write-Host "  - Desktop GUI: StreamDeck Configurator active" -ForegroundColor White
Write-Host "  - Android App: Open app, it will auto-scan & connect!" -ForegroundColor Yellow
Write-Host "========================================================" -ForegroundColor Cyan
