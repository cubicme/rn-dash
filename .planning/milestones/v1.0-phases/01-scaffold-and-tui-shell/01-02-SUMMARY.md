---
phase: 01-scaffold-and-tui-shell
plan: "02"
subsystem: app
tags: [rust, ratatui, crossterm, tokio, tea-pattern, event-loop, terminal-lifecycle, async, keybindings]

# Dependency graph
requires:
  - phase: 01-01
    provides: Module skeleton (domain/infra/ui), Cargo manifest with ratatui 0.30 + tokio

provides:
  - TEA application state machine: AppState, FocusedPanel, handle_key(), update()
  - Async event loop using tokio::select! over crossterm EventStream
  - Terminal lifecycle with guaranteed restore on all exit paths including panics
  - Full vim-style keybinding dispatch (hjkl, Tab/Shift-Tab, /, ?, q)
  - Action enum with all Phase 1 variants — extension point for all later phases
  - File-based logging via tracing-appender (never stdout in TUI)

affects:
  - 01-03-PLAN.md (help overlay, footer keybinding hints — consumes Action enum and AppState)
  - All subsequent phases — all state mutations go through update(), all actions via Action enum

# Tech tracking
tech-stack:
  added:
    - futures 0.3 (StreamExt trait for EventStream::next())
    - crossterm 0.29 event-stream feature (enables EventStream for async event polling)
  patterns:
    - TEA (Elm Architecture): handle_key() returns Option<Action> (pure), update() is sole mutation site
    - Panic hook installs ratatui::restore() BEFORE original hook — guarantees terminal restore on panic
    - ratatui::restore() called unconditionally after app::run() — no path can skip terminal restore
    - EventStream via ratatui::crossterm::event::EventStream (not standalone crossterm dep)
    - setup_logging() returns WorkerGuard held in main() for full program lifetime

key-files:
  created:
    - src/action.rs
    - src/event.rs
    - src/app.rs
    - src/tui.rs
  modified:
    - src/main.rs
    - src/ui/mod.rs
    - Cargo.toml
    - Cargo.lock

key-decisions:
  - "futures 0.3 added explicitly to Cargo.toml — EventStream requires StreamExt which is not transitively available in current dep tree (ratatui-crossterm does not enable event-stream feature by default)"
  - "crossterm 0.29 with event-stream feature added to Cargo.toml — same version as ratatui-crossterm uses, cargo feature-unifies so no duplication"
  - "event.rs From<CrosstermEvent> for Option<Event> impl replaced with free function from_crossterm() — orphan rule disallows implementing std traits for foreign input/output type combinations"
  - "setup_logging() returns color_eyre::Result (not anyhow::Result) — main() returns color_eyre::Result, ? operator requires compatible error types"
  - "#![allow(dead_code)] on event.rs and app.rs stub items — consistent with Plan 01 pattern, removed when stubs gain real implementations in later phases"

patterns-established:
  - "TEA invariant: handle_key() is pure (no side effects, returns Option<Action>); update() is sole mutation site — all later phase state changes must follow this pattern"
  - "Terminal lifecycle order: color_eyre::install → panic hook (with restore) → setup_logging → ratatui::init → app::run → ratatui::restore — any deviation causes hook chaining bugs"
  - "Panic safety: ratatui::restore() is called in panic hook before forwarding to original hook, AND unconditionally after run() — two-layer guarantee"
  - "Keybinding dispatch: overlay modes intercept keys first (help overlay, error overlay), then normal mode — priority order must be maintained in all future keybinding additions"

requirements-completed:
  - ARCH-04
  - ARCH-06
  - SHELL-01
  - SHELL-03

# Metrics
duration: 4min
completed: 2026-03-02
---

# Phase 1 Plan 02: Core Application Loop and Terminal Lifecycle Summary

**Async TEA event loop with guaranteed terminal restore on all exit paths (including panics), vim-style keybinding dispatch, and focus cycling via tokio::select! over crossterm EventStream**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-02T06:04:50Z
- **Completed:** 2026-03-02T06:08:58Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- TEA (Elm Architecture) pattern established: `handle_key()` as pure function mapping key events to `Option<Action>`, `update()` as sole state mutation site — all later phases extend these without changing the invariant
- Terminal lifecycle with two-layer panic safety: panic hook calls `ratatui::restore()` before forwarding, AND `ratatui::restore()` called unconditionally after `app::run()` on both Ok and Err paths
- Async event loop via `tokio::select!` over `crossterm::event::EventStream` with 250ms tick — ready for Phase 2 periodic status refreshes
- Full vim-style keybinding dispatch: hjkl navigation, Tab/Shift-Tab focus cycling, `/` search (stub), `?` help overlay, `q` quit with correct overlay-priority intercept order
- Non-blocking file logging via `tracing-appender` to `~/.config/ump-dash/logs/` — never corrupts TUI output

## Task Commits

Each task was committed atomically:

1. **Task 1: Create action.rs, event.rs, and AppState with TEA scaffold** - `282363e` (feat)
2. **Task 2: Implement main.rs terminal lifecycle with panic hook and logging** - `b1e5dbf` (feat)

