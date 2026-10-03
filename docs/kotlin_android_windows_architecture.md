# Architecture Specification: Kotlin Android Client & Windows Desktop Host

This document specifies the technical architecture for a **Stream Deck clone** built natively with **Kotlin (Jetpack Compose) for Android (Phone & Tablet)** and a companion **Windows Desktop Host**.

---

## 1. High-Level System Architecture

```mermaid
flowchart TB
    subgraph AndroidClient ["Android Controller (:androidApp)"]
        direction TB
        ImmersiveUI["Jetpack Compose UI (Adaptive Grid)"]
        Haptics["Haptic Feedback & Sound Engine"]
        CoilCache["Coil Image Cache (GIF / WebP / SVG)"]
        NSD["Android NsdManager (mDNS Discovery)"]
        NetClient["Ktor / OkHttp WebSocket Client"]
        LocalStore["DataStore / SQLite (Recent Hosts & Offline Cache)"]

        ImmersiveUI <--> Haptics
        ImmersiveUI <--> CoilCache
        ImmersiveUI <--> NetClient
        NSD --> NetClient
        NetClient <--> LocalStore
    end

    subgraph SharedLib [":shared (Kotlin Multiplatform / Common Code)"]
        PacketModels["Protocol Data Models (kotlinx.serialization)"]
        EventEnums["KeyEvents, ActionTypes, TileStates"]
        Validation["Auth Handshake & Token Validation"]
    end

    subgraph Transport ["Transport Layer (LAN or USB)"]
        mDNS["mDNS (_streamdeck._tcp.local)"]
        WS["WebSocket (Port 4455 / MessagePack or JSON)"]
        USB["ADB Reverse Port Forward (adb reverse tcp:4455 tcp:4455)"]
    end

    subgraph WindowsHost ["Windows Desktop Host"]
        direction TB
        HostServer["WebSocket Host Server (Ktor CIO or .NET Kestrel)"]
        ProfileEngine["Profile & Page Coordinator"]
        WinHook["WinEventHook (Foreground Window Tracker)"]
        WinInput["Windows SendInput / Virtual Key Driver"]
        WASAPI["WASAPI Core Audio (Volume & Mute Control)"]
        PluginBridge["Integration Hub (OBS WebSocket, Spotify, Discord)"]
        TrayUI["System Tray & Config Window"]

        HostServer <--> ProfileEngine
        ProfileEngine <--> WinHook
        ProfileEngine <--> WinInput
        ProfileEngine <--> WASAPI
        ProfileEngine <--> PluginBridge
        TrayUI <--> ProfileEngine
    end

    AndroidClient -.-> SharedLib
    WindowsHost -.-> SharedLib
    AndroidClient <==> WS <==> WindowsHost
    AndroidClient -.-> mDNS -.-> WindowsHost
    AndroidClient <..> USB <..> WindowsHost
```

---

## 2. Android Client Architecture (`:androidApp`)

The Android application targets both smartphones and tablets in horizontal or vertical orientations.

### 2.1 UI Layer: Jetpack Compose & Adaptive Design

| Device Form Factor | Typical Layout Grid | Navigation & Controls |
| :--- | :--- | :--- |
| **Compact Phone (Portrait)** | 3 cols × 5 rows | Bottom navigation bar / paging swipe. |
| **Phone (Landscape)** | 5 cols × 3 rows | Nav rail on left edge; maximized grid area. |
| **Tablet (Landscape)** | 8 cols × 4 rows (or 10×5) | Multi-pane: Left nav rail for profiles/pages, main area for keys, right pane for live audio dials. |
| **Foldables** | Responsive column count using `WindowSizeClass` (`calculateWindowSizeClass(activity)`). |

#### Critical Android Configurations:
- **Immersive Full-Screen Mode:** Hide system bars (status & navigation) and keep screen permanently active while running:
  ```kotlin
  // Keep screen on
  window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

  // Edge-to-edge full immersion
  WindowCompat.setDecorFitsSystemWindows(window, false)
  val controller = WindowCompat.getInsetsController(window, window.decorView)
  controller.hide(WindowInsetsCompat.Type.systemBars())
  controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
  ```
