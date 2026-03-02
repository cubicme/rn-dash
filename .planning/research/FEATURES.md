# Feature Research

**Domain:** TUI developer workspace/process manager (Rust/Ratatui, React Native worktrees)
**Researched:** 2026-03-02
**Confidence:** HIGH (core TUI patterns), MEDIUM (RN-specific workflow), HIGH (project-specific requirements from PROJECT.md)

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist in any TUI workspace/process manager. Missing these makes the tool feel broken or amateur.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Vim-style keybindings (hjkl, q, /, ?) | Every power-user TUI (lazygit, k9s, gitui) uses these. Doom Emacs user will expect them. | LOW | Must be keyboard-only. Mouse optional. |
| On-screen keybinding hints (footer bar) | lazygit, k9s, gitui all show context-sensitive shortcuts in a footer. Users don't memorize; they discover. | LOW | Context-aware: hints change per selected panel/mode. |
| Panel/pane layout with focus traversal | Split-pane layout is the standard TUI pattern (gitui, lazygit, k9s all use it). Tab or arrow to move between panels. | MEDIUM | Tab or shift-Tab between panels; enter to dive in. |
| Real-time status display | k9s pioneered auto-refresh. Users expect live process state, not stale snapshots. | MEDIUM | Tick-based polling for process status; async updates. |
| List view with selection highlight | Standard widget in every TUI. Arrow keys navigate; selected item is highlighted. | LOW | Ratatui's List widget covers this. |
| Scrollable log output | Any tool that exposes logs must allow scrolling. Users need to read past output. | LOW | Ratatui Paragraph with scroll offset. |
| ? or F1 for help overlay | gitui uses context-aware help; k9s uses ?; lazygit uses x for custom commands and ? for general help. | LOW | Full-screen or floating overlay listing all bindings. |
| Escape to cancel/go back | Universal TUI convention. Cancel modals, deselect, return to previous state. | LOW | Applies to modals, confirmations, command input. |
| Confirmation dialog for destructive actions | git reset --hard, rm node_modules — these are irreversible. lazygit prompts before destructive ops. | LOW | Simple y/N prompt or dedicated confirm modal. |
| Command output visible while running | Running yarn install blind is frustrating. Users expect to see streaming output. | MEDIUM | Spawn subprocess, stream stdout/stderr to a panel. |
| Worktree list as primary navigation surface | This is a worktree manager — the list of worktrees is the core of the tool. | MEDIUM | Sortable, filterable list with branch/ticket metadata. |
| Git branch name visible per worktree | Minimum context. Users need to know what branch each worktree represents. | LOW | Read from git; display next to path. |
| Process running/stopped status indicator | Only one metro allowed. Users must know at a glance which worktree (if any) has metro running. | LOW | Boolean indicator: running / stopped. Simple icon or color. |
| Error state visibility | When a command fails, the user must know. Silent failures are bugs. | LOW | Non-zero exit code → show error, offer retry/dismiss. |

### Differentiators (Competitive Advantage)

