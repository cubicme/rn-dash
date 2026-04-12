---
phase: 08-palette-and-keybinding-rework
plan: "04"
subsystem: keybindings, ui/footer, ui/help_overlay, app-state
tags: [palette, keybindings, metro, uat-gap-closure, ux]
dependency_graph:
  requires: ["08-01", "08-03"]
  provides: [lowercase-worktree-palette, metro-palette-removed, app-managed-metro-switch-fix]
  affects: [src/app.rs, src/action.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [lowercase-palette-keys, skip-external-detection-flag]
key_files:
  created: []
  modified:
    - src/app.rs
    - src/action.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
decisions:
  - "Metro palette removed entirely — no `m` key entry anywhere; context-sensitive R/J/Esc from 08-03 covers all metro control"
  - "Added `skip_external_metro_check: bool` to AppState — set in MetroExited (when pending_restart), consumed in MetroStart to bypass port detection"
  - "Worktree palette keys lowercased to match other palette conventions (g>, w>, c>)"
metrics:
  completed_date: "2026-04-05"
  tasks_completed: 2
  files_changed: 4
---

# Phase 08 Plan 04: Palette Cleanup + Metro Switch Fix Summary

**One-liner:** Closes three UAT gaps — lowercased worktree palette keys (w/d/b), fully removed the metro palette, and bypassed external-metro detection during app-managed worktree switches — plus drops the stale `j/k navigate` footer hint.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Palette cleanup — lowercase worktree keys, remove metro palette, drop j/k hint | c3613b6 | src/app.rs, src/action.rs, src/ui/footer.rs, src/ui/help_overlay.rs |
| 2 | Fix metro conflict detection during app-managed worktree switch | e2cae20 | src/app.rs |

## What Was Built

**Task 1 — Palette cleanup (UAT gaps test 4, 7 + user request):**
- `src/app.rs:378` — Worktree palette arm now matches `Char('w') | Char('d') | Char('b')` for Add/Remove/AddNewBranch.
- `src/action.rs` — `EnterMetroPalette` variant removed.
- `src/app.rs` — entire `PaletteMode::Metro` match arm, the `Char('m') => EnterMetroPalette` dispatch, and the `EnterMetroPalette` update handler all removed; `PaletteMode::Metro` enum variant deleted.
- `src/ui/footer.rs` — `PaletteMode::Metro` hints arm removed, `("m", "metro")` stripped from WorktreeTable hints, `("j/k", "navigate")` removed from WorktreeTable.
- `src/ui/help_overlay.rs` — Metro submenu section + `m Metro submenu` row removed from Worktree Table section.

**Task 2 — Metro switch conflict fix (UAT gap test 9):**
- `src/app.rs` — new `AppState::skip_external_metro_check: bool` field (default `false`).
- `MetroExited` handler now sets `state.skip_external_metro_check = true` before dispatching `MetroStart` when `pending_restart` was true (covers both manual restart and worktree switch flows, since switch goes through `pending_restart`).
- `MetroStart` handler consumes the flag: if set, clears it and sends `MetroStartConfirmed` directly, skipping `detect_external_metro(8081)` — avoids false "external conflict" when the just-killed own process hasn't released the port yet.

## Deviations from Plan

Plan proposed gating on `pending_switch_path`, then correctly noted it is consumed in `MetroExited` before `MetroStart` runs. Final implementation uses the dedicated `skip_external_metro_check` flag as recommended in the plan's revised fix.

## Known Stubs

None.

## Threat Flags

None — `skip_external_metro_check` is an internal boolean with no external input path (T-08-04-01 accepted).

## Self-Check: PASSED

- `grep -c "Char('W')" src/app.rs` = 0 ✓
- `grep -c "EnterMetroPalette"` across action.rs + app.rs = 0 ✓
- `grep -c "PaletteMode::Metro"` across app.rs + footer.rs = 0 ✓
- `grep -c "skip_external_metro_check" src/app.rs` = 5 (definition, init, set, check, consume) ✓
- `ExternalMetroConflict` still present for genuine external conflicts ✓
- commits c3613b6, e2cae20 — FOUND
- `cargo check` — PASSED
