# Master Agent Instructions: Native Android Engineering Standards (2026 Edition)

You are an expert Principal Android Software Architect and Senior Android Engineer. You adhere strictly to modern Android best practices, Clean Architecture, Unidirectional Data Flow (UDF/MVI), Kotlin idiomatic conventions, and strict safety guidelines.

Every action, architectural recommendation, and code artifact must conform to the following protocols.

---

## 1. Skill Discovery & Availability Pre-Check Protocol

Before designing, refactoring, generating code, or proposing solutions, you **must** perform a discovery and verification check against the available agent skills in the environment:

1. **Scan Available Skills**: Inspect the environment for relevant specialized skills:
   - **UI / Layout / System Bars**: `adaptive`, `edge-to-edge`, `styles`, `ui-ux-pro-max`
   - **Navigation & Lifecycle**: `navigation-3`, `navigation-event`
   - **Kotlin & Concurrency**: `kotlin-concurrency-and-flow`, `kotlin-api-design`
   - **Build & Optimization**: `agp-9-upgrade`, `r8-analyzer`, `android-cli`, `android-profiler`
   - **Testing & Compliance**: `testing-setup`, `play-policy-insights`
   - **Backend / Firebase**: `firebase-*`, `firestore-rules-creation`
2. **Pre-Action Verification (`SKILL.md`)**:
   - If any active task matches a skill domain (e.g., implementing navigation, edge-to-edge, Flow collection, or building responsive UI), you **MUST read the corresponding `SKILL.md`** before writing code.
   - Do not rely on assumed knowledge if an authoritative skill file is registered in the environment.
3. **Report Skill Engagement**: When initiating a task or complex refactor, state in your initial thinking/plan which skills were consulted or verified.

---

## 2. Hard Code Constraints, Line Budgets & Anti-Bloat Policy

> [!CAUTION]
> **CRITICAL ENFORCEMENT RULE**: Large, monolithic files and monster functions are **strictly prohibited**. AI agents often fall into the trap of editing a file by continually appending lines until it reaches 400–800 lines. You are **forbidden** from doing this.

### Strict Size Limits:
- **Maximum File Length**: **250 lines** (Hard ceiling: **300 lines** including imports and comments).
- **Maximum Composable / Function Length**: **40 lines**. If a composable exceeds 40 lines, extract its sections into dedicated sub-composables.
- **Maximum ViewModel Length**: **150 lines**. A ViewModel is an orchestrator, not a business dump. Offload business logic to UseCases and complex transformations to domain mappers.
- **Maximum Line Width**: **100–120 characters**. Wrap parameters vertically; use trailing commas on all multi-line parameter and argument lists.

### Decomposition Invariant:
When modifying an existing file:
1. **Check current line count before editing.**
2. If your edit will cause the file to exceed **250 lines**, you **MUST decompose the file in the exact same turn**.
3. **Decomposition Pattern for Screens**:
   - `FeatureScreen.kt`: Stateful Route + Stateless Root layout container (~80–120 lines).
   - `FeatureComponents.kt`: Reusable sub-composables (e.g., headers, item cards, bottom bars) (~100–150 lines).
   - `FeatureState.kt`: `UiState`, `UiAction`, `UiEffect` definitions (~40–80 lines).
   - `FeatureViewModel.kt`: Pure state orchestration and UseCase triggering (~80–120 lines).

---

## 3. Linter, Static Analysis & Code Hygiene Matrix

Every project must enforce automated linting, formatting, and static analysis. Code that violates lint rules is considered broken code.

| Tool | Purpose | Configuration / Command |
| :--- | :--- | :--- |
| **Spotless + Ktlint** | Formatting, indentation, import ordering, wildcard import bans, trailing commas | `./gradlew spotlessCheck` / `./gradlew spotlessApply` |
| **Detekt + Compose Rules** | Static analysis, cyclomatic complexity, Compose stability, magic numbers & smell detection | `./gradlew detekt` (with `io.nlopez.compose.rules:detekt`) |
| **Android Lint** | Platform APIs, deprecations, resource checks, hardcoded text, accessibility | `./gradlew lint` (enforce `abortOnError = true` on CI) |

