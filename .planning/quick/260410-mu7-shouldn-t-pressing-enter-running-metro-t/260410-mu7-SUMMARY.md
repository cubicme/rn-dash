---
phase: quick-260410-mu7
plan: "01"
subsystem: ui/app
tags: [modal, metro, worktree, stale-deps, sync]
dependency_graph:
  requires: []
  provides: [SyncBeforeMetro modal flow]
  affects: [WorktreeSwitchToSelected, CommandExited, MetroSpawnFailed, CommandCancel]
tech_stack:
  added: []
  patterns: [SyncBeforeRun mirror pattern, TEA action/update flow]
key_files:
  created: []
  modified:
    - src/domain/command.rs
    - src/action.rs
    - src/app.rs
    - src/ui/modals.rs
    - src/ui/footer.rs
decisions:
  - Tasks 1 and 2 implemented together because adding ModalState::SyncBeforeMetro immediately caused non-exhaustive match errors in modals.rs, footer.rs, and app.rs — required all arms to be present for cargo check to pass
metrics:
  duration: 12
  completed_date: "2026-04-10"
  tasks_completed: 2
  files_modified: 5
---

# Quick 260410-mu7: SyncBeforeMetro modal for stale-dep check on Enter

**One-liner:** SyncBeforeMetro modal mirrors SyncBeforeRun pattern — Enter on a stale worktree shows a Y/N prompt before starting metro, with yarn/pod sync dispatched on accept and metro auto-started after the queue drains.

## What Was Built

When the user presses Enter on a worktree in the WorktreeTable, `WorktreeSwitchToSelected` now checks `wt.stale` and `wt.stale_pods` before proceeding. If either is true, a `ModalState::SyncBeforeMetro` modal is shown instead of starting metro directly.

- **Accept (Y):** switches `active_worktree_path` immediately, stops metro if running (no auto-restart), dispatches yarn install and/or pod-install into the command queue, sets `pending_metro_after_sync = true`. When `CommandExited` fires and the queue is empty, metro is started.
- **Decline (N/Esc):** consumes `pending_switch_path` and proceeds with the original stop+restart or direct-start logic.
- **Fresh worktrees:** bypass the modal entirely — no change to existing behavior.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2 | Add types, actions, state field, wire all handlers | c6d703d | src/domain/command.rs, src/action.rs, src/app.rs, src/ui/modals.rs, src/ui/footer.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Tasks 1 and 2 combined into single commit**
- **Found during:** Task 1 verification
- **Issue:** Adding `ModalState::SyncBeforeMetro` caused 4 non-exhaustive match errors in `render_modal` (modals.rs), `key_hints_for` (footer.rs), `handle_key` (app.rs), and `update` (app.rs). cargo check cannot pass with partial implementation.
- **Fix:** Implemented all Task 2 changes immediately to satisfy exhaustive pattern matching, committed as one atomic unit.
- **Files modified:** all 5 plan files
- **Commit:** c6d703d

## Known Stubs

None. All data flows are wired: stale fields come from existing `wt.stale`/`wt.stale_pods`, sync commands use existing `CommandSpec::YarnInstall`/`YarnPodInstall`, metro start uses existing `Action::MetroStart`.

## Threat Flags

None. No new trust boundaries — all changes are internal UI state management with no external I/O, network calls, or user-supplied strings.

## Self-Check: PASSED

- src/domain/command.rs modified: FOUND
- src/action.rs modified: FOUND
- src/app.rs modified: FOUND
- src/ui/modals.rs modified: FOUND
- src/ui/footer.rs modified: FOUND
- Commit c6d703d: FOUND (git log confirmed)
- cargo check: Finished clean
