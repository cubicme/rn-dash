# Project Research Summary

**Project:** UMP Dashboard — Rust TUI for React Native Worktree Management
**Domain:** Terminal UI developer workspace and process manager (Rust/Ratatui, React Native worktrees, tmux, JIRA)
**Researched:** 2026-03-02
**Confidence:** HIGH

## Executive Summary

The UMP Dashboard is a domain-specific TUI application that manages React Native worktrees, enforces the "one metro bundler at a time" constraint, and integrates with JIRA and tmux. The closest analogues from the competitive landscape are k9s (process management + log streaming + domain-specific operations) and lazyworktree (worktree browsing), but neither addresses the combined problem of process lifecycle enforcement, RN-specific commands, and external API context in a single interface. The recommended approach is a Rust/Ratatui application following the Elm Architecture pattern: a typed Event/Action/Effect loop with centralized AppState, domain types in a zero-IO layer, and infrastructure adapters hidden behind clean interfaces.

The stack is well-established and low-risk at its core. Ratatui 0.30 + Tokio 1.49 + crossterm 0.29 is the current community standard for production Rust TUIs. The only flagged instability is `tmux_interface` (pre-1.0), which can be mitigated by keeping tmux interaction behind a `TmuxClient` trait that allows a swap to raw `std::process::Command` tmux calls if needed. The tui-textarea crate is incompatible with ratatui 0.30 at time of research; either pin ratatui to 0.29 or use the built-in Paragraph widget for text input until tui-textarea 0.8 is released.

The top operational risks are process lifecycle bugs (zombie metro leaving port 8081 occupied on worktree switch) and terminal state corruption on crash. Both must be addressed in Phase 1 and Phase 2 respectively — they are foundational and expensive to retrofit. JIRA integration carries moderate risks around startup latency and token security that are straightforward to avoid if designed async-first from the start. Architecture and feature scope are well-understood. The MVP is clearly defined: worktree list, metro start/stop/log, RN command palette, and git operations — all with vim-style keybindings.

## Key Findings

### Recommended Stack

The core stack (ratatui, crossterm, tokio) is the de-facto standard for Rust TUI applications in 2026. Ratatui 0.30 introduced modularization into `ratatui-core` + `ratatui-widgets` for better compile times and requires Rust 1.86. Tokio provides non-blocking process I/O, async HTTP, and the channel-based event multiplexing that the architecture depends on. Git interaction should use the `git` CLI via `tokio::process::Command` rather than `git2` in async contexts — `git2` is synchronous and will block the render loop. JIRA calls use `reqwest` with the `json` feature. Config uses `toml` + `serde` with XDG path resolution via `directories`.

**Core technologies:**
- `ratatui 0.30` + `crossterm 0.29`: TUI rendering and terminal backend — de-facto standard, actively maintained, HIGH confidence
- `tokio 1.49`: Async runtime — required for non-blocking process I/O, HTTP calls, and tmux interaction; entire stack depends on it
- `reqwest 0.13`: Async HTTP for JIRA API — shares tokio runtime, JSON deserialization via serde
- `git2 0.20`: Git worktree listing — use only for worktree enumeration (synchronous operations wrapped in `spawn_blocking`); shell out for all other git ops
- `tmux_interface 0.3.2`: Typed tmux command abstraction — pre-1.0, pin version, keep behind a trait for potential swap
- `tracing` + `tracing-appender`: Structured file logging — NEVER stdout in a ratatui app; file-only logging is mandatory
- `color-eyre 0.6`: Panic hook — restores terminal before printing panic output; must be installed in Phase 1

**Version compatibility flags:**
- ratatui 0.30 requires Rust 1.86 (MSRV)
- tui-textarea 0.7 is NOT compatible with ratatui 0.30 (targets 0.29); wait for 0.8 or use Paragraph
- Do NOT add `crossterm` as a direct Cargo dependency — import via `ratatui::crossterm` re-export only

### Expected Features

Research confirms a clear MVP boundary. The worktree list is the primary navigation surface; metro process control is the core differentiating function. The feature graph is well-defined: metro control depends on the process layer, worktree switching depends on metro control, JIRA fetching is isolated and optional, device selection is subordinate to the RN command palette.

