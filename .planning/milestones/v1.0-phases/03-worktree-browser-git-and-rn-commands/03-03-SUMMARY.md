---
phase: 03-worktree-browser-git-and-rn-commands
plan: 03
subsystem: ui
tags: [ratatui, tea, app-state, command-palette, modal, worktrees, labels]

# Dependency graph
requires:
  - phase: 03-01
    provides: CommandSpec, ModalState, Worktree, Action variants (WorktreeSelectNext/Prev, CommandRun, all modal variants)
  - phase: 03-02
    provides: spawn_command_task, list_worktrees, load_labels/save_labels, list_android_devices/list_ios_devices

provides:
  - Fully wired app.rs: AppState with all Phase 3 fields, handle_key routing, update() for all actions
  - PaletteMode enum (Git/Rn) for two-stroke command dispatch
  - Modal interception in handle_key preventing key leak to navigation
  - Lazy yarn install before run-android/run-ios on stale worktrees (WORK-06)
  - Worktree list loaded on startup and refreshed on demand
  - Label persistence via save_labels on SetLabel
  - Device enumeration flow: async task -> DevicesEnumerated -> DevicePicker modal

affects: [03-04, 04-jira-integration, 05-worktree-management]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - dispatch_command() helper skips pre-processing pipeline (used by ModalConfirm to avoid double-destructive-check)
    - pending_device_command + DevicesEnumerated pattern for async-to-modal bridge
    - pending_label_branch flag distinguishes label submit from command submit in ModalInputSubmit
    - palette_mode in AppState + handle_key reads it — two-key command dispatch without extra Action roundtrip
    - WORK-06: pending_command_after_install + CommandExited dispatch deferred command

key-files:
  created: []
  modified:
    - src/action.rs
    - src/app.rs

key-decisions:
  - "dispatch_command() helper introduced to skip pre-processing — ModalConfirm calls it directly to avoid re-triggering destructive confirmation"
  - "DevicesEnumerated added to Action enum — only way to bridge async device list call back into sync update(); stored spec in pending_device_command while in flight"
  - "pending_label_branch: Option<String> in AppState distinguishes label submit from command submit in ModalInputSubmit — avoids adding a ModalState::LabelInput variant"
  - "palette_mode: Option<PaletteMode> in AppState read by handle_key for two-stroke commands; update() clears it on CommandRun or ModalCancel"
  - "YarnLint used as sentinel pending_template in StartSetLabel's TextInput modal — consumed by pending_label_branch check before template is ever read"

patterns-established:
  - "Pattern: handle_key modal check -> palette check -> overlay check -> panel-specific check -> normal — strict priority order"
  - "Pattern: dispatch_command() for direct execution; CommandRun Action for pre-processing pipeline"

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
duration: 3min
completed: 2026-03-02
---

# Phase 3 Plan 03: App Wiring — Worktrees, Commands, Modals Summary

**TEA brain for Phase 3: AppState extended with 11 new fields, handle_key with modal+palette routing, update() covering all 20+ Phase 3 action arms, and worktree/label loading wired into run()**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-02T08:32:39Z
- **Completed:** 2026-03-02T08:35:45Z
- **Tasks:** 2 (combined into single commit — both modify same files)
- **Files modified:** 2

## Accomplishments

- AppState extended with worktrees, worktree_list_state, command_output, running_command, command_task, pending_command_after_install, modal, labels, repo_root, palette_mode, pending_device_command, pending_label_branch fields
- handle_key routing: modal interception first (prevents key leak), then palette mode, then overlays, then panel-specific (WorktreeList j/k/g/c/R/L, CommandOutput j/k/X/C), then normal navigation
- update() handles all Phase 3 arms: WorktreeSelectNext/Prev, WorktreesLoaded (applies labels, clamps index), RefreshWorktrees, CommandRun (full pre-processing pipeline), CommandOutputLine/Exited/Clear/Cancel, all modal arms, DevicesEnumerated, SetLabel/StartSetLabel, EnterGitPalette/EnterRnPalette
- WORK-06 lazy install: stale worktrees automatically get yarn install before run-android/run-ios via pending_command_after_install
- Labels loaded on startup, persisted on SetLabel, applied to worktrees on WorktreesLoaded
- Initial worktree list loaded async on run() startup

## Task Commits

Both tasks implemented together (same files — inseparable):

1. **Task 1 + Task 2: AppState, handle_key, update(), run()** - `dad03bd` (feat)

## Files Created/Modified

- `src/action.rs` - Added DevicesEnumerated, EnterGitPalette, EnterRnPalette variants
- `src/app.rs` - Full Phase 3 implementation: PaletteMode, extended AppState, handle_key routing, update() all arms, dispatch_command() helper, run() with startup loading

## Decisions Made

- **dispatch_command() helper:** Extracted "actually run command" logic into a helper function to skip the pre-processing pipeline (is_destructive, needs_text_input, needs_device_selection checks). ModalConfirm calls it directly, bypassing the destructive check that would otherwise re-trigger confirmation.
- **DevicesEnumerated in Action enum:** The only way to bridge async device enumeration back into sync update(). Spec stored in pending_device_command while async task is in flight; DevicesEnumerated delivers the result.
- **pending_label_branch flag:** Rather than adding ModalState::LabelInput variant to domain/command.rs, AppState holds a pending_label_branch: Option<String>. ModalInputSubmit checks this to distinguish label submit from command submit.
- **YarnLint as sentinel:** StartSetLabel uses YarnLint as the pending_template in the TextInput modal. It's never consumed via the command path — pending_label_branch check fires first.
- **Palette mode via AppState:** Two-stroke command dispatch (g then p = git pull) implemented by reading state.palette_mode in handle_key. update() clears palette_mode on CommandRun or ModalCancel.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed char dereference in modal TextInput arm**
- **Found during:** Task 1 (handle_key ModalState::TextInput arm)
- **Issue:** `Char(c) => Some(Action::ModalInputChar(*c))` — `c` is already a `char` by value in crossterm match, not a reference
- **Fix:** Removed dereference: `Char(c) => Some(Action::ModalInputChar(c))`
- **Files modified:** src/app.rs
- **Verification:** `cargo check` passes cleanly
- **Committed in:** dad03bd (same task commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Trivial compile fix. No scope change.

## Issues Encountered

None beyond the char dereference compile error.

## Next Phase Readiness

- app.rs is fully wired for Phase 3 behavior
- All command dispatch flows through the pre-processing pipeline correctly
- UI layer (Phase 3 Plan 04) can render worktree list, command palette hints, modals, and command output using the new AppState fields
- Phase 4 (JIRA) will add jira_title to Worktree — WorktreesLoaded already applies labels, same pattern can be used

---
*Phase: 03-worktree-browser-git-and-rn-commands*
*Completed: 2026-03-02*
