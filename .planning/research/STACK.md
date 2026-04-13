# Technology Stack

**Project:** rn-dash v1.3 — Per-Worktree Tasks + Architecture Audit
**Researched:** 2026-04-13
**Confidence:** HIGH (core additions verified via docs.rs and crates.io; arch_test_core flagged MEDIUM due to low activity/AGPL-3.0 concerns)

---

## Context: What Already Exists (Do NOT Re-Add)

The following are already in Cargo.toml and validated in v1.0–v1.2. Do not duplicate.

| Already Present | Version |
|-----------------|---------|
| ratatui | 0.30 |
| crossterm | 0.29 (event-stream) |
| tokio | 1.49 (full) |
| futures | 0.3 |
| anyhow / thiserror / color-eyre | 1.x / 2.x / 0.6 |
| tracing / tracing-subscriber / tracing-appender | 0.1 / 0.3 / 0.2 |
| serde / serde_json / toml | 1.x / 1.x / 0.8 |
| reqwest | 0.12 (json, rustls-tls) |
| libc | 0.2 |
| async-trait | 0.1 |

The existing event loop already has a `tokio::time::interval` at 250ms that fires a tick. Elapsed time tracking and spinner animation both hook into this without any new crate.

---

## New Dependencies Required for v1.3

### Runtime Dependencies (add to `[dependencies]`)

| Crate | Version | Purpose | Integration Point |
|-------|---------|---------|-------------------|
| `tokio-util` | `0.7` | `CancellationToken` for per-worktree individual task cancellation; `TaskTracker` for coordinated shutdown of per-worktree task sets | Replace `JoinHandle::abort()` pattern. Each worktree task slot gets a `CancellationToken`; the runner selects on `token.cancelled()` alongside stdout/stderr streams. Pairs with existing `tokio::spawn` in `command_runner.rs`. |
| `throbber-widgets-tui` | `0.11` | Stateful 6-frame animated spinner widget rendered in worktree table cells. Built-in symbol sets include `CLOCK`, `BRAILLE_*`, `BOX_DRAWING`, `VERTICAL_BLOCK` frames. Renders via `frame.render_stateful_widget()`. | Owned `ThrobberState` stored per active worktree task in `AppState`. Advance state by calling `state.calc_next()` in the existing 250ms tick handler in `app::run`. Render in `ui/panels.rs` worktree table rows. |

### Development / Audit Tools (install via `cargo install`, NOT Cargo.toml)

| Tool | Version | Purpose | How to Use |
|------|---------|---------|------------|
| `cargo-geiger` | `0.13` | Report unsafe block count across all crates and dependencies. Used in architecture audit phase to validate "safe Rust only" goal. | `cargo geiger` — run once and review output. Not a CI gate (slow, requires network). |
| `cargo-deny` | `0.19` | Lint dependency graph: disallow license categories, flag duplicate versions, check advisory database. Also enforces `allowed_sources`. | Add `.cargo/deny.toml` to repo. Run `cargo deny check` in CI. Gate on license + advisories checks. |
| `cargo-depgraph` | `1.6` | Generate Graphviz `.dot` dependency graph to visualize crate-to-crate relationships and confirm domain/infra/app/ui separation at the module level. | `cargo depgraph | dot -Tsvg > graph.svg`. One-time audit use; not CI. |

### Dev Dependencies (add to `[dev-dependencies]`)

| Crate | Version | Purpose | Integration Point |
|-------|---------|---------|-------------------|
| `arch_test_core` | `0.1.5` | Rule-based architecture test: declare layer access rules (`domain` may not access `infra`; `ui` may not access `infra` directly, etc.) and validate statically against the source tree using rust-analyzer's syntax parser. Write as `#[test]` in `tests/architecture.rs`. | **MEDIUM confidence — AGPL-3.0 license; check if license is compatible with project goals before adding. Last commit 2021 but crate still functional.** Alternative: enforce boundaries via `#[cfg(test)]` integration tests with manual `use` checks, or skip and use `cargo-depgraph` visually. |

---

## What NOT to Add