**Must have — table stakes (v1):**
- Worktree list panel with branch name display — core navigation surface
- Metro status indicator (running/stopped, which worktree) — the critical singleton constraint made visible
- Metro start/stop/restart controls — core action
- Metro log panel with show/hide toggle and scrolling — live feedback
- RN command palette: clean, rm node_modules, yarn install, yarn start --reset-cache
- Git operations per worktree: reset --hard, pull, push
- Vim-style keybindings (hjkl, q, /, ?) throughout — non-negotiable for the target user
- On-screen keybinding hints footer — context-sensitive, changes per active panel
- Confirmation dialogs for destructive actions
- Command output streaming in panel

**Should have — differentiators (v1.x after core is stable):**
- JIRA ticket title alongside branch name — branch context without leaving terminal
- Custom labels per worktree — user annotations stored in config
- Worktree switching orchestration (kill + yarn install + start in new worktree) — one-keystroke workflow
- Dependency staleness detection with lazy install warning
- Metro interactive commands (j/r/reload via stdin pipe)
- Device/simulator selection for run-android/run-ios
- Launch Claude Code in tmux tab at worktree cwd

**Defer to v2+:**
- CI/CD status per worktree (GitHub Actions) — requires second auth system
- Command flags configuration modal — polish, adds UI complexity
- yarn jest / lint additions — add when testing workflow is validated as a pain point

### Architecture Approach

The recommended architecture is a four-layer system: domain (pure Rust data types and business rules, zero IO), infrastructure (ProcessManager, JiraClient, GitClient, TmuxClient, ConfigStore), application (AppState + update() function implementing the Elm Architecture), and presentation (ratatui widgets that are pure functions of AppState). The Elm Architecture — Event → Action → update() → Effect → background task → Event — is the dominant pattern in production ratatui applications and eliminates callback hell. AppState is the single source of truth; widgets never mutate state. Effects (async work) are spawned from the main loop after `update()` returns and send their results back as Events through typed channels.

**Major components:**
1. `AppState` — single source of truth for all UI-visible data; cloned into render, mutated by update()
2. `ProcessManager` — owns all `tokio::process::Child` handles; exposes `spawn()`, `kill()`, `is_running()`; callers never touch raw handles
3. `WorktreeManager` (domain) — encodes the "one metro at a time" invariant as a pure Rust function; tested without IO
4. `JiraClient` — async HTTP via reqwest; returns domain types; optional and never blocks startup
5. `TmuxClient` — wraps `tmux_interface`; fire-and-forget window creation; behind a trait for swap-ability
6. Event loop (`tokio::select!`) — multiplexes terminal key events, process output, background task results, and tick events
7. UI widgets (presentation layer) — pure functions of `&AppState`; no logic, no async

**Build order dictated by dependencies:** domain/ first (testable in isolation), infra/ second (each adapter independently testable), app.rs third (wire domain + effects), tui/event scaffolding fourth, ui/ widgets last.

### Critical Pitfalls

1. **Terminal not restored on panic/crash** — Install `color-eyre` and `ratatui::restore()` panic hook in Phase 1 before any other code. Any code path that exits without cleanup leaves the user with a broken shell. Test by intentionally panicking in the main loop.

2. **Zombie metro process after kill (port 8081 stays bound)** — Always `child.kill().await` followed by `child.wait().await` before switching worktrees. Dropping a `Child` handle does NOT kill the process on Unix. Set `kill_on_drop(true)` as a safety net. After kill, validate port 8081 is free before starting metro in the new worktree.

3. **Blocking I/O on the async runtime (UI freeze)** — Never use `std::process::Command` in async code. Use `tokio::process::Command` for all subprocess spawning. Never use `git2` synchronous bindings in the event loop — either shell out via tokio or wrap in `spawn_blocking`. Any blocking call stalls the entire render loop.

4. **tmux send-keys race condition** — When opening Claude Code in a new tmux window, use `tmux new-window "claude"` to pass the command as the initial shell command rather than creating a window and sending keys. Shells take time to initialize; send-keys to a new pane is a race condition confirmed to affect Claude Code's own agent spawning.

5. **Crossterm version conflict** — Do not add `crossterm` as a direct Cargo.toml dependency. Import everything via `ratatui::crossterm`. Two crossterm versions in the binary cause invisible raw mode bugs. Verify with `cargo tree | grep crossterm` at project setup.

