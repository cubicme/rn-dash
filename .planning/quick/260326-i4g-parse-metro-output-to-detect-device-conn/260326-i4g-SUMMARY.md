---
phase: quick-260326-i4g
plan: 01
subsystem: metro
tags: [metro, activity, ui, table, parsing]
dependency_graph:
  requires: []
  provides: [metro-activity-inline-display]
  affects: [src/domain/metro.rs, src/action.rs, src/app.rs, src/ui/panels.rs]
tech_stack:
  added: []
  patterns: [TEA-action-flow, inline-detail-row, string-parsing-without-regex]
key_files:
  created: []
  modified:
    - src/domain/metro.rs
    - src/action.rs
    - src/app.rs
    - src/ui/panels.rs
decisions:
  - "parse_metro_line uses .contains() / .to_lowercase() only — no regex crate added"
  - "extract_percent scans bytes manually to find digit-run followed by '%'"
  - "MetroManager::clear() now resets activity to None (was missing before)"
  - "MetroManager::set_starting() sets activity to Starting immediately on spawn"
  - "Detail row placed in ticket column (Constraint::Min(20)) — widest flexible column"
  - "Logical-to-visual index remapping wraps render_stateful_widget call"
metrics:
  duration: 2 min
  completed_date: "2026-03-27"
  tasks_completed: 2
  files_modified: 4
---

# Quick Task 260326-i4g: Parse Metro Output to Detect Device Connection

**One-liner:** Real-time metro activity (Ready/Bundling N%/Device connected/Error) parsed from stdout and displayed as an inline Cyan detail row beneath the metro-running worktree in the table.

## What Was Built

### Task 1 — MetroActivity type, action variant, parsing

`MetroActivity` enum was already defined in `src/domain/metro.rs` from earlier work. Added:

- `MetroActivityUpdate(MetroActivity)` variant to `src/action.rs` (after `MetroSpawnFailed`)
- `parse_metro_line(&str) -> Option<MetroActivity>` in `src/app.rs` — matches "Welcome to Metro" / "Fast - Scalable - Integrated" → Ready; "BUNDLE" → Bundling{percent}; "client connected" → DeviceConnected; "error" (excluding source-map/deprecated) → Error
- `extract_percent(&str) -> Option<u8>` — byte-scan for digit-run followed by '%'
- `drain_metro_output` now accepts `action_tx: UnboundedSender<Action>` and sends `MetroActivityUpdate` for each parsed line
- `update()` handler stores activity: `state.metro.activity = Some(activity)`
- `MetroManager::clear()` now also sets `self.activity = None`
- `MetroManager::set_starting()` now also sets `self.activity = Some(MetroActivity::Starting)`

### Task 2 — Detail row rendering in worktree table

`render_worktree_table()` refactored from `.map().collect()` to an explicit loop. After each worktree row where `metro_status == Running`, if `state.metro.activity` is `Some(activity)`, a non-selectable detail row is pushed with:
- Columns 1–3 and 5 are empty
- Column 4 (ticket, flexible width) shows `"\u{2502} {activity}"` in Cyan on Rgb(0,60,0) background

Selection remapping: before `render_stateful_widget`, logical index is offset upward past any detail rows inserted before it; after render, restored back to logical index so AppState stays consistent with `state.worktrees` indexing.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | b4477f4 | feat(quick-260326-i4g): add MetroActivity parsing and MetroActivityUpdate action |
| 2 | dde2715 | feat(quick-260326-i4g): render metro activity as detail row in worktree table |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] MetroManager::clear() did not reset activity field**
- **Found during:** Task 1 implementation review
- **Issue:** `clear()` set handle/status to stopped but left `activity` populated — stale activity would show after metro stopped
- **Fix:** Added `self.activity = None` to `clear()`
- **Files modified:** src/domain/metro.rs
- **Commit:** b4477f4

**2. [Rule 2 - Missing] MetroManager::set_starting() did not set activity to Starting**
- **Found during:** Task 1 implementation per plan spec
- **Issue:** Plan specified this should happen but the existing implementation didn't include it
- **Fix:** Added `self.activity = Some(MetroActivity::Starting)` to `set_starting()`
- **Files modified:** src/domain/metro.rs
- **Commit:** b4477f4

## Known Stubs

None — metro activity flows through the TEA loop to UI rendering. Data is live when metro is running.

## Self-Check: PASSED

- b4477f4 exists in git log: confirmed
- dde2715 exists in git log: confirmed
- src/domain/metro.rs modified: confirmed
- src/action.rs modified (MetroActivityUpdate added): confirmed
- src/app.rs modified (drain_metro_output, parse_metro_line, update handler): confirmed
- src/ui/panels.rs modified (detail row loop): confirmed
- `cargo check` passes with no errors (7 pre-existing warnings only): confirmed
