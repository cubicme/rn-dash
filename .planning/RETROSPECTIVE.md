# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — MVP

**Shipped:** 2026-04-05
**Phases:** 8 | **Plans:** 37 | **Tasks:** 59

### What Was Built
- Full Rust/Ratatui terminal dashboard for React Native worktree management
- Metro process control with single-instance enforcement and external conflict detection
- Worktree browser with JIRA integration, custom labels, and staleness detection
- Complete git operations and RN command palette with streaming output
- 5-palette submenu keybinding scheme (a/i/x/s/g) with vim-style navigation
- Command queue system with per-worktree output persistence
- Multiplexer abstraction (tmux + zellij) for terminal tab management
- Worktree creation/removal, metro auto-prerequisite for RN runs

### What Worked
- TEA (Elm Architecture) pattern scaled well through 8 phases — AppState remained the single source of truth
- Domain/infra/app/ui separation kept modules deep and changes localized
- Quick tasks (/gsd-quick) were effective for post-milestone polish — 10 quick tasks shipped iteratively
- Decimal phase insertion (05.1, 05.2) handled real-usage feedback without disrupting the roadmap
- Per-worktree output persistence and command queue emerged from real usage, not upfront design

### What Was Inefficient
- Two feedback phases (05.1 + 05.2) with 18 plans total — could have been one phase with better upfront UX design
- Some gap closure plans were just fixing argv strings — these should have been caught in initial plan review
- Staleness detection went through 3 rewrites (.yarn-integrity, .yarn-state, install-state.gz) — should have researched yarn versioning upfront

### Patterns Established
- `CommandSpec::needs_metro()` pattern for prerequisite gating
- `refresh_needed()` pure domain function mapping commands to refresh sets
- `preferred_prefix()` for consistent display naming across UI surfaces
- Labels follow branches, not worktrees (persisted in labels.json keyed by branch name)
- External process conflict detection via lsof port lookup

### Key Lessons
1. Real-usage feedback phases are inevitable — budget for them in milestone planning
2. Device enumeration (adb/xcrun) has platform quirks that only surface on real hardware — test early
3. Metro log streaming generates high-frequency events — auto-follow with manual scroll override is the right UX
4. Port conflict detection is essential for any process manager — users always have stale processes

### Cost Observations
- Model mix: opus for planning, sonnet for execution — balanced profile
- 207 commits over 34 days
- 5,491 LOC Rust

## Cross-Milestone Trends

| Metric | v1.0 |
|--------|------|
| Phases | 8 |
| Plans | 37 |
| Tasks | 59 |
| LOC | 5,491 |
| Days | 34 |
| Quick Tasks | 10 |
