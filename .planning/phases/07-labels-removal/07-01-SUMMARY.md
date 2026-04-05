---
phase: 07-labels-removal
plan: "01"
subsystem: infra/domain/app/ui
tags: [cleanup, removal, refactor]
dependency_graph:
  requires: []
  provides: [clean-codebase-no-labels]
  affects: [src/infra/config.rs, src/domain/worktree.rs, src/action.rs, src/app.rs, src/ui/panels.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [module-relocation, dead-code-removal]
key_files:
  created: []
  modified:
    - src/infra/config.rs
    - src/infra/mod.rs
    - src/infra/jira_cache.rs
    - src/infra/android_prefs.rs
    - src/infra/sim_history.rs
    - src/infra/worktrees.rs
    - src/domain/worktree.rs
    - src/action.rs
    - src/app.rs
    - src/ui/panels.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
  deleted:
    - src/infra/labels.rs
decisions:
  - "config_dir() relocated to config.rs — natural home for config-directory function"
  - "Both tasks committed as one atomic commit — Task 1 alone does not compile"
metrics:
  duration_minutes: 12
  completed_date: "2026-04-05T16:10:13Z"
  tasks_completed: 2
  files_changed: 13
requirements: [CLN-01]
---

# Phase 07 Plan 01: Remove Labels Feature Summary

**One-liner:** Deleted `labels.rs`, relocated `config_dir()` to `config.rs`, stripped label field from `Worktree`, `SetLabel`/`StartSetLabel` from `Action`, all label handlers from `app.rs`, and the label column + keybinding hints from the UI — codebase compiles cleanly with zero label references.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Relocate config_dir and remove labels infra + domain + action | b736c17 | labels.rs (deleted), config.rs, mod.rs, jira_cache.rs, android_prefs.rs, sim_history.rs, worktrees.rs, worktree.rs, action.rs |
| 2 | Remove label references from app.rs and UI, verify clean build | b736c17 | app.rs, panels.rs, footer.rs, help_overlay.rs |

Note: Tasks 1 and 2 were committed together as a single atomic commit because Task 1 alone (with `label` field removed from `Worktree`) would fail to compile until app.rs and UI references were also removed.

## What Was Done

1. **Deleted `src/infra/labels.rs`** — removed `load_labels()`, `save_labels()`, `labels_path()` and the `#[allow(dead_code)]` guard that had been keeping it alive.

2. **Relocated `config_dir()` to `src/infra/config.rs`** — the function is now the natural home for this helper (config directory path lives next to config loading/saving). Updated the module doc comment to remove the stale cross-reference to `labels.rs`.

3. **Updated 3 `config_dir` consumers** — `jira_cache.rs`, `android_prefs.rs`, and `sim_history.rs` each changed from `use crate::infra::labels::config_dir` to `use crate::infra::config::config_dir`.

4. **Removed `pub mod labels` from `src/infra/mod.rs`.**

5. **Cleaned `src/domain/worktree.rs`** — removed `pub label: Option<String>` field, updated `display_name()` and `preferred_prefix()` to skip the label priority check, updated doc comments.

6. **Cleaned `src/infra/worktrees.rs`** — removed `label: None` from the `Worktree` constructor.

7. **Cleaned `src/action.rs`** — removed `SetLabel { branch, label }` and `StartSetLabel` variants and their phase comment.

8. **Cleaned `src/app.rs`** — removed:
   - `pub labels: HashMap<String, String>` state field
   - `pub pending_label_branch: Option<String>` state field
   - Both `Default` initializers for those fields
   - `Char('L') => Action::StartSetLabel` keybinding
   - Label reload block in `WorktreesLoaded` handler
   - Label submit block in `ModalInputSubmit` handler (converted `else if` chain to plain `if`)
   - `Action::SetLabel` and `Action::StartSetLabel` match arms with phase comment
   - Startup `load_labels()` call

9. **Cleaned `src/ui/panels.rs`** — removed `label` local variable, `Cell::from(truncate(label, 12))` from main row, corresponding empty cell from detail row, and `Constraint::Length(14)` from table constraints.

10. **Cleaned `src/ui/footer.rs`** — removed `("L", "label")` from WorktreeTable hint list.

11. **Cleaned `src/ui/help_overlay.rs`** — removed `Row::new(vec!["L", "Set custom branch label"])`.

## Verification

- `cargo build` — succeeded, zero errors
- `cargo clippy` — zero label-related warnings
- `grep -rn "infra::labels" src/` — 0 matches
- `test -f src/infra/labels.rs` — file does not exist
- All 14 acceptance criteria verified green

## Deviations from Plan

None — plan executed exactly as written, with the sole note that Tasks 1 and 2 were batched into a single commit (the intermediate state between tasks is not a valid compilation unit).

## Known Stubs

None.

## Threat Flags

None — this is a pure deletion/relocation; no new trust boundaries introduced.

## Self-Check: PASSED

- `src/infra/labels.rs` — MISSING (correctly deleted)
- `src/infra/config.rs` — FOUND, contains `pub fn config_dir`
- `src/domain/worktree.rs` — FOUND, no `label` field
- `src/action.rs` — FOUND, no `SetLabel`/`StartSetLabel`
- Commit b736c17 — FOUND
