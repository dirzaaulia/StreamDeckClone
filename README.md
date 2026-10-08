# StreamDeck Clone: Full System Documentation

A high-performance, ultra-low latency, cross-platform Stream Deck system connecting a mobile controller to a Windows workstation over local Wi-Fi and high-speed USB tethering.

---

## 1. System Architecture Overview

```text
┌──────────────────────────────────────────────────────────────┐
│                    Android Mobile Client                     │
│         Kotlin 2.1 • Jetpack Compose • Material 3            │
│  - 3x3 Key Matrix (host-provided titles and actions)    │
│  - Hybrid Discovery: mDNS (NsdManager) + TCP Subnet Scanner  │
│  - WebSocket Client with Protocol Buffers Serialization      │
└──────────────────────────────┬───────────────────────────────┘
                               │
            Pinned WSS (Protobuf) over Port 4455
            • Local Wi-Fi (e.g. wss://192.168.110.7:4455)
            • ADB Reverse USB (wss://127.0.0.1:4455)
                               │
┌──────────────────────────────▼───────────────────────────────┐
│               Windows Host Engine (host-desktop)              │
│               Rust Edition 2024 • Tokio Runtime              │
│  - Tokio-Tungstenite WebSocket Server                        │
│  - Win32 Native Input Injection (Atomic SendInput Chords)    │
│  - WASAPI Direct Audio Engine (IMMDeviceEnumerator)          │
│  - Window Focus Profile Auto-Switcher (GetForegroundWindow)  │
│  - Zeroconf mDNS Broadcaster (_streamdeck._tcp.local.)       │
└──────────────────────────────▲───────────────────────────────┘
                               │
                          IPC / Spawn
                               │
┌──────────────────────────────┴───────────────────────────────┐
│              Desktop Configurator (desktop-gui)              │
│                     Tauri v2 + Vite + Web                    │
│  - Embedded Webview GUI (Zero 404 dependencies)              │
│  - Real-time Host Engine status & network SSID display       │
│  - Background daemon lifecycle management                    │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. Component Directory Structure

```text
StreamDeckClone/
├── .agents/                               # Engineering Standards & Automation
│   ├── engineering_standards_android.md   # Android/KMP/CMP Master Standards
│   ├── engineering_standards_desktop.md   # Rust & Tauri Desktop Master Standards
│   ├── hooks.json                         # Antigravity tool interception rules
│   └── scripts/check_line_limit.py        # Real-time line count & hygiene guard
├── android/                               # Android Native Client
│   ├── app/src/main/
│   │   ├── java/com/streamdeck/client/
│   │   │   ├── data/net/                  # DeckWebSocketClient, DeckDiscoveryService
│   │   │   └── presentation/deck/         # DeckScreen, ConnectionScreen, DeckViewModel
│   │   └── res/values/                    # strings.xml, themes, colors (zero magic values)
│   └── app/build.gradle.kts               # Android app module
├── gradle/                                # Single version catalog and Gradle wrapper
├── settings.gradle.kts                    # Includes android/app as :app
├── build.gradle.kts                       # checkLineBudget verification task
├── gradlew.bat                            # Run Gradle from the repository root
├── host-desktop/                          # Native Windows Rust Host Engine
│   ├── src/
│   │   ├── main.rs                        # Main entry & tracing initializer
│   │   ├── server.rs                      # WebSocket accept loop & message dispatch
│   │   ├── input.rs                       # Win32 SendInput atomic hotkey executor
│   │   ├── audio.rs                       # WASAPI master volume / mute controller
│   │   ├── profiles.rs                    # Active window polling auto-profile switcher
│   │   ├── mdns.rs                        # mDNS service announcer
│   │   └── layout.rs                      # Default 3x3 key matrix configuration
│   └── Cargo.toml                         # Windows crate features & dependencies
├── desktop-gui/                           # Tauri v2 Desktop Configurator
│   ├── src-tauri/                         # Rust Tauri v2 core & IPC commands
│   ├── src/                               # Web frontend (Vite, CSS, JS)
│   └── dist/                              # Embedded web assets (standalone release)
├── proto/                                 # Single Source of Truth
│   └── streamdeck.proto                   # Protobuf definitions for DeckMessage & Events
├── run.ps1                                # 1-Command All-In-One Launcher (Admin, Local)
├── run-tailscale.ps1                      # 1-Command Tailscale Remote Launcher (Admin, Remote)
├── run-tailscale.bat                      # Double-Click Tailscale Remote Launcher
├── start-host.bat                         # Host Engine Launcher (Admin)
└── start-gui.bat                          # Desktop GUI Launcher
```

---

## 3. Communication Protocol (`proto/streamdeck.proto`)

Communication is bidirectional using binary Protocol Buffers over WebSocket:

- **Handshake (`HandshakeRequest` / `HandshakeResponse`)**: A new phone sends the current pairing code from the desktop app; the host remembers that phone for later connections. Removing the phone from the desktop app also disconnects its active session.
- **Heartbeat (`Heartbeat`)**: Sent every 5 seconds to track latency and keep connection alive.
- **Key Event (`KeyEvent`)**:
  - `key_index`: Slot index (0–8 in 3x3 default grid).
  - `event_type`: `DOWN` (0) or `UP` (1).
- **Layout Update (`PageLayoutUpdate`)**: Sent from PC host to Android client with slot titles, background colors, and badge texts.
- **Profile Switch (`ProfileSwitchEvent`)**: Emitted when PC detects an application switch (VS Code, Chrome, etc.).

---

## 4. Default Key Action Layout (3x3 Grid)

| Slot | Title | Default Action | Implementation |
| :---: | :--- | :--- | :--- |
| **0** | **Mute Audio** | Toggle Master Mute | WASAPI COM Endpoint + `VK_VOLUME_MUTE` |
| **1** | **Vol Down** | Lower Master Volume 5% | WASAPI COM Endpoint + `VK_VOLUME_DOWN` |
| **2** | **Vol Up** | Raise Master Volume 5% | WASAPI COM Endpoint + `VK_VOLUME_UP` |
| **3** | **Play / Pause** | Media Play/Pause | Win32 `VK_MEDIA_PLAY_PAUSE` |
| **4** | **Prev Track** | Previous Track | Win32 `VK_MEDIA_PREV_TRACK` |
| **5** | **Next Track** | Next Track | Win32 `VK_MEDIA_NEXT_TRACK` |
| **6** | **Desktop** | Show/Hide Desktop | Atomic `SendInput` (`Win + D`) |
| **7** | **Task Mgr** | Open Task Manager | Atomic `SendInput` (`Ctrl + Shift + Esc`) |
| **8** | **Screenshot** | Modern Screen Snipping | Atomic `SendInput` (`Win + Shift + S`) |

---

## 5. Network Discovery & Connectivity

1. **Auto-Discovery**:
   - **mDNS Broadcaster**: Host announces `_streamdeck._tcp.local.` on port `4455`.
   - **Subnet Port Scanner**: If router drops multicast packets (AP isolation), the Android app concurrently scans its local subnet (`192.168.x.x`) on TCP port `4455`, discovering the host within 300ms.
2. **Manual IP Connection**:
   - Enter `host_ip:4455`, the six-digit code, and the certificate SHA-256 fingerprint shown under desktop **Connection details**. Compare the fingerprint on the trusted PC before connecting. The phone remembers the encrypted credential and pin for reconnecting.
3. **USB Mode (ADB Reverse)**:
   - When plugged in via USB: `adb reverse tcp:4455 tcp:4455`.
   - Tap **"Use USB (127.0.0.1)"** and use the same certificate fingerprint; USB traffic is pinned WSS too.

---

## 6. How to Build & Run (Runbook)

### Recommended: start and stop from the desktop app
Build the host and desktop app as below, then open the desktop configurator. It shows **Host stopped** until you press **Start Host** (unless a host is already running). Closing the desktop window sends the app to the tray; **Stop Host** or tray **Quit** stops only a host the desktop app started. A host started separately, especially as Administrator, remains owned by its original launcher and cannot be stopped by the desktop app. The connection controls are under **Connection settings**; pairing and editing buttons are the primary workflow. The desktop app embeds its frontend in release builds and does not run a separate Vite web server; `tauri dev` starts a Vite server only for development.

### All-in-one development launcher (Local Wi-Fi / USB):
Connect one Android device (or set `ANDROID_SERIAL` if several are connected), then run from PowerShell:
```powershell
.\run.ps1
```
The script requests Administrator access for Windows input injection, rebuilds the Windows host and GUI in release mode, builds and tests the Android debug app, installs and opens it on the selected device, configures ADB reverse when available, and starts both Windows programs. It stops on a build failure and will not replace an already running host. With no device, it builds Android and starts the desktop programs; connect a device and rerun to install it. This separately launched host is external to the GUI, so **Stop Host** cannot stop it; close the host process yourself.

### Tailscale remote launcher (Remote from Work):
When remoting into your home/work PC over Tailscale:
```powershell
.\run-tailscale.ps1
# or double-click run-tailscale.bat
```
This launcher:
1. **Detects the PC's Tailscale IPv4 address** (`100.x.y.z`) automatically and sets `STREAMDECK_HOST_IP`.
2. **Rebuilds host & desktop GUI** in release mode with Tailscale IP configuration and QR payloads.
3. **Builds Android debug APK** (`checkLineBudget`, unit tests, `assembleDebug`).
4. **Auto-discovers Android peers** via Tailscale or active network ADB (`<tailscale-ip>:5555`).
5. **Configures ADB reverse port forwarding** (`tcp:4455 tcp:4455`) over the network tunnel.
6. **Launches the Android app with Intent extras** (`--es host_address "$TailscaleIp:4455" --es fingerprint "$Fingerprint"`), automatically filling in your Tailscale connection parameters so you don't need to manually type a 64-character SHA-256 fingerprint on mobile.
7. **Displays a summary banner** with the Tailscale address, port, and fingerprint for quick verification.

### Manual Step-by-Step:
1. **Build Host Engine**:
   ```powershell
   cd host-desktop
   cargo build --release
   ```
2. **Build Desktop GUI**:
   ```powershell
   cd desktop-gui
   npm run build
   cargo build --release --manifest-path src-tauri/Cargo.toml
   ```
3. **Run Android App**:
   ```powershell
   # From the repository root:
   .\gradlew.bat checkLineBudget :app:testDebugUnitTest :app:assembleDebug
   adb install -r android/app/build/outputs/apk/debug/app-debug.apk
   ```
4. **Start the host**:
   Open the desktop app and press **Start Host**. Use `start-host.bat` only if you need a separately elevated host; that host cannot be stopped from the desktop app.

---

## 7. Desktop Configurator Profiles

The desktop configurator reads profiles from the host on connection. Save/Clear sends a validated nine-key profile to the host, which persists it in the user's configuration directory (`StreamDeckClone/config.json`) and broadcasts changes to connected Android decks when that profile is active. Foreground-window detection switches between Default, Browser, VSCode, OBS, and VisualStudio. The configurator must connect locally; remote LAN clients cannot edit profiles. Actions include volume and media controls, desktop/task-manager/screenshot shortcuts, Ctrl+C/V/Z/S/W, and F5. Empty actions do nothing. A malformed configuration file is left untouched and the host uses defaults until corrected. Profile saves reject unsupported actions or overlong fields instead of silently changing them.

The editor supports drag-and-drop key swapping as well as an accessible keyboard alternative (select key, press 'M', select target key, Esc to cancel). Key swaps save persistently to host profiles. The host stores profile schema version 1; when it reads an older unversioned profile file, it preserves the original as `config.legacy.json` before migrating. A corrupt, unsupported, or conflicting backup file causes startup to fail rather than replacing saved profiles. Keep both files for manual repair; do not delete the backup without checking its contents. Release builds launch without terminal windows (`windows_subsystem = "windows"` and `CREATE_NO_WINDOW`), and the GUI layout spans full height.

**Pair a phone:** Click **Pair my phone** in the desktop app and scan its QR code (`streamdeck-pair:v2:<ip>:<port>:<code>:<certificate-sha256>`) or manually enter the address, six-digit code, and certificate fingerprint shown under **Connection details**. Verify the fingerprint on the trusted PC. Codes expire after five minutes or ten failed attempts. Remove a paired phone under **Connected phones** to revoke access.

**Secure-by-default migration:** Old v1 QR codes and unencrypted phone WebSockets are not supported. Existing phones must re-pair. Host certificate regeneration changes the fingerprint: verify the new one on the PC and re-pair rather than accepting a silent rotation. Phone credentials are encrypted using Android Keystore; host device tokens are hashed on disk. The desktop control socket is bound to `127.0.0.1:4456` and Tauri's native layer supplies the local control secret. A separately launched host works only when it uses the same user's accessible host identity; an elevated or different-user host may fail authentication and must be restarted under the correct user. Do not expose either port to the internet. Real phone-to-PC security verification is still pending.

---

## 8. Engineering Standards & Quality Gates

Track implementation in [.agents/roadmap.md](.agents/roadmap.md) and separate automated versus user-run checks in [.agents/testing-results.md](.agents/testing-results.md). Update both after each meaningful phase.

All development adheres strictly to the rules in `.agents/`:
- **Line Budgets**: Max 250 lines per file (hard ceiling 300), max 40 lines per function, max 150 lines per ViewModel.
- **Android Quality**: Run `./gradlew.bat checkLineBudget` from the repository root. Zero hardcoded strings, zero raw hex colors in UI.
- **Rust Quality**: `cargo clippy -- -D warnings` must pass with 0 errors and 0 warnings.
- **Input Injection**: Atomic chords via `SendInput` with `wScan: 0` running in an elevated interactive session.
