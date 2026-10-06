# StreamDeck Clone — testing results

Last updated: 2026-10-04. This is the ongoing verification log for [.agents/roadmap.md](roadmap.md). **Automated pass ≠ real-use pass.** Record the date and result again whenever changes affect the flow; never carry a previous pass forward as proof of new code.

## Automated checks — 2026-10-04 (Desktop GUI & Host Updates)

Run after terminal-free release launch, full-height window contents, persistent key drag swap/keyboard alternative, and 6-digit pairing QR payload updates.

| Check | Command / scope | Result |
|---|---|---|
| Desktop GUI JS tests & build | `cd desktop-gui; node --test src/*.test.js; npm run build` | PASS: 3 JS unit tests pass (`keyMove.test.js`, `qrPayload.test.js`); Vite build succeeds in ~300ms |
| Host-desktop formatting & lint | `cd host-desktop; cargo fmt --check; cargo clippy --all-targets -- -D warnings` | PASS: 0 errors, 0 warnings |
| Host-desktop tests | `cd host-desktop; cargo test` | PASS: 5 unit tests (`test_profile_validation`, `layout_uses_stable_slot_ids`, `pairing_survives_restart_and_revoke`, `test_config_default_and_persistence`, `pairing_blocks_unknown_phones_and_revocation_closes_active_session`) and 1 integration test (`test_handshake_and_key_event`) |
| Desktop Tauri native checks | `cd desktop-gui/src-tauri; cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo check` | PASS: 0 errors, 0 warnings |

### What the JS & Rust unit tests cover

- `desktop-gui/src/keyMove.test.js`: verifies key content swapping while preserving physical slot IDs (0..8) and handling invalid/out-of-bound IDs.
- `desktop-gui/src/qrPayload.test.js`: verifies formatting of `streamdeck-pair:v1:<ip>:<port>:<code6>` for valid inputs and rejecting invalid IP/ports or non-6-digit codes.
- `desktop-gui/src-tauri/src/host_process.rs`: verifies default unmanaged status for Tauri host process state.
- `host-desktop/src/server_test.rs`: verifies production WebSocket server, 6-digit pairing code generation, credential issuance, and session revocation.

### Real-app verification vs skipped

- **Automated verification**:
  - All desktop web assets built cleanly with Vite.
  - Rust binaries for `host-desktop` and `desktop-gui/src-tauri` compile and pass strict clippy linting and unit tests.
  - Terminal-free release launch verified via `#![cfg_attr(..., windows_subsystem = "windows")]` and `CREATE_NO_WINDOW` (0x08000000) flags.
- **Skipped / Pending User Trial**:
  - Physical Android device connection and real-app QR camera scan / 6-digit code entry on actual hardware.
  - Execution of live PC hotkey actions (`Win + D`, volume control) from physical phone touch.
  - Note: External host process was **not terminated**.

## Automated checks — 2026-10-05 (connection security in progress)

| Check | Result |
|---|---|
| Host `cargo test --offline` | PASS: 11 unit tests and 1 integration test, including repeated-handshake, malformed/oversized-frame rejection, and bounded device-ID attempt tracking. |
| Host `cargo clippy --offline --all-targets -- -D warnings` | PASS after latest host changes. |
| Host `cargo fmt --check` | FAIL: formatting differences in security implementation. |
| Desktop `npm test` and `npm run build` | PASS: 3 JS tests, including QR v2; Vite production build succeeded. |
| Tauri `cargo check --offline` | PASS: native control bridge compiles. |
| Tauri `cargo test --offline` | FAIL on this Windows environment: test process exits `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139); linker also warns about multiple manifests. |
| Android Gradle gates | PASS after current Android edits: `checkLineBudget`, `:app:testDebugUnitTest`, `:app:assembleDebug`. Earlier run reported 5 tests; latest build succeeded. |

Manual pinned-TLS, real phone pairing, reconnect, revocation, and live PC controls are **NOT RUN**. Global/per-IP concurrent connection caps were added, but peer-based failed-attempt throttling, identity file ACLs, and full negative transport coverage still need work; no security-complete claim yet.

## Real-use trial — waiting for user

Status: **NOT RUN**. The user cannot test right now. Leave these unchecked until they report what happened; a developer should not fill in their result by inference. Use a trusted local network, do not expose port 4455 to the internet, and keep actual pairing codes/credentials out of this file.

When ready, record: date, phone model/Android version, Windows version, Wi-Fi or USB, host/desktop/phone build or commit, and a brief result or screenshot reference. Do not paste a code or credential.

- [ ] Open the desktop app: it shows **Host stopped** until you press **Start Host**; stopping in the app disconnects the phone. Result/date: _pending_.
- [ ] Start the Windows host and desktop app; both show the host as available. Result/date: _pending_.
- [ ] With a new phone, connect without a code; the phone is asked to pair and cannot press PC buttons. Result/date: _pending_.
- [ ] Click **Pair a phone** on desktop, enter the shown code on the phone and connect; nine keys appear. Result/date: _pending_.
- [ ] Press a safe button (for example volume), confirm the PC responds once; avoid **Task Manager** or **Screenshot** if inconvenient. Result/date: _pending_.
- [ ] Change one label/action on desktop and save; confirm phone updates and the intended button action runs. Result/date: _pending_.
- [ ] Disconnect/reconnect the phone without a new code; confirm its deck returns. Result/date: _pending_.
- [ ] Remove the phone on desktop while it is connected; confirm it disconnects and cannot reconnect without a fresh code. Result/date: _pending_.
- [ ] Restart desktop host and app; confirm saved profile edits and paired-phone access still work as expected. Result/date: _pending_.

### User-reported issues and retest

| Date | Step | What happened / expected | Fix reference | Retest result |
|---|---|---|---|---|
| _pending_ | — | — | — | — |

## Rule for future updates

For each meaningful app update, add a new dated automated-check subsection with **actual commands, pass/fail/skipped and failures**. Add or revise real-use checklist items for the changed behavior. Keep the user's own observations and retest outcomes in the real-use section rather than labeling automated checks as user acceptance. Update [roadmap.md](roadmap.md) only after source work is implemented, and keep real-use verification pending until the user confirms it.
