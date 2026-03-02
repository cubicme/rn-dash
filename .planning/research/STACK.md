# Stack Research

**Domain:** Rust TUI dashboard — process management, git worktrees, REST API, tmux interaction
**Researched:** 2026-03-02
**Confidence:** HIGH (core stack verified via official docs and docs.rs; tmux crate flagged MEDIUM due to pre-1.0 status)

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| ratatui | 0.30.0 | TUI rendering framework | De-facto standard for Rust TUIs. Forked and actively maintained successor to archived tui-rs. 0.30 modularizes into ratatui-core + ratatui-widgets for better compile times and API stability. MSRV 1.86. |
| crossterm | 0.29.0 | Terminal backend + raw mode + events | Default backend for ratatui; cross-platform (macOS/Linux/Windows). Provides `event-stream` feature for async keyboard/mouse input with Tokio. Only enable one backend. |
| tokio | 1.49.0 | Async runtime | Required for non-blocking process I/O, HTTP calls, and tmux commands. `tokio::process` handles child process spawning with async stdout/stderr streaming. `tokio::sync` provides channels for actor-style state management. Use `features = ["full"]`. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| git2 | 0.20.4 | Git worktree listing and operations | Use `Repository::worktrees()` to enumerate worktrees, `Worktree::path()` to resolve paths, and branch HEAD resolution for current branch name. Preferred over shelling out to `git` — type-safe and no subprocess overhead. |
| reqwest | 0.13.2 | Async HTTP client for JIRA API | Makes REST calls to JIRA `/rest/api/3/issue/{key}` using Basic Auth (email + API token in Base64). Enable `features = ["json"]` for automatic serde deserialization of response bodies. Shares the Tokio runtime. |
| serde | 1.x | Serialization/deserialization framework | Required for JIRA API response structs and TOML config deserialization. Use `features = ["derive"]` to enable `#[derive(Serialize, Deserialize)]`. |
| serde_json | 1.0.149 | JSON parsing for JIRA API responses | Used alongside reqwest for typed deserialization of JIRA issue objects. Current version is actively maintained (1.0.149 released 2026-01-06). |
| toml | 0.8.x | Config file parsing | Parse `~/.config/ump-dash/config.toml` for JIRA token, worktree labels, and preferences. Pairs with serde for struct deserialization. |
| directories | 5.x | XDG config directory resolution | Provides `ProjectDirs::from()` to resolve `~/.config/ump-dash/` in a platform-correct way. Avoids hard-coding path separators or env vars. |
| tmux_interface | 0.3.2 | Tmux command abstraction | Wraps `tmux new-window`, `send-keys`, `kill-pane` etc. as typed Rust structs instead of raw string commands. Supports tmux 3.3 (stable feature flag). Pre-1.0, API may change — pin the version. Alternative: shell out directly (see "Alternatives Considered"). |
| anyhow | 1.x | Application error handling | Use at binary level (`main.rs`, command handlers). `anyhow::Result` + `.context()` gives readable error chains without defining custom error types for every call site. |
| thiserror | 2.x | Library error type definitions | Use for domain-layer error types (e.g., `WorktreeError`, `JiraError`, `MetroError`). Provides `#[derive(Error)]` to avoid boilerplate. Use thiserror for domain modules, anyhow at the application boundary. |
| tracing | 0.1.x | Structured diagnostic logging | Write debug logs to file (not stdout — the TUI owns stdout). Pairs with `tracing-subscriber` + `tracing-appender` for rolling file output to `~/.config/ump-dash/logs/`. Maintained by the Tokio project. |
| tracing-subscriber | 0.3.x | Tracing subscriber/formatter | Configure the file-based log subscriber with `RollingFileAppender`. Never log to stdout in a ratatui app — it corrupts the TUI rendering. |
| tracing-appender | 0.2.x | Non-blocking rolling file log writer | `tracing_appender::rolling::daily()` writes to dated log files. Non-blocking variant avoids stalling the async event loop during log writes. |
| color-eyre | 0.6.x | Panic hook + error display | Install in `main()` with `color_eyre::install()`. Provides the critical pattern: restore terminal before printing error output, preventing corrupted terminal state on crash. Recommended by official ratatui error handling docs. |
| tui-textarea | 0.7.0 | Text input widget | For any in-TUI text entry (label editing, filter fields). Currently targets ratatui 0.29; watch for a 0.8 release that tracks ratatui 0.30. If ratatui 0.30 is used immediately, pin tui-textarea to a compatible ratatui version or use ratatui's built-in `Paragraph` for simple single-line input. |
| tokio-process-stream | 0.4.x | Interleaved stdout+stderr stream from subprocess | Wraps `tokio::process::Child` as a `futures::Stream` yielding `Item::Stdout` / `Item::Stderr` lines in arrival order. Needed for live metro log streaming in the dashboard. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo-generate | Bootstrap from ratatui official templates | `cargo generate ratatui/templates` then choose "component" template for async + tokio + crossterm structure. Use as starting point; adapt to project architecture. |
| cargo-watch | Rebuild on file change during development | `cargo watch -x run` — speeds up iteration on TUI layout changes. |
| tokio-console | Async task debugging | `tokio-console` provides a live view of tokio tasks, channels, and wakers. Invaluable when debugging deadlocks in the event loop. Enable with `TOKIO_UNSTABLE=1`. |
| rust-analyzer | IDE language server | Requires no configuration; works with standard `cargo` workspaces. |

