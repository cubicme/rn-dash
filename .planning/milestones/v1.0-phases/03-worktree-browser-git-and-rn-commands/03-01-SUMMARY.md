---
phase: 03-worktree-browser-git-and-rn-commands
plan: 01
subsystem: domain
tags: [rust, ratatui, domain-types, command-spec, worktree, serde, serde_json, tea-pattern]

# Dependency graph
requires:
  - phase: 01-scaffold-and-tui-shell
    provides: Action enum, domain/mod.rs structure, TEA pattern
  - phase: 02-metro-process-control
    provides: MetroHandle, metro domain types
provides:
  - CommandSpec enum (17 variants) with to_argv(), is_destructive(), needs_text_input(), needs_device_selection(), label()
  - ModalState enum (Confirm, TextInput, DevicePicker) and DeviceInfo struct in domain/command.rs
  - Fully-populated Worktree struct (8 fields) with display_name() method and WorktreeMetroStatus enum
  - Action enum extended with 21 Phase 3 variants (worktree nav, command lifecycle, modal flow, label management)
  - serde + serde_json explicit deps in Cargo.toml
affects:
  - 03-02 (app logic: WorktreeManager, CommandRunner, ModalState machine)
  - 03-03 (UI: WorktreeList widget, CommandOutputPanel, modal overlays)
  - 04-jira-integration (Worktree.jira_title field)
  - 05-worktree-switching (Worktree struct)

# Tech tracking
tech-stack:
  added:
    - serde = { version = "1", features = ["derive"] }
    - serde_json = "1"
  patterns:
    - CommandSpec enum as pure data — to_argv() converts to process args, no I/O in domain layer
    - ModalState enum covers all three UI modal flows (confirm, text input, device picker)
    - display_name() priority chain: label > jira_title > branch

key-files:
  created:
    - src/domain/command.rs
  modified:
    - src/domain/worktree.rs
    - src/domain/mod.rs
    - src/action.rs
    - src/app.rs
    - Cargo.toml

key-decisions:
  - "CommandSpec.to_argv() uses yarn check-types --incremental per CLAUDE.md project note"
  - "Worktree derives PartialEq (PathBuf, String, bool all implement it) so Action enum can keep its PartialEq derive"
  - "Phase 3 Action variants added as exhaustive stubs in app.rs update() match — implemented in Plan 03-02"
  - "DeviceInfo derives PartialEq alongside Debug and Clone for consistency with rest of domain types"

patterns-established:
  - "Pattern 1: CommandSpec is pure data; infrastructure layer calls to_argv() and spawns the process"
  - "Pattern 2: ModalState covers all three flows needed by Phase 3 UI (confirm, text-input, device-picker)"
  - "Pattern 3: display_name() implements label > JIRA title > branch fallback chain"

requirements-completed:
  - WORK-01
  - WORK-02
  - WORK-03
  - WORK-05
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

# Phase 03 Plan 01: Domain Types — CommandSpec, Worktree, Action Extensions Summary

**Pure Rust domain types: CommandSpec (17 variants), expanded Worktree struct (8 fields), ModalState, and 21 new Action variants establishing all Phase 3 type contracts**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-02T08:22:50Z
- **Completed:** 2026-03-02T08:25:05Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Created `src/domain/command.rs` with CommandSpec (17 variants), to_argv(), is_destructive(), needs_text_input(), needs_device_selection(), label(), plus ModalState and DeviceInfo types
- Expanded `src/domain/worktree.rs` from a 2-field stub to an 8-field struct with WorktreeMetroStatus enum and display_name() priority chain
- Extended `src/action.rs` with 21 Phase 3 variants covering worktree navigation, command lifecycle, modal flow, and label management
- Added serde + serde_json to Cargo.toml as explicit deps for labels.json persistence in Plans 02+

## Task Commits

Each task was committed atomically:

1. **Task 1: Add serde deps, create command.rs, expand worktree.rs** - `02eba27` (feat)
2. **Task 2: Extend Action enum with Phase 3 variants** - `ad81173` (feat)

**Plan metadata:** (docs commit to follow)

## Files Created/Modified

- `src/domain/command.rs` - CommandSpec (17 variants), ModalState (3 variants), DeviceInfo — core type contracts for Phase 3
- `src/domain/worktree.rs` - Expanded from stub to full 8-field Worktree struct with WorktreeMetroStatus and display_name()
- `src/domain/mod.rs` - Added `pub mod command;` export
- `src/action.rs` - 21 new Phase 3 Action variants appended (worktree nav, command lifecycle, modal flow, label management)
- `src/app.rs` - Phase 3 stub match arms added to update() to maintain exhaustive matching
- `Cargo.toml` + `Cargo.lock` - serde and serde_json deps added

## Decisions Made

- `CommandSpec.to_argv()` uses `yarn check-types --incremental` per CLAUDE.md project note
- `Worktree` derives `PartialEq` (all fields support it) so `Action` enum can keep its `PartialEq` derive on `WorktreesLoaded(Vec<Worktree>)`
- Phase 3 Action variants added as no-op stubs in `app.rs update()` — full implementation in Plan 03-02
- `DeviceInfo` derives `PartialEq` for consistency with other domain types

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Phase 3 stub match arms in app.rs update()**
- **Found during:** Task 2 (Extend Action enum with Phase 3 variants)
- **Issue:** Adding 21 new Action variants to the enum made the existing exhaustive match in `app.rs update()` fail to compile (E0004: non-exhaustive patterns)
- **Fix:** Added a grouped match arm covering all 21 Phase 3 variants with `// Phase 3 stubs — implemented in Plan 03-02` comment body
- **Files modified:** `src/app.rs`
- **Verification:** `cargo check` passes with 0 errors
- **Committed in:** `ad81173` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 - blocking compilation issue)
**Impact on plan:** Required change to maintain compilability. No scope creep. Stubs will be replaced by real implementations in Plan 03-02.

## Issues Encountered

None beyond the auto-fixed compilation issue above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All domain type contracts established — Plans 03-02 (app logic) and 03-03 (UI) can implement against these types
- CommandSpec.to_argv() is complete — infra layer only needs to call it and spawn the process
- ModalState covers all three UI flows needed by Phase 3 (confirm destructive, text input, device picker)
- Worktree.display_name() implements the label > JIRA > branch fallback chain used in all list rendering

---
*Phase: 03-worktree-browser-git-and-rn-commands*
*Completed: 2026-03-02*
