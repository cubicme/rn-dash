---
phase: quick
plan: 260322-niz
subsystem: app
tags: [palette, metro, ux-bug]
dependency_graph:
  requires: []
  provides: [metro-palette-auto-dismiss]
  affects: [src/app.rs]
tech_stack:
  added: []
  patterns: [palette_mode = None on action dispatch]
key_files:
  created: []
  modified:
    - src/app.rs
decisions:
  - "palette_mode cleared at the top of each metro action arm — mirrors the CommandRun pattern at line 754"
metrics:
  duration: 3min
  completed: 2026-03-22
---

# Quick Task 260322-niz: Metro Palette Should Reset to Root After Command

## One-liner

Metro palette (m>) now dismisses immediately on any command key (s/x/r/j/R) by clearing palette_mode in all five metro action handlers.

## What Was Done

Added `state.palette_mode = None;` as the first statement inside each of the five metro Action match arms in `update()`:

1. `Action::MetroStart` — before the `is_running()` check
2. `Action::MetroStop` — before `take_handle()`
3. `Action::MetroRestart` — before the `is_running()` branch
4. `Action::MetroSendDebugger` — before the `is_running()` check
5. `Action::MetroSendReload` — before the `is_running()` check

This matches the existing pattern used by `Action::CommandRun` (which already cleared palette_mode). The user pressing m>s, m>x, m>r, m>j, or m>R now returns to normal key handling mode immediately.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1    | 662c8ff | fix(quick-260322-niz): clear palette_mode on all five metro action handlers |

## Verification

`cargo check` passes with 0 errors, 7 pre-existing warnings (unchanged).

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `src/app.rs` modified: FOUND
- Commit 662c8ff: FOUND
