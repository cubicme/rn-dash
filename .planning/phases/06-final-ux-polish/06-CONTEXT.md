# Phase 06: Final UX Polish - Context

**Gathered:** 2026-03-12
**Status:** Ready for planning
**Source:** User feedback (direct)

<domain>
## Phase Boundary

Final round of UX polish based on real usage testing. Six discrete fixes spanning metro log filtering, multiplexer integration, visual indicators, and naming conventions.

</domain>

<decisions>
## Implementation Decisions

### Metro Log Filtering
- Remove junk/noise logs from metro output — watchman warnings and other non-useful output should be filtered out before display

### Multiplexer Tab from Worktree
- Need a command to open a tmux/zellij tab from the selected worktree (not just Claude Code — a general-purpose shell tab)

### Metro Running Indicator
- Show a green play icon at the start of the worktree row when metro is running for that worktree
- This is the primary visual indicator that metro is active

### Prefix Ordering Fix
- The prefix for tmux tab naming should be `e2e-claude` not `claude-e2e`
- preferred_prefix() should come first, then the tab type suffix

### Optional Name for Claude Tab
- When opening a new Claude Code tab, allow an optional name for the part after the prefix
- Default to "claude" if the user just presses Enter without typing
- If the user types something, use that as the suffix instead

### Double Border on Title
- Show the double border around the title bar/header itself too (currently only on panes)

### Claude's Discretion
- Metro log filtering patterns (which specific log lines to suppress)
- Key binding choice for the new "open tab" command
- Input modal UX for the optional claude tab name

</decisions>

<specifics>
## Specific Ideas

- Green play icon (unicode) prepended to worktree row text when metro is running
- Prefix format: `{preferred_prefix}-{tab_type}` e.g. `e2e-claude`, `e2e-shell`
- Filter patterns for metro: watchman warnings, other noisy lines that aren't useful
- Double border (BorderType::Double) on the main title/header block

</specifics>

<deferred>
## Deferred Ideas

None — all items are in scope for this phase.

</deferred>

---

*Phase: 06-final-ux-polish*
*Context gathered: 2026-03-12 via user feedback*
