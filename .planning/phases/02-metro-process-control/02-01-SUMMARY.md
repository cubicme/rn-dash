---
phase: 02-metro-process-control
plan: "01"
subsystem: infra
tags: [rust, tokio, ratatui, process-management, domain, tui]

# Dependency graph
requires:
  - phase: 01-scaffold-and-tui-shell
    provides: TEA scaffold (action.rs, app.rs, AppState), domain/worktree.rs pattern, infra/mod.rs stub

provides:
  - MetroManager struct with single-instance invariant via Option<MetroHandle>
  - MetroHandle with tokio channel and JoinHandle fields for async coordination
  - MetroStatus enum (Stopped/Running/Starting/Stopping) for UI consumption
  - ProcessClient trait with TokioProcessClient implementation (async-trait)
  - port_is_free() utility via TcpListener::bind probe
  - Action enum extended with 10 metro variants (control + background events)
  - AppState extended with metro, metro_logs, log panel state fields
  - Pure state mutation handlers for MetroToggleLog/ScrollUp/ScrollDown/LogLine/Exited

affects:
  - 02-02 (metro runtime — uses MetroManager, ProcessClient, MetroHandle to spawn/kill)
  - 02-03 (metro UI — uses MetroStatus, metro_logs, log_panel_visible, log_scroll_offset)
  - 03 (worktree list — uses active_worktree_path stub in AppState)

# Tech tracking
tech-stack:
  added:
    - async-trait = "0.1" — enables async fn in traits via #[async_trait::async_trait]
  patterns:
    - MetroManager/Option<MetroHandle> structural single-instance enforcement
    - ProcessClient trait boundary (ARCH-02 compliant)
    - MetroHandle as infrastructure-bridging type in domain/ with architectural justification comment
    - Pure state mutations in update(), async runtime deferred to Plan 02

key-files:
  created:
    - src/domain/metro.rs — MetroManager, MetroHandle, MetroStatus
    - src/infra/process.rs — ProcessClient trait + TokioProcessClient
    - src/infra/port.rs — port_is_free()
  modified:
    - src/domain/mod.rs — added pub mod metro
    - src/infra/mod.rs — added pub mod process; pub mod port
    - src/action.rs — 10 metro Action variants added
    - src/app.rs — AppState extended with metro fields, manual Default impl, update() metro arms
    - Cargo.toml — async-trait added

key-decisions:
  - "MetroHandle lives in domain/ referencing tokio types: infrastructure-bridging type, ARCH-01 maintained because domain/mod.rs imports no infra"
  - "No new mandatory crates beyond async-trait — tokio::process and tokio::sync::mpsc already in tokio features=full"
  - "MetroManager::register() panics if handle exists — callers must take_handle() and kill before registering new instance"
  - "Pure metro state mutations (toggle, scroll, log append, clear) implemented in update() — async spawn/kill deferred to Plan 02"
  - "Debug derive added to MetroHandle and MetroManager after AppState derive(Debug) compilation failure — Rule 1 auto-fix"

patterns-established:
  - "Pattern: Option<MetroHandle> structural invariant — type system makes two simultaneous handles impossible"
  - "Pattern: Background task contract — stream_task and stdin_task JoinHandles stored in MetroHandle for Plan 02 lifecycle management"
  - "Pattern: MetroStatus is the observable projection of MetroHandle state — UI reads status, not the handle directly"

requirements-completed: [METRO-01, METRO-09]

# Metrics
duration: 3min
completed: 2026-03-02
---

# Phase 02 Plan 01: Metro Type Contracts Summary

**MetroManager single-instance enforcer, ProcessClient trait boundary, and full Action/AppState metro interface layer — all contracts Plan 02 runtime and Plan 03 UI build against**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-02T06:57:39Z
- **Completed:** 2026-03-02T07:00:39Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Domain metro types: MetroManager (single-instance invariant via Option<MetroHandle>), MetroHandle (tokio channel/JoinHandle bridge), MetroStatus (Stopped/Running/Starting/Stopping)
- Infra trait boundary: ProcessClient async trait + TokioProcessClient (process_group(0), kill_on_drop, piped IO, DEBUG=Metro:* filter support)
- port_is_free() utility via TcpListener::bind probe for post-kill port verification
- Action enum extended with 10 metro variants; AppState extended with all metro fields; update() handles 5 pure state mutations immediately

## Task Commits

Each task was committed atomically:

1. **Task 1: Domain metro types and infra trait contracts** - `5d2d104` (feat)
2. **Task 2: Action enum and AppState extensions for metro** - `0dc0ed2` (feat)

## Files Created/Modified

- `src/domain/metro.rs` — MetroManager (single-instance enforcer), MetroHandle (tokio bridge type), MetroStatus enum
- `src/infra/process.rs` — ProcessClient trait (async-trait) + TokioProcessClient implementation
- `src/infra/port.rs` — port_is_free(u16) -> bool via TcpListener::bind probe
- `src/domain/mod.rs` — added pub mod metro
- `src/infra/mod.rs` — replaced stub with pub mod process + pub mod port
- `src/action.rs` — 10 metro variants: MetroStart/Stop/Restart/ToggleLog/ScrollUp/ScrollDown/SendDebugger/SendReload/LogLine/Exited
- `src/app.rs` — AppState metro fields, manual Default impl, MAX_LOG_LINES constant, update() metro arms
- `Cargo.toml` — async-trait = "0.1" added

## Decisions Made

- MetroHandle lives in domain/ and references tokio types: it is an infrastructure-bridging type, not pure domain logic. ARCH-01 (domain imports no infra) is maintained because domain/mod.rs itself has no infra imports.
- No additional crates beyond async-trait — tokio::process, tokio::sync::mpsc, and tokio::task are all part of tokio features="full" already in Cargo.toml.
- MetroManager::register() panics on double-registration — enforces that callers must take_handle() and kill before spawning a new instance. Explicit invariant over silent overwrite.
- Pure state mutations (MetroToggleLog, MetroScrollUp/Down, MetroLogLine, MetroExited) implemented in update() now. Async spawn/kill actions (MetroStart/Stop/Restart/SendDebugger/SendReload) are stubs for Plan 02.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added #[derive(Debug)] to MetroHandle and MetroManager**
- **Found during:** Task 2 (AppState extensions)
- **Issue:** AppState derives Debug, which requires all fields to implement Debug. MetroManager (and transitively MetroHandle) lacked Debug derives — compilation error E0277.
- **Fix:** Added `#[derive(Debug)]` to both MetroHandle and MetroManager structs. Both tokio::sync::mpsc::UnboundedSender<Vec<u8>> and tokio::task::JoinHandle<()> implement Debug, so derive works without manual impl.
- **Files modified:** src/domain/metro.rs
- **Verification:** cargo build passes with zero errors after the fix
- **Committed in:** 0dc0ed2 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - missing derive)
**Impact on plan:** Necessary for compilation — AppState::derive(Debug) requires the field types to implement Debug. No scope creep.

## Issues Encountered

None — plan executed cleanly with one auto-fix for the missing Debug derive.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- All type contracts established — Plan 02 (metro runtime) can implement MetroStart/Stop/Restart spawn/kill logic using the MetroManager, ProcessClient, and MetroHandle types defined here
- Plan 03 (metro UI) can render MetroStatus and metro_logs fields already present in AppState
- ProcessClient::spawn_metro takes a `filter: bool` parameter ready for log filter mode (Plan 02 or 03)
- Action variants MetroSendDebugger/MetroSendReload are stub arms in update() — Plan 02 wires them to MetroManager::send_stdin()

---
*Phase: 02-metro-process-control*
*Completed: 2026-03-02*
