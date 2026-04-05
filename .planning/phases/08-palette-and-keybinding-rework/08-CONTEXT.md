# Phase 08: Palette and Keybinding Rework - Context

**Gathered:** 2026-04-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Users interact with a clean, context-sensitive keybinding scheme — yarn palette, worktree palette, metro keys only when relevant, dynamic hints. All hint text is derived from available actions, not hardcoded strings.

</domain>

<decisions>
## Implementation Decisions

### Palette Restructuring
- Rename (s)ync palette to (y)arn palette
- Move clean commands (clean android, clean cocoapods, rm node_modules) into (y)arn palette alongside yarn install, pod-install
- Extract worktree commands from (g)it into new (w)orktree palette
- (w)orktree palette contains: create worktree (existing), remove worktree (existing), create worktree with new branch (new command)

### New Worktree Creation
- New command in (w)orktree palette: create worktree with a new branch
- Must ask user for the base branch (interactive picker showing available branches)
- Creates a new branch from the selected base and a new worktree for it

### Metro Key Context
- R (reload) and J (debugger) keys should ONLY appear/work when metro is running
- Remove metro restart key entirely — RET (Enter) already handles restart via worktree switch
- ESC stops metro when metro is running

### Dynamic Hints
- Footer hint line must be derived from currently available actions, not hardcoded strings
- When metro is not running, metro-specific keys (R/J) should not appear in hints
- When in a modal or palette, hints should reflect that mode's available actions
- Remove the stale '▶=metro  ⚠=stale' legend from footer

### Claude's Discretion
- Internal implementation of the dynamic hint system (data structure, refresh strategy)
- How to structure the worktree palette mode enum variant

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- PaletteMode enum in domain types — add Yarn and Worktree variants
- handle_key() already has palette routing logic
- Existing worktree create/remove commands can be moved to new palette

### Established Patterns
- Palettes are PaletteMode variants dispatched in handle_key()
- Footer hints are currently hardcoded strings in footer.rs
- Help overlay has a static list of keybindings

### Integration Points
- src/domain/command.rs — PaletteMode enum
- src/app.rs — handle_key() palette routing
- src/ui/footer.rs — hint line rendering
- src/ui/help_overlay.rs — help text

</code_context>

<specifics>
## Specific Ideas

- User explicitly stated: "instead of R and j inside of metro, let's have R and J only when metro is running"
- User explicitly stated: "we don't need metro restart as RET will do that"
- User explicitly stated: "let's have ESC as stop for now"
- User explicitly stated: "hints should be actually derived from what's available not hardcoded"
- User explicitly stated: "let's change (s)ync to (y)arn"
- User explicitly stated: "let's move clean also to y"
- User explicitly stated: "get worktree out of (g)it and make it (w)orktree"
- User explicitly stated: "new command there that creates a worktree with a new branch which should ask for the base of the new branch"

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>
