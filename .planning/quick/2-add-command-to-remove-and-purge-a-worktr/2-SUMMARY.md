---
phase: quick-2
plan: 1
subsystem: worktree-management
tags: [worktree, git, removal, tui, keybinding]
dependency_graph:
  requires: [src/action.rs, src/app.rs, src/infra/worktrees.rs]
  provides: [worktree-removal-command]
  affects: [src/ui/help_overlay.rs, src/ui/footer.rs]
tech_stack:
  added: []
  patterns: [TEA action/update, tokio::spawn async removal, sentinel CommandSpec pattern]
key_files:
  created: []
  modified:
    - src/action.rs
    - src/infra/worktrees.rs
    - src/app.rs
    - src/ui/help_overlay.rs
    - src/ui/footer.rs
decisions:
  - "Used pending_worktree_removal: Option<(WorktreeId, PathBuf, String)> on AppState — checked in ModalConfirm before the normal CommandSpec dispatch, matching the sentinel pattern already used for label input and claude tab name"
  - "Used GitPull as sentinel CommandSpec in Confirm modal — same proven pattern as YarnLint in StartSetLabel and OpenClaudeCode handlers"
  - "git worktree remove --force chosen unconditionally — avoids needing to detect dirty state; user already confirmed the destructive action via the modal"
  - "ModalCancel also clears pending_worktree_removal — prevents state leak if user cancels after g>D"
  - "Metro stopped synchronously via update() recursive call before async spawn — ensures stop signal is sent even if metro is on the same worktree being removed"
metrics:
  duration: "2 min"
  completed_date: "2026-03-21"
  tasks: 2
  files_modified: 5
---

# Quick Task 2: Add g>D Command to Remove and Purge a Worktree

**One-liner:** Worktree removal via g>D with confirm modal, main-worktree guard, metro stop, per-worktree state cleanup, and auto-refresh using `git worktree remove --force` + `git worktree prune`.

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Add WorktreeRemove action, remove_worktree infra, and confirm modal flow | 6f23503 | src/action.rs, src/infra/worktrees.rs, src/app.rs |
| 2 | Update help overlay and footer hints for g>D | 2704df4 | src/ui/help_overlay.rs, src/ui/footer.rs |

## What Was Built

### New Action Variants (src/action.rs)
- `WorktreeRemove` — user-triggered from git palette via g>D
- `WorktreeRemoved(String)` — background callback on success (carries path for logging)
- `WorktreeRemoveFailed(String)` — background callback on failure (carries error message)

### New Infra Function (src/infra/worktrees.rs)
`remove_worktree(repo_root, worktree_path)` runs:
1. `git worktree remove --force <path>` — unconditional removal
2. `git worktree prune` — cleans stale `.git/worktrees/<name>` entries
3. Logs warning if directory still exists post-removal (safety check)

### App Logic (src/app.rs)

**AppState addition:**
```rust
pub pending_worktree_removal: Option<(WorktreeId, PathBuf, String)>,
```

**handle_key Git palette:** `Char('D') => Some(Action::WorktreeRemove)`

**Action::WorktreeRemove handler:**
- Guard: if `wt.path == state.repo_root` → error "Cannot remove the main worktree"
- Stores `pending_worktree_removal` with id, path, branch
- Shows Confirm modal with branch name; adds "(metro will be stopped)" note if metro is active on that path
- Clears palette_mode

**Action::ModalConfirm modification:**
- Checks `pending_worktree_removal` first (before normal CommandSpec dispatch)
- Stops metro if it's running on the targeted path
- Cleans up `command_output_by_worktree` and `command_output_scroll_by_worktree` for the removed worktree
- Spawns async `remove_worktree()` — sends WorktreeRemoved or WorktreeRemoveFailed

**Action::WorktreeRemoved:** Spawns `list_worktrees` → `WorktreesLoaded` to refresh the table.

**Action::WorktreeRemoveFailed:** Sets error_state with the git error message.

**Action::ModalCancel:** Also clears `pending_worktree_removal` to prevent stale state.

### UI Updates
- `src/ui/help_overlay.rs`: Added `Row::new(vec!["D", "Remove worktree (purge)"])` in Git submenu section
- `src/ui/footer.rs`: Added `("D", "remove wt")` in PaletteMode::Git hints

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo check` passes with no errors (7 pre-existing dead_code warnings, unchanged)
- `cargo clippy` passes with no new errors
- Manual test required to confirm: g>D on non-main worktree shows modal, Y removes it, list refreshes; g>D on main worktree shows error

## Self-Check: PASSED

Files exist:
- src/action.rs — modified, WorktreeRemove/WorktreeRemoved/WorktreeRemoveFailed added
- src/infra/worktrees.rs — modified, remove_worktree added
- src/app.rs — modified, pending_worktree_removal field + all handlers
- src/ui/help_overlay.rs — modified, D row in Git section
- src/ui/footer.rs — modified, D hint in Git palette

Commits exist:
- 6f23503 feat(quick-2): add WorktreeRemove action, remove_worktree infra, and confirm modal flow
- 2704df4 feat(quick-2): update help overlay and footer hints for g>D
