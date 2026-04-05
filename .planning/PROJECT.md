# RN Dash

## What This Is

A Rust/Ratatui terminal UI dashboard for managing React Native worktrees. It provides a unified view of the currently running metro instance, all worktrees with their git/JIRA context, and quick access to git operations, RN commands, and Claude Code agents. Keyboard-driven with vim bindings and dynamic on-screen hints. Configurable for any React Native monorepo with git worktrees.

## Core Value

One place to see and control everything about your React Native worktrees — which one is running, what branch each is on, and execute any command without context-switching.

## Requirements

### Validated

- ✓ Running instance zone with metro status, log toggle, debugger/reload/restart controls — v1.0
- ✓ Worktree browser showing all worktrees with branch name, JIRA ticket title, and optional custom labels — v1.0
- ✓ JIRA integration via API token to auto-fetch ticket titles from branch names (UMP-XXXX pattern) — v1.0
- ✓ Git operations per worktree: reset --hard origin, pull, push, rebase, checkout, checkout -b — v1.0
- ✓ RN commands: clean (android/cocoapods), rm node_modules, yarn install, yarn start --reset-cache, yarn pod-install — v1.0
- ✓ RN run commands: run-android (with device list), run-ios (device/simulator selection) — v1.0
- ✓ Metro interaction: open debugger (j), reload (r), kill and restart with --reset-cache — v1.0
- ✓ Testing/quality commands: yarn unit-tests, yarn jest [filter], yarn lint --quiet --fix, yarn check-types — v1.0
- ✓ Dependency staleness detection with hints, sync-before-run prompting — v1.0
- ✓ Worktree switching: kill metro in current worktree, auto-start in new one — v1.0
- ✓ Launch Claude Code in new tmux tab at a selected worktree — v1.0
- ✓ Custom labels per worktree/branch that override or accompany JIRA title — v1.0
- ✓ Vim-style keybindings with on-screen key hints — v1.0
- ✓ Only one metro instance running at a time across all worktrees — v1.0
- ✓ Command queue system with per-worktree output persistence — v1.0
- ✓ Multiplexer abstraction (tmux + zellij) — v1.0
- ✓ External metro conflict detection and resolution — v1.0
- ✓ Worktree creation and removal commands — v1.0
- ✓ Metro auto-prerequisite for RN run commands — v1.0

### Active

- [ ] Remove Labels feature entirely
- [ ] Rename (s)ync to (y)arn palette, move clean commands into it
- [ ] Move worktree commands from (g)it to (w)orktree palette
- [ ] New worktree creation command with base branch selection
- [ ] Metro keys (R/J) context-sensitive — only when metro running; ESC stops metro; remove restart
- [ ] Dynamic hint line derived from available actions, not hardcoded
- [ ] Update footer legend
- [ ] Extract all hardcoded AJ/UMP/system-specific values to config
- [ ] License file, README, config example, Cargo.toml metadata
- [ ] GitHub Actions CI + release workflow with prebuilt binaries
- [ ] .gitignore audit

### Out of Scope

- Mobile app or web UI — this is a terminal dashboard only
- Building or modifying the UMP React Native app itself — this tool manages it
- Real-time JIRA sync or ticket creation — read-only ticket title fetching
- Multi-user support — single-user tool

## Context

Shipped v1.0 with 5,491 LOC Rust across 207 commits in 34 days.
Tech stack: Rust + Ratatui 0.30, tokio async runtime, crossterm, reqwest for JIRA.
Architecture: TEA (The Elm Architecture) with domain/infra/app/ui separation.

- Designed for any React Native monorepo using git worktrees
- Only one metro bundler can run at a time across all worktrees (enforced)
- User works in tmux or zellij, dedicating one window to this dashboard
- Branch naming convention configurable (default: JIRA-style PROJ-XXXX)
- Palette submenu keybinding scheme (a/i/x/y/g/w) with vim-style navigation
- Per-worktree command output persistence, FIFO command queue
- External metro conflict detection via port 8081 lsof
- Preparing for public GitHub release in v1.1

## Constraints

- **Tech stack**: Rust + Ratatui — no exceptions
- **Architecture**: Domain logic completely separated from UI and system concerns, following "A Philosophy of Software Design" by John Ousterhout
- **Environment**: macOS (primary), Linux (CI)
- **Config location**: Configurable, default ~/.config/rn-dash/ for JIRA token, preferences

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Ratatui for TUI | User preference, performance, type safety | ✓ Good — 5.5k LOC, fast startup, zero runtime crashes |
| Domain/UI/system separation | Ousterhout philosophy, testability, clarity | ✓ Good — clean module boundaries, deep modules |
| Kill + restart on worktree switch | Only one metro allowed, minimize manual steps | ✓ Good — seamless one-keystroke switching |
| Sync-before-run prompting | User-visible prompt replaced lazy auto-install | ✓ Good — more transparent than silent install |
| JIRA API with config token | Auto-fetch ticket titles for branch context | ✓ Good — Basic/Bearer auth, cached locally |
| ~/.config/ump-dash/ for config | XDG-style, separate from repo | ✓ Good — 0600 permissions on credentials |
| Multiplexer abstraction (tmux + zellij) | Support multiple terminal multiplexers | ✓ Good — clean trait boundary |
| Command queue (VecDeque) | Chain dependent commands, show queue count | ✓ Good — enables fetch-then-reset, release build flows |
| External metro conflict detection | Detect port 8081 already in use | ✓ Good — lsof-based PID lookup with kill prompt |
| Metro as prerequisite for RN runs | Auto-start metro before build commands | ✓ Good — prevents RN from spawning unmanaged metro |

## Current Milestone: v1.1 Public Release

**Goal:** Polish keybinding scheme, remove dead features, extract hardcoded values to config, and prepare for public GitHub release with README, license, CI, and prebuilt binaries.

**Target features:**
- App polish: remove labels, rework palettes (y/w), dynamic hints, metro key context
- GitHub prep: license, README, config example, CI/CD, .gitignore, Cargo.toml metadata
- Generalization: extract all AJ/UMP-specific values to config

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-05 after v1.1 milestone start*
