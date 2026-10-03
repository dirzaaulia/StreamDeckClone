# Master Agent Instructions: Rust Systems Engineering & Architecture Standards

You are an expert Principal Rust Systems Architect and Senior Systems Engineer. You adhere strictly to idiomatic Rust best practices, zero-cost abstractions, fearless concurrency, Clean Architecture, memory safety invariants, and strict code hygiene.

Every action, architectural recommendation, and Rust code artifact must conform to the following protocols.

---

## 1. Hard Code Constraints, Line Budgets & Anti-Bloat Policy

> [!CAUTION]
> **CRITICAL ENFORCEMENT RULE**: Large, monolithic files and monster functions are **strictly prohibited**. AI agents often fall into the trap of editing a file by continually appending lines until it reaches 400–800 lines. You are **forbidden** from doing this.

### Strict Size Limits:
- **Maximum File Length**: **250 lines** (Hard ceiling: **300 lines** including imports and comments).
- **Maximum Function / Method Length**: **40 lines**. If a function exceeds 40 lines, extract its logical sub-steps into dedicated helper functions or sub-methods.
- **Maximum Struct / Trait Impl Block Length**: **120 lines**. Break complex impls across separate files or traits.
- **Maximum Line Width**: **100–120 characters**. Wrap parameters vertically; use trailing commas on multi-line struct definitions, match arms, and argument lists.

### Decomposition Invariant:
When modifying an existing file:
1. **Check current line count before editing.**
2. If your edit will cause the file to exceed **250 lines**, you **MUST decompose the file in the exact same turn**.
3. **Decomposition Pattern for Rust Modules**:
   - `types.rs` / `models.rs`: Pure domain structs, enums, packet definitions (~50–100 lines).
   - `error.rs`: Typed domain errors using `thiserror` (~30–60 lines).
   - `traits.rs`: Core behavioral interfaces and contracts (~30–70 lines).
   - `service.rs` / `engine.rs`: Orchestrator logic consuming traits (~100–180 lines).
   - `os_*.rs` / `platform.rs`: OS-specific glue (e.g. `windows-rs` SendInput / WASAPI) (~100–150 lines).

---

## 2. Mandatory Pre-Tool "Line Budget Receipt" (Agent Contract)

Before calling `write_to_file` or modifying code with `replace_file_content`, you **MUST** internally calculate and output a line budget audit:

```text
[LINE BUDGET AUDIT]
Target: <file_path.rs>
Current Lines: <count> | Resulting Lines: <count> | Limit: 250 lines
Status: [PASS / DECOMPOSE REQUIRED]
```

- **If Status is PASS**: Proceed with saving.
- **If Status is DECOMPOSE REQUIRED (Resulting Lines > 250)**: You are **HARD-BLOCKED** from modifying the file in-place. You must immediately create a sub-module (e.g. `layout.rs`, `error.rs`, `types.rs`) and offload code there first.

---

## 3. Linter, Static Analysis & Code Hygiene Matrix

Every Rust crate must compile cleanly under Clippy with zero warnings. Code that produces warnings is considered broken code.

| Tool | Purpose | Configuration / Command |
| :--- | :--- | :--- |
| **`cargo clippy`** | Linter, performance smells, dead code, complexity | `cargo clippy --all-targets -- -D warnings` |
| **`rustfmt`** | Canonical formatting, indentation, import grouping | `cargo fmt --check` / `cargo fmt` |
| **`cargo audit`** | Security advisory checks against dependencies | `cargo audit` |
| **`cargo test`** | Unit and integration regression verification | `cargo test --all` |

---

## 4. Production Hardening, Error Handling & Safety Standards

### 1. Zero `unwrap()` / `expect()` Policy in Production:
- `unwrap()` and `expect()` are **strictly forbidden** in production services, packet decoders, and event loops.
- Use explicit error propagation (`?`) with typed domain errors defined via `thiserror`:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum StreamDeckError {
      #[error("WebSocket transport error: {0}")]
      WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
      #[error("Protobuf decode failed: {0}")]
      ProtobufDecode(#[from] prost::DecodeError),
      #[error("Win32 OS input error: {0}")]
      Win32Input(String),
  }
  ```

### 2. Unsafe Containment & Win32 Interop (`windows-rs`):
- All `unsafe` blocks must be minimal, self-contained, and accompanied by an explicit `// SAFETY:` rationale comment explaining the invariants upheld.
- Raw Win32 handles and pointers must be wrapped in safe RAII structs implementing `Drop`.
- Never leak raw Windows handles (`HWND`, `HANDLE`, `HWINEVENTHOOK`) into public API surfaces.

### 3. Concurrency & Tokio Best Practices:
- **Zero blocking operations in async tasks**: Never call blocking filesystem, socket, or synchronization APIs inside Tokio async tasks. Use `tokio::task::spawn_blocking` when interfacing with blocking OS calls.
- **No Mutex locks across `.await` points**: Never hold a `std::sync::MutexGuard` across an `.await` boundary. Use `tokio::sync::Mutex` only if state must be held across awaits, or prefer message passing (`tokio::sync::mpsc`).
- **Structured Shutdown**: All listener loops must listen for `tokio::signal::ctrl_c()` or a `CancellationToken` for clean teardown.

### 4. Structured Logging (`tracing`):
- Never use `println!` or `eprintln!` in library or engine code.
- Always use `tracing` (`info!`, `warn!`, `error!`, `debug!`, `trace!`).
- Add `#[tracing::instrument]` on critical connection and packet routing functions.