## Files Created/Modified

- `src/action.rs` - Action enum with all Phase 1 variants; extension point for all later phases
- `src/event.rs` - Internal Event type wrapping crossterm events; from_crossterm() converter
- `src/app.rs` - AppState, FocusedPanel, handle_key() pure mapping, update() TEA mutation, async run() loop
- `src/tui.rs` - setup_logging() with tracing-appender non-blocking file appender; lifecycle doc comments
- `src/main.rs` - Canonical 6-step terminal lifecycle (color_eyre, panic hook, logging, init, run, restore)
- `src/ui/mod.rs` - Added stub view() function (Plan 03 replaces with real rendering)
- `Cargo.toml` - Added futures 0.3 and crossterm 0.29 event-stream feature
- `Cargo.lock` - Updated with new dependency resolutions

## Decisions Made

- `futures 0.3` added explicitly to Cargo.toml because `StreamExt` (needed for `EventStream::next()`) is not transitively available — `ratatui-crossterm` does not enable the `event-stream` feature on crossterm by default
- `crossterm 0.29` with `event-stream` feature added directly (same version as ratatui-crossterm uses, so cargo feature-unifies with no duplication — satisfies the "no crossterm duplication" constraint)
- `event.rs` uses a free function `from_crossterm()` instead of `impl From<CrosstermEvent> for Option<Event>` — Rust orphan rules disallow implementing a std trait (`From`) for a foreign-crate output type (`Option`)
- `setup_logging()` returns `color_eyre::Result` — required for `?` to work in `main()` which returns `color_eyre::Result<()>`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed orphan rule violation in event.rs**
- **Found during:** Task 1 (Create action.rs, event.rs, and AppState)
- **Issue:** Plan spec shows `impl From<CrosstermEvent> for Option<Event>` — Rust orphan rules disallow implementing `From` (std trait) for `Option<Event>` (output type is `Option` from std, not crate-local)
- **Fix:** Replaced with `pub fn from_crossterm(ev: CrosstermEvent) -> Option<Event>` free function
- **Files modified:** src/event.rs
- **Verification:** `cargo build` produced zero errors after fix
- **Committed in:** 282363e (Task 1 commit)

**2. [Rule 1 - Bug] Changed setup_logging() return type from anyhow::Result to color_eyre::Result**
- **Found during:** Task 1 build (error[E0277] in main.rs)
- **Issue:** Plan spec shows `anyhow::Result<WorkerGuard>` but main() returns `color_eyre::Result`, and `?` cannot convert between incompatible error types
- **Fix:** Changed return type to `color_eyre::Result<tracing_appender::non_blocking::WorkerGuard>`
- **Files modified:** src/tui.rs
- **Verification:** `cargo build` produced zero errors after fix
- **Committed in:** 282363e (Task 1 commit)

**3. [Rule 1 - Bug] Added dead_code suppression to event.rs and app.rs**
- **Found during:** Task 1 verification (3 warnings on stub items)
- **Issue:** `Event` enum, `from_crossterm()`, `ErrorState` fields all unused — generates dead_code warnings; plan convention (established in 01-01) requires zero warnings
- **Fix:** Added `#![allow(dead_code)]` inner attribute to event.rs and app.rs
- **Files modified:** src/event.rs, src/app.rs
- **Verification:** `cargo build` produced zero warnings after fix
- **Committed in:** 282363e (Task 1 commit)

**4. [Rule 3 - Blocking] Added futures 0.3 and crossterm event-stream feature to Cargo.toml**
- **Found during:** Task 1 implementation (EventStream requires event-stream feature + StreamExt trait)
- **Issue:** Plan says "futures::StreamExt transitively available" — but cargo tree shows futures is NOT in the dependency tree and crossterm's event-stream feature is disabled by default in ratatui-crossterm
- **Fix:** Added `futures = "0.3"` and `crossterm = { version = "0.29", features = ["event-stream"] }` to Cargo.toml; cargo feature-unifies with the existing crossterm 0.29 already in the tree
- **Files modified:** Cargo.toml, Cargo.lock
- **Verification:** EventStream import succeeds, cargo build passes
- **Committed in:** 282363e (Task 1 commit)

---

**Total deviations:** 4 auto-fixed (3 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All auto-fixes necessary for compilation correctness. No scope creep. The plan's note about futures being "transitively available" was inaccurate for the current ratatui 0.30 workspace structure — explicit dep is the correct approach per ratatui 0.30 modularization.

## Issues Encountered

None beyond the auto-fixed compilation errors above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- App loop is ready for Plan 03 (panel layout, help overlay, footer keybinding hints)
- `crate::ui::view()` stub in ui/mod.rs compiles cleanly and is the exact function Plan 03 will replace
- `Action` enum has all Phase 1 variants; Plan 03 adds `ShowHelp`/`DismissHelp` rendering (already dispatched)
- `FocusedPanel` enum exposes `WorktreeList`, `MetroPane`, `CommandOutput` for panel border styling in Plan 03
- No blockers for Plan 03

---
*Phase: 01-scaffold-and-tui-shell*
*Completed: 2026-03-02*