## Implications for Roadmap

Based on the combined research — the dependency graph from FEATURES.md, the build order from ARCHITECTURE.md, and the phase-to-pitfall mappings from PITFALLS.md — the following phase structure is recommended:

### Phase 1: Project Scaffold and TUI Shell

**Rationale:** Every subsequent phase inherits the terminal lifecycle, event loop architecture, and keybinding layer established here. Terminal cleanup bugs and crossterm version conflicts are foundational and expensive to retrofit. This phase has no feature dependencies but everything depends on it.

**Delivers:** A running ratatui app with correct terminal init/restore, panic hook, event-driven render loop (not fixed 60fps), single draw-per-tick, and a stub UI shell with keybinding hints footer. All vim keybindings wired through an Action enum. App tested inside tmux to verify key passthrough.

**Addresses from FEATURES.md:** Vim-style keybindings, on-screen keybinding hints, escape/modal conventions.

**Avoids from PITFALLS.md:**
- Terminal raw mode on panic (install color-eyre + panic hook)
- Crossterm version mismatch (verify `cargo tree` before first commit)
- Fixed 60fps CPU waste (event-driven render from the start)
- Vim key conflicts with tmux (test all bindings in tmux on day one)
- Multiple draw() calls per iteration (single draw established in skeleton)

**Research flag:** Standard patterns, well-documented. No additional research phase needed.

---

### Phase 2: Metro Process Management

**Rationale:** Metro process control is the core differentiating feature and the dependency anchor for worktree switching, log streaming, and interactive commands. Building this correctly (with kill+wait semantics, port verification, and port-based status detection) before any downstream features prevents the most expensive bugs in the project.

**Delivers:** ProcessManager with spawn/kill/wait/is-running, metro status indicator (using port 8081 probe, not PID alone), metro start/stop/restart actions, metro log streaming via channel, log panel with toggle and scroll. The "one metro at a time" invariant enforced in domain/WorktreeManager.

**Addresses from FEATURES.md:** Metro status indicator (P1), metro start/stop/restart (P1), metro log panel (P1).

**Avoids from PITFALLS.md:**
- Zombie metro / port 8081 not released (kill+wait+port probe)
- Blocking I/O on async runtime (all process I/O via tokio)
- Metro status via PID only (cross-validate with port 8081 check)
- Stdout/stderr pipe deadlock (separate tokio tasks per stream)

**Research flag:** Standard patterns. Tokio process docs are comprehensive. No additional research needed.

---

### Phase 3: Worktree List and Git Operations

**Rationale:** The worktree list panel is the primary navigation surface but depends on stable process management to display accurate metro status per worktree. Git operations are the daily driver commands that make the tool usable for its intended workflow.

**Delivers:** Worktree list panel showing path + branch name + metro status badge. Git operations per worktree: reset --hard (with confirmation dialog), pull, push. Confirmation dialog pattern established here for reuse. Command output streaming for git operations.

**Addresses from FEATURES.md:** Worktree list panel (P1), git operations (P1), confirmation dialog (P1), command output streaming (P1).

**Avoids from PITFALLS.md:**
- Git operations blocking the main async thread (shell out via `tokio::process::Command`, not git2 in async context)
- Stdout/stderr pipe deadlock (reuse ProcessManager pattern from Phase 2)
- No feedback during slow operations (stream git output to panel)
- Config struct breaking changes (version config schema now)

**Research flag:** Standard patterns. May want a quick research pass on `git worktree list --porcelain` output format before implementation to avoid parsing bugs.

---

### Phase 4: RN Command Palette and Config/Labels

**Rationale:** The RN command palette is the second major differentiator and completes the MVP. Config and labels are low-complexity but enable JIRA integration in the next phase. Establishing config with correct file permissions now avoids a security retrofit.

**Delivers:** RN command palette (clean, rm node_modules, yarn install, yarn start --reset-cache) with output streaming. Config store at `~/.config/ump-dash/` with versioned schema and 0600 permissions. Custom labels per worktree stored in config. Dependency staleness detection (package.json mtime vs node_modules mtime).

