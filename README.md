# StreamDeck Clone: Full System Documentation

A high-performance, ultra-low latency, cross-platform Stream Deck system connecting a mobile controller to a Windows workstation over local Wi-Fi and high-speed USB tethering.

---

## 1. System Architecture Overview

```text
┌──────────────────────────────────────────────────────────────┐
│                    Android Mobile Client                     │
│         Kotlin 2.1 • Jetpack Compose • Material 3            │
│  - Adaptive Key Matrix (3x3 Grid, Custom Colors & Badges)    │
│  - Hybrid Discovery: mDNS (NsdManager) + TCP Subnet Scanner  │
│  - WebSocket Client with Protocol Buffers Serialization      │
└──────────────────────────────┬───────────────────────────────┘
                               │
            WebSocket (Protobuf) over Port 4455
            • Local Wi-Fi (e.g. ws://192.168.110.7:4455)
            • ADB Reverse USB (ws://127.0.0.1:4455)
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
│   └── build.gradle.kts                   # checkLineBudget verification task
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
├── run.ps1                                # 1-Command All-In-One Launcher (Admin)
├── start-host.bat                         # Host Engine Launcher (Admin)
└── start-gui.bat                          # Desktop GUI Launcher
```

---

## 3. Communication Protocol (`proto/streamdeck.proto`)

Communication is bidirectional using binary Protocol Buffers over WebSocket:

- **Handshake (`HandshakeRequest` / `HandshakeResponse`)**: Exposes client device details and protocol version compatibility.
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
   - Users can manually type `host_ip:4455` and tap **Connect**. The app stores the address and automatically reconnects on future launches.
3. **USB Mode (ADB Reverse)**:
   - When plugged in via USB: `adb reverse tcp:4455 tcp:4455`.
   - Tap **"Use USB (127.0.0.1)"** for zero-latency, router-free operation.

---

## 6. How to Build & Run (Runbook)

### 1-Command All-In-One Launcher:
Open PowerShell and run:
```powershell
.\run.ps1
```
*Auto-elevates as Administrator, configures ADB reverse, builds release binaries if needed, and launches both the Host Engine and Desktop GUI.*

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
   cd android
   .\gradlew.bat checkLineBudget assembleDebug
   adb install -r app/build/outputs/apk/debug/app-debug.apk
   ```
4. **Start Host as Administrator**:
   Double-click `start-host.bat`.

---

## 7. Engineering Standards & Quality Gates

All development adheres strictly to the rules in `.agents/`:
- **Line Budgets**: Max 250 lines per file (hard ceiling 300), max 40 lines per function, max 150 lines per ViewModel.
- **Android Quality**: `./gradlew.bat checkLineBudget` must pass. Zero hardcoded strings, zero raw hex colors in UI.
- **Rust Quality**: `cargo clippy -- -D warnings` must pass with 0 errors and 0 warnings.
- **Input Injection**: Atomic chords via `SendInput` with `wScan: 0` running in an elevated interactive session.