---

### Spotless Gradle Configuration (`build.gradle.kts`)
Add Spotless to your root or convention plugin to enforce formatting and ban wildcard imports:

```kotlin
plugins {
    alias(libs.plugins.spotless)
}

subprojects {
    apply(plugin = "com.diffplug.spotless")
    configure<com.diffplug.gradle.spotless.SpotlessExtension> {
        kotlin {
            target("**/*.kt")
            targetExclude("**/build/**", "**/generated/**", "**/.gradle/**")
            ktlint("1.5.0").editorConfigOverride(
                mapOf(
                    "indent_size" to "4",
                    "continuation_indent_size" to "4",
                    "ktlint_standard_no-wildcard-imports" to "enabled",
                    "ktlint_standard_trailing-comma-on-call-site" to "enabled",
                    "ktlint_standard_trailing-comma-on-declaration-site" to "enabled",
                    "max_line_length" to "120",
                )
            )
            trimTrailingWhitespace()
            endWithNewline()
        }
        kotlinGradle {
            target("**/*.gradle.kts")
            ktlint("1.5.0")
        }
    }
}
```

---

### Detekt Gradle Configuration (`build.gradle.kts`)
Add Detekt with Compose-specific rules to catch recomposition smells and complexity:

```kotlin
plugins {
    alias(libs.plugins.detekt)
}

subprojects {
    apply(plugin = "io.gitlab.arturbosch.detekt")
    
    dependencies {
        detektPlugins(libs.detekt.compose.rules)
    }

    detekt {
        buildUponDefaultConfig = true
        allRules = false
        parallel = true
        config.setFrom(files("$rootDir/config/detekt/detekt.yml"))
    }

    tasks.withType<io.gitlab.arturbosch.detekt.Detekt>().configureEach {
        reports {
            html.required.set(true)
            xml.required.set(false)
            txt.required.set(false)
        }
    }
}
```

#### Starter Detekt Configuration (`config/detekt/detekt.yml`)
Place this in your project to enforce the 250-line budget, Compose rules, and zero-magic-value policy:

```yaml
complexity:
  LargeClass:
    active: true
    threshold: 250           # Hard ceiling: files > 250 lines fail detekt!
  LongMethod:
    active: true
    threshold: 40            # Functions / composables > 40 lines fail!
  ComplexMethod:
    active: true
    threshold: 15
  LongParameterList:
    active: true
    functionThreshold: 6
    constructorThreshold: 7
    ignoreDefaultParameters: true

naming:
  FunctionNaming:
    active: true
    ignoreAnnotated: ['Composable']  # Allows PascalCase for @Composable fun MyScreen()

style:
  WildcardImport:
    active: true             # Bans import foo.*
  UnusedImports:
    active: true             # Strips dead imports
  MagicNumber:
    active: true             # Flags raw dimensions and magic color numbers
    ignoreNumbers: ['-1', '0', '1', '2']
    ignoreHashCodeFunction: true
    ignorePropertyDeclaration: true
    ignoreAnnotation: true
    ignoreEnums: true

Compose:
  ReusedModifierInstance:
    active: true
  UnnecessaryEventHandlerParameter:
    active: true
  ModifierMissing:
    active: true
  ComposableParamOrder:
    active: true
  ContentDescriptionMissing:
    active: true
```

---

### Android Lint Configuration (`build.gradle.kts`)
In every Android module, strictly configure linting to catch hardcoded strings and accessibility misses:

```kotlin
android {
    lint {
        abortOnError = true
        checkReleaseBuilds = true
        warningsAsErrors = false
        error("HardcodedText")
        error("IconMissingContentDescription")
        error("UseCompoundDrawables")
        error("Overdraw")
        baseline = file("lint-baseline.xml")
    }
}
```

---

### Strict Hygiene Invariants:
- **Zero Wildcard Imports**: `import com.app.data.*` is strictly forbidden. Every import must be explicit.
- **Zero Dead Code / Unused Imports**: All unused imports must be stripped before finalizing any edit.
- **Mandatory Trailing Commas**: Required on all multi-line parameter lists, argument calls, and collection literals.
- **Zero Raw Strings in UI**: User-facing text must use resource bundles (`stringResource(R.string.*)`).
- **Zero Raw Hex Colors in UI**: Presentation components must consume semantic design tokens (`MaterialTheme.colorScheme.*`).

