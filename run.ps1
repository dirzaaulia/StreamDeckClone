# StreamDeck Clone - build and launch Windows host, desktop GUI, and Android app.
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

try {
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
    foreach ($tool in @("cargo", "npm", "adb")) {
        if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
            throw "Missing $tool. Install it and add it to PATH before running this launcher."
        }
    }
    $Gradle = Join-Path $ScriptDir "gradlew.bat"
    if (-not (Test-Path $Gradle)) { throw "Missing Gradle wrapper: $Gradle" }

    # Stop existing host or GUI instances so release builds can overwrite executables and ports 4455/4456 are freed
    Stop-Process -Name "host-desktop", "streamdeck-gui" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500

    Invoke-Checked "Windows host (release)" { cargo build --release --manifest-path (Join-Path $ScriptDir "host-desktop\Cargo.toml") }
    $HostExe = Join-Path $ScriptDir "host-desktop\target\release\host-desktop.exe"
    if (-not (Test-Path $HostExe)) { throw "Host release executable was not produced." }

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

    Invoke-Checked "Android app (debug)" { & $Gradle checkLineBudget :app:testDebugUnitTest :app:assembleDebug }
    $Apk = Join-Path $ScriptDir "android\app\build\outputs\apk\debug\app-debug.apk"
    if (-not (Test-Path $Apk)) { throw "Android debug APK was not produced." }

    $serial = $env:ANDROID_SERIAL
    $devices = @(Get-AndroidDevices)
    if ($serial) {
        if ($serial -notin $devices) { throw "ANDROID_SERIAL '$serial' is not an online ADB device." }
    } elseif ($devices.Count -eq 1) {
        $serial = $devices[0]
    } elseif ($devices.Count -gt 1) {
        Write-Warning "Multiple Android devices found. Set ANDROID_SERIAL and rerun to install and launch Android."
    } else {
        Write-Warning "No online Android device found. Android was built but cannot be installed or launched."
    }

    if ($serial) {
        Invoke-Checked "Install Android app on $serial" { adb -s $serial install -r $Apk }
        $reverseOutput = & adb -s $serial reverse tcp:4455 tcp:4455 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Warning "ADB reverse unavailable for $serial; use the host's Wi-Fi address instead. $reverseOutput"
        } else {
            Write-Host "[OK] Port 4455 reversed for $serial (pinned TLS still required)." -ForegroundColor Gray
        }
    }


    foreach ($port in @(4455, 4456)) {
        if (Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue) {
            throw "Port $port is already in use. Close the listener before launching the Windows host."
        }
    }
    Write-Host "[RUN] Starting Windows host..." -ForegroundColor Green
    $hostProcess = Start-Process -FilePath $HostExe -WorkingDirectory (Join-Path $ScriptDir "host-desktop") -PassThru
    Start-Sleep -Milliseconds 750
    $hostProcess.Refresh()
    if ($hostProcess.HasExited) {
        throw "Host exited during startup (code $($hostProcess.ExitCode)). Check its console output and credentials."
    }
    Write-Host "[RUN] Starting desktop GUI..." -ForegroundColor Green
    Start-Process -FilePath $GuiExe -WorkingDirectory (Join-Path $ScriptDir "desktop-gui")

    if ($serial) {
        Invoke-Checked "Open Android app on $serial" {
            adb -s $serial shell am start -n com.streamdeck.client/.MainActivity
        }
    }
    Write-Host "Done. Windows host uses pinned WSS on port 4455 and loopback controls on port 4456." -ForegroundColor Green
    if (-not $serial) { Write-Warning "Connect one Android device and rerun to install and open the app." }
} catch {
    Write-Error $_
    exit 1
}
