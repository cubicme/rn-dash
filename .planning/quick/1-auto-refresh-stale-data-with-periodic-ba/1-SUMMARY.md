---
phase: quick
plan: 1
subsystem: app-event-loop
tags: [refresh, polling, labels, worktrees]
dependency_graph:
  requires: []
  provides: [periodic-auto-refresh]
  affects: [src/app.rs]
tech_stack:
  added: []
  patterns: [tokio-interval-in-select, label-reload-on-worktrees-loaded]
key_files:
  created: []
  modified:
    - src/app.rs
decisions:
  - "60-second interval via tokio::time::interval — no last_refresh field needed, timer handles timing internally"
  - "Label reload placed at top of WorktreesLoaded handler — covers both periodic and manual Shift-R paths automatically"
  - "Refresh skipped when running_command is Some — prevents interference with active command output"
metrics:
  duration: 4min
  completed: "2026-03-14T12:41:37Z"
  tasks_completed: 1
  files_modified: 1
---

# Quick Task 1: Auto-refresh Stale Data with Periodic Background Polling — Summary

**One-liner:** 60-second background polling via tokio interval fires RefreshWorktrees when idle, with label disk-reload on every WorktreesLoaded.

## What Was Built

Added periodic background refresh to the dashboard event loop. Every 60 seconds (when no command is running), `Action::RefreshWorktrees` is dispatched automatically — the same action that Shift-R triggers. This causes `list_worktrees` to run in the background, then `WorktreesLoaded` updates worktrees, staleness indicators, metro status, and JIRA titles (via the existing cache+re-fetch logic already in the handler).

Labels are now reloaded from disk at the top of every `WorktreesLoaded` handler, meaning both periodic and manual refreshes always pick up external label edits (e.g., edits made in another terminal).

## Changes

### src/app.rs

- Added `refresh_interval` (60s tokio interval) after the existing 250ms `tick` in `run()`.
- Consumed the immediate first tick of `refresh_interval` to avoid firing at startup (startup already loads worktrees).
- Added new `select!` branch: `_ = refresh_interval.tick()` → fires `RefreshWorktrees` when `state.running_command.is_none()`.
- Added `state.labels = crate::infra::labels::load_labels().unwrap_or_default();` at the top of the `WorktreesLoaded` handler.

## Deviations from Plan

None — plan executed exactly as written. The plan note to NOT add `last_refresh` to AppState was followed; the tokio interval handles timing internally.

## Verification

- `cargo check` passes with no errors (7 pre-existing warnings, none new)
- Implementation covers all must-haves:
  - Worktree data auto-refreshes every 60 seconds without user interaction
  - Labels reload from disk on each periodic refresh
  - Manual Shift-R still works as before (goes through same RefreshWorktrees → WorktreesLoaded path)
  - Refresh does not fire when a command is running

## Self-Check: PASSED

- `src/app.rs` modified: confirmed
- Commit `13b4e82` exists: confirmed
