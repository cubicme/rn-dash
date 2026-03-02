# Roadmap: UMP Dashboard

## Overview

Five phases build the UMP Dashboard from a bare terminal skeleton to a fully orchestrated React Native worktree manager. Phase 1 establishes the terminal lifecycle and event architecture that every later phase inherits. Phase 2 adds the core differentiating feature — metro process control with the single-instance invariant. Phase 3 delivers the full worktree browser, git operations, and RN command palette, making the tool usable for day-to-day workflow. Phase 4 adds JIRA context and config storage so branches show ticket titles. Phase 5 wires worktree switching orchestration and Claude Code tmux integration as the final polish layer.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Scaffold and TUI Shell** - Running ratatui app with correct terminal lifecycle, event loop, and vim-style keybinding layer
- [ ] **Phase 2: Metro Process Control** - Metro start/stop/restart/log with the single-instance invariant enforced in domain
- [ ] **Phase 3: Worktree Browser, Git, and RN Commands** - Full worktree list, all git operations, and complete RN command palette with output streaming
- [ ] **Phase 4: Config and JIRA Integration** - Config store at ~/.config/ump-dash/, JIRA title fetching with caching, and graceful degradation
- [ ] **Phase 5: Worktree Switching and Claude Code** - One-keystroke worktree switching orchestration and Claude Code tmux tab launch

## Phase Details

### Phase 1: Scaffold and TUI Shell
**Goal**: A running ratatui application with correct terminal init/restore, panic recovery, async event loop, and the full vim-style keybinding layer — the foundation every later phase inherits
**Depends on**: Nothing (first phase)
**Requirements**: ARCH-01, ARCH-02, ARCH-03, ARCH-04, ARCH-05, ARCH-06, SHELL-01, SHELL-02, SHELL-03, SHELL-04, SHELL-05
**Success Criteria** (what must be TRUE):
  1. User can launch the dashboard and the terminal is fully restored (raw mode off, cursor visible) on exit, crash, or panic — no broken shell
  2. User can navigate between panels using hjkl, Tab, and Shift-Tab, and the footer updates to show context-sensitive keybinding hints for whichever panel is focused
  3. User can open a help overlay (? or F1) that lists all available keybindings, and dismiss it with q or Escape
  4. User sees a clear error state with retry and dismiss options when any operation fails with a non-zero exit code
  5. App compiles without warnings, domain logic has no direct dependency on ratatui or process crates (verifiable via cargo tree), and there is exactly one crossterm version in the dependency graph
**Plans**: TBD

### Phase 2: Metro Process Control
**Goal**: Users can start, stop, restart, and monitor metro with guaranteed single-instance enforcement — the zombie-process and port-binding bugs are addressed before any downstream features depend on this layer
**Depends on**: Phase 1
**Requirements**: METRO-01, METRO-02, METRO-03, METRO-04, METRO-05, METRO-06, METRO-07, METRO-08, METRO-09
**Success Criteria** (what must be TRUE):
  1. User can see at a glance which worktree (if any) has metro running with a status indicator (running/stopped), and the indicator accurately reflects reality even if metro was killed outside the dashboard
  2. User can start metro (yarn start --reset-cache) from the focused worktree with one keystroke, and starting metro when another instance is already running automatically kills the existing one first
  3. User can stop and restart metro with single keystrokes, and after stop the port 8081 is verified free before the UI shows status as stopped
  4. User can toggle a log panel that shows metro output only when filtered, can scroll through log history, and can send debugger (j) and reload (r) commands to the running metro instance
**Plans**: TBD

### Phase 3: Worktree Browser, Git, and RN Commands
**Goal**: Users can see all worktrees in a browsable list, run any git operation or RN command on a selected worktree, and watch streaming output — completing the core daily-driver workflow
**Depends on**: Phase 2
**Requirements**: WORK-01, WORK-02, WORK-03, WORK-05, WORK-06, GIT-01, GIT-02, GIT-03, GIT-04, GIT-05, GIT-06, RN-01, RN-02, RN-03, RN-04, RN-05, RN-06, RN-07, RN-08, RN-09, RN-10, RN-11, RN-12
**Success Criteria** (what must be TRUE):
  1. User sees a list of all worktrees with branch name, metro status badge, JIRA ticket title placeholder (or branch name if not yet fetched), and any custom label — all populated from disk without any network call
  2. User can set a custom label on a worktree that persists across sessions and follows the branch (not the worktree path)
  3. User can run any git operation (reset --hard, pull, push, rebase, checkout, checkout -b) on a selected worktree and watch streaming output in a panel; destructive operations (reset --hard) show a confirmation prompt before executing
  4. User can run any RN command (clean android, clean cocoapods, rm node_modules, yarn install, pod-install, run-android with device selection, run-ios with device/simulator selection, unit-tests, jest with filter, lint, check-types) on a selected worktree with streaming output
  5. Dashboard shows a staleness hint when node_modules appears outdated relative to package.json or yarn.lock, and lazily installs dependencies before launching the app if the user has not done so manually
**Plans**: TBD

### Phase 4: Config and JIRA Integration
**Goal**: Users see JIRA ticket titles next to branch names, the JIRA API token is stored securely with correct file permissions, and the dashboard degrades gracefully when JIRA is unreachable
**Depends on**: Phase 3
**Requirements**: INTG-01, INTG-02, INTG-03, INTG-05
**Success Criteria** (what must be TRUE):
  1. User can place a JIRA API token in ~/.config/ump-dash/ (config file has 0600 permissions on first write) and the dashboard reads it without any other configuration needed
  2. User sees JIRA ticket titles automatically appear next to branch names for any branch matching the UMP-XXXX pattern — titles load in the background without blocking dashboard startup
  3. Fetched JIRA titles persist to a local cache so they appear on next launch without a network call; when JIRA is unreachable the branch name is shown in place of the title with no error shown to the user
  4. Dashboard correctly detects it is running inside tmux and gates tmux-dependent features (tab creation) on that detection
**Plans**: TBD

### Phase 5: Worktree Switching and Claude Code
**Goal**: Users can switch the active worktree with one keystroke triggering full metro orchestration, and can open Claude Code in a new tmux tab at any worktree directory
**Depends on**: Phase 4
**Requirements**: WORK-04, INTG-04
**Success Criteria** (what must be TRUE):
  1. User can switch the "running" worktree with one keystroke and the dashboard automatically kills metro in the current worktree, waits for port 8081 to free, and starts metro in the newly selected worktree — progress is visible during the transition
  2. User can open Claude Code in a new tmux tab at a selected worktree's directory with one keystroke; the tab opens with claude as the initial shell command (not via send-keys) so there is no race condition on shell initialization
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Scaffold and TUI Shell | 0/TBD | Not started | - |
| 2. Metro Process Control | 0/TBD | Not started | - |
| 3. Worktree Browser, Git, and RN Commands | 0/TBD | Not started | - |
| 4. Config and JIRA Integration | 0/TBD | Not started | - |
| 5. Worktree Switching and Claude Code | 0/TBD | Not started | - |
