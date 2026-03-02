---
phase: 04-config-and-jira-integration
plan: 03
subsystem: ui
tags: [ratatui, worktree-list, display, uat, gap-closure]

# Dependency graph
requires:
  - phase: 04-config-and-jira-integration
    provides: Worktree struct with jira_title, label, stale fields; JIRA title fetching wired into TEA loop
provides:
  - Worktree list renders branch name first (always visible), JIRA title/label second in Gray
  - Staleness shown as Unicode warning icon U+26A0 instead of [stale] text
  - No DarkGray+DIM secondary text in worktree list
affects: [05-worktree-switching, visual-verification]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Worktree list accesses struct fields directly (wt.branch, wt.label, wt.jira_title) rather than using display_name() helper -- gives layout control to renderer"
    - "Unicode icons preferred over text badges for compact status indicators"

key-files:
  created: []
  modified:
    - src/ui/panels.rs
    - src/domain/worktree.rs

key-decisions:
  - "panels.rs accesses wt.branch directly instead of display_name() for list rendering -- display_name() still available for single-string contexts (modals, status messages)"
  - "display_name() annotated with #[allow(dead_code)] -- preserved for future modal/status use even though no current call sites exist"
  - "Color::Gray used for secondary text (not DarkGray or DIM) -- legible on dark terminals"

patterns-established:
  - "List widget rendering: fields directly, not via aggregating helpers, when layout control matters"

requirements-completed: [WORK-02, WORK-05]

# Metrics
duration: 3min
completed: 2026-03-02
---

# Phase 4 Plan 03: Worktree List Display Fix Summary

**Worktree list reordered to show branch name first (always visible), JIRA title/label second in legible Gray, with Unicode warning icon U+26A0 replacing the noisy [stale] text badge**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-02T13:55:33Z
- **Completed:** 2026-03-02T13:58:33Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Branch name always rendered as primary text in the worktree list -- long JIRA titles can no longer push the branch name out of view
- Secondary text (label > jira_title) displayed with `Color::Gray` and " - " dash separator, replacing the barely-visible DarkGray+DIM parenthetical
- Stale worktrees now show yellow Unicode warning sign (U+26A0) instead of `[stale]` text -- compact and universally recognizable in terminals
- display_name() helper preserved in domain layer with #[allow(dead_code)] for future modal/status message use

## Task Commits

Each task was committed atomically:

1. **Task 1: Reorder worktree list rendering -- branch first, title second, icon staleness** - `1294a6b` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/ui/panels.rs` - Rewritten render_worktree_list() closure: Span::raw(&wt.branch) as primary, Color::Gray secondary text, U+26A0 staleness icon
- `src/domain/worktree.rs` - Added #[allow(dead_code)] to display_name() method

## Decisions Made

- `panels.rs` now accesses struct fields directly (`wt.branch`, `wt.label`, `wt.jira_title`) for layout control rather than using `display_name()`. The `display_name()` helper remains in the domain layer for contexts where a single aggregated string is needed (modal titles, status messages).
- `#[allow(dead_code)]` added to `display_name()` because removing the only call site created a new compiler warning. The method is intentionally preserved for future use.
- `Color::Gray` chosen for secondary text (vs DarkGray or DIM) because UAT Test 4 specifically flagged the barely-visible DarkGray+DIM parenthetical as a legibility problem.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Suppressed new dead_code warning on display_name()**
- **Found during:** Task 1 (post-verification cargo check)
- **Issue:** Removing `display_name()` call from panels.rs created a new `dead_code` warning that didn't exist before this plan
- **Fix:** Added `#[allow(dead_code)]` to the `display_name()` method in worktree.rs; updated doc comment to clarify the method is preserved for single-string contexts
- **Files modified:** src/domain/worktree.rs
- **Verification:** cargo check passes with 3 warnings (all pre-existing, none from our changes)
- **Committed in:** 1294a6b (part of task commit)

**2. [Rule 1 - Bug] Fixed clippy uninlined_format_args in new code**
- **Found during:** Task 1 (clippy verification)
- **Issue:** `format!(" - {}", label)` and `format!(" - {}", title)` trigger clippy's uninlined_format_args lint -- would introduce 2 new clippy errors vs pre-existing baseline
- **Fix:** Changed both to inline format syntax: `format!(" - {label}")` and `format!(" - {title}")`
- **Files modified:** src/ui/panels.rs
- **Verification:** clippy error count matches pre-existing baseline (14 errors, none from modified files)
- **Committed in:** 1294a6b (part of task commit)

---

**Total deviations:** 2 auto-fixed (2 Rule 1 bug fixes)
**Impact on plan:** Both auto-fixes necessary to avoid introducing new warnings/errors vs pre-existing baseline. No scope creep.

## Issues Encountered

None -- plan executed straightforwardly. Pre-existing clippy failures (14 errors, all in other files) were verified as pre-existing and not addressed per scope boundary rules.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Worktree list display now matches UAT Test 4 expectations
- display_name() available in domain layer for Phase 5 modal/status contexts
- Phase 5 (worktree switching) can build on the clean list rendering foundation

---
*Phase: 04-config-and-jira-integration*
*Completed: 2026-03-02*