**Addresses from FEATURES.md:** RN command palette (P1), custom labels (P2), dependency staleness detection (P2).

**Avoids from PITFALLS.md:**
- JIRA token stored without permission hardening (set 0600 on first write)
- Config schema breaking changes (use `#[serde(default)]` on all fields, version field from day one)
- Command injection via branch name in shell args (always use `Command::new()` with separate arg elements)

**Research flag:** Standard patterns. No additional research needed.

---

### Phase 5: JIRA Integration

**Rationale:** JIRA integration is isolated — it only needs an HTTP client and a config token. It adds meaningful context but is not required for the core workflow. Building it after config is established means the token storage pattern is already in place. Must be designed async-first with graceful degradation from day one.

**Delivers:** Background JIRA title fetching triggered by worktree focus. In-memory cache with TTL. Disk cache at `~/.config/ump-dash/cache/` for offline startup. Graceful fallback showing branch name when JIRA is unreachable. HTTP 429 handling with backoff. JIRA API token support for both Cloud (Basic Auth) and Data Center (Bearer PAT).

**Addresses from FEATURES.md:** JIRA ticket title alongside branch name (P2).

**Avoids from PITFALLS.md:**
- JIRA API calls blocking startup (show dashboard first, fetch in background)
- JIRA token exposure (0600 permissions, support `JIRA_API_TOKEN` env var)
- Background auto-refresh rate limiting (fetch on demand, cache with TTL)
- Logging JIRA API token in debug output (sanitize Authorization headers in tracing)

**Research flag:** Validate JIRA auth method (Cloud vs. Data Center) before implementation. Cloud uses Basic Auth (email:token); Data Center uses Bearer PAT. The API endpoint version also differs (v3 vs v2).

---

### Phase 6: Worktree Switching Orchestration and Advanced Features

**Rationale:** Worktree switching orchestration requires stable metro control (Phase 2) and worktree list (Phase 3) to be solid. Device selection for run-android/ios is complex and dependent on the RN command palette (Phase 4). Metro interactive commands need the stdin pipe established in Phase 2. Claude Code tmux integration needs pitfall-aware implementation.

**Delivers:** One-keystroke worktree switching (kill current metro, optional yarn install if stale, start metro in new worktree) with visible progress. Device/simulator selection panel for run-android/run-ios (adb devices + xcrun simctl). Metro interactive commands (j/r/reload) via stdin pipe. Launch Claude Code via `tmux new-window "claude"` at worktree cwd.

**Addresses from FEATURES.md:** Worktree switching orchestration (P2/HIGH), device selector (P2/HIGH), metro interactive commands (P2), Claude Code tmux integration (P2).

**Avoids from PITFALLS.md:**
- tmux send-keys race condition (use `new-window "command"` not send-keys for Claude Code)
- Single-metro invariant during switch (atomic kill+start via WorktreeManager domain logic)

**Research flag:** Device selection (adb + xcrun simctl output parsing) likely needs a research pass — parsing these tool outputs is brittle and platform-specific.

---

### Phase Ordering Rationale

- **Phase 1 before everything:** Terminal lifecycle and event loop architecture are inherited by all later phases. Getting it wrong means retrofitting across the entire codebase.
- **Phase 2 before Phase 3:** Worktree list displays metro status; it cannot be complete without a working ProcessManager.
- **Phase 3 before Phase 6 (switching):** Worktree switching orchestration depends on the stable list panel and metro control.
- **Phase 4 before Phase 5:** Config storage and file permissions must exist before JIRA token can be stored securely.
- **Phase 5 isolated:** JIRA is the only phase with no downstream dependencies other than Phase 6 cosmetics. It can be parallelized with Phase 4 if needed.
- **Phase 6 last:** All high-complexity integrations (orchestration, device selection, tmux) depend on earlier phases being solid.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 5 (JIRA):** Confirm JIRA auth method (Cloud Basic Auth vs. Data Center Bearer PAT) and API version (v3 vs v2) before writing the client. Auth mismatch means zero calls succeed.
- **Phase 6 (Device selection):** `adb devices` and `xcrun simctl list instruments -s simulators` output format needs targeted research — parsing these reliably is non-trivial.