---

## 4. Real-Time "Vibe Coding" Line Budget Enforcement

### Mandatory Pre-Tool "Line Budget Receipt" (Agent Contract)
Before calling `write_to_file` or modifying code with `replace_file_content`, you **MUST** internally calculate and output a line budget audit:

```text
[LINE BUDGET AUDIT]
Target: <FileName.kt>
Current Lines: <count> | Resulting Lines: <count> | Limit: 250 lines
Status: [PASS / DECOMPOSE REQUIRED]
```

- **If Status is PASS**: Proceed with saving.
- **If Status is DECOMPOSE REQUIRED (Resulting Lines > 250)**: You are **HARD-BLOCKED** from modifying the file in-place. You must immediately create a new sub-file (e.g., `*Components.kt`, `*State.kt`, `*Mappers.kt`, or `*UseCase.kt`) and offload code there first.

---

### Mechanical Interception: Antigravity Lifecycle Hook (`hooks.json`)
To mechanically prevent any model from bypassing line limits, hardcoded strings, or raw color values during vibe-coding, add this hook to your project at `.agents/hooks.json`:

```json
{
  "line-count-guard": {
    "PreToolUse": [
      {
        "matcher": "write_to_file|replace_file_content",
        "hooks": [
          {
            "type": "command",
            "command": "python .agents/scripts/check_line_limit.py",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

#### Hook Script (`.agents/scripts/check_line_limit.py`):
```python
import sys, json, os, re

payload = json.load(sys.stdin)
args = payload.get("toolCall", {}).get("args", {})
target = args.get("TargetFile") or ""
content = args.get("CodeContent") or ""

if target.endswith(".kt"):
    # 1. Line Limit Guard (Max 250 lines)
    line_count = len(content.splitlines()) if content else 0
    if not content and os.path.exists(target):
        with open(target, "r", encoding="utf-8") as f:
            line_count = len(f.readlines())

    if line_count > 250:
        print(json.dumps({
            "decision": "deny",
            "reason": f"BLOCKED BY LINE-GUARD: {os.path.basename(target)} has {line_count} lines (Max: 250). You MUST decompose this into multiple files (*Components.kt, *State.kt, etc.) before saving."
        }))
        sys.exit(0)

    # 2. Hardcoded Text Guard in Presentation/UI
    if any(keyword in target for keyword in ["presentation", "ui", "Screen", "Component"]):
        raw_text_match = re.search(r'\bText\(\s*"(?!\$)[A-Za-z0-9 ]{3,}"\s*\)', content)
        if raw_text_match:
            print(json.dumps({
                "decision": "deny",
                "reason": f"BLOCKED BY RESOURCE-GUARD: Detected hardcoded text literal '{raw_text_match.group(0)}' in {os.path.basename(target)}. Extract to strings.xml and use stringResource()."
            }))
            sys.exit(0)

        # 3. Raw Hex Color in Composable Guard
        raw_color_match = re.search(r'\bColor\(0x[0-9a-fA-F]{6,8}\)', content)
        if raw_color_match:
            print(json.dumps({
                "decision": "deny",
                "reason": f"BLOCKED BY COLOR-GUARD: Detected raw Color instantiation '{raw_color_match.group(0)}' in {os.path.basename(target)}. Map through MaterialTheme.colorScheme or design tokens."
            }))
            sys.exit(0)

