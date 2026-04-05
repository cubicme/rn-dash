---
phase: 03-worktree-browser-git-and-rn-commands
plan: "04"
subsystem: ui
tags: [ratatui, worktree, modals, command-output, footer, help-overlay, stateful-widget]

requires:
  - phase: 03-03
    provides: AppState with worktrees/command_output/modal/palette_mode fields wired

provides:
  - Real worktree list with StatefulWidget selection, metro badges, staleness hints, and labels
  - Scrollable command output panel with running-command title indicator
  - Three modal overlay types: Confirm (Y/N), TextInput (char-by-char), DevicePicker (list)
  - Context-sensitive footer hints for palette mode, modal state, and per-panel focus
  - Expanded help overlay with Worktree List, Git Palette, RN Palette keybinding sections

affects:
  - Phase 4 (JIRA, settings UI)
  - Phase 5 (worktree switching, device enrichment)

tech-stack:
  added: []
  patterns:
    - StatefulWidget pattern via render_stateful_widget for List+ListState
    - view() accepts &mut AppState to allow StatefulWidget rendering
    - Modal dispatch via single render_modal() entry point in modals.rs
    - Separate centered_rect() per overlay module (no cross-widget coupling)

key-files:
  created:
    - src/ui/modals.rs
  modified:
    - src/ui/mod.rs
    - src/ui/panels.rs
    - src/ui/footer.rs
    - src/ui/help_overlay.rs
    - src/app.rs

key-decisions:
  - "view() signature changed to &mut AppState — required by render_stateful_widget needing &mut ListState"
  - "modals.rs is a separate module with its own centered_rect() — avoids cross-widget coupling (same pattern as error_overlay.rs)"
  - "DevicePicker modal uses local ListState (let mut ls) for rendering — modal owns selected index, not a ListState"
  - "Help overlay height increased from 70% to 80% and first column from 18 to 28 chars to fit Phase 3 rows"

patterns-established:
  - "StatefulWidget pattern: render_stateful_widget(widget, area, &mut state.list_state)"
  - "Modal overlay pattern: Clear first, then widget, using centered_rect() helper"
  - "Footer priority order: palette mode → modal state → overlay modes → panel-specific hints"

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

duration: 10min
completed: 2026-03-02
---

# Phase 3 Plan 04: UI Rendering — Worktrees, Modals, Footer, Help Summary

**Ratatui UI layer completed: real worktree List+StatefulWidget with metro badges, scrollable command output, three modal overlay types (confirm/text-input/device-picker), context-sensitive footer hints for palette and modal states, and expanded help overlay with all Phase 3 keybindings**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-03-02T08:31:00Z
- **Completed:** 2026-03-02T08:41:01Z
- **Tasks:** 2
- **Files modified:** 5 modified, 1 created

## Accomplishments

- Replaced worktree list placeholder with real `List` + `ListState` StatefulWidget rendering metro badges (`[M]`/`[ ]`), display names (label > jira_title > branch), branch-in-dim when label set, and `[stale]` hint
- Replaced command output placeholder with scrollable `Paragraph` + `ScrollbarState`, auto-scrolling to bottom, with running command name and `[running]` indicator in the title
- Created `modals.rs` with `render_modal()` dispatching to `render_confirm_modal`, `render_text_input_modal`, and `render_device_picker_modal` — each using Clear + centered_rect overlay pattern
- Updated footer to show palette-mode hints (12 keys for Git/Rn), modal hints (Confirm/TextInput/DevicePicker), and updated panel hints with `g`/`c`/`L` for WorktreeList and `Ctrl-c` when command running
- Expanded help overlay from 15 to 47 rows covering Worktree List, Git Palette, and RN Palette sections; increased height from 70% to 80%

## Task Commits

Each task was committed atomically:

1. **Task 1: Real worktree list, command output, modal dispatch** - `cfb55f9` (feat)
2. **Task 2: Footer hints and help overlay Phase 3 content** - `577245a` (feat)

**Plan metadata:** (docs commit — see final_commit below)

## Files Created/Modified

- `src/ui/modals.rs` - New: render_modal dispatch + confirm/text-input/device-picker renderers
- `src/ui/mod.rs` - view() signature changed to &mut AppState, added pub mod modals, modal render call
- `src/ui/panels.rs` - Real worktree List+StatefulWidget, real scrollable command output with scrollbar
- `src/ui/footer.rs` - Palette mode hints, modal hints, updated WorktreeList/CommandOutput panel hints
- `src/ui/help_overlay.rs` - Added Phase 3 keybinding sections, 80% height, 28-char first column
- `src/app.rs` - terminal.draw call updated to pass &mut state

## Decisions Made

- `view()` signature changed to `&mut AppState` — required because `render_stateful_widget` needs `&mut ListState`; all render functions that don't use StatefulWidget still take `&AppState` (only `render_worktree_list` needs mut)
- `modals.rs` gets its own `centered_rect()` — follows existing pattern of one per overlay module to avoid cross-widget coupling
- `DevicePicker` modal creates a local `ListState` for rendering (the selected index lives in `ModalState::DevicePicker.selected`, not a separate `ListState` field)
- Help overlay first column widened from 18 to 28 characters to fit section header text like "Git Palette (g then...)"

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 3 is now 100% complete: domain types (01), infra (02), app wiring (03), UI rendering (04)
- All Phase 3 requirements (WORK-01–03, WORK-05–06, GIT-01–06, RN-01–12) are satisfied
- Phase 4 (JIRA integration) can begin — needs JIRA auth method confirmed (Cloud vs Data Center)
- Phase 5 (worktree switching) has device selection UI already implemented in Phase 3

## Self-Check: PASSED

- `src/ui/modals.rs` exists and compiles
- `src/ui/panels.rs` contains `render_stateful_widget`
- `src/ui/modals.rs` contains `render_confirm_modal`
- `src/ui/footer.rs` contains `PaletteMode`
- `src/ui/help_overlay.rs` contains "Worktree"
- `cargo build` passes with 0 errors (3 pre-existing dead code warnings only)
- Commits cfb55f9 and 577245a exist

---
*Phase: 03-worktree-browser-git-and-rn-commands*
*Completed: 2026-03-02*
