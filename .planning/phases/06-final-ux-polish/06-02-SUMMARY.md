---
phase: 06-final-ux-polish
plan: "02"
subsystem: ui/action
tags: [shell-tab, prefix-ordering, keybinding, footer, help-overlay]
dependency_graph:
  requires: [src/infra/multiplexer.rs]
  provides: [OpenShellTab action, T key binding, {prefix}-shell tab name, fixed {prefix}-claude ordering]
  affects: [src/action.rs, src/app.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [spawn_blocking for multiplexer calls, $SHELL env var fallback to /bin/zsh]
key_files:
  created: []
  modified:
    - src/action.rs
    - src/app.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
decisions:
  - OpenShellTab uses $SHELL env var with /bin/zsh fallback — consistent with system shell preference
  - Prefix ordering fixed to {prefix}-type for both claude and shell tab names
metrics:
  duration: "3 min"
  completed: "2026-03-12"
  tasks_completed: 2
  files_modified: 4
---

# Phase 06 Plan 02: Open Shell Tab Command and Prefix Ordering Fix Summary

OpenShellTab action (T key) opens $SHELL in a new tmux/zellij tab at the selected worktree with {prefix}-shell naming; OpenClaudeCode prefix ordering fixed from claude-{prefix} to {prefix}-claude.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add OpenShellTab action and fix prefix ordering | 07235bd | src/action.rs, src/app.rs |
| 2 | Update footer hints and help overlay for T key | 438fcc8 | src/ui/help_overlay.rs (footer.rs already applied in 06-01) |

## What Was Built

### OpenShellTab Action
- New `OpenShellTab` variant added to `Action` enum with comment
- `Char('T')` keybinding mapped in the `WorktreeTable` arm of `handle_key()`
- Handler in `update()` follows identical pattern to `OpenClaudeCode`:
  - Guards on `state.multiplexer.is_none()` → error_state with descriptive message
  - Gets selected worktree with index bounds clamping
  - Names tab as `{preferred_prefix}-shell` (e.g., "e2e-shell")
  - Reads `$SHELL` env var (fallback `/bin/zsh`)
  - Spawns via `tokio::task::spawn_blocking` → `detect_multiplexer().new_window()`

### Prefix Ordering Fix (OpenClaudeCode)
- `format!("claude-{}", wt.preferred_prefix())` → `format!("{}-claude", wt.preferred_prefix())`
- Consistent with OpenShellTab naming convention: `{prefix}-{type}`

### Footer and Help Overlay Updates
- Footer: `("T", "shell tab")` added after `("C", "claude")` in WorktreeTable hints
- Help overlay: `Row::new(vec!["T", "Open shell tab at worktree"])` after Claude Code row

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

All key files present. Both task commits verified (07235bd, 438fcc8). Build completes without errors.