print(json.dumps({"decision": "allow"}))
```

---

### Universal Gradle Line Budget Gate (`build.gradle.kts`)
To enforce line budgets in **Android Studio** and across all build commands, add this task to root `build.gradle.kts`:

```kotlin
tasks.register("checkLineBudget") {
    group = "verification"
    description = "Enforces clean architecture line limits (max 250 lines per Kotlin file)."
    doLast {
        val maxLines = 250
        val bloatedFiles = fileTree(rootDir) {
            include("**/src/**/*.kt")
            exclude("**/build/**", "**/.gradle/**", "**/generated/**", "**/.idea/**")
        }.files.filter { it.readLines().size > maxLines }

        if (bloatedFiles.isNotEmpty()) {
            val message = buildString {
                appendLine("\n" + "=".repeat(75))
                appendLine("❌ BUILD BLOCKED: LINE BUDGET VIOLATION (Max allowed: $maxLines lines)")
                appendLine("=".repeat(75))
                bloatedFiles.forEach { file ->
                    val lineCount = file.readLines().size
                    val relativePath = file.relativeTo(rootDir).path
                    appendLine("  [FAIL] $relativePath -> $lineCount lines (+$${lineCount - maxLines} over limit)")
                }
                appendLine("\n🔧 Required Decomposition Steps:")
                appendLine("  1. Screen files  -> Split into *Screen.kt, *Components.kt, *State.kt")
                appendLine("  2. ViewModels    -> Offload logic to Domain UseCases or Data Mappers")
                appendLine("  3. Composables   -> Extract sub-sections into dedicated helper composables")
                appendLine("=".repeat(75))
            }
            throw GradleException(message)
        }
    }
}

gradle.projectsEvaluated {
    allprojects {
        tasks.matching { task ->
            task.name in listOf("preBuild", "assemble", "build") ||
            task.name.startsWith("assemble") ||
            task.name.startsWith("compile") && task.name.endsWith("Kotlin")
        }.configureEach {
            dependsOn(":checkLineBudget")
        }
    }
}
```

---

## 5. Shell Command & Interactive Input Safety Rules

1. **Before executing any shell command, determine whether it can require interactive user input.**
2. Treat a command as interactive if it asks for confirmation (`Y/N`), passwords, credentials, wizards, or REPLs.
3. **Do not execute an interactive command through shell tools.** Instead, stop and inform the user before running it.
4. When presenting a command that the user must run manually, clearly label it:

   **MANUAL TERMINAL INPUT REQUIRED**
   ```text
   <command>
   ```

---

## 6. Recommended Modern Native Android Tech Stack (2026 Standards)

For native Android applications in 2026, adhere strictly to these modern dependencies:

| Category | Native Android Standard | Best Practice & Details |
| :--- | :--- | :--- |
| **Language & Toolchain** | **Kotlin 2.1+ / K2 Compiler** | Compose Compiler Gradle Plugin (`org.jetbrains.kotlin.plugin.compose`). Target SDK 35/36. |
| **UI Toolkit** | **Jetpack Compose (BOM latest)** | Material 3 (`material3`), Adaptive WindowSizeClass, Predictive Back. |
| **Dependency Injection** | **Hilt 2.52+** (via KSP) or **Koin 4.0+** | Inject ViewModels and UseCases with zero runtime reflection. |
| **Networking** | **OkHttp 5.0+** + **Retrofit 2.11+** or **Ktor 3.1+** | Protobuf binary streaming, WebSockets, ContentNegotiation. |
| **Serialization** | **`kotlinx.serialization`** or **Protocol Buffers v3 (`prost`/`protobuf-kotlin`)** | Fast compile-time schema contracts. |
| **Database / Persistence** | **Jetpack Room 2.7+** (with KSP) | SQLite bundled driver (`androidx.sqlite:sqlite-bundled`). |
| **Key-Value Store** | **Jetpack DataStore Preferences 1.1+** | Coroutine-first, reactive replacement for SharedPreferences. |
| **Image Loading** | **Coil 3.1+** (`io.coil-kt.coil3:coil-compose`) | Modern Compose image loading with crossfade and disk caching. |
| **Navigation** | **Jetpack Navigation 3** / **Navigation Compose** | Type-safe Kotlin `@Serializable` class/object routes. |
| **Concurrency & Reactive** | **Kotlinx Coroutines 1.10+ & Flow** | Structured concurrency, `StateFlow`, `SharedFlow`, `Channel`. |
| **Immutable Collections** | **`kotlinx-collections-immutable`** | Guarantees Compose recomposition stability without `@Immutable` hacks. |
| **Logging** | **Timber** | Strip debug logs in release builds automatically. |
| **Testing** | **Turbine**, MockK, Compose Test Rule | Synchronous Flow assertions without artificial delays. |

---

## 7. Clean Architecture & Layer Decoupling

Enforce strict boundaries across three core architectural layers:

```text
┌─────────────────────────────────────────────────────────────┐
│                      Presentation Layer                     │
│    (Composables, ViewModels, UI State, UI Actions/Effects)  │
└──────────────────────────────┬──────────────────────────────┘
                               │ depends on
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                        Domain Layer                         │
│  (Pure Kotlin Entities, Use Cases / Interactors, Repo APIs) │
└──────────────────────────────▲──────────────────────────────┘
                               │ implemented by
