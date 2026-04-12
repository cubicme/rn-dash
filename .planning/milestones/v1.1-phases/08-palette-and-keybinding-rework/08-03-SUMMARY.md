---
phase: 08-palette-and-keybinding-rework
plan: "03"
subsystem: keybindings, ui/footer, app-state
tags: [keybindings, metro, footer, context-sensitive, ux]
dependency_graph:
  requires: [08-01, 08-02]
  provides: [context-sensitive-metro-keys, dynamic-footer-hints]
  affects: [src/app.rs, src/action.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [conditional-key-dispatch, dynamic-hint-derivation]
key_files:
  created: []
  modified:
    - src/app.rs
    - src/action.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
decisions:
  - "MetroRestart action variant removed entirely — no internal dispatch, only user-triggered"
  - "Esc stops metro from worktree table when running; does nothing otherwise (safe, no prior mapping)"
  - "Footer renders single full-width hint line — no horizontal split, no static legend"
metrics:
  duration_min: 12
  completed_date: "2026-04-05T16:33:33Z"
  tasks_completed: 2
  files_modified: 4
---

# Phase 08 Plan 03: Context-Sensitive Metro Keys Summary

Context-sensitive R/J/Esc metro keys, dynamic footer hints, and stale footer legend removal — users now see only currently actionable keys.

## Tasks Completed

### Task 1: Make metro keys context-sensitive and add ESC-to-stop
**Commit:** `ca33695`

- `R` in worktree table: dispatches `MetroSendReload` when metro running, `RefreshWorktrees` when not
- `J` in worktree table: dispatches `MetroSendDebugger` only when metro running; no-op otherwise
- `Esc` in worktree table: dispatches `MetroStop` when metro running; no-op otherwise (safe — no prior ESC mapping in worktree table normal mode)
- Metro palette `Char('r') => MetroRestart` key mapping removed
- `Action::MetroRestart` variant removed from `action.rs` (user-only, no internal dispatches)
- `update()` handler for `MetroRestart` removed from `app.rs`
- `help_overlay.rs`: `R` row updated to "Reload metro (when running) / Refresh list"; new `J` and `Esc` rows added for context-sensitive actions; `r Restart metro` row removed from Metro section

### Task 2: Replace hardcoded footer with dynamic hints and remove legend
**Commit:** `71f30a5`

- `render_footer()`: removed right-aligned legend (▶=metro ⚠=stale), removed horizontal `Layout::Horizontal` split, renders single full-width hint line
- `key_hints_for()`: WorktreeTable arm replaced with dynamic builder — R/J/Esc hints appended only when `state.metro.is_running()`
- Metro palette hints: removed `("r", "restart")` entry
- Removed unused `Color` and `Style` imports from `footer.rs`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed BranchPickerFilter dereference error**
- **Found during:** Task 1 (cargo check)
- **Issue:** `Char(c) => Some(Action::BranchPickerFilter(*c))` — `c` is `char` (Copy type), not a reference; `*c` is invalid
- **Fix:** Changed `*c` to `c`
- **Files modified:** `src/app.rs` line 305
- **Commit:** `ca33695` (included in Task 1 commit)

**2. [Rule 2 - Pre-existing clippy] Clippy warnings in modified files**
- **Found during:** Final verification
- **Issue:** `format!(" {}  ", desc)` in `footer.rs:19` triggers `uninlined_format_args`; `app.rs` has additional pre-existing clippy warnings
- **Assessment:** All pre-existing, not introduced by this plan. Scope boundary: not fixed here.
- **Deferred:** Pre-existing clippy warnings across the codebase are out of scope for this plan

## Known Stubs

None — all functionality is fully wired.

## Threat Flags

None — changes are purely UI hint rendering and key routing logic (no new trust boundaries per threat model).

## Self-Check

Files exist:
- src/app.rs - modified
- src/action.rs - modified
- src/ui/footer.rs - modified
- src/ui/help_overlay.rs - modified

Commits exist:
- ca33695 (Task 1)
- 71f30a5 (Task 2)

## Self-Check: PASSED
