---
phase: quick-260331-cw5
plan: "01"
subsystem: infra/android
tags: [android, devices, adb, mode, prefs]
dependency_graph:
  requires: []
  provides: [android-model-names, android-mode-persistence]
  affects: [src/infra/devices.rs, src/infra/android_prefs.rs, src/domain/command.rs, src/app.rs, src/ui/footer.rs]
tech_stack:
  added: []
  patterns: [serde_json::json! for serialization, config_dir pattern]
key_files:
  created:
    - src/infra/android_prefs.rs
  modified:
    - src/infra/devices.rs
    - src/infra/mod.rs
    - src/domain/command.rs
    - src/app.rs
    - src/ui/footer.rs
decisions:
  - default android_mode to Some("debugOptimized") when no prefs file exists
  - parse_adb_devices handles both old (adb devices) and new (adb devices -l) output formats
  - mode saved at both device-confirm sites (single-device auto-select and device picker confirm)
metrics:
  duration: "5 min"
  completed: "2026-03-31"
  tasks: 2
  files: 5
---

# Quick Task 260331-cw5: Fix Android Device Listing, Add Mode / Debug Persistence Summary

**One-liner:** `adb devices -l` model name parsing with `--mode debugOptimized` wired into run-android via persisted android_prefs.json.

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Fix adb parser for model names, add android_prefs module | b9d9904 | src/infra/devices.rs, src/infra/android_prefs.rs, src/infra/mod.rs |
| 2 | Wire android_mode into RnRunAndroid and app state | 6c1ce96 | src/domain/command.rs, src/app.rs, src/ui/footer.rs |

## What Was Built

### adb devices -l Parser

`parse_adb_devices()` now handles both output formats:
- **Old format** (`adb devices`): `R58MA1XR0XE\tdevice` — falls back to serial as name
- **New format** (`adb devices -l`): `R58MA1XR0XE\tdevice product:a52sxq model:SM_A525F ...` — extracts `model:` field, replaces underscores with spaces for display name

`list_android_devices()` now passes `-l` flag: `.args(["devices", "-l"])`.

### Android Prefs Persistence

New module `src/infra/android_prefs.rs` following `sim_history.rs` pattern:
- `android_prefs_path()` → `~/.config/ump-dash/android_prefs.json`
- `load_android_mode() -> Option<String>` — reads `{"mode": "debugOptimized"}`
- `save_android_mode(mode: &str)` — writes JSON, creates config dir if needed

### Mode Field in RnRunAndroid

`RnRunAndroid { device_id: String, mode: Option<String> }` — `to_argv()` inserts `--mode <value>` before `--deviceId` when mode is Some.

### AppState.android_mode

`android_mode: Option<String>` field added to `AppState`:
- Initialized from `load_android_mode()` or defaults to `Some("debugOptimized".to_string())`
- Passed as template mode in palette Android handler (both 'd' and 'e' keys)
- Propagated through `ModalDeviceConfirm` and `DevicesEnumerated` single-device auto-select
- Saved to disk at both dispatch sites via `save_android_mode()`

### Footer

Android palette hint for 'd' updated from "run-android" to "run-android (debug)".

## Deviations from Plan

None - plan executed exactly as written. The constraint to default `android_mode` to `Some("debugOptimized")` on first run was applied as specified.

## Self-Check: PASSED

- FOUND: src/infra/android_prefs.rs
- FOUND: src/infra/devices.rs
- FOUND commit b9d9904 (task 1)
- FOUND commit 6c1ce96 (task 2)
- cargo check: Finished with 0 errors