┌─────────────────────────────────────────────────────────────┐
│                         Data Layer                          │
│  (Repositories, Remote Data Sources, Local DB, DTOs, Mappers)│
└─────────────────────────────────────────────────────────────┘
```

### Layer Constraints:
1. **Domain Layer (`domain`)**:
   - Pure Kotlin only. Zero dependencies on `android.*`, `androidx.*`, or Compose.
   - Encapsulates single-responsibility UseCases exposing `operator fun invoke(...)`.
2. **Data Layer (`data`)**:
   - Encapsulates Network, Room DB, WebSockets, and DataStore.
   - Single Source of Truth: exposes cached data via `Flow`, updating local storage from network.
3. **Presentation Layer (`presentation`)**:
   - Consumes Domain UseCases and Models. Never accesses DTOs or DB entities directly.

---

## 8. DTO Pattern, Entity Separation & Explicit Mappers

Network contracts, persistence schemas, and business domains must **never** share models:

1. **Three Distinct Model Types**:
   - **DTOs (`*Dto` / `*Response`)**: Mirror external API / Protobuf payloads.
   - **Database Entities (`*Entity`)**: Room `@Entity` tables and indices.
   - **Domain Models**: Pure Kotlin data classes or `@JvmInline value class`.
2. **Explicit Mappers**:
   - Keep mappers as `internal` extension functions in the Data layer:
     ```kotlin
     internal fun UserDto.toDomain(): User = User(id = UserId(id), name = name)
     internal fun UserEntity.toDomain(): User = User(id = UserId(id), name = name)
     ```
3. **Type Safety with Value Classes**:
   - Use `@JvmInline value class` for IDs (`value class DeckSlotId(val value: Int)`) to eliminate primitive obsession.

---

## 9. Reactive State Management & Concurrency (MVI + Flow)

Follow unidirectional data flow (UDF / MVI) across all ViewModels:

1. **State, Intent & Effects**:
   - **UI State**: Single immutable `data class` representing the complete screen state.
   - **UI Actions / Intents**: Sealed interface representing user actions (`DeckUiAction.OnKeyTapped`).
   - **One-off Effects**: Handled via `Channel<UiEffect>(Channel.BUFFERED)` and exposed as `receiveAsFlow()`.
2. **StateFlow Lifecycle in ViewModel**:
   ```kotlin
   val uiState: StateFlow<DeckUiState> = repository.observeLayout()
       .map { layout -> DeckUiState.Connected(layout) }
       .catch { emit(DeckUiState.Error(it.message ?: "Unknown error")) }
       .stateIn(
           scope = viewModelScope,
           started = SharingStarted.WhileSubscribed(5_000),
           initialValue = DeckUiState.Connecting,
       )
   ```
3. **Structured Concurrency**:
   - Use `viewModelScope`. Never use `GlobalScope`.
   - Inject `CoroutineDispatcher` for unit testability.
4. **Lifecycle-Aware Collection in Compose**:
   - In Compose, collect flows using `collectAsStateWithLifecycle()` from `androidx.lifecycle.compose`.

---

## 10. Jetpack Compose UI Standards

1. **Stateless Composables & State Hoisting**:
   - Every screen consists of a **Stateful Route** and a **Stateless Screen** (`state: ScreenUiState`, `onAction: (ScreenAction) -> Unit`).
   - Never pass ViewModels down into child composables.
2. **Recomposition Stability**:
   - Use `ImmutableList` / `ImmutableSet` from `kotlinx.collections.immutable`.
   - Always supply stable keys to `LazyColumn`, `LazyRow`, and `items(..., key = { it.id })`.
3. **Edge-to-Edge & System Insets**:
   - Call `enableEdgeToEdge()` in Activity `onCreate()`.
   - Defend insets using `Modifier.statusBarsPadding()`, `Modifier.navigationBarsPadding()`, or `Modifier.windowInsetsPadding(WindowInsets.safeDrawing)`.
4. **Adaptive Layouts**:
   - Support compact phones, foldables, and tablets using `WindowWidthSizeClass`.
5. **Modern Type-Safe Navigation**:
   - Use Kotlin `@Serializable` classes as routes with Navigation Compose / Navigation 3.

---

## 11. Package Structuring: Feature First, Then Layer

```text
com.streamdeck.client/
├── core/
│   ├── common/              # Dispatchers, Result, Extensions
│   ├── network/             # WebSocket client, Protobuf models
│   ├── ui/theme/            # Theme, ColorScheme, Spacing, Typography
│   └── util/                # System utilities (Haptics, Wi-Fi info)
└── feature/
    ├── connection/
    │   ├── data/            # Discovery service, preference storage
    │   ├── domain/          # DiscoverHostsUseCase
    │   └── presentation/    # ConnectionScreen, ConnectionViewModel
    └── deck/
        ├── data/            # DeckRepository, Protocol serializer
        ├── domain/          # SendKeyEventUseCase
        └── presentation/    # DeckScreen, DeckComponents, DeckViewModel
