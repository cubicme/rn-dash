---
phase: 08-palette-and-keybinding-rework
plan: "02"
subsystem: ui/modals, infra/worktrees, app-state
tags: [worktree, branch-picker, modal, git, keybindings]
dependency_graph:
  requires: [08-01]
  provides: [branch-picker-modal, new-branch-worktree-flow]
  affects: [src/app.rs, src/action.rs, src/domain/command.rs, src/infra/worktrees.rs, src/ui/footer.rs, src/ui/modals.rs]
tech_stack:
  added: []
  patterns: [tokio::process::Command, spawned-background-task, modal-state-machine, type-to-filter]
key_files:
  created: []
  modified:
    - src/infra/worktrees.rs
    - src/action.rs
    - src/app.rs
    - src/domain/command.rs
    - src/ui/footer.rs
    - src/ui/modals.rs
decisions:
  - "Branch picker follows DevicePicker filter pattern — typing any char appends to filter; arrows navigate"
  - "add_worktree_new_branch places new worktree at repo_root.parent().join(new_branch)"
  - "Remote branch list strips 'origin/' prefix, excludes HEAD pointers, sorted + deduped"
metrics:
  completed_date: "2026-04-05"
  tasks_completed: 2
  files_changed: 6
---

# Phase 08 Plan 02: Worktree — New Branch Flow Summary

**One-liner:** `w>B` now opens a filterable remote-branch picker, prompts for a new branch name, then creates a worktree on that branch from the selected base via `git worktree add -b`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add `list_remote_branches` and `add_worktree_new_branch` infra functions | 63bfb6b | src/infra/worktrees.rs |
| 2 | Wire `WorktreeAddNewBranch` action with branch picker + name input flow | 004150a | src/action.rs, src/app.rs, src/domain/command.rs, src/ui/footer.rs, src/ui/modals.rs |

## What Was Built

- `src/infra/worktrees.rs:285` — `list_remote_branches(repo_root)` shells out to `git branch -r --no-color`, strips `origin/`, filters HEAD pointers, sorts+dedupes.
- `src/infra/worktrees.rs:309` — `add_worktree_new_branch(repo_root, new_branch, base_branch)` runs `git worktree add -b <new> <path> origin/<base>`, refusing to clobber an existing directory.
- `src/action.rs:140` — new variants: `BranchesLoaded`, `BranchPickerConfirm`, `BranchPickerNext/Prev`, `BranchPickerFilter(char)`, `BranchPickerBackspace`, `WorktreeNewBranchCreated`, `WorktreeNewBranchFailed`.
- `src/domain/command.rs` — new `ModalState::BranchPicker { branches, selected, filter }`.
- `src/app.rs:378` — `Char('b') => WorktreeAddNewBranch` wired in worktree palette; `Action::WorktreeAddNewBranch` handler (line 1924) spawns `list_remote_branches`, opens `BranchPicker`, then on confirm stashes the base in `pending_new_branch_base` and opens a `TextInput` modal for the new branch name. `ModalInputSubmit` branches on `pending_new_branch_worktree` to call `add_worktree_new_branch` and refresh the list.
- `src/ui/modals.rs` — `BranchPicker` rendering: filtered list with highlight, filter string displayed.
- `src/ui/footer.rs` — BranchPicker hints: `Enter select / Up-Down navigate / type filter / Esc cancel`.

## Deviations from Plan

None — the key dispatch ended up lowercase `b` (not uppercase `B`) after Plan 04's palette casing rework, but the wiring path is identical.

## Known Stubs

None.

## Threat Flags

None — branch name passes to git via `args[]` (not shell-interpolated), per threat model T-08-03.

## Self-Check: PASSED

- `src/infra/worktrees.rs` — `list_remote_branches` FOUND (line 285), `add_worktree_new_branch` FOUND (line 309)
- `src/action.rs:140` — `WorktreeAddNewBranch` FOUND; `BranchesLoaded`, `BranchPickerConfirm`, `WorktreeNewBranchCreated` FOUND
- `src/domain/command.rs` — `BranchPicker` variant FOUND
- `src/app.rs` — 18 references to `BranchPicker`, handler at line 1924
- `src/ui/modals.rs` — BranchPicker rendering FOUND
- commits 63bfb6b, 004150a — FOUND in git log
- `cargo check` — PASSED
