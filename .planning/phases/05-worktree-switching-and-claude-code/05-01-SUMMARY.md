---
phase: 05-worktree-switching-and-claude-code
plan: 01
subsystem: worktree-switching
tags: [worktree, tmux, metro, claude-code, keybindings]
dependency_graph:
  requires: [src/action.rs, src/app.rs, src/infra/mod.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
  provides: [WorktreeSwitchToSelected, OpenClaudeCode, pending_switch_path, open_claude_in_worktree, footer-hints, help-overlay-entries]
  affects: [src/app.rs, src/action.rs, src/infra/tmux.rs]
tech_stack:
  added: [src/infra/tmux.rs]
  patterns: [TEA action dispatch, tokio::spawn for blocking process calls, pending_switch_path captured at action time]
key_files:
  created:
    - src/infra/tmux.rs
  modified:
    - src/action.rs
    - src/app.rs
    - src/infra/mod.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
decisions:
  - "pending_switch_path captured at WorktreeSwitchToSelected dispatch time — user navigation during async stop gap cannot change the target"
  - "open_claude_in_worktree uses shell-command form (not send-keys) to eliminate race condition with shell init"
  - "OpenClaudeCode spawned via tokio::spawn to avoid blocking event loop on synchronous std::process::Command::status()"
  - "tmux -d flag used to prevent focus switch away from dashboard when opening Claude Code window"
  - "Enter keybinding placed in WorktreeList-specific block — no conflict with modal Enter or other contexts"
metrics:
  duration: 2 min
  completed: 2026-03-02
  tasks_completed: 2
  files_modified: 5
  files_created: 1
---

# Phase 5 Plan 01: Worktree Switch and Claude Code Integration Summary

**One-liner:** Enter-key metro worktree switching with pending_switch_path capture + C-key Claude Code tmux tab launch using shell-command form.

## What Was Built

Wired two new keybindings in the WorktreeList panel completing the final two v1 requirements:

1. **Enter key (WorktreeSwitchToSelected):** When pressed on a worktree, captures the target path immediately (before any navigation can change selection), then either stops metro + sets pending_restart (if running) or starts metro directly in the new worktree (if stopped). MetroExited handler consumes `pending_switch_path` to redirect `active_worktree_path` before restarting metro.

2. **C key (OpenClaudeCode):** When pressed, checks `tmux_available` — shows error overlay if not in tmux session, otherwise spawns `tmux new-window -d -c <path> -n "claude:<branch>" claude` via `tokio::spawn`. Uses shell-command form (not send-keys) to eliminate race conditions.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Action variants, AppState field, key bindings, update() handlers | 95034b5 | src/action.rs, src/app.rs, src/infra/mod.rs, src/infra/tmux.rs |
| 2 | Footer hints and help overlay | 6c064f2 | src/ui/footer.rs, src/ui/help_overlay.rs |

## Verification Results

- `cargo check` passes with zero errors
- No new warnings introduced by Phase 5 code (3 pre-existing warnings remain)
- `WorktreeSwitchToSelected` and `OpenClaudeCode` variants present in action.rs
- `pending_switch_path: Option<PathBuf>` field exists in AppState with None default
- MetroExited handler applies `pending_switch_path.take()` before MetroStart dispatch
- `open_claude_in_worktree()` uses `tmux new-window -d -c <path> -n <name> claude` (shell-command form, not send-keys)
- Footer shows "Enter switch" and "C claude" hints in WorktreeList panel
- Help overlay shows "Enter — Switch metro to worktree" and "C (shift-c) — Open Claude Code (tmux)"

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- FOUND: src/infra/tmux.rs
- FOUND: src/action.rs (WorktreeSwitchToSelected and OpenClaudeCode variants)
- FOUND: src/app.rs (pending_switch_path field, Enter/C key bindings, update() handlers)
- FOUND: 05-01-SUMMARY.md
- FOUND commit 95034b5: feat(05-01): add WorktreeSwitchToSelected and OpenClaudeCode actions with tmux infra
- FOUND commit 6c064f2: feat(05-01): update footer hints and help overlay with Phase 5 keybindings