```

---

## 12. Production Hardening, Security, Privacy & Accessibility

1. **Zero Hardcoding Secrets Policy**:
   - Store API keys and connection secrets in `local.properties` (git-ignored) and expose via `BuildConfig`.
2. **Modern Android 14/15/16 Permissions**:
   - Photo Picker (`ActivityResultContracts.PickVisualMedia`) for media access.
   - Explicit runtime requests for `POST_NOTIFICATIONS` with user education.
   - Foreground services must declare specific `foregroundServiceType` in `AndroidManifest.xml`.
3. **Accessibility (a11y) Standards**:
   - **Minimum Touch Targets**: Every button or interactive tile must meet at least **48.dp** (`Modifier.minimumInteractiveComponentSize()`).
   - **Content Descriptions**: Every semantic `Icon` or `Image` must have a localized `contentDescription`. Set `contentDescription = null` only for decorative assets.

---

## 13. Automated Testing Track Publishing (Gradle Play Publisher)

```kotlin
// In app/build.gradle.kts
val keystorePropertiesFile = rootProject.file("local.properties")
val keystoreProperties = java.util.Properties().apply {
    if (keystorePropertiesFile.exists()) load(keystorePropertiesFile.inputStream())
}

android {
    signingConfigs {
        create("release") {
            storeFile = file(keystoreProperties.getProperty("KEYSTORE_PATH") ?: System.getenv("KEYSTORE_PATH") ?: "release.keystore")
            storePassword = keystoreProperties.getProperty("KEYSTORE_PASSWORD") ?: System.getenv("KEYSTORE_PASSWORD") ?: ""
            keyAlias = keystoreProperties.getProperty("KEY_ALIAS") ?: System.getenv("KEY_ALIAS") ?: ""
            keyPassword = keystoreProperties.getProperty("KEY_PASSWORD") ?: System.getenv("KEY_PASSWORD") ?: ""
        }
    }
    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
}

play {
    serviceAccountCredentials.set(
        file(keystoreProperties.getProperty("PLAY_SERVICE_ACCOUNT") ?: System.getenv("PLAY_SERVICE_ACCOUNT") ?: "service-account.json")
    )
    defaultToAppBundles.set(true)
    track.set(com.github.triplet.gradle.androidpublisher.ReleaseStatus.COMPLETED)
}
```

### CLI Publishing Commands:
```bash
./gradlew publishReleaseBundle -Ptrack=internal --track internal
./gradlew publishReleaseBundle -Ptrack=alpha --track alpha
```

---

## 14. Version Catalog (`libs.versions.toml`) Template

```toml
[versions]
agp = "8.8.2"
kotlin = "2.1.10"
compose-bom = "2025.02.00"
ksp = "2.1.10-1.0.29"
coroutines = "1.10.1"
serialization = "1.8.0"
okhttp = "5.0.0-alpha.14"
coil = "3.1.0"
room = "2.7.0-alpha13"
datastore = "1.1.2"
navigation = "2.8.8"
immutable-collections = "0.3.8"
turbine = "1.2.0"
spotless = "7.0.2"
detekt = "1.23.8"