- **Adaptive Grid Implementation:**
  ```kotlin
  @Composable
  fun StreamDeckGrid(
      keys: List<DeckKey>,
      columns: Int,
      rows: Int,
      onKeyPress: (keyId: String, event: KeyEventType) -> Unit,
      modifier: Modifier = Modifier
  ) {
      BoxWithConstraints(modifier = modifier.fillMaxSize().padding(8.dp)) {
          val tileWidth = (maxWidth - (12.dp * (columns - 1))) / columns
          val tileHeight = (maxHeight - (12.dp * (rows - 1))) / rows
          
          LazyVerticalGrid(
              columns = GridCells.Fixed(columns),
              userScrollEnabled = false,
              horizontalArrangement = Arrangement.spacedBy(12.dp),
              verticalArrangement = Arrangement.spacedBy(12.dp)
          ) {
              items(keys, key = { it.id }) { key ->
                  DeckKeyTile(
                      key = key,
                      modifier = Modifier.size(tileWidth, tileHeight),
                      onPress = { onKeyPress(key.id, KeyEventType.DOWN) },
                      onRelease = { onKeyPress(key.id, KeyEventType.UP) }
                  )
              }
          }
      }
  }
  ```
- **Low-Latency Touch Handling & Haptics:**
  Use `pointerInput` with `awaitPointerEventScope` to trigger immediately on `PointerEventType.Press` (not waiting for tap release), triggering `HapticFeedbackType.LongPress` or `LocalHapticFeedback.current.performHapticFeedback(HapticFeedbackType.TextHandleMove)`.
- **Dynamic Asset Rendering:** Use **Coil 3** (Kotlin Multiplatform / Android) with `GifDecoder` and `SvgDecoder` for animated icon packs.

### 2.2 Local Discovery via `NsdManager`
Using Android's native Network Service Discovery (Zeroconf / mDNS) to discover the Windows PC:
```kotlin
class HostDiscoveryManager(private val context: Context) {
    private val nsdManager = context.getSystemService(Context.NSD_SERVICE) as NsdManager

    fun startDiscovery(onHostFound: (host: String, port: Int) -> Unit) {
        val listener = object : NsdManager.DiscoveryListener {
            override fun onServiceFound(serviceInfo: NsdServiceInfo) {
                if (serviceInfo.serviceType == "_streamdeck._tcp.") {
                    nsdManager.resolveService(serviceInfo, object : NsdManager.ResolveListener {
                        override fun onServiceResolved(resolved: NsdServiceInfo) {
                            onHostFound(resolved.host.hostAddress, resolved.port)
                        }
                        override fun onResolveFailed(info: NsdServiceInfo, errorCode: Int) {}
                    })
                }
            }
            // ... other lifecycle callbacks
        }
        nsdManager.discoverServices("_streamdeck._tcp.", NsdManager.PROTOCOL_DNS_SD, listener)
    }
}
```

---

## 3. Windows Desktop Host Architecture

For the Windows Host, two strong tech stacks stand out:

### Tech Stack Comparison for Windows

| Feature | **Option 1: Kotlin (Compose Multiplatform Desktop / JVM)** | **Option 2: C# .NET 8 (WPF / WinUI 3)** *(Best Native)* |
| :--- | :--- | :--- |
| **Code Sharing** | **100% shared protocol, models, and WebSocket engine** with Android. | Separate C# codebase, shared protocol defined via JSON schema/Protobuf. |
| **Win32 Input (SendInput)** | Via **JNA** (Java Native Access) or Java 22 **FFM** (Foreign Function & Memory). | Native `[DllImport("user32.dll")]` or CsWin32 source generator. |
| **WASAPI Audio Control** | Requires JNA COM bindings. | Native support via `NAudio.CoreAudioApi` (direct per-app audio mixer access). |
| **System Tray & Footprint** | System tray supported; ~80–120 MB RAM (JVM). | Native Windows tray, single-file native AOT; ~35–50 MB RAM. |
| **Dev Speed** | High (if developer is primarily focused on Kotlin). | High (native Windows ecosystem tooling). |

> **Recommendation:**
> - If you want **one single language (Kotlin)** across the entire repository with a Gradle multi-project build (`:androidApp`, `:desktopApp`, `:shared`), use **Compose Multiplatform Desktop (JVM)**.
> - If you need **direct, deep Windows audio/process hooks** with minimal native bridging hassle, build the host in **C# .NET 8**.

---

## 4. Windows Native OS Integrations (Deep Dive)

### 4.1 Simulating Keystrokes & Hotkeys (`SendInput`)
Standard `keybd_event` is deprecated. Windows requires `SendInput` with `INPUT` structures containing `KEYBDINPUT`:

```cpp
// Windows C++ / Win32 concept (mirrored in JNA or P/Invoke)
void SendKeyPress(WORD vkCode, bool keyUp) {
    INPUT input = {0};
    input.type = INPUT_KEYBOARD;
    input.ki.wVk = vkCode;
    input.ki.dwFlags = keyUp ? KEYEVENTF_KEYUP : 0;
    SendInput(1, &input, sizeof(INPUT));
}
```
*Note on Admin/Game restrictions:* To inject keys into elevated games or programs running as Administrator, the Windows Host application manifest must include:
`<requestedExecutionLevel level="asInvoker" uiAccess="true" />` (requires signing) OR the app must be started as Administrator.

