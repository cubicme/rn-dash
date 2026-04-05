---
phase: quick-260405-ijq
plan: 01
subsystem: app-state, command-pipeline
tags: [metro, command-dispatch, prerequisite-gate, rn-run]
dependency_graph:
  requires: []
  provides: [metro-prerequisite-gate]
  affects: [src/app.rs, src/domain/command.rs]
tech_stack:
  added: []
  patterns: [TEA-prerequisite-gate, pending-field-pattern]
key_files:
  created: []
  modified:
    - src/domain/command.rs
    - src/app.rs
decisions:
  - "Metro prerequisite gate placed after palette_mode clear and before sync-before-run check — ensures stashed command re-enters full pipeline on Ready (sync check, device picker still apply)"
  - "Re-enter via Action::CommandRun (not dispatch_command directly) so all pre-processing runs on the stashed command"
  - "MetroActivity::Clone needed for activity.clone() in MetroActivityUpdate to allow both storing and matching"
metrics:
  duration: 8min
  completed: "2026-04-05"
  tasks_completed: 2
  files_modified: 2
---

# Quick 260405-ijq: Make Metro a Prerequisite for Running iOS/Android — Summary

**One-liner:** Auto-start metro before RN run commands (run-ios, run-android, run-ios-device, release-build) and dispatch only after metro reports Ready, preventing React Native CLI from spawning an unmanaged metro instance.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add needs_metro() to CommandSpec and pending_metro_run to AppState | 021d794 | src/domain/command.rs, src/app.rs |
| 2 | Wire metro prerequisite gate in CommandRun pipeline and auto-dispatch on Ready | 900f79a | src/app.rs |

## What Was Built

### Task 1: needs_metro() method and pending_metro_run field

Added `needs_metro()` to `CommandSpec` in `src/domain/command.rs` — returns `true` for `RnRunAndroid`, `RnRunIos`, `RnRunIosDevice`, and `RnReleaseBuild`. Follows the same pattern as `is_destructive()` and `needs_device_selection()`.

Added `pending_metro_run: Option<CommandSpec>` to `AppState` in `src/app.rs`, initialized to `None` in `Default`. Field stores a stashed RN run command while metro is starting.

### Task 2: Metro prerequisite gate

Three integration points in `src/app.rs`:

1. **`Action::CommandRun` handler** — after `palette_mode = None`, before the sync-before-run check: if `spec.needs_metro() && !state.metro.is_running()`, stash spec into `pending_metro_run` and dispatch `MetroStart`, then return early.

2. **`Action::MetroActivityUpdate(Ready)` handler** — after setting `state.metro.activity`, if activity is `Ready` and `pending_metro_run` is `Some`, take it and call `update(state, Action::CommandRun(run_spec), ...)`. This re-enters the full CommandRun pipeline so sync-before-run and device picker still apply.

3. **`Action::MetroExited` and `Action::MetroSpawnFailed` handlers** — clear `pending_metro_run = None` at the top of each handler to prevent stale dispatch if metro fails to start or exits unexpectedly.

## Verification

- `cargo check`: passes, no new errors
- `cargo clippy`: no new warnings (39 pre-existing warnings unchanged)
- Logic: when metro is stopped and user triggers a>e or i>e, metro auto-starts; run command dispatches only after metro logs "Welcome to Metro" (which triggers `MetroActivity::Ready`); if metro is already running, run commands dispatch immediately with no behavioral change

## Deviations from Plan

**1. [Rule 1 - Bug] MetroActivity::Clone needed for activity.clone() in MetroActivityUpdate**

- **Found during:** Task 2
- **Issue:** The `MetroActivityUpdate` handler needed to both store `activity` (`state.metro.activity = Some(activity.clone())`) and match on it in the same arm. Without `clone()` the move would have been consumed before the match. `MetroActivity` already derived `Clone` (confirmed in domain/metro.rs), so `.clone()` works cleanly.
- **Fix:** Changed `state.metro.activity = Some(activity)` to `state.metro.activity = Some(activity.clone())` and matched on `activity` directly after.
- **Files modified:** src/app.rs
- **Commit:** 900f79a

## Known Stubs

None.

## Threat Flags

None — all changes are internal state management within the TEA update loop. No new trust boundaries introduced.

## Self-Check: PASSED

- src/domain/command.rs exists and contains needs_metro()
- src/app.rs exists and contains pending_metro_run field and all three wiring points
- Commit 021d794 exists
- Commit 900f79a exists