## Installation

```toml
# Cargo.toml

[dependencies]
# TUI core
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = { version = "0.29", features = ["event-stream"] }

# Async runtime
tokio = { version = "1.49", features = ["full"] }
tokio-process-stream = "0.4"

# Git
git2 = "0.20"

# HTTP / JIRA
reqwest = { version = "0.13", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Config
toml = "0.8"
directories = "5"

# Error handling
anyhow = "1"
thiserror = "2"
color-eyre = "0.6"

# Logging (file, not stdout)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# Tmux
tmux_interface = { version = "0.3", features = ["tmux_stable"] }

# Text input widget
tui-textarea = "0.7"    # watch for 0.8 when ratatui 0.30 support lands
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| ratatui 0.30 | tui-rs | Never — tui-rs is archived (unmaintained since 2023). ratatui is its maintained successor. |
| crossterm backend | termion backend | Only if macOS/Linux-only and you want a lighter dependency. Crossterm is recommended because it handles macOS correctly and is the default ratatui backend. |
| tokio | async-std | No. tokio is the ecosystem standard in 2025. reqwest, tracing, and tokio-process-stream all depend on tokio. Mixing runtimes causes panics. |
| git2 | `git` CLI subprocess | Acceptable for operations not well-supported in git2 (e.g., complex rebase sequences), but `git worktree list` and status are cleanly supported by git2. Use git2 first; fall back to subprocess for operations that are easier as shell commands. |
| tmux_interface | Raw `std::process::Command` tmux calls | Shell-out is a valid alternative given tmux_interface's pre-1.0 instability. If tmux_interface's API churn becomes painful, `Command::new("tmux").args(["new-window", "-n", "name"])` is simple and predictable. |
| reqwest | ureq (blocking) | Only if you want to avoid async complexity for JIRA calls specifically. Since tokio is already in the stack for process management, reqwest is the natural fit. |
| anyhow + thiserror | eyre | eyre is a solid alternative to anyhow. color-eyre extends eyre, so if color-eyre is used for panic hooks, you can optionally use `eyre::Result` instead of `anyhow::Result`. Either is fine — pick one and be consistent. |
| tracing + tracing-appender | tui-logger widget | tui-logger embeds logs as a TUI panel widget. Interesting if you want a visible log panel in the dashboard itself. Not recommended as primary logging — always write to file too, since the TUI panel is lost when the app exits. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| tui-rs | Archived in 2023. No bug fixes, no new widget support, no ratatui 0.x compatibility. | ratatui |
| termion | Unix-only, not macOS-optimized, less active than crossterm. Ratatui's crossterm backend is the community default. | crossterm |
| log + env_logger | The `log` crate does not support structured/async-aware logging. In a tokio app with async tasks, log events lose task context. | tracing + tracing-subscriber |
| println! / eprintln! | Writes to stdout/stderr which ratatui owns in raw mode. Will corrupt the TUI display. | tracing with file appender |
| async-std | Incompatible runtime with tokio. reqwest, tracing, and tokio-process-stream all require tokio. Mixing causes panics at runtime. | tokio |
| pty crates (pty-process, portable-pty) | Not needed for this project. Metro runs detached via tmux; interaction goes through tmux's `send-keys`. PTY would be required only if capturing metro output without tmux (more complex, out of scope). | tokio::process + tokio-process-stream for non-PTY streaming; tmux_interface for tmux-hosted processes |
| reqwest 0.12.x | Outdated. Current stable is 0.13.2. Dependency version conflicts likely with other updated crates. | reqwest 0.13 |

## Stack Patterns by Variant

**If JIRA instance is self-hosted (Data Center, not Cloud):**
- Use Personal Access Token (PAT) as a Bearer token in the `Authorization` header
- API endpoint base changes: `https://your-jira.company.com/rest/api/2/issue/{key}` (not v3)
- Cloud JIRA uses Basic Auth: `Authorization: Basic base64(email:api_token)`
- Verify which auth method your JIRA uses before implementation

