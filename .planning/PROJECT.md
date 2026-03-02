# UMP Dashboard

## What This Is

A Rust/Ratatui terminal UI dashboard for managing React Native worktrees in the UMP project. It provides a dedicated tmux window with a unified view of the currently running metro instance, all worktrees with their git/JIRA context, and quick access to git operations, RN commands, and Claude Code agents. Built for a Doom Emacs user comfortable with vim bindings — keyboard-driven with on-screen hints.

## Core Value

One place to see and control everything about UMP worktrees — which one is running, what branch each is on, and execute any command without context-switching.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Running instance zone with metro status, log toggle, debugger/reload/restart controls
- [ ] Worktree browser showing all worktrees with branch name, JIRA ticket title, and optional custom labels
- [ ] JIRA integration via API token to auto-fetch ticket titles from branch names (UMP-XXXX pattern)
- [ ] Git operations per worktree: reset --hard origin, pull, push, rebase, checkout, checkout -b
- [ ] RN commands: clean (android/cocoapods), rm node_modules, yarn install, yarn start --reset-cache, yarn pod-install
- [ ] RN run commands: run-android (with device list), run-ios (device/simulator selection)
- [ ] Metro interaction: open debugger (j), reload (r), kill and restart with --reset-cache
- [ ] Testing/quality commands: yarn unit-tests, yarn jest [filter], yarn lint --quiet --fix, yarn check-types
- [ ] Dependency staleness detection with hints, lazy auto-install before app launch
- [ ] Worktree switching: kill metro in current worktree, auto-start in new one
- [ ] Launch Claude Code in new tmux tab at a selected worktree
- [ ] Custom labels per worktree/branch that override or accompany JIRA title
- [ ] Vim-style keybindings with on-screen key hints
- [ ] Only one metro instance running at a time across all worktrees
- [ ] Command options/flags visible and configurable when executing commands

### Out of Scope

- Mobile app or web UI — this is a terminal dashboard only
- Building or modifying the UMP React Native app itself — this tool manages it
- Real-time JIRA sync or ticket creation — read-only ticket title fetching
- Multi-user support — single-user tool

## Context

- Main repo at ~/aljazeera/ump with multiple git worktrees
- Worktree paths based on original branch names but can contain any content
- Only one metro bundler can run at a time across all worktrees
- User works in tmux, dedicating one window to this dashboard
- Branch naming convention: UMP-XXXX-description (maps to JIRA tickets)
- Existing workflow involves ad-hoc tmux windows and manual command execution
- Claude Code agents are run at worktrees for development work

## Constraints

- **Tech stack**: Rust + Ratatui — no exceptions
- **Architecture**: Domain logic completely separated from UI and system concerns, following "A Philosophy of Software Design" by John Ousterhout
- **Environment**: macOS, runs inside tmux
- **Config location**: ~/.config/ump-dash/ for JIRA token, worktree labels, preferences

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Ratatui for TUI | User preference, performance, type safety | — Pending |
| Domain/UI/system separation | Ousterhout philosophy, testability, clarity | — Pending |
| Kill + restart on worktree switch | Only one metro allowed, minimize manual steps | — Pending |
| Lazy dependency install before run | Don't block workflow, but ensure app is ready when needed | — Pending |
| JIRA API with config token | Auto-fetch ticket titles for branch context | — Pending |
| ~/.config/ump-dash/ for config | XDG-style, separate from repo | — Pending |
| Tmux integration for Claude Code | Open agent in new tab at worktree directory | — Pending |

---
*Last updated: 2026-03-02 after initialization*
