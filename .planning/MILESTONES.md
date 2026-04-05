# Milestones

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