Features that distinguish this tool from generic TUI workspace managers. These map directly to the project's core value: "one place to see and control everything about UMP worktrees."

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| JIRA ticket title alongside branch name | Branch names like UMP-1234-fix-crash are opaque. Showing "Fix crash on Android login" gives instant context without leaving the terminal. | MEDIUM | Fetch from JIRA REST API v3 using API token. Cache locally (in-memory or ~/.config/ump-dash/). Extract UMP-XXXX from branch with regex. |
| Single-metro enforcement with auto-switch | No other tool enforces the "only one bundler" constraint. Auto-killing metro when switching worktrees removes a daily friction point. | HIGH | Track which worktree owns the running metro process. On switch: SIGTERM metro, verify dead, start in new worktree. Race condition risk if process doesn't die cleanly. |
| Custom labels per worktree/branch | JIRA titles are sometimes uninformative. Custom labels let users annotate worktrees (e.g., "QA build", "blocked by backend"). | LOW | Store in ~/.config/ump-dash/labels.toml. Display alongside JIRA title. |
| Dependency staleness detection with lazy install | Automatically detect when node_modules is stale (package.json mtime vs node_modules mtime). Offer to install before launching. No manual "did I forget to yarn install?" | MEDIUM | Compare file modification times. Present warning badge. Lazy-install before run-android/run-ios, not proactively. |
| Metro log toggle (live tail, show/hide) | Metro logs are noisy. Being able to show/hide them in-panel without killing the process is a workflow upgrade. | MEDIUM | Attach to the metro process stdout/stderr pipe. Toggle visibility. Separate from the "running" state. |
| Metro interactive commands (j/r/reload) | Metro responds to keyboard input (j for debugger, r for reload). Wrapping those inside the dashboard means no terminal-switching. | MEDIUM | Send keystrokes to the metro subprocess's stdin. Requires process stdin pipe. |
| Device/simulator selection for run-android/run-ios | Picking the right device is a manual step in every RN workflow. Surfacing adb devices and xcrun simctl list inside the TUI eliminates CLI-switching. | HIGH | Run adb devices and xcrun simctl list instruments -s simulators, parse output, present selection list. |
| Launch Claude Code in tmux tab at worktree | Developer workflow: pick worktree, launch agent. No other tool integrates AI agent spawning. | MEDIUM | tmux new-window -c <worktree-path> 'claude'. Requires tmux session detection. |
| RN-specific command palette (clean, pod-install, lint, etc.) | Generic TUI workspace tools have no RN awareness. Exposing yarn pod-install, yarn check-types, clean per-platform makes this indispensable for the RN team. | MEDIUM | Configurable command list with per-worktree execution. Output streamed to panel. |
| Command flags configuration before execution | Power users want --reset-cache, --fix, --quiet exposed. lazygit exposes options before running some commands. | MEDIUM | Pre-execution modal: show command, allow flag toggling, confirm. |
| Worktree switching orchestration | When switching active worktree: kill metro, optionally run yarn install, start metro. One-keystroke workflow. | HIGH | Sequence of async operations with visible progress. Rollback on failure. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Mouse support as primary interaction | GUIs feel more approachable. | TUI tools targeting Doom Emacs/vim users should be keyboard-first. Mouse support adds complexity for minimal gain given the audience. | Keyboard-only with discoverable shortcuts. Add basic mouse scrolling only if trivial. |
| Real-time JIRA sync / ticket write-back | "Update ticket status from the dashboard" sounds convenient. | Scope creep. JIRA write requires OAuth scopes, error handling, conflict resolution. Read-only title fetch is already sufficient and simple. | Read-only JIRA ticket title via API token. |
| Full git log / diff viewer | lazygit does this better. Users will reach for lazygit for deep git operations. | Duplicates lazygit. Adds significant complexity (diff rendering, hunk navigation). | Offer a single keybinding to open lazygit at the worktree instead. |
| Multi-user / team sync | "Share which worktree is being worked on" sounds useful. | Single-user tool by definition. Network sync introduces auth, conflicts, reliability concerns. | Out of scope. PROJECT.md explicitly excludes multi-user support. |
| Plugin/extension system | Power users want customization. | Massively increases scope and maintenance burden. Not needed for a focused single-team tool. | Hardcode the RN command palette. Add config file for simple customization (labels, commands). |
| Built-in terminal emulator / shell | Running arbitrary commands is tempting to expose. | This becomes a tmux replacement. Scope explosion. | Use tmux new-window for ad-hoc shells. The dashboard executes specific commands only. |
| Background auto-refresh of all JIRA titles on startup | Feels like a nice UX touch. | JIRA rate limits API calls. Cold-start latency. Refreshing all worktrees on launch could be slow or fail. | Fetch on demand (when worktree is focused) with in-memory cache. Configurable TTL. |
| CI/CD status per worktree | lazyworktree does this via GitHub Actions integration. | Requires GitHub API auth (different from JIRA), polling, and rate limiting. Adds two auth systems. | Defer to v2 if the team requests it. Not blocking core value. |

## Feature Dependencies

