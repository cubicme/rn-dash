---
phase: 02-metro-process-control
plan: "02"
subsystem: app
tags: [rust, tokio, ratatui, process-management, async, mpsc, oneshot]

# Dependency graph
requires:
  - phase: 02-metro-process-control
    plan: "01"
    provides: MetroManager, MetroHandle, MetroStatus, ProcessClient trait, port_is_free(), Action metro variants, AppState metro fields

provides:
  - Metro keybinding dispatch in handle_key() for MetroPane focus (s/x/r/l/J/R)
  - Async metro spawn via tokio::spawn(spawn_metro_task) — never blocks event loop
  - Kill sequence via oneshot channel to metro_process_task, port-free polling, MetroExited delivery
  - Restart flow via pending_restart flag: MetroStop + auto-MetroStart on MetroExited
  - stdin forwarding to metro via stdin_writer task (MetroSendDebugger/Reload)
  - Log streaming via BufReader::lines() from stdout+stderr to MetroLogLine actions
  - External death detection via child.wait() in metro_process_task
  - MetroHandle extended with kill_tx: Option<oneshot::Sender<()>>
  - AppState extended with pending_restart: bool
  - update() wired with metro_tx and handle_tx channel params
  - run() select! loop handles metro_rx and handle_rx arms

affects:
  - 02-03 (metro UI — renders MetroStatus/metro_logs; keybindings now live)
  - 03 (worktree list — active_worktree_path feeds spawn path)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Oneshot channel kill signal pattern — MetroHandle.kill_tx triggers metro_process_task to kill Child and poll port
    - Separate handle_rx channel for non-Clone MetroHandle delivery (cannot go through Action enum)
    - Recursive update() dispatch for restart chain (MetroRestart → MetroStop → on MetroExited → MetroStart)
    - Background task trilogy: spawn_metro_task (coordination) + metro_process_task (Child owner) + stream_metro_logs (IO) + stdin_writer (stdin)

key-files:
  created: []
  modified:
    - src/app.rs — handle_key() metro-pane branch, update() async metro arms, run() channel wiring, 4 async helper functions
    - src/domain/metro.rs — MetroHandle extended with kill_tx field

key-decisions:
  - "kill_tx: Option<oneshot::Sender<()>> added to MetroHandle — oneshot is the right primitive for a one-time kill signal; Option<> allows take() exactly once"
  - "Separate handle_rx channel for MetroHandle delivery — MetroHandle contains JoinHandle which is not Clone, so it cannot travel via Action enum; separate channel keeps the type boundary clean"
  - "pending_restart: bool in AppState drives restart chain — MetroRestart/MetroToggleLog/MetroStart-while-running all set this flag; MetroExited checks it and re-dispatches MetroStart"
  - "stream_metro_logs as separate task inside metro_process_task — process task aborts log task first on kill, preventing log lines from racing with MetroExited"
  - "MetroToggleLog now sets log_filter_active = log_panel_visible and restarts if running — log filter change requires restart to pass updated filter to spawn_metro"

patterns-established:
  - "Pattern: Background task trilogy — spawn_metro_task coordinates, metro_process_task owns Child, stream_metro_logs reads IO. Clear ownership, no shared mutable state."
  - "Pattern: Recursive update() dispatch — update calls itself for nested action chains (MetroRestart → MetroStop). Safe because Rust is not async here and there is no stack overflow risk at 2-3 levels deep."
  - "Pattern: handle_rx for non-Clone types — when a background task needs to return a value that cannot go through Action enum, use a dedicated channel arm in the select! loop."

requirements-completed: [METRO-02, METRO-03, METRO-04, METRO-07, METRO-08]

# Metrics
duration: 5min
completed: 2026-03-02
---

# Phase 02 Plan 02: Metro Runtime Wiring Summary

