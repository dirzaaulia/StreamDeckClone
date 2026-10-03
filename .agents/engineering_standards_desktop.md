# Master Agent Instructions: Desktop App (Rust & Tauri v2) Engineering Standards (2026 Edition)

You are an expert Principal Systems Architect and Senior Windows / Rust / Tauri Engineer. You adhere strictly to modern 2026 Rust idioms, Win32 API native automation, Tauri v2 security practices, Clean Architecture, and strict line budget limits.

Every action, architectural recommendation, and code artifact for `host-desktop` and `desktop-gui` must conform to the following protocols.

---

## 1. Core Architecture & Modern Tech Stack Matrix (2026 Standards)

| Component | Technology | Version / Standard | Architectural Role |
| :--- | :--- | :--- | :--- |
| **Host Engine (`host-desktop`)** | **Rust (Edition 2024 / 1.85+)** | Tokio 1.43+, `windows` 0.58+ | Zero-overhead native Windows daemon. Handles Protobuf over WebSocket (`tokio-tungstenite`), native Win32 input injection (`SendInput`), WASAPI direct audio (`IMMDeviceEnumerator`), and mDNS broadcasting. |
| **Configurator GUI (`desktop-gui`)** | **Tauri v2 + Vite 6+** | `@tauri-apps/api` 2.2+, WebView2 | High-performance desktop configurator with embedded web frontend. Standalone release binary with zero external web dependencies and zero 404 errors. |
| **Wire Protocol (`proto/`)** | **Protocol Buffers v3** | `prost` 0.13+ / `prost-build` | Strongly-typed, cross-platform wire format shared between Android (Kotlin) and PC (Rust). Precompiled via `build.rs`. |
| **System Audio Controller** | **Windows WASAPI** | `IMMDeviceEnumerator`, `IAudioEndpointVolume` | Hardware-direct master audio volume manipulation and mute toggling via COM without keyboard input emulation lag. |
| **Automation & Launchers** | **PowerShell & Batch** | `run.ps1` (Admin) & `start-host.bat` | Automated UAC elevation, ADB reverse port forwarding (`4455`), release compilation, and process lifecycle management. |

---

## 2. Hard Code Constraints & Line Budgets

> [!CAUTION]
> **CRITICAL ENFORCEMENT RULE**: Large, monolithic files and monster functions are **strictly prohibited**. AI agents must not continually append lines to files.

### Strict Size Limits:
- **Maximum File Length**: **250 lines** (Hard ceiling: **300 lines** including imports and comments).
- **Maximum Function Length**: **40 lines**. If a function exceeds 40 lines, decompose it into private helper functions.
- **Mandatory Line Budget Receipt**: Every Rust source file must start with a line budget audit header:
  ```rust
  // [LINE BUDGET AUDIT] <current_lines>/250
  ```
- **Line Width**: Max **100–120 characters**. Wrap parameters vertically with trailing commas.

---

## 3. Rust Quality, Static Analysis & Error Handling Standards

1. **Clippy Clean Invariant**:
   - `cargo clippy -- -D warnings` **must pass with zero errors and zero warnings**.
   - No dead code, unused imports, or redundant closures permitted.
2. **Defensive Error Handling**:
   - **No `unwrap()` or `expect()` in production runtime paths.**
   - Use `Result<T, E>` with `thiserror` for library domain errors or `anyhow::Result` in top-level binaries.
   - Use `tracing::info!`, `tracing::warn!`, and `tracing::error!` for all logging with `EnvFilter`.
3. **Async Runtime Discipline**:
   - Use **Tokio** (`#[tokio::main]`) for asynchronous networking.
   - Never execute blocking operations (like synchronous file I/O or long polling loops) directly on Tokio worker threads; offload to `tokio::task::spawn_blocking`.

---

## 4. Win32 Native Input & OS Automation Standards

1. **Atomic Key-Chord Injection**:
   - Multi-key hotkeys (e.g. `Win + D`, `Ctrl + Shift + Esc`, `Win + Shift + S`) must be dispatched in a **single atomic `SendInput` call**:
     ```rust
     let mut inputs = Vec::with_capacity(keys.len() * 2);
     for &vk in keys { inputs.push(Self::create_key_input(vk, false)); }
     for &vk in keys.iter().rev() { inputs.push(Self::create_key_input(vk, true)); }
     SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
     ```
   - **Never** inject artificial `thread::sleep` delays between chord presses that prevent the Windows shell from detecting hotkeys.
2. **Scan Code Discipline**:
   - Do **NOT** populate `wScan` with `MapVirtualKeyW` unless `KEYEVENTF_SCANCODE` is explicitly set in `dwFlags`.
   - Setting non-zero `wScan` on virtual keys without scan-code flags causes Windows UIPI to reject `SendInput` with `ERROR_ACCESS_DENIED (WIN32_ERROR 5)`.
   - Keep `wScan: 0` for pure virtual key simulation.
3. **Process Elevation & UIPI (User Interface Privilege Isolation)**:
   - Injecting input into elevated/system windows requires the host process to run with **Administrator privileges**.
   - Do not attempt to embed broken UAC manifests in MinGW GNU toolchains (`ld.exe: multiple non-default manifests`).
   - Use the PowerShell launcher (`run.ps1`) or `start-host.bat` with `Start-Process -Verb RunAs` to guarantee interactive session elevation.
4. **COM Apartment Initialization**:
   - Tokio worker threads are uninitialized by default. If calling Win32 Shell COM interfaces (e.g., `IShellDispatch4`), you **must** call:
     ```rust
     let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
     ```
   - Never assume COM is initialized on background worker threads.

---

## 5. Network, Protocol & Discovery Standards

1. **mDNS Service Broadcasting**:
   - Broadcast service type `_streamdeck._tcp.local.` using `mdns-sd`.
   - **Zero Vendor Hardcoding Policy**: Never hardcode specific adapter names (e.g. `"Tailscale"`, `"vEthernet"`). Follow standard RFC mDNS networking protocols.
   - The Android client employs a hybrid approach (mDNS + concurrent TCP port 4455 subnet scanner) to ensure 100% connectivity even across routers with AP isolation.
2. **WebSocket & Protobuf**:
   - WebSocket operates on port `4455`.
   - All payloads are serialized with Protocol Buffers (`DeckMessage`).
   - Keep network decode handlers fast and dispatch input actions immediately without blocking the receive loop.

---

## 6. Tauri v2 Desktop GUI Standards

1. **Zero-404 Embedded Assets Policy**:
   - The production GUI binary (`streamdeck-gui.exe`) must embed web assets from `dist/` at compile time via `tauri::generate_context!()`.
   - Always run `npm run build` in `desktop-gui/` before building the release binary.
2. **Host Process Coordination**:
   - The GUI queries running host status via `tasklist` and can start the host silently in the background if not already running.
   - Supports both `host-desktop/target/release/host-desktop.exe` and debug paths.
3. **Clean IPC**:
   - Use strongly-typed Tauri commands (`#[tauri::command]`) for all frontend-to-Rust communication (`host_pid`, `launch_host`, `get_network_info`).
