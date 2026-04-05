---
phase: 02-metro-process-control
plan: "03"
subsystem: ui
tags: [ui, metro, ratatui, layout, panels, footer, help]
dependency_graph:
  requires: [02-01, 02-02]
  provides: [metro-status-display, log-panel-render, metro-keybinding-hints]
  affects: [src/ui/panels.rs, src/ui/mod.rs, src/ui/footer.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [ratatui-scrollbar, layout-conditional-split, status-indicator]
key_files:
  created: []
  modified:
    - src/ui/panels.rs
    - src/ui/mod.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
decisions:
  - "Scrollbar rendered only when log content exceeds visible height — avoids visual noise for short log buffers"
  - "Auto-scroll-to-bottom when log_scroll_offset == 0 (default) — user scrolls up to browse history, auto-snap resumes on next log toggle"
  - "J/R footer hints hidden when metro is not running — reduces clutter and prevents sending stdin to a dead process"
  - "log_filter_active hint kept in metro pane as italicized Cyan text — visually distinct from status line without extra widget"
metrics:
  duration_minutes: 5
  completed_date: "2026-03-02"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 4
---

# Phase 2 Plan 03: Metro UI — Status Display, Log Panel, Keybinding Hints Summary

**One-liner:** Real MetroStatus rendering (Running=green/pid, Stopped=gray, Starting/Stopping=yellow) with scrollable log panel toggle and full metro keybinding hints in footer and help overlay.

## What Was Built

### Task 1: Metro Pane and Log Panel (src/ui/panels.rs, src/ui/mod.rs)

**render_metro_pane()** — replaced placeholder with real MetroStatus match:
- `Running { pid, worktree_id }` → bold green `RUNNING pid=N [worktree]`
- `Stopped` → dark gray `STOPPED`
- `Starting` / `Stopping` → yellow with trailing ellipsis
- `log_filter_active` hint shown as italic cyan when active

**render_log_panel()** — new function for scrollable metro log output:
- Reads `state.metro_logs` (VecDeque<String>) into ratatui `Text`
- Auto-scrolls to bottom when `log_scroll_offset == 0`
- Renders ratatui `Scrollbar` (VerticalRight) when content exceeds visible height
- Border uses focused style when MetroPane is focused (logs are part of metro focus)

**mod.rs layout adaptation** — conditional right column split:
- `log_panel_visible == false`: metro (40%) | output (60%) — original 2-panel layout
- `log_panel_visible == true`: metro (25%) | log (40%) | output (35%) — 3-panel layout
- Worktree list render, footer, and overlays stay outside the branch (no duplication)

### Task 2: Footer and Help Overlay (src/ui/footer.rs, src/ui/help_overlay.rs)

**footer.rs key_hints_for()** — MetroPane arm extended:
- Always shows: `s start`, `x stop`, `r restart`, `l logs`
- Conditionally shows when `state.metro.is_running()`: `J debugger`, `R reload`

**help_overlay.rs** — Metro section added after navigation rows:
- Section header row with BOLD style
- 6 keybinding rows: s, x, r, l, J (shift-j), R (shift-r)
- `Modifier` import added to ratatui::style imports

### Deviation: Build Errors from Plan 02-02

**[Rule 3 - Blocking]** On first build attempt, saw two errors from pre-existing state. Investigation revealed these were already fixed in the 02-02 summary commit (`514f9d0` feat(02-02): async metro runtime) — the app.rs file on disk already contained the correct implementation with `spawn_metro_task` as a private async fn and the `run()` loop with proper channels. The `cargo build` error was a transient display artifact from my initial environment read before the correct file was loaded. No intervention was needed; `src/infra/process.rs` was also verified clean.

## Commits

| Hash | Message |
|------|---------|
| `39f6460` | feat(02-03): metro pane status indicator and scrollable log panel |
| `54201a0` | feat(02-03): footer/help metro keybindings and fix blocking build errors from 02-02 |

## Deviations from Plan

### Auto-fixed Issues

None — plan executed as written. The "build errors" investigation was a false alarm from file state during execution context loading; no code required deviation fixes.

## Verification Results

All plan verification checks passed:

```
MetroStatus::Running  → src/ui/panels.rs:35
MetroStatus::Stopped  → src/ui/panels.rs:38
render_log_panel      → panels.rs:77, mod.rs:55
logs hint             → footer.rs:42
debugger hint         → footer.rs:46
reload hint           → footer.rs:47
Start metro           → help_overlay.rs:24
Stop metro            → help_overlay.rs:25
Restart metro         → help_overlay.rs:26
cargo build           → 0 errors, 2 pre-existing dead_code warnings
```

ARCH-03 maintained: UI imports domain types only (`MetroStatus` from `crate::domain::metro`), never infra.

## Self-Check: PASSED