| Do Not Add | Why |
|------------|-----|
| `indicatif` | CLI progress bars for stdout — not compatible with ratatui raw-mode TUI. Corrupts terminal output. Use `throbber-widgets-tui` instead. |
| `tokio-task-supervisor` | Thin wrapper around `tokio_util::task::TaskTracker` + `CancellationToken`. Adds a dependency for nothing the project cannot do directly with `tokio-util 0.7`. |
| `tachyonfx` | Ratatui effects/animation library. Full animation engine with particles and easing — overkill for 6-frame spinners. Use `throbber-widgets-tui` directly. |
| `pty-process` / `portable-pty` | PTY process handling. Not needed — commands run detached via `tokio::process::Command` with piped stdout/stderr, same as the current `command_runner.rs`. |
| `cargo-audit` (as a dep) | Already covered by `cargo-deny` which includes advisory database checking. Duplicate. |
| `arch_test_sdk` | Blockchain SDK — completely unrelated name collision on crates.io. |
| Any new async runtime crate | Tokio is the runtime. Adding `async-std`, `smol`, or `monoio` would cause incompatible runtime panics. |
| A second HTTP client | `reqwest 0.12` is already present. No new HTTP work needed for this milestone. |

---

## Elapsed Time Tracking: No New Crate Needed

Elapsed time is tracked with `std::time::Instant` (from the standard library — no crate required). Store `Instant::now()` when a task starts; compute `.elapsed()` in the tick handler or at render time to display `"12s"` / `"1m 03s"` strings.

```rust
// In AppState — no new import needed:
pub task_started_at: HashMap<WorktreeId, std::time::Instant>,
```

`tokio::time::Instant` is equivalent for this use case but `std::time::Instant` is simpler since the value is read in the UI render path (synchronous), not in an async select.

---

## Per-Worktree Parallel Execution: No New Crate Needed

The per-worktree task map is a plain `HashMap<WorktreeId, WorktreeTaskSlot>` in `AppState` where `WorktreeTaskSlot` holds:

```rust
struct WorktreeTaskSlot {
    spec: CommandSpec,
    cancel: tokio_util::sync::CancellationToken,  // new: from tokio-util
    started_at: std::time::Instant,               // no crate: std
    // JoinHandle held only if abort-without-token is needed as fallback
}
```

No `HashMap` crate needed — `std::collections::HashMap` is already used in `AppState` (see `command_output_by_worktree`, `jira_title_cache`).

---

## Cancellation Pattern (replaces `.abort()`)

Current pattern: `command_task: Option<JoinHandle<()>>` with `task.abort()`.

Replacement pattern for per-worktree cancellation:

```rust
// Spawn:
let token = CancellationToken::new();
let child_token = token.child_token();
tokio::spawn(async move {
    tokio::select! {
        _ = child_token.cancelled() => { /* graceful stop: kill process */ }
        _ = run_command(spec, tx) => {}
    }
});

// Cancel:
token.cancel();  // propagates to child_token, wakes the task
```

`CancellationToken::child_token()` is used so cancelling one worktree's token does not affect other worktrees' tokens. This is the canonical tokio-util pattern.

---

## Spinner Animation Pattern (using throbber-widgets-tui 0.11)

`throbber-widgets-tui 0.11` requires `ratatui ^0.30.0` — confirmed compatible with the existing `ratatui 0.30` dependency.

MSRV for `throbber-widgets-tui` is Rust 1.88.0. The project's current MSRV is set by ratatui 0.30 (1.86.0). This means building after adding `throbber-widgets-tui` requires Rust 1.88+. Run `rustup update stable` and update any CI toolchain pins.

Animation state per active worktree:

```rust
// In AppState:
pub throbber_states: HashMap<WorktreeId, throbber_widgets_tui::ThrobberState>,

// In tick handler (existing 250ms interval in app::run):
for state in app.throbber_states.values_mut() {
    state.calc_next();
}

// In render (ui/panels.rs):
frame.render_stateful_widget(
    throbber_widgets_tui::Throbber::default()
        .style(Style::default().fg(Color::Yellow))
        .throbber_set(throbber_widgets_tui::symbols::throbber::BRAILLE_EIGHT_DOUBLE),
    cell_area,
    &mut throbber_states[&worktree_id],
);
```

