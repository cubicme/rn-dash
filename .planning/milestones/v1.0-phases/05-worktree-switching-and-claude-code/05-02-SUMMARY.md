---
phase: 05-worktree-switching-and-claude-code
plan: "02"
subsystem: metro-ux
tags: [metro, error-handling, ui, streaming]
dependency_graph:
  requires: []
  provides: [metro-spawn-error-overlay, metro-log-streaming-in-pane]
  affects: [src/action.rs, src/app.rs, src/infra/process.rs, src/ui/panels.rs]
tech_stack:
  added: []
  patterns: [MetroSpawnFailed action, always-on DEBUG=Metro:* filter, auto-scroll log rendering]
key_files:
  created: []
  modified:
    - src/action.rs
    - src/app.rs
    - src/infra/process.rs
    - src/ui/panels.rs
decisions:
  - "MetroSpawnFailed(String) action variant chosen over MetroExited for error case — surfaces error message to user via error_state overlay with can_retry: true"
  - "filter parameter removed from spawn_metro entirely — always set DEBUG=Metro:* since streaming is always desired"
  - "MetroToggleLog decoupled from metro restart — toggle only shows/hides dedicated log panel, no restart needed"
  - "log_filter_active kept in AppState (always true) to avoid refactoring render code that reads it"
  - "pid suppressed from metro pane status line — worktree name is the user-relevant context"
metrics:
  duration: "4 min"
  completed: "2026-03-03"
  tasks: 2
  files_modified: 4
---

# Phase 5 Plan 02: Metro Error Surfacing and Log Streaming Summary

**One-liner:** MetroSpawnFailed action surfaces spawn errors to error overlay; metro pane now renders scrolling stdout/stderr output with always-on DEBUG=Metro:* filter.

## What Was Built

Two UAT Test 1 gaps closed:

**A. Metro spawn errors surfaced (was: silently swallowed)**
- Added `MetroSpawnFailed(String)` action variant to `action.rs`
- `spawn_metro_task` now sends `MetroSpawnFailed(format!("{e}"))` on error instead of `MetroExited`
- `update()` handler for `MetroSpawnFailed` sets `error_state` overlay with `can_retry: true`, clears `pending_restart` and `pending_switch_path` to prevent stuck worktree switches
- User sees: "Metro failed to start: [error message]" overlay with retry option

**B. Metro pane shows actual output (was: static status string only)**
- `render_metro_pane` rewritten to show status line at top + scrolling metro_logs below
- Auto-scrolls to bottom when `log_scroll_offset == 0` (latest output always visible)
- Scrollbar appears when content exceeds visible height (matches render_log_panel pattern)
- Both metro pane and dedicated log panel ('l' key) read from the same `state.metro_logs` VecDeque

**C. Filter always active**
- `ProcessClient::spawn_metro` filter parameter removed — `DEBUG=Metro:*` always set
- `MetroStart` no longer reads `log_filter_active` — calls `spawn_metro_task` with 3 args
- `MetroToggleLog` simplified to toggle-only — no metro restart, no filter coupling
- `log_filter_active` defaults to `true` in `AppState::default()` (field kept to avoid render refactor)

## Commits

- `8dab14d` feat(05-02): surface metro spawn errors and always-stream metro output
- `2e2d511` feat(05-02): render metro logs directly in metro pane with auto-scroll

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- src/action.rs: MetroSpawnFailed variant present
- src/app.rs: MetroSpawnFailed handler sets error_state, spawn_metro_task has 3 params
- src/infra/process.rs: filter param removed, DEBUG=Metro:* always set
- src/ui/panels.rs: render_metro_pane renders metro_logs with auto-scroll
- cargo check: passed (3 pre-existing dead_code warnings, no new warnings)
