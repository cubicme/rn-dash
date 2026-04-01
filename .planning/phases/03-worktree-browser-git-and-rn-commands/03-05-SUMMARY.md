---
phase: 03-worktree-browser-git-and-rn-commands
plan: 05
subsystem: domain
tags: [rust, command-spec, react-native, yarn, argv]

# Dependency graph
requires:
  - phase: 03-worktree-browser-git-and-rn-commands
    provides: CommandSpec enum with to_argv() and label() methods
provides:
  - Corrected to_argv() for 6 CommandSpec variants matching RN-01, RN-02, RN-06, RN-07, RN-08, RN-10
  - Corrected label() for 4 CommandSpec variants reflecting updated commands
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "to_argv() is the sole source of truth for process argv — no argv construction outside domain"

key-files:
  created: []
  modified:
    - src/domain/command.rs

key-decisions:
  - "RnRunAndroid uses npx react-native run-android (not yarn android) per RN-06"
  - "RnRunIos uses yarn react-native run-ios (not yarn ios) per RN-07 — yarn as launcher, react-native as sub-command"
  - "RnCleanAndroid and RnCleanCocoapods both use npx react-native clean --include <target> (not gradlew/pod) per RN-01 and RN-02"

patterns-established:
  - "Gap closure plans are single-task, single-file — verify with grep + cargo build"

requirements-completed:
  - WORK-01
  - WORK-02
  - WORK-03
  - WORK-05
  - WORK-06
  - GIT-01
  - GIT-02
  - GIT-03
  - GIT-04
  - GIT-05
  - GIT-06
  - RN-01
  - RN-02
  - RN-03
  - RN-04
  - RN-05
  - RN-06
  - RN-07
  - RN-08
  - RN-09
  - RN-10
  - RN-11
  - RN-12

# Metrics
duration: 2min
completed: 2026-03-02
---

# Phase 3 Plan 05: CommandSpec to_argv() Gap Closure Summary

**Six CommandSpec variants corrected to emit requirement-compliant argv: react-native clean --include, run-android/run-ios via npx/yarn react-native, yarn unit-tests, and yarn lint --quiet --fix**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-02T08:59:11Z
- **Completed:** 2026-03-02T09:01:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Fixed RnCleanAndroid: `./gradlew clean` -> `npx react-native clean --include android` (RN-01)
- Fixed RnCleanCocoapods: `pod deintegrate` -> `npx react-native clean --include cocoapods` (RN-02)
- Fixed RnRunAndroid: `yarn android --deviceId` -> `npx react-native run-android --deviceId` (RN-06)
- Fixed RnRunIos: `yarn ios --udid` -> `yarn react-native run-ios --udid` (RN-07)
- Fixed YarnUnitTests: `yarn test` -> `yarn unit-tests` (RN-08)
- Fixed YarnLint: `yarn lint` -> `yarn lint --quiet --fix` (RN-10)
- Updated label() for 4 variants to reflect corrected commands

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix CommandSpec to_argv() and label() for 6 requirement-deviating variants** - `68476f7` (fix)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified

- `/Users/cubicme/aljazeera/dashboard/src/domain/command.rs` - Corrected to_argv() and label() for 6 CommandSpec variants

## Decisions Made

- RnRunIos keeps `yarn` as the launcher per RN-07 spec, with `react-native run-ios` as the sub-command (not `npx`)
- No other methods (is_destructive, needs_text_input, needs_device_selection) were touched — scope strictly limited to to_argv() and label()

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None — single-file change, cargo build passed on first attempt with only pre-existing warnings (3 dead code warnings unrelated to this change).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All 17 CommandSpec variants now produce requirement-compliant argv
- Phase 3 gap closed: truth #22 in 03-VERIFICATION.md is now satisfied
- Phase 4 (JIRA integration) can proceed — JIRA auth method confirmation still required before implementation

## Self-Check: PASSED

- FOUND: src/domain/command.rs
- FOUND: 03-05-SUMMARY.md
- FOUND: task commit 68476f7 (fix(03-05): correct 6 CommandSpec to_argv() deviations)
- FOUND: metadata commit 6c5fce8 (docs(03-05): complete gap closure plan)

---
*Phase: 03-worktree-browser-git-and-rn-commands*
*Completed: 2026-03-02*