**If tui-textarea is not yet ratatui 0.30 compatible when development starts:**
- Pin ratatui to `0.29` until tui-textarea releases a 0.8 version
- Or use ratatui's built-in `Paragraph` widget for simple single-line text fields (sufficient for label editing)
- Monitor https://github.com/rhysd/tui-textarea/releases for ratatui 0.30 compatibility

**If tmux_interface API churn becomes painful:**
- Drop it and use `std::process::Command::new("tmux").args([...])` directly
- The tmux CLI is stable and well-documented; direct shell calls are completely acceptable
- Keep tmux interaction behind a `TmuxClient` trait so the implementation can be swapped

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| ratatui 0.30.0 | crossterm 0.29 | ratatui 0.30 added feature flags for multiple crossterm versions. Default is latest (0.29). |
| ratatui 0.30.0 | tui-textarea 0.7.x | NOT compatible. tui-textarea 0.7 targets ratatui 0.29. Wait for tui-textarea 0.8+. |
| tokio 1.49 | reqwest 0.13.2 | Compatible — reqwest 0.13 requires tokio 1.x. |
| tokio 1.49 | tokio-process-stream 0.4.x | Compatible — wraps tokio::process::Child from tokio 1.x. |
| git2 0.20.4 | libgit2 1.9.x | git2 0.20 requires libgit2 1.9.0+. libgit2-sys bundles the correct version, no pre-install needed. |
| serde 1.x | serde_json 1.x | Always compatible within the same serde major version. |
| ratatui 0.30.0 | MSRV | Rust 1.86.0 required. Verify `rustup update stable` before building. |

## Sources

- https://ratatui.rs/highlights/v030/ — ratatui 0.30 stable release notes, MSRV 1.86 confirmed (HIGH confidence)
- https://docs.rs/ratatui/latest/ratatui/ — ratatui 0.30.0 module structure and backends (HIGH confidence)
- https://docs.rs/tokio/latest/tokio/ — tokio 1.49.0 features: process, sync, task (HIGH confidence)
- https://docs.rs/reqwest/latest/reqwest/ — reqwest 0.13.2, async + JSON features (HIGH confidence)
- https://docs.rs/git2/latest/git2/ — git2 0.20.4, Worktree struct API (HIGH confidence)
- https://docs.rs/git2/latest/git2/struct.Worktree.html — Worktree methods confirmed (HIGH confidence)
- https://docs.rs/tmux_interface/latest/tmux_interface/ — tmux_interface 0.3.2, commands supported (MEDIUM confidence — pre-1.0)
- https://ratatui.rs/recipes/apps/color-eyre/ — color-eyre integration pattern for ratatui (HIGH confidence)
- https://ratatui.rs/concepts/event-handling/ — event handling patterns in ratatui (HIGH confidence)
- https://github.com/rhysd/tui-textarea/releases — tui-textarea 0.7.0 targets ratatui 0.29, not 0.30 (HIGH confidence)
- https://github.com/ratatui/awesome-ratatui — ecosystem widget crates survey (MEDIUM confidence)
- WebSearch: tokio-process-stream for interleaved stdout+stderr (MEDIUM confidence — version unverified via official docs)
- WebSearch: directories crate 5.x for XDG paths (MEDIUM confidence)

---
*Stack research for: Rust TUI dashboard — React Native worktree management*
*Researched: 2026-03-02*
