---
phase: quick
plan: 260322-7pi
subsystem: command-dispatch
tags: [ios, device, command-spec, footer, help-overlay]
dependency_graph:
  requires: []
  provides: [RnRunIosDevice command variant, i>d auto-device keybinding]
  affects: [src/domain/command.rs, src/app.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [new CommandSpec variant, bypass needs_device_selection pipeline]
key_files:
  created: []
  modified:
    - src/domain/command.rs
    - src/app.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
decisions:
  - RnRunIosDevice has no fields — --device flag auto-selects the first connected physical device without needing a UDID
  - RnRunIosDevice excluded from needs_device_selection to bypass enumeration and device picker entirely
  - needs_pods check extended to include RnRunIosDevice so pod staleness triggers SyncBeforeRun prompt on iOS device runs
metrics:
  duration: 5min
  completed_date: "2026-03-22"
  tasks: 2
  files: 4
---

# Quick Task 260322-7pi: Skip iOS Device Picker Summary

**One-liner:** Added `RnRunIosDevice` variant using `run-ios --device` so `i>d` auto-selects the first connected physical iOS device without showing a device picker.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add RnRunIosDevice variant and wire through command dispatch | 330d834 | src/domain/command.rs, src/app.rs |
| 2 | Update footer hints and help overlay text | 25a4592 | src/ui/footer.rs, src/ui/help_overlay.rs |

## What Was Built

- New `RnRunIosDevice` variant added to `CommandSpec` enum with `to_argv()` producing `["yarn", "react-native", "run-ios", "--device"]`
- `i>d` in the iOS palette now dispatches `RnRunIosDevice` instead of `RnRunIos` — no device enumeration, no picker modal
- `i>e` continues dispatching `RnRunIos { device_id }` which goes through the simulator picker as before
- `SyncBeforeRun` check extended to include `RnRunIosDevice` — stale worktrees still trigger the sync prompt
- `needs_pods` check extended to include `RnRunIosDevice` — pod staleness detection works for physical device runs
- Footer hints updated: `d` = "run-ios --device", `e` = "simulator list"
- Help overlay updated: `d` row = "run-ios --device (auto-select)", `e` row = "Simulator list (xcrun)"

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

Files created/modified:
- FOUND: src/domain/command.rs
- FOUND: src/app.rs
- FOUND: src/ui/footer.rs
- FOUND: src/ui/help_overlay.rs

Commits:
- FOUND: 330d834
- FOUND: 25a4592

cargo check: passes cleanly (7 pre-existing warnings, 0 errors)