```
[Worktree List Panel]
    └──requires──> [Git Process Layer] (read branch names)
    └──requires──> [Config Storage] (~/.config/ump-dash/)
    └──enhances──> [JIRA Title Fetcher] (add ticket context)
    └──enhances──> [Custom Labels] (add user annotations)

[Metro Process Control]
    └──requires──> [Process Spawner/Monitor] (spawn, kill, detect status)
    └──requires──> [Worktree List Panel] (know which worktree is active)
    └──enhances──> [Log Panel] (stream metro output)
    └──enhances──> [Metro Interactive Commands] (stdin pipe to metro)

[RN Command Palette]
    └──requires──> [Worktree List Panel] (know target worktree)
    └──requires──> [Command Runner] (subprocess execution with output streaming)
    └──enhances──> [Device Selector] (for run-android / run-ios)
    └──enhances──> [Dependency Staleness Detector] (pre-run check)

[Worktree Switching Orchestration]
    └──requires──> [Metro Process Control] (kill current metro)
    └──requires──> [Worktree List Panel] (select target)
    └──requires──> [Command Runner] (start metro in new worktree)
    └──enhances──> [Dependency Staleness Detector] (auto-install before start)

[JIRA Title Fetcher]
    └──requires──> [Config Storage] (API token, cache)

[Device Selector]
    └──requires──> [Command Runner] (adb devices, xcrun simctl)
    └──enhances──> [RN Command Palette] (run-android, run-ios)

[Launch Claude Code]
    └──requires──> [tmux Integration] (new-window with cwd)
    └──requires──> [Worktree List Panel] (target directory)

[Keybinding Hints Footer]
    └──enhances──> all panels (contextual hints per active panel)

[Confirmation Dialog]
    └──requires──> [Modal System]
    └──enhances──> [Git Operations] (before reset --hard)
    └──enhances──> [Metro Process Control] (before kill)

[Metro Interactive Commands (j/r/reload)]
    └──requires──> [Metro Process Control] (must be running)
    └──requires──> [Process stdin pipe] (send keystrokes)
```

### Dependency Notes

- **Metro Process Control requires Process Spawner/Monitor:** Everything that talks to metro (logs, interactive commands, status) is downstream of actually owning the process handle. Build the process layer first.
- **Worktree Switching requires Metro Process Control:** Switching must kill-then-start. Metro control must be stable before orchestration is safe to build.
- **Device Selector enhances RN Command Palette:** Device selection is only needed when running the app. It's a popup within the run command flow, not a separate panel.
- **JIRA Fetcher is isolated:** It only needs an HTTP client and the config token. It doesn't depend on metro or git beyond reading the branch name.
- **Custom Labels conflict with nothing:** Pure config read — zero dependencies on live processes.

## MVP Definition

### Launch With (v1)

Minimum viable product — what proves the concept and replaces the current ad-hoc tmux workflow.

- [ ] Worktree list panel showing path + branch name — core navigation surface
- [ ] Metro status indicator (running/stopped, which worktree) — the critical constraint made visible
- [ ] Metro start/stop/restart controls (from the active worktree) — core action
- [ ] Metro log panel with toggle (show/hide, scroll) — live feedback
- [ ] RN command palette: clean, rm node_modules, yarn install, yarn start --reset-cache — most-used commands
- [ ] Git operations per worktree: reset --hard origin, pull, push — the daily git operations
- [ ] Vim-style keybindings throughout — non-negotiable for the user
- [ ] On-screen keybinding hints in footer — discovery mechanism
- [ ] Confirmation dialog for destructive actions — safety gate
- [ ] Command output streaming in panel — visibility into what commands are doing

### Add After Validation (v1.x)

Features to add once the core workflow is proven.

- [ ] JIRA ticket title fetching from branch name — add once the list is stable; requires HTTP/config
- [ ] Custom labels per worktree — simple config enhancement after list is working
- [ ] Worktree switching orchestration (kill + restart) — build after metro control is solid
- [ ] Dependency staleness detection — low risk enhancement once command runner is stable
- [ ] run-android / run-ios with device selection — more complex; needs device listing subprocess
- [ ] Metro interactive commands (j/r) — nice UX polish; needs stdin pipe to metro
- [ ] Launch Claude Code in tmux tab — useful but not blocking core value

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] CI/CD status per worktree (GitHub Actions) — requires second auth system; not in core scope
- [ ] Command flags configuration modal before execution — polish feature, adds UI complexity
- [ ] yarn jest [filter] / yarn lint in palette — add when testing workflow is validated as pain point
- [ ] yarn pod-install / run-ios simulator selection — iOS workflow; add when Android path is stable

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Worktree list panel | HIGH | MEDIUM | P1 |
| Metro status indicator | HIGH | LOW | P1 |
| Metro start/stop/restart | HIGH | MEDIUM | P1 |
| Metro log panel (toggle + scroll) | HIGH | MEDIUM | P1 |
| Vim keybindings + footer hints | HIGH | LOW | P1 |
| Confirmation dialog | HIGH | LOW | P1 |
| Command output streaming | HIGH | MEDIUM | P1 |
| Git operations (reset, pull, push) | HIGH | MEDIUM | P1 |
| RN command palette (core) | HIGH | MEDIUM | P1 |
| JIRA title fetching | MEDIUM | MEDIUM | P2 |
| Custom labels | MEDIUM | LOW | P2 |
| Worktree switching orchestration | HIGH | HIGH | P2 |
| Dependency staleness detection | MEDIUM | MEDIUM | P2 |
| Device selector (run-android/ios) | MEDIUM | HIGH | P2 |
| Metro interactive commands (j/r) | MEDIUM | MEDIUM | P2 |
| Launch Claude Code in tmux | MEDIUM | MEDIUM | P2 |
| Command flags configuration modal | LOW | MEDIUM | P3 |
| CI/CD status per worktree | LOW | HIGH | P3 |
| yarn test / jest filter | LOW | LOW | P3 |

