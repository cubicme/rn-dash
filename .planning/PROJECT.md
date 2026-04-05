# UMP Dashboard

## What This Is

A Rust/Ratatui terminal UI dashboard for managing React Native worktrees in the UMP project. It provides a dedicated tmux window with a unified view of the currently running metro instance, all worktrees with their git/JIRA context, and quick access to git operations, RN commands, and Claude Code agents. Built for a Doom Emacs user comfortable with vim bindings — keyboard-driven with on-screen hints.

## Core Value

One place to see and control everything about UMP worktrees — which one is running, what branch each is on, and execute any command without context-switching.

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

(Fresh for next milestone)

### Out of Scope

- Mobile app or web UI — this is a terminal dashboard only
- Building or modifying the UMP React Native app itself — this tool manages it
- Real-time JIRA sync or ticket creation — read-only ticket title fetching
- Multi-user support — single-user tool

## Context

Shipped v1.0 with 5,491 LOC Rust across 207 commits in 34 days.
Tech stack: Rust + Ratatui 0.30, tokio async runtime, crossterm, reqwest for JIRA.
Architecture: TEA (The Elm Architecture) with domain/infra/app/ui separation.

- Main repo at ~/aljazeera/ump with multiple git worktrees
- Only one metro bundler can run at a time across all worktrees (enforced)
- User works in tmux or zellij, dedicating one window to this dashboard
- Branch naming convention: UMP-XXXX-description (maps to JIRA tickets)
- 5-palette submenu keybinding scheme (a/i/x/s/g) with vim-style navigation
- Per-worktree command output persistence, FIFO command queue
- External metro conflict detection via port 8081 lsof

## Constraints

- **Tech stack**: Rust + Ratatui — no exceptions
- **Architecture**: Domain logic completely separated from UI and system concerns, following "A Philosophy of Software Design" by John Ousterhout
- **Environment**: macOS, runs inside tmux
- **Config location**: ~/.config/ump-dash/ for JIRA token, worktree labels, preferences

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

---
*Last updated: 2026-04-05 after v1.0 milestone*