The project spec calls for 6-frame yellow spinner. `throbber-widgets-tui` ships several 6-frame symbol sets. `BRAILLE_SIX` or `VERTICAL_BLOCK` are both 6-frame. Verify exact symbol set names at compile time via the crate docs.

---

## Architecture Audit Tools: CLI Only (No Cargo.toml)

All audit tools are one-time or CI-gate runners, not runtime dependencies. Install once:

```bash
cargo install cargo-geiger    # v0.13 — unsafe scan
cargo install cargo-deny      # v0.19 — license/advisory/dupe check
cargo install cargo-depgraph  # v1.6  — dependency graph visualization
```

`cargo-deny` is the only one that should enter CI. Add `.cargo/deny.toml` with:
- `[licenses]` — deny GPL (except LGPL), deny unlicensed
- `[advisories]` — deny RUSTSEC vulnerabilities at severity HIGH+
- `[bans]` — flag duplicate major versions of key crates (ratatui, tokio, serde)

`cargo-geiger` and `cargo-depgraph` are audit-phase tools only (slow, require graphviz).

---

## Updated Cargo.toml Delta

Only these lines need to be added:

```toml
[dependencies]
# --- NEW in v1.3 ---
tokio-util = { version = "0.7", features = ["rt"] }
throbber-widgets-tui = "0.11"

[dev-dependencies]
# --- NEW in v1.3 (evaluate AGPL-3.0 license before adding) ---
# arch_test_core = "0.1.5"
```

`tokio-util` feature `"rt"` enables `CancellationToken` and `TaskTracker`; `"codec"` and others are not needed.

---

## Compatibility Matrix (New Additions Only)

| Package | Compatible With | Verified |
|---------|-----------------|---------|
| `throbber-widgets-tui 0.11` | `ratatui ^0.30.0` | YES — crate specifies `ratatui ^0.30.0` as dependency (docs.rs confirmed) |
| `throbber-widgets-tui 0.11` | MSRV 1.88.0 | YES — requires Rust 1.88+, up from ratatui's 1.86 floor |
| `tokio-util 0.7` | `tokio 1.x` | YES — tokio-util is the official companion crate, same team |
| `arch_test_core 0.1.5` | Any Rust project (uses `ra_ap_syntax`) | YES — static source analysis, no runtime |
| `cargo-deny 0.19` | macOS + Linux CI | YES — binary tool, no library dep |

---

## Sources

- https://docs.rs/throbber-widgets-tui/latest/throbber_widgets_tui/ — ratatui `^0.30.0` requirement confirmed, MSRV 1.88.0 (HIGH confidence)
- https://github.com/arkbig/throbber-widgets-tui — v0.11.0 released 2026-02-22 (HIGH confidence)
- https://docs.rs/tokio-util/0.7.18/tokio_util/sync/struct.CancellationToken.html — CancellationToken API, child_token propagation semantics (HIGH confidence)
- https://docs.rs/tokio-util/0.7.18/tokio_util/task/struct.TaskTracker.html — TaskTracker API (HIGH confidence)
- https://crates.io/crates/cargo-geiger — v0.13.0, active as of 2025-08-31 (HIGH confidence)
- https://crates.io/crates/cargo-deny — v0.19.1 (HIGH confidence)
- https://crates.io/crates/cargo-depgraph — v1.6.0, graphviz dependency graphs (HIGH confidence)
- https://docs.rs/arch_test_core — v0.1.5, AGPL-3.0, uses ra_ap_syntax for static analysis (MEDIUM confidence — last commit 2021, AGPL-3.0 flag)
- https://crates.io/crates/arch_test_core — 0.1.5 current version confirmed (MEDIUM confidence)
- cargo search results (local) — versions cross-checked against crates.io (HIGH confidence)
- Existing source audit: `src/app.rs`, `src/infra/command_runner.rs`, `src/action.rs` — confirmed existing 250ms tick, single `command_task: Option<JoinHandle<()>>`, no existing CancellationToken usage

---

*Stack additions for: rn-dash v1.3 — Per-Worktree Tasks + Architecture Audit*
*Researched: 2026-04-13*
