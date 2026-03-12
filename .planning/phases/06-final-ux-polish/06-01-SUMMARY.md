---
phase: 06-final-ux-polish
plan: "01"
subsystem: ui
tags: [ratatui, metro, log-filtering, icons, rust]

# Dependency graph
requires:
  - phase: 05.2-milestone-feedbacks
    provides: metro streaming with DEBUG=Metro:* always active, Y/P icons in worktree table
provides:
  - should_suppress_metro_line() filter function suppressing watchman and empty lines before metro_logs VecDeque
  - Green play triangle (U+25B6) as metro running indicator in worktree table and footer legend
affects: [future metro log filtering expansions]

# Tech tracking
tech-stack:
  added: []
  patterns: [log line filter function placed adjacent to stream_metro_logs for proximity]

key-files:
  created: []
  modified:
    - src/app.rs
    - src/ui/panels.rs
    - src/ui/footer.rs

key-decisions:
  - "should_suppress_metro_line() filters only watchman warnings and empty lines — conservative scope to avoid suppressing legitimate build warnings"
  - "U+25B6 BLACK RIGHT-POINTING TRIANGLE replaces bullet as metro running icon for clearer play-state semantics"

patterns-established:
  - "Log noise filter: pure fn next to async streaming fn, called inline before Action dispatch"

requirements-completed: [UX-06-01, UX-06-03]

# Metrics
duration: 5min
completed: 2026-03-12
---

# Phase 06 Plan 01: Metro Log Noise Suppression and Play Icon Indicator Summary

**Watchman warning filter added to metro log streaming pipeline and bullet replaced with green play triangle in worktree table and footer legend**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-12T09:01:00Z
- **Completed:** 2026-03-12T09:06:46Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `should_suppress_metro_line()` to drop watchman warnings and empty lines before they reach the metro log VecDeque
- Filter applied in both stdout and stderr reader arms of `stream_metro_logs`
- Replaced `●` bullet with `\u{25B6}` play triangle in worktree table metro indicator
- Updated footer legend to match, ensuring visual consistency

## Task Commits

Each task was committed atomically:

1. **Task 1: Add metro log line filter in stream_metro_logs** - `9196ce8` (feat)
2. **Task 2: Change metro indicator from bullet to play triangle** - `b5e7895` (feat)

## Files Created/Modified

- `src/app.rs` - Added `should_suppress_metro_line()` filter fn; modified both stdout/stderr arms in `stream_metro_logs` to call it
- `src/ui/panels.rs` - Metro running icon changed from `●` to `\u{25B6}` in `render_worktree_table()`
- `src/ui/footer.rs` - Icon legend updated to match play triangle

## Decisions Made

- `should_suppress_metro_line()` is conservative — only watchman warnings and empty lines are suppressed. Lines containing "warn" are NOT filtered to preserve legitimate build warnings.
- U+25B6 (BLACK RIGHT-POINTING TRIANGLE) chosen for its universal "play" semantics, signaling metro is actively running rather than just present.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Metro log display is cleaner — watchman noise and blank lines suppressed
- Play icon visually distinguishes running metro from stopped worktrees
- Ready to continue Phase 06 remaining plans

---
*Phase: 06-final-ux-polish*
*Completed: 2026-03-12*
