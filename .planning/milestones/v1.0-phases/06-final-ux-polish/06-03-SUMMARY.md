---
phase: 06-final-ux-polish
plan: "03"
subsystem: app-ui
tags:
  - modal
  - claude-code
  - title-bar
  - layout
dependency_graph:
  requires:
    - 06-02
  provides:
    - claude-tab-name-modal
    - title-bar-widget
  affects:
    - src/app.rs
    - src/ui/mod.rs
    - src/ui/panels.rs
tech_stack:
  added: []
  patterns:
    - pending field pattern (pending_claude_open mirrors pending_label_branch)
    - 4-row normal layout with title area
key_files:
  created: []
  modified:
    - src/app.rs
    - src/ui/mod.rs
    - src/ui/panels.rs
decisions:
  - "pending_claude_open stores worktree dir name (not full path) to survive async gap"
  - "TextInput modal sentinel uses YarnLint CommandSpec (same pattern as StartSetLabel)"
  - "Title bar uses Constraint::Length(3): exactly 2 border lines + 1 content line"
  - "Fullscreen branch intentionally unchanged — no title bar in fullscreen to preserve vertical space"
metrics:
  duration: "1 min"
  completed_date: "2026-03-12"
  tasks_completed: 2
  files_modified: 3
---

# Phase 6 Plan 3: Optional Claude Tab Name Modal and Title Bar Summary

**One-liner:** TextInput modal before Claude tab spawn lets users name tabs with custom suffix, plus double-bordered title bar in normal layout.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add optional Claude tab name modal flow | ecf6236 | src/app.rs |
| 2 | Add title bar with double border to normal layout | b9bc610 | src/ui/panels.rs, src/ui/mod.rs |

## What Was Built

**Task 1 — Claude tab name modal:**
- Added `pending_claude_open: Option<String>` field to `AppState` (stores worktree dir name)
- `OpenClaudeCode` handler now opens `ModalState::TextInput` with prompt "Claude tab suffix:" instead of immediately spawning
- `ModalInputSubmit` handles `pending_claude_open`: empty buffer uses "claude" as suffix, typed text becomes custom suffix
- Tab name format: `{preferred_prefix}-{suffix}` (e.g. "ump-1234-claude" or "ump-1234-debug")
- `ModalCancel` clears `pending_claude_open` to prevent state leak on Esc

**Task 2 — Title bar:**
- Added `render_title_bar()` to `src/ui/panels.rs` with `BorderType::Double` and bold "UMP Dashboard" title
- Extended normal layout from 3-row to 4-row: `[title(3), top(min8), table(fixed), footer(1)]`
- `panels::render_title_bar()` called before the top section in normal layout
- Fullscreen branch is unchanged — no title bar wasted space in fullscreen mode

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

Files created/modified:
- FOUND: src/app.rs (pending_claude_open field + modal flow)
- FOUND: src/ui/panels.rs (render_title_bar function)
- FOUND: src/ui/mod.rs (4-row layout with title_area)

Commits:
- FOUND: ecf6236 feat(06-03): add optional Claude tab name modal flow
- FOUND: b9bc610 feat(06-03): add title bar with double border to normal layout

Build: cargo build passes with 0 errors (6 pre-existing warnings unrelated to this plan).