Phases with standard patterns (skip research-phase):
- **Phase 1:** Ratatui + crossterm + Tokio event loop setup is well-documented in official ratatui docs.
- **Phase 2:** Tokio process management patterns are well-documented; `kill().await` + `wait().await` semantics are in the official docs.
- **Phase 3:** Git CLI output via `git worktree list --porcelain` is stable and documented.
- **Phase 4:** Config with serde + toml is boilerplate; RN command invocations are known.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core stack (ratatui, tokio, reqwest, serde) verified via official docs. One caveat: tui-textarea 0.7 incompatibility with ratatui 0.30 confirmed; tokio-process-stream version unverified via official docs (MEDIUM). |
| Features | HIGH | MVP derived from PROJECT.md requirements. Competitor analysis grounded in real tools (lazygit, gitui, k9s, lazyworktree). Feature dependency graph is internally consistent. |
| Architecture | HIGH | Elm Architecture pattern confirmed as official ratatui recommendation. Component boundaries derived from gitui and lazygit source analysis plus official ratatui patterns docs. |
| Pitfalls | HIGH | All critical pitfalls verified against official Ratatui, Tokio, and Atlassian docs. tmux send-keys race condition confirmed via Claude Code issue tracker. |

**Overall confidence:** HIGH

### Gaps to Address

- **JIRA auth method:** Not confirmed whether the team's JIRA instance is Cloud (Basic Auth + email:token) or Data Center (Bearer PAT). Must verify before Phase 5 implementation. Wrong choice means zero successful API calls.
- **tui-textarea 0.30 compatibility:** At research time, tui-textarea 0.7 targets ratatui 0.29. Monitor https://github.com/rhysd/tui-textarea/releases for 0.8 release. Decision point: pin ratatui to 0.29 or use Paragraph for text input.
- **tmux_interface stability:** Pre-1.0 API may change between patch versions. Pin exact version in Cargo.toml. Decision rule: if API churn is painful during development, drop it and use `Command::new("tmux").args([...])` directly — this is explicitly acceptable per research.
- **tokio-process-stream version:** Version 0.4.x noted but not verified against official crates.io. Validate before use; the `tokio::process::Child` + `BufReader::lines()` approach is a safe fallback.

## Sources

### Primary (HIGH confidence)
- https://ratatui.rs/highlights/v030/ — ratatui 0.30 release notes, MSRV 1.86
- https://ratatui.rs/concepts/application-patterns/the-elm-architecture/ — TEA pattern, official ratatui docs
- https://ratatui.rs/concepts/event-handling/ — event loop patterns
- https://ratatui.rs/faq/ — panic hook and terminal cleanup
- https://ratatui.rs/recipes/apps/panic-hooks/ — color-eyre integration
- https://ratatui.rs/recipes/apps/color-eyre/ — color-eyre pattern for ratatui
- https://docs.rs/tokio/latest/tokio/process/index.html — tokio process, zombie prevention
- https://docs.rs/reqwest/latest/reqwest/ — reqwest 0.13.2 async + JSON
- https://docs.rs/git2/latest/git2/ — git2 0.20.4 Worktree API
- https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/ — JIRA Cloud REST API
- https://developer.atlassian.com/cloud/jira/platform/rate-limiting/ — JIRA rate limiting
- https://github.com/ratatui/ratatui/issues/1298 — crossterm version incompatibility advisory
- https://github.com/anthropics/claude-code/issues/23513 — tmux send-keys race condition confirmed

### Secondary (MEDIUM confidence)
- https://github.com/gitui-org/gitui — source code architecture analysis
- https://deepwiki.com/jesseduffield/lazygit — lazygit architecture
- https://github.com/chmouel/lazyworktree — worktree TUI patterns
- https://docs.rs/tmux_interface/latest/tmux_interface/ — tmux_interface 0.3.2 (pre-1.0)
- https://github.com/ratatui/awesome-ratatui — ecosystem widget survey
- WebSearch: tokio-process-stream 0.4.x — version unverified via official docs

### Tertiary (LOW confidence — validate during implementation)
- tui-textarea 0.8 ratatui 0.30 compatibility — not yet released at research time; monitor releases
- JIRA Data Center PAT auth specifics — not validated against a live instance

---
*Research completed: 2026-03-02*
*Ready for roadmap: yes*
