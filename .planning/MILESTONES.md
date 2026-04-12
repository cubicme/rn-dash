# Milestones

## v1.1 Public Release (Shipped: 2026-04-13)

**Phases completed:** 4 phases, 9 plans

**Delivered:** Polished keybinding scheme, removed dead features, generalized config for any RN monorepo, and shipped to public GitHub with CI and prebuilt binaries.

**Stats:**
- 60 files changed, +5,139 / -546 LOC
- Timeline: 2026-04-05 → 2026-04-12 (8 days)
- Git range: `1740f25` (milestone start) → `98500fb` (v1.2.0 release)

**Key accomplishments:**

- Labels feature deleted entirely — `labels.rs` removed, `config_dir()` relocated, `Worktree.label` stripped, all `SetLabel` actions and UI handlers gone; codebase compiles with zero label references
- Sync palette renamed to Yarn (absorbing 3 clean commands); Worktree palette extracted from Git with create/remove/new-branch commands under lowercase `w` key
- `w>B` opens a filterable remote-branch picker, prompts for a new branch name, then creates a worktree on that branch via `git worktree add -b`
- Context-sensitive metro keys (R/J/Esc only when metro running), dynamic footer hints derived from available actions, MetroRestart action removed entirely
- Package renamed to `rn-dash`; all hardcoded AJ/UMP values extracted to `DashConfig` (repo_root, jira_project_prefix, app_title) with a fully documented example config
- TOML config format replaces JSON using `toml::from_str`/`toml::to_string_pretty`, with annotated `config.example.toml`
- MIT license, Cargo.toml crates.io metadata, comprehensive README (build, config, usage), and `.gitignore` audit
- GitHub Actions CI (`cargo build / clippy -D warnings / test` on macOS+Linux) and tag-triggered release workflow publishing tar.gz binaries for Apple Silicon, Intel Mac, and Linux x86_64
- macOS codesigning + notarization added to release pipeline; Gatekeeper workaround documented in README

**Post-ship quick tasks (merged into v1.2.0 release):**
- `SyncBeforeMetro` modal for stale-dep check on Enter key
- `auto_sync` config param to skip sync confirmation modals
- Consolidated yarn clean commands into single selection menu
- Bug fixes: lsof hang, process-group kill, race conditions, sync modal for pods-only staleness

---

## v1.0 MVP (Shipped: 2026-04-05)

**Phases completed:** 8 phases, 37 plans, 59 tasks

**Key accomplishments:**

- Cargo project initialized with ratatui 0.30 + tokio + logging stack; domain/infra/ui module skeleton with enforced architecture boundaries and zero build warnings
- Async TEA event loop with guaranteed terminal restore on all exit paths (including panics), vim-style keybinding dispatch, and focus cycling via tokio::select! over crossterm EventStream
- Ratatui three-panel layout with cyan/gray focus borders, context-sensitive footer, centered help overlay (? / F1), and error overlay — all read-only renders from AppState
- MetroManager single-instance enforcer, ProcessClient trait boundary, and full Action/AppState metro interface layer — all contracts Plan 02 runtime and Plan 03 UI build against
- tokio::spawn-based metro lifecycle — spawn/kill/restart/stdin forwarding and external death detection wired into TEA event loop via dual mpsc channels (metro_rx for actions, handle_rx for MetroHandle delivery)
- One-liner:
- Pure Rust domain types: CommandSpec (17 variants), expanded Worktree struct (8 fields), ModalState, and 21 new Action variants establishing all Phase 3 type contracts
- Four infra modules behind clean function boundaries: git porcelain parser, generic async command runner with mpsc streaming, JSON label persistence, and adb/xcrun device list parsers
- TEA brain for Phase 3: AppState extended with 11 new fields, handle_key with modal+palette routing, update() covering all 20+ Phase 3 action arms, and worktree/label loading wired into run()
- Ratatui UI layer completed: real worktree List+StatefulWidget with metro badges, scrollable command output, three modal overlay types (confirm/text-input/device-picker), context-sensitive footer hints for palette and modal states, and expanded help overlay with all Phase 3 keybindings
- Six CommandSpec variants corrected to emit requirement-compliant argv: react-native clean --include, run-android/run-ios via npx/yarn react-native, yarn unit-tests, and yarn lint --quiet --fix
- reqwest-based JIRA HTTP client with Basic/Bearer auth dispatch, DashConfig 0600-protected credentials file, and title cache persistence following the labels.rs NotFound pattern
- Background JIRA title fetching wired into TEA loop: config loaded on startup, titles cached and re-applied on WorktreesLoaded, JiraTitlesFetched handler persists to disk and updates worktree display names
- Worktree list reordered to show branch name first (always visible), JIRA title/label second in legible Gray, with Unicode warning icon U+26A0 replacing the noisy [stale] text badge
- One-liner:
- One-liner:
- FIFO command queue (VecDeque<CommandSpec>) and per-worktree output persistence (HashMap<WorktreeId, VecDeque<String>>) replace shared command_output and pending_command_after_install in AppState
- One-liner:
- Complete handle_key() rewrite with 5-palette submenu scheme (a/i/x/s/g), multiplexer field replacing tmux_available bool, and OpenClaudeCode using claude_flags from config
- CleanToggle and SyncBeforeRun modals fully wired into command queue with rendered UI — replacing WORK-06 lazy install with user-visible sync prompt
- Android release two-step (assembleRelease + adb install), GitResetHardFetch chaining via queue, iOS simulator sort-by-recent with type-to-filter picker, and live queue count in output panel title
- Double-line borders on all panes, dynamic metro/output titles, 4-column table restructure, and metro-active row highlight fix
- Pure domain refresh_needed() function mapping CommandSpec to RefreshSet, wired into CommandExited with .yarn-integrity staleness sentinel
- Universal vim-style G/gg scrolling with metro auto-follow, fullscreen Tab exit, physical iOS device picker via xctrace, and metro debugger visual feedback
- Port 8081 external metro detection with lsof-based PID lookup, conflict modal showing PID and working directory, kill-and-auto-start resolution flow
- White pane titles, per-category status icons (metro/yarn/pods), and green bold metro-active row highlight
- Multi-version yarn staleness detection with .yarn-integrity/.yarn-state sentinels and all-worktree refresh
- Fixed scrollbar position mismatch on G scroll, Tab-in-fullscreen pane cycling, and metro debugger toggle command byte
- Fixed metro-active row highlight by deriving status from MetroManager, replaced icon dots with always-visible Y/P letters, added Dir column to worktree table
- Robust yarn staleness via Berry install-state.gz sentinel, metro debugger sends j keystroke
- Single domain function preferred_prefix() for consistent worktree naming in Claude tabs and metro pane title
- Watchman warning filter added to metro log streaming pipeline and bullet replaced with green play triangle in worktree table and footer legend
- One-liner:

---
