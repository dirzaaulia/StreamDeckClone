# StreamDeck Clone - 1-Click Tailscale Remote Launcher
# Builds and launches Windows host, desktop GUI, and Android app over Tailscale.
$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $ScriptDir

function Invoke-Checked {
    param([string]$Description, [scriptblock]$Command)
    Write-Host "[BUILD] $Description" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed (exit code $LASTEXITCODE). Nothing has been launched."
    }
}

function Get-AndroidDevices {
    $output = & adb devices
    if ($LASTEXITCODE -ne 0) { throw "Could not list Android devices with ADB." }
    @($output | Select-Object -Skip 1 | Where-Object { $_ -match '^([^\s]+)\s+device\s*$' } |
        ForEach-Object { $Matches[1] })
}

function Get-TailscaleIp {
    # 1. Try tailscale CLI
    if (Get-Command tailscale -ErrorAction SilentlyContinue) {
        $ip = (& tailscale ip -4 2>&1).Trim()
        if ($LASTEXITCODE -eq 0 -and $ip -match '^\d{1,3}(\.\d{1,3}){3}$') {
            return $ip
        }
    }
    # 2. Try default install location
    $tsExe = Join-Path $env:ProgramFiles "Tailscale\tailscale.exe"
    if (Test-Path $tsExe) {
        $ip = (& $tsExe ip -4 2>&1).Trim()
        if ($LASTEXITCODE -eq 0 -and $ip -match '^\d{1,3}(\.\d{1,3}){3}$') {
            return $ip
        }
    }
    # 3. Fallback to Windows network adapter search (100.64.0.0/10 CGNAT range)
    $adapter = Get-NetIPAddress -AddressFamily IPv4 -ErrorAction SilentlyContinue |
        Where-Object { ($_.InterfaceAlias -match "Tailscale" -or $_.IPAddress -like "100.*") -and $_.IPAddress -notmatch '^127\.' } |
        Select-Object -First 1
    if ($adapter) {
        return $adapter.IPAddress
    }
    return $null
}