### 4.2 Foreground Window Tracking (Automatic Profile Switch)
Use `SetWinEventHook` to listen to window focus changes without polling:
```csharp
// Windows Win32 Hook
HWINEVENTHOOK hHook = SetWinEventHook(
    EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND,
    IntPtr.Zero, WinEventDelegate, 0, 0,
    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
);

void WinEventDelegate(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime) {
    GetWindowThreadProcessId(hwnd, out uint processId);
    Process proc = Process.GetProcessById((int)processId);
    // e.g. proc.ProcessName == "Photoshop" -> switch client to "photoshop_profile"
    ProfileManager.ActivateProfileForProcess(proc.ProcessName);
}
```

### 4.3 Audio & Application Volume Mixer (WASAPI)
Allows creating dial buttons or volume sliders for specific apps (e.g., Discord or Spotify):
- Access the `IAudioSessionManager2` from the default audio device (`IMMDeviceEnumerator`).
- Enumerate audio sessions (`IAudioSessionEnumerator`) to find sessions matching target process IDs.
- Call `ISimpleAudioVolume::SetMasterVolume` or `SetMute`.

---

## 5. Shared Codebase & Protocol Specification (`:shared`)

Using Kotlin Multiplatform (`kmp`) with `kotlinx.serialization`:

### 5.1 Protocol Message Models
```kotlin
package com.streamdeck.protocol

import kotlinx.serialization.Serializable

@Serializable
sealed class DeckMessage {
    @Serializable
    data class Handshake(
        val deviceName: String,
        val deviceModel: String,
        val appVersion: String,
        val authToken: String? = null
    ) : DeckMessage()

    @Serializable
    data class KeyEvent(
        val profileId: String,
        val pageId: String,
        val keyIndex: Int,
        val type: EventType // DOWN, UP, LONG_PRESS
    ) : DeckMessage()

    @Serializable
    data class PageLayoutUpdate(
        val profileId: String,
        val pageId: String,
        val columns: Int,
        val rows: Int,
        val keys: List<KeySlotConfig>
    ) : DeckMessage()

    @Serializable
    data class KeyStateUpdate(
        val keyIndex: Int,
        val label: String? = null,
        val badgeText: String? = null,
        val badgeColor: String? = null,
        val state: Int = 0, // Toggle 0/1
        val iconChecksum: String? = null
    ) : DeckMessage()
}

@Serializable
enum class EventType { DOWN, UP, LONG_PRESS, DIAL_ROTATE }

@Serializable
data class KeySlotConfig(
    val index: Int,
    val title: String,
    val iconUrl: String?,
    val backgroundColor: String? = null,
    val hasLiveFeedback: Boolean = false
)
```

---

## 6. Zero-Lag USB Tethering Support (ADB Reverse)

For esports streamers, low-latency gaming, or venues without stable Wi-Fi:
1. Connect Android phone/tablet to PC via USB cable.
2. Enable USB Debugging on Android.
3. Windows Host automatically runs (via embedded ADB command):
   ```cmd
   adb reverse tcp:4455 tcp:4455
   ```
4. The Android app connects directly to `ws://localhost:4455`, achieving **< 1ms round-trip latency** with zero wireless packet drops.

---

## 7. Project Structure & Gradle Setup

```
streamdeck-project/
├── build.gradle.kts
├── settings.gradle.kts
│
├── shared/                         # Kotlin Multiplatform Module
│   ├── build.gradle.kts
│   └── src/commonMain/kotlin/      # Shared protocol, serialization, models
│
├── androidApp/                     # Android Phone & Tablet App
│   ├── build.gradle.kts
│   └── src/main/
│       ├── AndroidManifest.xml
│       └── kotlin/com/streamdeck/client/
│           ├── ui/                 # Jetpack Compose Screens (Adaptive Grid, KeyViews)
│           ├── discovery/          # NsdManager Discovery & QR Scanner
│           ├── net/                # Ktor / OkHttp WebSocket Client
│           └── viewmodel/          # StateFlow & DeckViewModel
│
└── desktopHost/                    # Windows Desktop (Compose Desktop or C# Host)
    ├── build.gradle.kts
    └── src/main/kotlin/com/streamdeck/host/
        ├── net/                    # Ktor WebSocket Server & mDNS Announcer
        ├── win32/                  # JNA SendInput, WinEventHook, WASAPI bindings
        ├── profiles/               # Profile storage & Window Switcher
        ├── plugins/                # OBS-WebSocket & Spotify Connectors
        └── ui/                     # Desktop Configurator (Tray & Setup GUI)
```
