# Requirements: UMP Dashboard

**Defined:** 2026-03-02
**Core Value:** One place to see and control everything about UMP worktrees — which one is running, what branch each is on, and execute any command without context-switching.

## v1 Requirements

### Architecture

- [x] **ARCH-01**: Domain logic (worktree model, metro state machine, command dependencies, staleness rules) is pure Rust with zero dependencies on UI or system crates
- [x] **ARCH-02**: Infrastructure layer (process spawning, git operations, JIRA HTTP, tmux interaction, file I/O) is behind trait boundaries so implementations can be swapped or tested
- [x] **ARCH-03**: UI layer (ratatui widgets, rendering, layout) depends on domain types but never on infrastructure directly
- [x] **ARCH-04**: Application layer uses The Elm Architecture (TEA): AppState (model) → Action enum (update) → View functions (render)
- [x] **ARCH-05**: Code follows "A Philosophy of Software Design" by John Ousterhout — deep modules with simple interfaces, minimize shallow abstractions
- [x] **ARCH-06**: Domain invariants (e.g., "only one metro at a time") are enforced in domain types, not scattered across UI or infra code

### TUI Shell

- [x] **SHELL-01**: User can navigate the dashboard using vim-style keybindings (hjkl, q, /, ?)
- [x] **SHELL-02**: User sees context-sensitive keybinding hints in a footer bar that update per active panel/mode
- [x] **SHELL-03**: User can move focus between panels using Tab/Shift-Tab or arrow keys
- [x] **SHELL-04**: User can open a help overlay (? or F1) listing all available keybindings
- [x] **SHELL-05**: User sees error states clearly when commands fail (non-zero exit, with retry/dismiss)

### Metro

- [x] **METRO-01**: User can see at a glance which worktree (if any) has metro running, with status indicator (running/stopped)
- [x] **METRO-02**: User can start metro (yarn start --reset-cache) from the active worktree
- [x] **METRO-03**: User can stop the running metro instance
- [x] **METRO-04**: User can restart metro (kill + start) with one keystroke
- [x] **METRO-05**: User can view metro log output in a dedicated panel only when a log filter is applied (metro does not stream logs by default)
- [x] **METRO-06**: User can scroll through metro log history in the log panel
- [x] **METRO-07**: User can send debugger command (j) to the running metro instance
- [x] **METRO-08**: User can send reload command (r) to the running metro instance
- [x] **METRO-09**: Only one metro instance can run at a time across all worktrees (enforced by the dashboard)

### Worktree Management

- [x] **WORK-01**: User sees a list of all worktrees with their current branch name
- [x] **WORK-02**: User sees the JIRA ticket title next to the branch name (fetched via API from UMP-XXXX pattern)
- [x] **WORK-03**: User can set a custom label on a branch that persists across worktrees (label follows the branch, not the worktree)
- [x] **WORK-04**: User can switch the "running" worktree which auto-kills metro in current and starts it in the new one
- [x] **WORK-05**: User sees dependency staleness hints when node_modules is outdated relative to package.json/yarn.lock
- [x] **WORK-06**: Stale dependencies are lazily installed before launching the app if user hasn't manually installed

### Git Operations

- [x] **GIT-01**: User can run git reset --hard origin/<current-branch> on a selected worktree
- [x] **GIT-02**: User can run git pull on a selected worktree
- [x] **GIT-03**: User can run git push on a selected worktree
- [x] **GIT-04**: User can run git rebase origin/<target-branch> on a selected worktree
- [x] **GIT-05**: User can run git checkout <branch> to switch branches in a worktree
- [x] **GIT-06**: User can run git checkout -b <branch> to create and switch to a new branch

### RN Commands

- [x] **RN-01**: User can run npx react-native clean --include 'android' on a selected worktree
- [x] **RN-02**: User can run npx react-native clean --include 'cocoapods' on a selected worktree
- [x] **RN-03**: User can run rm -rf node_modules on a selected worktree
- [x] **RN-04**: User can run yarn install on a selected worktree
- [x] **RN-05**: User can run yarn pod-install on a selected worktree
- [x] **RN-06**: User can run npx react-native run-android on a selected worktree with device selection (from adb devices list)
- [x] **RN-07**: User can run yarn react-native run-ios on a selected worktree with device/simulator selection
- [x] **RN-08**: User can run yarn unit-tests on a selected worktree
- [x] **RN-09**: User can run yarn jest with a test filter on a selected worktree
- [x] **RN-10**: User can run yarn lint --quiet --fix on a selected worktree
- [x] **RN-11**: User can run yarn check-types --incremental on a selected worktree
- [x] **RN-12**: User sees streaming command output in a panel while commands execute

