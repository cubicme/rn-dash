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

## Milestone: v1.1 — Public Release

**Shipped:** 2026-04-13
**Phases:** 4 | **Plans:** 9

### What Was Built
- Labels feature removed entirely — cleaner domain, no hidden state
- Palette scheme reworked: (y)arn absorbs clean commands, (w)orktree extracted from git, lowercase keys, new-branch worktree creation with interactive base branch picker
- Context-sensitive metro keys (R/J/Esc only when running), dynamic footer hints, MetroRestart removed
- Generalized: package renamed `ump-dash` → `rn-dash`, all hardcoded AJ/UMP values extracted to DashConfig
- TOML config format replaces JSON with annotated `config.example.toml`
- Public GitHub release: MIT license, README, Cargo.toml metadata, `.gitignore` audit
- GitHub Actions CI (build + clippy -D warnings + test on macOS+Linux)
- Tag-triggered release workflow publishing signed+notarized macOS binaries and Linux tar.gz

### What Worked
- Phase 07 (labels removal) as a dedicated cleanup phase before keybinding rework — simplified downstream work
- Gap closure plans (08-04, 08-05) absorbed UAT feedback without needing a separate feedback phase
- CI/release isolated to final Phase 10 — no app code risk, shipped cleanly
- TOML migration handled in-phase as gap closure rather than deferred — avoided carrying JSON debt forward

### What Was Inefficient
- REQUIREMENTS.md traceability table got stale (still showed "Pending" for shipped reqs) — manual checkbox updates didn't keep pace with execution
- Two gap closure plans (08-04, 08-05) hint that UAT earlier in Phase 08 would have caught these issues during wave-1
- `config_dir()` relocation in Phase 07 was a minor refactor bundled with label removal — could have been its own plan or caught in Phase 09 generalization

### Patterns Established
- Gap closure plans at end of a phase for UAT-discovered issues (vs. new phase)
- Config-driven generalization (DashConfig with serde defaults) for any-repo portability
- macOS codesign + notarize in release workflow — avoid Gatekeeper friction
- Lowercase palette key convention (w/d/b) for consistency

### Key Lessons
1. Maintain REQUIREMENTS.md traceability checkboxes in the SAME commit as plan completion — stale tracking erodes trust in the doc
2. Name things for the audience, not the origin — `ump-dash` → `rn-dash` was trivial but should have been day-one naming
3. Plan for macOS Gatekeeper early if distributing binaries — notarization is non-trivial and user-facing
4. Post-ship quick tasks (8 in the 7 days since shipping) indicate the right moment to cut v1.2.0 from accumulated polish

### Cost Observations
- Post-ship activity: 8 quick tasks in 7 days → rolled into v1.2.0 release
- 60 files changed, +5,139 / -546 LOC across milestone
- 8-day active window (2026-04-05 → 2026-04-12)

## Cross-Milestone Trends

| Metric      | v1.0  | v1.1  |
|-------------|-------|-------|
| Phases      | 8     | 4     |
| Plans       | 37    | 9     |
| LOC (net)   | 5,491 | +4,593 |
| Days        | 34    | 8     |
| Quick Tasks | 10    | 8     |
