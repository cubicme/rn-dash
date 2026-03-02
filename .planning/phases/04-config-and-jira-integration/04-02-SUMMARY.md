---
phase: 04-config-and-jira-integration
plan: 02
subsystem: infra
tags: [jira, tmux, cache, async, tokio, tea]

# Dependency graph
requires:
  - phase: 04-01
    provides: config.rs, jira.rs, jira_cache.rs infra modules (HttpJiraClient, load_config, load_jira_cache, extract_jira_key, is_inside_tmux)
  - phase: 03-worktree-browser
    provides: Worktree.jira_title field, WorktreesLoaded action, display_name() priority cascade

provides:
  - JiraTitlesFetched action variant wired into Action enum and update() handler
  - AppState Phase 4 fields: tmux_available, jira_title_cache, jira_client
  - Startup config + cache load in run() with silent degradation on missing config
  - Background JIRA title fetch after WorktreesLoaded (fire-and-forget tokio::spawn)
  - Cache re-applied on every WorktreesLoaded to prevent flash-then-disappear
  - UI rendering already correct (display_name() + dim branch parenthetical verified)

affects: [05-tmux-integration, ui/panels.rs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Fire-and-forget JIRA fetch: Arc<dyn JiraClient> cloned into tokio::spawn, results sent via metro_tx as JiraTitlesFetched"
    - "Cache-then-fetch: cached titles applied synchronously in WorktreesLoaded, network fetch only for uncached keys"
    - "Silent degradation: missing config or JIRA unreachable causes branch name fallback with no user-visible error"

key-files:
  created: []
  modified:
    - src/action.rs
    - src/app.rs
    - src/infra/jira.rs

key-decisions:
  - "JiraClient trait requires Debug supertrait — AppState derives Debug so Arc<dyn JiraClient> must satisfy Debug"
  - "HttpJiraClient derives Debug — reqwest::Client implements Debug, so #[derive(Debug)] works"
  - "panels.rs required no changes — display_name() and dim branch parenthetical were already implemented in Phase 3"

patterns-established:
  - "Phase 4 cache pattern: load_jira_cache on startup, re-apply on WorktreesLoaded, persist on JiraTitlesFetched"
  - "Trait object debug: any trait stored as Arc<dyn Trait> in AppState must include Debug as supertrait"

requirements-completed: [INTG-01, INTG-02, INTG-03, INTG-05]

# Metrics
duration: 2min
completed: 2026-03-02
---

# Phase 4 Plan 02: JIRA App Wiring Summary

**Background JIRA title fetching wired into TEA loop: config loaded on startup, titles cached and re-applied on WorktreesLoaded, JiraTitlesFetched handler persists to disk and updates worktree display names**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-02T12:57:33Z
- **Completed:** 2026-03-02T13:04:33Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- JiraTitlesFetched(Vec<(String, String)>) action variant added to Action enum
- AppState extended with tmux_available, jira_title_cache, jira_client (Arc<dyn JiraClient>)
- run() startup sequence: detect tmux, load DashConfig, build HttpJiraClient, load cache — all silently skipped on missing config
- WorktreesLoaded handler: cached titles re-applied before worktrees stored, background fetch spawned for uncached JIRA keys
- JiraTitlesFetched handler: in-memory cache updated, persisted via save_jira_cache, applied to active worktrees
- panels.rs verified as already correct — display_name() cascade (label > jira_title > branch) and dim branch parenthetical already present

## Task Commits

Each task was committed atomically:

1. **Task 1: Add JiraTitlesFetched action variant and wire AppState + update() + run()** - `392ef26` (feat)
2. **Task 2: Update worktree list rendering to show JIRA titles** - no commit required (already correct)

**Plan metadata:** (see final commit below)

## Files Created/Modified
- `src/action.rs` - Added JiraTitlesFetched(Vec<(String, String)>) variant to Action enum
- `src/app.rs` - Phase 4 fields in AppState, Default impl, run() startup wiring, WorktreesLoaded handler updates, JiraTitlesFetched handler
- `src/infra/jira.rs` - Added Debug supertrait to JiraClient, derive Debug on HttpJiraClient

## Decisions Made
- JiraClient trait requires `Debug` supertrait: AppState derives Debug so any field stored as `Arc<dyn Trait>` requires the trait to also bound on Debug. Added `std::fmt::Debug` to JiraClient's supertrait list and `#[derive(Debug)]` to HttpJiraClient.
- panels.rs required no changes: the Phase 3 implementation already used `wt.display_name()` and showed branch name in dim parentheses when `wt.label.is_some() || wt.jira_title.is_some()` — exactly what this plan required.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added Debug supertrait to JiraClient to satisfy AppState derive**
- **Found during:** Task 1 (AppState Phase 4 fields)
- **Issue:** `AppState` derives `Debug`, but `Arc<dyn JiraClient>` does not implement `Debug` because `JiraClient` did not have `Debug` as a supertrait.
- **Fix:** Added `std::fmt::Debug` to JiraClient supertrait bounds; added `#[derive(Debug)]` to HttpJiraClient struct.
- **Files modified:** `src/infra/jira.rs`
- **Verification:** cargo check passes with no errors
- **Committed in:** `392ef26` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - compilation bug)
**Impact on plan:** Required for the code to compile. No scope creep.

## Issues Encountered
- None beyond the Debug supertrait requirement (auto-fixed).

## User Setup Required
None - no external service configuration required for this plan. JIRA integration silently skips when `~/.config/ump-dash/config.json` is absent.

## Next Phase Readiness
- Phase 4 JIRA integration complete: titles fetch in background, cache persists, UI shows them
- tmux_available flag set on startup, gates Phase 5 tmux features
- Phase 5 (tmux integration) can rely on state.tmux_available and the established TEA action pattern

---
*Phase: 04-config-and-jira-integration*
*Completed: 2026-03-02*
