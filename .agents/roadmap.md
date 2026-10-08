# StreamDeck Clone — implementation roadmap

Last updated: 2026-10-07. Keep this file for **what to build and what changed**. Record verification separately in [testing-results.md](testing-results.md). A checked implementation item means the code exists; it does **not** mean a person has tried the complete phone-and-PC flow.

## Product goal

Turn an Android phone into a customizable button deck for a Windows PC. The Windows host owns saved profiles and runs actions; the desktop app edits them; the phone displays and presses the configured buttons. Grow toward the everyday capabilities of Stream Deck-style apps without copying proprietary plugins or branding. Keep the existing nine buttons and basic actions free.

## Phase 0 — dependable core (implemented in source; real-flow review pending)

- [x] Save desktop key edits to host-owned profiles and show updated active layouts on the phone.
- [x] Match displayed key slots to the actions they run, even if saved keys are reordered.
- [x] Reject invalid profile edits rather than silently changing them.
- [x] Avoid blocking the host connection loop while running button actions; close the foreground-process handle.
- [x] Make the desktop editor wait for save confirmation and recover from save errors.
- [ ] Verify the complete edit → save → phone display → PC action → restart flow with the real applications.

## Desktop host controls & GUI UX (implemented; user trial pending)

- [x] Show **Start Host** / **Stop Host** in the desktop app instead of silently launching the host when the app opens.
- [x] Close to tray without stopping the app-owned host; stop it on explicit tray Quit or app exit. An external host remains untouched.
- [x] Single-instance restoration, native tray menu with host status and safe managed-host restart, and Windows 11 Mica attempt with solid fallback.
- [ ] Review Mica appearance, tray behavior, keyboard/editing, and WebView2 process-tree memory in the actual release window.
- [x] Identify a host started elsewhere without claiming the desktop app can stop it (including an elevated host).
- [x] Terminal-free release launch: ensure host and desktop configurator launch without console windows in release mode (`windows_subsystem = "windows"` and `CREATE_NO_WINDOW` 0x08000000).
- [x] Full-height window layout: configure `html, body` 100% height flex layouts with vertical scrolling only inside grid, sidebar, and inspector panels.
- [x] Key reorder & drag swap: support drag-and-drop key content swapping as well as keyboard alternative (select key, press 'M', select destination key, Esc to cancel) with persistent profile save to host.
- [x] Six-digit pairing code & QR payload: host generates a six-digit code; desktop app renders `streamdeck-pair:v2:<ip>:<port>:<code>:<fingerprint>` as a QR image.
- [ ] Confirm the same flow with the user's own ordinary phone-and-PC usage and a packaged Windows installer.

> Launch `run.ps1` and `start-host.bat` separately from this flow: they start an external host that the desktop app does not own. Start Host in the desktop app to make Stop Host available there.

## Phase 1 — phone pairing (pinned TLS and native controls implemented in source; hardening and real-device verification pending)

- [x] Show a pairing code in the desktop app; accept it in the Android connection screen.
- [x] Remember a paired phone so it can reconnect without entering a new code.
- [x] Let the desktop app remove a phone; close that phone's existing connection and reject its saved credential.
- [x] Expire a code after five minutes or ten failed attempts.
- [x] Test the real host connection handler with simulated WebSocket clients.
- [ ] Try the pairing and removal flow with the actual desktop app and phone.
- [x] Require pinned WSS on the phone listener and a QR v2 certificate fingerprint; reject old unencrypted phone traffic.
- [x] Move desktop controls to a separately authenticated loopback endpoint; keep the control secret in native Tauri code rather than browser JavaScript.
- [x] Encrypt Android pairing credentials and hash host device tokens at rest; reject plaintext legacy records.
- [x] Bound failed phone attempts by network peer in addition to device-ID throttling, and use crash-safer replacement for profiles and paired-device records; add negative WebSocket tests.
- [ ] Enforce restricted Windows ACLs and atomic identity/secret creation; complete TLS-level negative transport tests (including real-device pin mismatch).
- [ ] Verify real phone pairing, pin mismatch, reconnect and revocation with the packaged apps.

> Security hardening is not complete. Keep ports off the internet and do not claim the connection is production-ready until the remaining security checks and device trial pass.

## Phase 2 — pages, layouts, and everyday actions (planned)

- [x] Version the current host profile file and migrate existing unversioned 3×3 profiles without losing keys; preserve a legacy backup. Future page/slot schema still needs its own design.
- [ ] Add pages/folders, key reorder, grid sizes, editing preview, icons/styles, undo and backup/export.
- [ ] Add safe, bounded action sequences and common actions; validate edits on the host.
- [ ] Keep layouts usable on phone, landscape and larger screens, with accessible touch targets.

## Phase 3 — paid unlock (planned; product decision required)

- [ ] Keep nine keys and core connection, pairing, editing and accessibility free.
- [ ] Decide Pro limits and price from product research; proposed option is a one-time unlock for larger layouts and advanced editing/actions.
- [ ] Design verified Play Billing fulfillment, restore, refunds, offline behavior and host-side enforcement before accepting purchases.
- [ ] Preserve saved paid layouts if access lapses; never delete user data as a downgrade mechanism.

## Phase 4 — live controls (planned)

- [ ] Show host-provided button states, badges and useful status without flooding the connection.
- [ ] Add sliders/widgets, appearance customization and device-size previews after the layout model is stable.

## Phase 5 — integrations and distribution (planned)

- [ ] Add first-party integrations individually (for example OBS and media controls), with permission and failure handling.
- [ ] Design plugin isolation and credential handling before third-party plugins or a marketplace.
- [ ] Validate packaged Windows install/update and Android distribution on clean devices.

## Update rule

When starting or finishing a phase, update its checkboxes and date here. Add a dated entry with commands, pass/fail details and **separate user trial status** in [testing-results.md](testing-results.md). Do not mark a phase fully verified solely because builds or automated tests pass. The original expanded research is in the IDE's `implementation_plan.artifact.md`; this file is the repository's ongoing, shorter progress tracker.
