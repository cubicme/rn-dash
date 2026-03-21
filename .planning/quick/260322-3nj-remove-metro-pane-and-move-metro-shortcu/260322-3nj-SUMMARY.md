---
phase: quick
plan: 260322-3nj
subsystem: ui
tags: [metro, palette, layout, focus, cleanup]
dependency_graph:
  requires: []
  provides: [metro-palette-m, two-panel-layout]
  affects: [src/app.rs, src/action.rs, src/ui/mod.rs, src/ui/panels.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [palette-mode, drain-pattern]
key_files:
  created: []
  modified:
    - src/action.rs
    - src/app.rs
    - src/ui/mod.rs
    - src/ui/panels.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
decisions:
  - Metro pane removed — logs are noise, drain_metro_output prevents pipe buffer blocking without display
  - Metro control moved to 'm>' palette (s=start, x=stop, r=restart, j=debugger, R=reload)
  - Focus cycle reduced to WorktreeTable <-> CommandOutput (two panels only)
  - MetroSendDebugger/Reload now use tracing::warn/info instead of MetroLogMessage channel
metrics:
  duration: 5min
  completed: 2026-03-22
---

# Quick Task 260322-3nj: Remove Metro Pane and Move Metro Shortcuts to 'm' Palette

**One-liner:** Removed metro log pane entirely, replacing it with a two-panel layout (command output + worktree table) and a new 'm>' metro control palette accessible from the worktree table.

## What Was Built

- **Two-panel layout**: Command output takes the full top area; metro pane and all log display are gone
- **Focus cycling**: `WorktreeTable -> CommandOutput -> WorktreeTable` (removed `MetroPane` variant)
- **Metro 'm>' palette**: Press `m` from worktree table to enter metro submenu — `s`=start, `x`=stop, `r`=restart, `j`=debugger (HTTP POST /open-debugger), `R`=reload (HTTP POST /reload), `Esc`=cancel
- **drain_metro_output**: Replaces `stream_metro_logs` — still drains stdout/stderr to prevent pipe buffer blocking, but discards all lines instead of sending them as actions
- **Footer and help overlay**: Both updated to show `m` hint in WorktreeTable and a Metro (m>) palette section

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Remove metro pane from rendering, layout, and focus cycling | 8a2cddb | src/action.rs, src/app.rs, src/ui/mod.rs, src/ui/panels.rs, src/ui/footer.rs, src/ui/help_overlay.rs |
| 2 | Add 'm' metro palette with start/stop/restart/debugger/reload shortcuts | 5d1ee23 | src/action.rs, src/app.rs, src/ui/footer.rs, src/ui/help_overlay.rs |

## Actions Removed

- `MetroToggleLog` — no log panel to toggle
- `MetroScrollUp` / `MetroScrollDown` — no log panel to scroll
- `MetroLogMessage(String)` — no log buffer to push to
- `MetroLogLine(String)` — no log buffer
- `LogPanelClear` — no log panel

## State Fields Removed

- `metro_logs: VecDeque<String>` — log buffer
- `log_scroll_offset: usize`
- `log_panel_visible: bool`
- `log_filter_active: bool`
- `metro_log_auto_follow: bool`

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `/Users/cubicme/aljazeera/dashboard/.planning/quick/260322-3nj-remove-metro-pane-and-move-metro-shortcu/260322-3nj-SUMMARY.md` — created
- Commit `8a2cddb` — FOUND
- Commit `5d1ee23` — FOUND
- No `MetroPane` references in src/ — CONFIRMED
- No metro log state fields in src/ — CONFIRMED
- `PaletteMode::Metro` present in footer.rs, handle_key, update() — CONFIRMED