**Priority key:**
- P1: Must have for launch (v1)
- P2: Should have, add after core is stable (v1.x)
- P3: Nice to have, future consideration (v2+)

## Competitor Feature Analysis

| Feature | lazygit | gitui | k9s | lazyworktree | UMP Dashboard |
|---------|---------|-------|-----|--------------|---------------|
| Panel-based layout | YES | YES | YES | YES | YES — planned |
| Vim keybindings | YES | YES | YES | YES | YES — required |
| On-screen key hints | YES (footer) | YES (context panel) | YES (header) | YES | YES — footer bar |
| Real-time status | Partial | Partial | YES (auto-refresh) | Partial | YES — metro status |
| Process management | NO | NO | YES (pods) | NO | YES — metro |
| Log streaming | NO | NO | YES (pod logs) | NO | YES — metro logs |
| Git operations | YES (deep) | YES (core) | NO | YES (basic) | YES (subset) |
| Worktree management | YES (basic) | NO | NO | YES (primary) | YES (primary) |
| External API integration | NO | NO | NO | YES (GitHub) | YES (JIRA) |
| Custom labels/notes | NO | NO | NO | YES (markdown notes) | YES (simple labels) |
| Device selection | NO | NO | NO | NO | YES — differentiator |
| tmux integration | NO | NO | NO | YES (sessions) | YES (Claude Code) |
| Single-process enforcement | NO | NO | YES (k8s constraint) | NO | YES — metro singleton |
| Domain-specific commands | NO | NO | YES (k8s ops) | NO | YES — RN commands |

**Key insight from competitor analysis:** lazygit and gitui are git-only tools that don't touch processes. k9s is the closest analogue — it manages running processes (pods) in real-time, shows logs, and enforces cluster constraints. The UMP dashboard is effectively "k9s for React Native worktrees" with git and JIRA integration added. lazyworktree is the closest feature-wise on the worktree side but doesn't manage processes at all.

## Sources

- [lazygit GitHub — keybindings and worktree features](https://github.com/jesseduffield/lazygit)
- [lazygit Keybindings Reference](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
- [gitui GitHub — panel architecture and design decisions](https://github.com/gitui-org/gitui)
- [k9s official site — feature overview](https://k9scli.io/)
- [lazyworktree GitHub — worktree TUI patterns](https://github.com/chmouel/lazyworktree)
- [lazyworktree HN discussion](https://news.ycombinator.com/item?id=46474066)
- [Prox process manager TUI patterns](https://github.com/craigderington/prox)
- [Ratatui popup/dialog patterns](https://ratatui.rs/examples/apps/popup/)
- [awesome-ratatui — TUI app examples](https://github.com/ratatui/awesome-ratatui)
- [Essential CLI/TUI tools analysis](https://itnext.io/essential-cli-tui-tools-for-developers-7e78f0cd27db)
- [bottom (btm) — process monitoring TUI](https://github.com/ClementTsang/bottom)
- [JIRA Cloud REST API docs](https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/)
- [Nerdlog — log streaming TUI patterns](https://github.com/dimonomid/nerdlog)
- [PROJECT.md — project requirements and constraints](/Users/cubicme/aljazeera/dashboard/.planning/PROJECT.md)

---
*Feature research for: Rust/Ratatui TUI dashboard for React Native worktree management (UMP Dashboard)*
*Researched: 2026-03-02*