### Integrations

- [x] **INTG-01**: Dashboard reads JIRA API token from ~/.config/ump-dash/ config
- [x] **INTG-02**: Dashboard fetches JIRA ticket titles by extracting UMP-XXXX from branch names and querying the JIRA REST API
- [x] **INTG-03**: Fetched JIRA titles are cached locally to avoid redundant API calls
- [x] **INTG-04**: User can launch Claude Code in a new tmux tab at a selected worktree's directory
- [x] **INTG-05**: Dashboard detects it is running inside tmux for tmux-dependent features

## v2 Requirements

### Enhanced UX

- **UX-01**: Confirmation dialog before destructive actions (git reset --hard, rm node_modules)
- **UX-02**: Command flags configuration modal before execution (toggle --reset-cache, --fix, etc.)
- **UX-03**: Basic mouse scrolling support in log panels

### CI/CD

- **CICD-01**: Show CI/CD pipeline status per worktree (GitHub Actions)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full git log/diff viewer | lazygit does this better — offer keybinding to open lazygit instead |
| Real-time JIRA write-back | Scope creep — read-only title fetch is sufficient |
| Multi-user / team sync | Single-user tool by design |
| Plugin/extension system | Focused tool — hardcode RN commands, config file for customization |
| Built-in terminal emulator | Would become tmux replacement — use tmux new-window for ad-hoc shells |
| Mobile or web UI | Terminal dashboard only |
| Mouse-first interaction | Keyboard-first for vim/Doom Emacs user |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ARCH-01 | Phase 1 | Complete |
| ARCH-02 | Phase 1 | Complete |
| ARCH-03 | Phase 1 | Complete |
| ARCH-04 | Phase 1 | Complete |
| ARCH-05 | Phase 1 | Complete |
| ARCH-06 | Phase 1 | Complete |
| SHELL-01 | Phase 1 | Complete |
| SHELL-02 | Phase 1 | Complete |
| SHELL-03 | Phase 1 | Complete |
| SHELL-04 | Phase 1 | Complete |
| SHELL-05 | Phase 1 | Complete |
| METRO-01 | Phase 2 | Complete |
| METRO-02 | Phase 2 | Complete |
| METRO-03 | Phase 2 | Complete |
| METRO-04 | Phase 2 | Complete |
| METRO-05 | Phase 2 | Complete |
| METRO-06 | Phase 2 | Complete |
| METRO-07 | Phase 2 | Complete |
| METRO-08 | Phase 2 | Complete |
| METRO-09 | Phase 2 | Complete |
| WORK-01 | Phase 3 | Complete |
| WORK-02 | Phase 3 | Complete |
| WORK-03 | Phase 3 | Complete |
| WORK-04 | Phase 5 | Complete |
| WORK-05 | Phase 3 | Complete |
| WORK-06 | Phase 3 | Complete |
| GIT-01 | Phase 3 | Complete |
| GIT-02 | Phase 3 | Complete |
| GIT-03 | Phase 3 | Complete |
| GIT-04 | Phase 3 | Complete |
| GIT-05 | Phase 3 | Complete |
| GIT-06 | Phase 3 | Complete |
| RN-01 | Phase 3 | Complete |
| RN-02 | Phase 3 | Complete |
| RN-03 | Phase 3 | Complete |
| RN-04 | Phase 3 | Complete |
| RN-05 | Phase 3 | Complete |
| RN-06 | Phase 3 | Complete |
| RN-07 | Phase 3 | Complete |
| RN-08 | Phase 3 | Complete |
| RN-09 | Phase 3 | Complete |
| RN-10 | Phase 3 | Complete |
| RN-11 | Phase 3 | Complete |
| RN-12 | Phase 3 | Complete |
| INTG-01 | Phase 4 | Complete |
| INTG-02 | Phase 4 | Complete |
| INTG-03 | Phase 4 | Complete |
| INTG-04 | Phase 5 | Complete |
| INTG-05 | Phase 4 | Complete |

**Coverage:**
- v1 requirements: 45 total
- Mapped to phases: 45
- Unmapped: 0

---
*Requirements defined: 2026-03-02*
*Last updated: 2026-03-02 — phase traceability complete after roadmap creation*