try {
    # Auto-elevate to Administrator for Win32 SendInput injection rights
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator
    )
    if (-not $isAdmin) {
        Write-Host "Requesting Administrator access for Windows input injection..." -ForegroundColor Yellow
        Start-Process powershell.exe -Verb RunAs -WorkingDirectory $ScriptDir -ArgumentList @(
            "-NoExit", "-ExecutionPolicy", "Bypass", "-File", "`"$PSCommandPath`""
        )
        return
    }

    # Ensure required CLI tools exist
    foreach ($tool in @("cargo", "npm", "adb")) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
            throw "Missing $tool. Install it and add it to PATH before running this launcher."
        }
    }

    # Add Tailscale to PATH if located in Program Files
    $tsDir = Join-Path $env:ProgramFiles "Tailscale"
    if ((Test-Path $tsDir) -and ($env:PATH -notmatch [regex]::Escape($tsDir))) {
        $env:PATH = "$tsDir;$env:PATH"
    }

    $Gradle = Join-Path $ScriptDir "gradlew.bat"
    if (-not (Test-Path $Gradle)) { throw "Missing Gradle wrapper: $Gradle" }

    # Detect Host Tailscale IP
    Write-Host "[TAILSCALE] Detecting Tailscale IP address..." -ForegroundColor Cyan
    $TailscaleIp = Get-TailscaleIp
    if (-not $TailscaleIp) {
        throw "Could not detect Tailscale IP (100.x.y.z). Please ensure Tailscale is running and logged in."
    }
    Write-Host "[TAILSCALE] Host IP detected: $TailscaleIp" -ForegroundColor Green
    $env:STREAMDECK_HOST_IP = $TailscaleIp

    # Stop existing host or GUI instances so ports 4455/4456 are freed and binaries can be replaced
    Stop-Process -Name "host-desktop", "streamdeck-gui" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500

    # Build Windows host (release)
    Invoke-Checked "Windows host (release)" { cargo build --release --manifest-path (Join-Path $ScriptDir "host-desktop\Cargo.toml") }
    $HostExe = Join-Path $ScriptDir "host-desktop\target\release\host-desktop.exe"
    if (-not (Test-Path $HostExe)) { throw "Host release executable was not produced." }

    # Build Desktop GUI (release)
    Push-Location (Join-Path $ScriptDir "desktop-gui")
    try {
        if (-not (Test-Path "node_modules\.bin\vite.cmd")) {
            Invoke-Checked "Desktop JavaScript dependencies" { npm ci }
        }
        Invoke-Checked "Desktop GUI (release)" { npx tauri build --no-bundle }
    } finally {
        Pop-Location
    }
    $GuiExe = Join-Path $ScriptDir "desktop-gui\src-tauri\target\release\streamdeck-gui.exe"
    if (-not (Test-Path $GuiExe)) { throw "Desktop GUI release executable was not produced." }

    # Build Android app (debug)
    Invoke-Checked "Android app (debug)" { & $Gradle checkLineBudget :app:testDebugUnitTest :app:assembleDebug }
    $Apk = Join-Path $ScriptDir "android\app\build\outputs\apk\debug\app-debug.apk"
    if (-not (Test-Path $Apk)) { throw "Android debug APK was not produced." }

    # Look for existing TLS fingerprint
    $CertPath = Join-Path $env:APPDATA "StreamDeckClone\identity.cert"
    $Fingerprint = $null
    if (Test-Path $CertPath) {
        $Fingerprint = (Get-FileHash -Algorithm SHA256 $CertPath).Hash.ToLower()
    }

    # Discover Android device via ADB
    $serial = $env:ANDROID_SERIAL
    $devices = @(Get-AndroidDevices)

    # If no device connected via ADB, check if an Android device is listed in Tailscale peers
    if (-not $serial -and $devices.Count -eq 0 -and (Get-Command tailscale -ErrorAction SilentlyContinue)) {
        $statusOutput = & tailscale status 2>&1
        $androidPeer = $statusOutput | Where-Object { $_ -match '(\b100\.\d{1,3}\.\d{1,3}\.\d{1,3}\b).*?android' } | Select-Object -First 1
        if ($androidPeer -and $androidPeer -match '(\b100\.\d{1,3}\.\d{1,3}\.\d{1,3}\b)') {
            $peerIp = $Matches[1]
            Write-Host "[ADB] Attempting to connect to Tailscale Android peer: $peerIp:5555" -ForegroundColor Yellow
            & adb connect "$peerIp`:5555" | Out-Null
            Start-Sleep -Seconds 1
            $devices = @(Get-AndroidDevices)
        }
    }

    if ($serial) {
        if ($serial -notin $devices) { throw "ANDROID_SERIAL '$serial' is not an online ADB device." }
    } elseif ($devices.Count -eq 1) {
        $serial = $devices[0]
    } elseif ($devices.Count -gt 1) {
        $serial = $devices[0]
        Write-Host "[INFO] Multiple Android devices found. Selected: $serial" -ForegroundColor Yellow
    } else {
        Write-Warning "No online Android device found via ADB. Android APK was built and ready."
    }

    if ($serial) {
        Invoke-Checked "Install Android app on $serial" { adb -s $serial install -r $Apk }
        $reverseOutput = & adb -s $serial reverse tcp:4455 tcp:4455 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] ADB reverse active: localhost:4455 <-> PC:4455 on $serial" -ForegroundColor Gray
        }
    }

    # Ensure ports are free
    foreach ($port in @(4455, 4456)) {
        if (Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue) {
            throw "Port $port is already in use. Close the listener before launching."
        }
    }

    # Launch Windows host
    Write-Host "[RUN] Starting Windows host (Admin)..." -ForegroundColor Green
    $hostProcess = Start-Process -FilePath $HostExe -WorkingDirectory (Join-Path $ScriptDir "host-desktop") -PassThru
    Start-Sleep -Milliseconds 750
    $hostProcess.Refresh()
    if ($hostProcess.HasExited) {
        throw "Host exited during startup (code $($hostProcess.ExitCode)). Check logs."
    }

    # Refresh certificate fingerprint if newly generated
    if (-not $Fingerprint -and (Test-Path $CertPath)) {
        $Fingerprint = (Get-FileHash -Algorithm SHA256 $CertPath).Hash.ToLower()
    }

    # Launch desktop GUI
    Write-Host "[RUN] Starting Desktop GUI (Tailscale mode)..." -ForegroundColor Green
    Start-Process -FilePath $GuiExe -WorkingDirectory (Join-Path $ScriptDir "desktop-gui")

    # Launch Android app with Tailscale IP and Fingerprint pre-configured
    if ($serial) {
        Invoke-Checked "Launch Android app on $serial with Tailscale parameters" {
            $amArgs = @("-s", $serial, "shell", "am", "start", "-n", "com.streamdeck.client/.MainActivity",
                        "--es", "host_address", "$($TailscaleIp):4455")
            if ($Fingerprint) {
                $amArgs += @("--es", "fingerprint", $Fingerprint)
            }
            & adb $amArgs
        }
    }

    Write-Host ""
    Write-Host "=================================================================" -ForegroundColor Green
    Write-Host "   StreamDeck Tailscale Remote Session Ready!                    " -ForegroundColor Green
    Write-Host "=================================================================" -ForegroundColor Green
    Write-Host " Host Tailscale IP : $TailscaleIp" -ForegroundColor Cyan
    Write-Host " StreamDeck Port   : 4455" -ForegroundColor Cyan
    Write-Host " Connection URL    : wss://$($TailscaleIp):4455/" -ForegroundColor Cyan
    if ($Fingerprint) {
        Write-Host " TLS Fingerprint   : $Fingerprint" -ForegroundColor Yellow
    }
    if ($serial) {
        Write-Host " Android Device    : $serial (Connected & Configured)" -ForegroundColor Green
        Write-Host " ADB Loopback      : 127.0.0.1:4455 (Reversed)" -ForegroundColor Gray
    } else {
        Write-Host " Android Manual    : Open app on phone -> enter $TailscaleIp:4455" -ForegroundColor Yellow
    }
    Write-Host "=================================================================" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Error $_
    exit 1
}