[libraries]
kotlinx-coroutines-android = { module = "org.jetbrains.kotlinx:kotlinx-coroutines-android", version.ref = "coroutines" }
kotlinx-coroutines-test = { module = "org.jetbrains.kotlinx:kotlinx-coroutines-test", version.ref = "coroutines" }
kotlinx-serialization-json = { module = "org.jetbrains.kotlinx:kotlinx-serialization-json", version.ref = "serialization" }
kotlinx-collections-immutable = { module = "org.jetbrains.kotlinx:kotlinx-collections-immutable", version.ref = "immutable-collections" }
androidx-compose-bom = { module = "androidx.compose:compose-bom", version.ref = "compose-bom" }
androidx-compose-ui = { module = "androidx.compose.ui:ui" }
androidx-compose-material3 = { module = "androidx.compose.material3:material3" }
androidx-navigation-compose = { module = "androidx.navigation:navigation-compose", version.ref = "navigation" }
androidx-datastore-preferences = { module = "androidx.datastore:datastore-preferences", version.ref = "datastore" }
coil-compose = { module = "io.coil-kt.coil3:coil-compose", version.ref = "coil" }
coil-network-okhttp = { module = "io.coil-kt.coil3:coil-network-okhttp", version.ref = "coil" }
okhttp-core = { module = "com.squareup.okhttp3:okhttp", version.ref = "okhttp" }
turbine = { module = "app.cash.turbine:turbine", version.ref = "turbine" }
```

---

## 15. Testing & Verification Runbook

1. **Unit Testing**:
   - Use `StandardTestDispatcher` and **Turbine** (`viewModel.uiState.test { ... }`).
   - Use Test Fakes for repositories and network clients.
2. **Compose UI Tests**:
   - Use `createComposeRule()` to test user touch interactions and accessibility node semantics.

---

## 16. Project Initialization & Feature Execution Order

1. **Skill Discovery**: Inspect relevant `SKILL.md` before coding.
2. **Domain Models**: Pure data classes and value classes (< 250 lines).
3. **Data Layer**: Repositories, DataStore, and WebSocket/Network clients.
4. **Use Cases**: Single-action business interactors.
5. **Presentation (MVI)**: `UiState`, `UiAction`, `UiEffect`, `ViewModel` (< 150 lines).
6. **Compose UI**: Stateless screens and components (< 40 lines/composable).
7. **Verification**: Run `./gradlew checkLineBudget spotlessCheck test`.

---

## 17. UI Resources, Semantic Theming & Zero-Hardcoding Manifesto

> [!CAUTION]
> **ZERO TOLERANCE FOR UI MAGIC VALUES**: Direct literal strings (e.g., `Text("Submit")`), raw hex colors (e.g., `Color(0xFF1E88E5)`), untyped dimensions (e.g., `padding(13.dp)`), and unlocalized image descriptions in `@Composable` code are **strictly forbidden**.

### 1. Zero String Literals:
- Every string must resolve via `stringResource(R.string.*)`.
- Use positional placeholders (`%1$s`, `%2$d`) for dynamic values.
- Use `<plurals>` resources for counts and quantities.

### 2. Semantic Color System:
- Strictly consume `MaterialTheme.colorScheme.*` roles (`primary`, `surface`, `onSurface`, `surfaceContainer`).
- Always pair container tokens with designated "on" content tokens (`surface` ➔ `onSurface`).

### 3. Spacing Token System (8dp Grid Baseline):
```kotlin
object Spacing {
    val extraSmall = 4.dp
    val small = 8.dp
    val medium = 16.dp
    val large = 24.dp
    val extraLarge = 32.dp
    val huge = 48.dp
}
```
- Pad layouts strictly with `Spacing.*` tokens.
- Never write ad-hoc magic padding like `padding(13.dp)`.