**tokio::spawn-based metro lifecycle — spawn/kill/restart/stdin forwarding and external death detection wired into TEA event loop via dual mpsc channels (metro_rx for actions, handle_rx for MetroHandle delivery)**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-02T07:02:00Z
- **Completed:** 2026-03-02T07:07:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Metro keybinding dispatch: s/x/r/l/J/R when MetroPane focused, with correct overlay/error priority ordering
- Full async metro lifecycle: spawn_metro_task (process creation) → metro_process_task (Child owner, kill+port-free) → stream_metro_logs (stdout/stderr BufReader) → stdin_writer (stdin forwarding)
- Restart chain via pending_restart flag covering MetroRestart, MetroStart-while-running, and MetroToggleLog-while-running
- External death detection via child.wait() racing against kill_rx in metro_process_task

## Task Commits

Each task was committed atomically:

1. **Task 1: Metro keybinding dispatch in handle_key()** - `8fd8b77` (feat)
2. **Task 2: Async metro runtime — spawn, kill, restart, stdin, death detection** - `514f9d0` (feat)

## Files Created/Modified

- `src/app.rs` — handle_key() metro-pane branch (s/x/r/l/J/R), update() async metro arms (MetroStart/Stop/Restart/Toggle/SendDebugger/Reload), run() with metro_rx+handle_rx select! arms, plus 4 async helpers: spawn_metro_task, metro_process_task, stream_metro_logs, stdin_writer
- `src/domain/metro.rs` — MetroHandle extended with `kill_tx: Option<tokio::sync::oneshot::Sender<()>>`

## Decisions Made

- Added `kill_tx: Option<oneshot::Sender<()>>` to MetroHandle: Plan 01's MetroHandle lacked a kill mechanism. Oneshot is the correct primitive for a one-time kill signal. Option wrapping allows `take()` exactly once without clone.
- Separate `handle_rx` channel for MetroHandle delivery: MetroHandle contains `JoinHandle<()>` which is not Clone, preventing it from traveling through the `Action` enum. A dedicated unbounded channel with its own `select!` arm cleanly delivers the handle to the main loop.
- `pending_restart: bool` flag in AppState drives all restart chains: MetroRestart, MetroStart-while-running, and MetroToggleLog-while-running all converge on the same stop→pending→start sequence.
- `stream_metro_logs` is a separate task spawned inside `metro_process_task`: this allows `metro_process_task` to `log_task.abort()` before killing the child, preventing log lines from racing with `MetroExited` delivery.
- `MetroToggleLog` now sets `log_filter_active = log_panel_visible` and triggers a restart if running: changing the filter requires a new spawn with/without `DEBUG=Metro:*` env var.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added kill_tx field to MetroHandle**
- **Found during:** Task 2 (async metro runtime)
- **Issue:** Plan 01's MetroHandle struct lacked a `kill_tx` field. Plan 02 explicitly anticipated this and instructed the executor to add it. Without kill_tx, MetroStop has no way to signal the process task to kill the child.
- **Fix:** Added `pub kill_tx: Option<tokio::sync::oneshot::Sender<()>>` to MetroHandle in src/domain/metro.rs
- **Files modified:** src/domain/metro.rs
- **Verification:** cargo build passes with zero errors after the addition
- **Committed in:** 514f9d0 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 - missing critical field anticipated by plan)
**Impact on plan:** Required for MetroStop to function. Plan 02 explicitly noted this would be needed. No scope creep.

## Issues Encountered

None — plan executed cleanly with one anticipated field addition to MetroHandle.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- All metro runtime behavior implemented — Plan 03 (metro UI) can render MetroStatus, metro_logs, and the log panel using the fully functional state machine
- Keybindings live and dispatching correct actions — UI needs to show what keys are available (help text in metro pane)
- MetroStatus transitions (Starting/Running/Stopping/Stopped) drive UI status indicator in Plan 03
- log_panel_visible and log_scroll_offset are ready for the scrollable log panel in Plan 03

---
*Phase: 02-metro-process-control*
*Completed: 2026-03-02*
