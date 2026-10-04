# Task: Rebuild StreamDeck Desktop GUI with Svelte 5, Fluent CSS Tokens & Tauri v2

## 1. Objective
Refactor and upgrade the desktop configurator (`desktop-gui`) in `d:\Android\Projects\StreamDeckClone\desktop-gui` using **Svelte 5 (Runes)** and a custom **Windows 11 Fluent CSS Design System**. The resulting desktop app must look, feel, and behave indistinguishably from a native Windows 11 .NET (WinUI 3) system utility (like Windows 11 Settings), while maintaining an ultra-lightweight memory footprint (**<25 MB RAM**).

Follow all rules in `.agents/engineering_standards_desktop.md` strictly:
- Max 250 lines per file (hard ceiling 300).
- Max 40 lines per function.
- Rust code must be `cargo clippy -- -D warnings` clean.
- Embedded assets release build: zero 404 dependencies.

---

## 2. Technical Stack & Architecture

- **Frontend**: Svelte 5 (`svelte@^5.0.0`) + Vite (`vite@^6.0.0`) using modern Runes (`$state`, `$derived`, `$effect`).
- **Styling**: Windows 11 Fluent 2 CSS Design Tokens (`fluent.css`) with translucent elevation surfaces.
- **Desktop Runtime**: Tauri v2 (`@tauri-apps/api@^2.0.0`, `@tauri-apps/plugin-shell@^2.0.0`).
- **OS Window FX**: `window-vibrancy` crate for native Windows 11 DWM **Mica** frosted glass.
- **Process & Tray Integration**: Native System Tray with minimize-to-tray lifecycle management.

---

## 3. Step-by-Step Implementation Plan

### Step 1: Scaffold Svelte 5 in `desktop-gui`
1. Update `desktop-gui/package.json`:
   - Add `@sveltejs/vite-plugin-svelte: ^4.0.0` and `svelte: ^5.0.0` to `devDependencies`.
   - Remove any legacy unused packages.
2. Configure `desktop-gui/vite.config.js`:
   - Import `defineConfig` from `vite` and `svelte` from `@sveltejs/vite-plugin-svelte`.
   - Configure server port 1420 (strictPort: true).
3. Update `desktop-gui/index.html` to mount Svelte 5 root `<div id="app"></div>` and load `/src/main.js`.
4. Update `desktop-gui/src/main.js` using Svelte 5's `mount(App, { target: document.getElementById('app') })`.

---

### Step 2: Implement Windows 11 Fluent CSS Tokens (`src/styles/fluent.css`)
Create a dedicated design system stylesheet adhering to WinUI 3 specifications:

1. **Kill all Web Quirks**:
   - Universal `user-select: none;` (only `input, textarea` set to `user-select: text`).
   - Universal `-webkit-user-drag: none;` on images, buttons, and SVGs.
   - Body & HTML set to `background: transparent;` so Windows 11 Mica glass shines through.
   - Official Windows 11 typography stack: `font-family: "Segoe UI Variable Text", "Segoe UI Variable Display", "Segoe UI", system-ui, sans-serif;`.
   - Subpixel rendering: `-webkit-font-smoothing: antialiased; text-rendering: optimizeLegibility;`.

2. **Fluent Surface Elevations & Colors**:
   - Canvas: `rgba(32, 32, 32, 0.75)`
   - Cards / Containers: `rgba(255, 255, 255, 0.05)` with `1px solid rgba(255, 255, 255, 0.08)` border.
   - Card Hover: `rgba(255, 255, 255, 0.08)`
   - Control Buttons: `rgba(255, 255, 255, 0.06)` with `1px solid rgba(255, 255, 255, 0.12)` border.
   - Accent Blue: `#0078d4` (Hover: `#1084d9`)
   - Radii: `4px` for interactive controls, `8px` for surface cards.

3. **Fluent Micro-Interactions (Spring Press)**:
   - On `:active`, scale buttons and key tiles by `transform: scale(0.97)` with `transition: transform 70ms cubic-bezier(0, 0, 0, 1)`.

4. **Auto-Hide Native Scrollbars**:
   - Style scrollbars with ultra-thin 4px rounded thumbs matching Windows 11 specs.

---

### Step 3: Build Svelte 5 UI Components (`src/App.svelte` & components)
Break the UI into clean sub-components (each under 100 lines):

1. **`App.svelte`** (Main Orchestrator):
   - Global right-click context menu interceptor (`oncontextmenu = (e) => e.preventDefault()`).
   - Svelte 5 Runes for reactive state:
     - `let hostRunning = $state(false);`
     - `let networkInfo = $state({ ip: '127.0.0.1', wifi: null, port: 4455 });`
     - `let selectedKeyIndex = $state(0);`
     - `let keys = $state([...]);`
   - Polls Tauri IPC for host engine status.

2. **`components/TitleBar.svelte`**:
   - Windows 11 header bar showing app title, Wi-Fi badge, and Host Engine status pill (`Running` / `Stopped`).
   - Button to start or restart Host Engine via Rust `launch_host`.

3. **`components/DeckMatrix.svelte`**:
   - 3x3 interactive Key Matrix matching the Android client grid.
   - Shows badge text (`Win+D`, `Win+Shift+S`, etc.), slot title, and background colors.
   - Click to select slot for editing in the side panel.

4. **`components/SlotEditor.svelte`**:
   - Fluent side panel to customize title, badge text, background color, and assigned Win32 hotkey action.

---

### Step 4: Native Windows 11 Mica & System Tray in Tauri Rust Core
1. In `desktop-gui/src-tauri/Cargo.toml`, add:
   - `window-vibrancy = "0.5"`
2. In `desktop-gui/src-tauri/tauri.conf.json`:
   - Set `"transparent": true` on the `main` window.
   - Keep `"decorations": true` to retain Windows 11 native titlebar caption buttons and Snap Layouts.
3. In `desktop-gui/src-tauri/src/lib.rs`:
   - In `setup()`, call `window_vibrancy::apply_mica(&window, Some(true))` for dark mode Mica glass.
   - Add a native **System Tray** (`tauri::tray::TrayIconBuilder`) with menu:
     - "Open Configurator"
     - "Host Status"
     - "Restart Host Engine"
     - "Quit"
   - Intercept `CloseRequested` window event to hide the window to tray instead of killing the process, keeping background hotkey execution alive.

---

## 4. Verification & Validation Steps
1. In `desktop-gui`: run `npm install`, then `npm run build` — must build to `dist/` with zero errors.
2. In `desktop-gui/src-tauri`: run `cargo clippy -- -D warnings` — must pass with 0 errors and 0 warnings.
3. Build the release binary:
   - `cargo build --release --manifest-path desktop-gui/src-tauri/Cargo.toml`
4. Verify execution:
   - Run `streamdeck-gui.exe`.
   - Verify native Windows 11 frosted Mica glass backdrop.
   - Right-click anywhere — browser inspect element menu must NOT appear.
   - Verify tactile Fluent press physics on buttons.
   - Verify memory usage in Task Manager is below **25 MB RAM**.
   - Verify closing the window minimizes to tray and keeps host engine running.
