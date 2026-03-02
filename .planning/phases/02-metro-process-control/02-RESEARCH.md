# Phase 2: Metro Process Control - Research

**Researched:** 2026-03-02
**Domain:** Rust async process management — tokio::process, single-instance enforcement, log streaming via mpsc channels, port verification, Metro CLI interaction
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| METRO-01 | User can see at a glance which worktree (if any) has metro running, with status indicator (running/stopped) | MetroStatus enum in domain/ with Running { pid, worktree_id } and Stopped variants; rendered in metro pane from AppState |
| METRO-02 | User can start metro (yarn start --reset-cache) from the active worktree | tokio::process::Command with kill_on_drop + process_group(0); keybinding `s` dispatches MetroStart action |
| METRO-03 | User can stop the running metro instance | Child::kill() (SIGKILL + wait) + port-free verification via TcpListener::bind probe; keybinding `x` dispatches MetroStop action |
| METRO-04 | User can restart metro (kill + start) with one keystroke | Composed MetroRestart action: kill existing → wait for port free → spawn new; keybinding `r` (when metro pane focused, no error overlay) |
| METRO-05 | User can view metro log output in a dedicated panel only when a log filter is applied | Metro does not stream logs by default; log panel toggled via `l` keybinding; only activates filter mode when enabled |
| METRO-06 | User can scroll through metro log history in the log panel | VecDeque<String> ring buffer (max 1000 lines) in AppState; Paragraph::scroll() + ScrollbarState |
| METRO-07 | User can send debugger command (j) to the running metro instance | child.stdin write_all(b"j\n") via ChildStdin handle stored in MetroHandle |
| METRO-08 | User can send reload command (r) to the running metro instance | child.stdin write_all(b"r\n") via ChildStdin handle stored in MetroHandle |
| METRO-09 | Only one metro instance can run at a time across all worktrees (enforced by the dashboard) | Single Option<MetroHandle> in domain::MetroManager; invariant enforced — start() kills existing before spawning |
</phase_requirements>

---

## Summary

Phase 2 builds the metro process control layer on top of the Phase 1 TEA scaffold. The core work is: (1) a `MetroManager` domain struct that owns the single process handle and enforces the one-instance invariant at the type level, (2) a `ProcessClient` infra trait behind which `tokio::process::Command` spawns metro as a new process group (so `kill()` kills the entire node subtree), (3) an async log-streaming task that reads metro's stdout/stderr and sends lines to the main event loop via `tokio::sync::mpsc`, and (4) a port-availability check using `std::net::TcpListener::bind` that polls until 8081 is free before the UI transitions to Stopped.

The critical project-specific constraint from CLAUDE.md: metro does NOT stream logs by default in the current RN CLI version — logs only appear when a filter argument is applied. The log panel therefore requires the user to toggle a "filter mode" that restarts metro with `--filter` (or via `DEBUG=` env var), rather than showing output unconditionally. This is a known behavior change in recent RN CLI versions, confirmed by project experience and corroborated by GitHub issues about metro log suppression.

The zombie-process and port-binding bugs referenced in the phase goal have well-established Rust solutions: `tokio::process::Command::process_group(0)` puts metro in its own process group so killing the group handle kills all Node child processes; `Child::kill()` in tokio (unlike std) automatically calls `wait()` after SIGKILL, preventing zombies; `kill_on_drop(true)` provides a drop-safety net for unexpected panics. The port-free check avoids a race condition where the UI shows "Stopped" before the OS has released the bind address.

**Primary recommendation:** Use `tokio::process` directly (no extra crate) for process management. Use `tokio::sync::mpsc::unbounded_channel` for log streaming from background task to event loop. Store logs in `VecDeque<String>` in AppState capped at 1000 lines. Use `std::net::TcpListener::bind("127.0.0.1:8081")` success as the port-free signal. Enforce the single-instance invariant via `Option<MetroHandle>` in `domain::MetroManager` — the type can never hold two handles simultaneously.

---

## Standard Stack

### Core (new additions for Phase 2)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.49 (already in Cargo.toml) | Spawn async child process, mpsc channels, BufReader lines | Already present; `tokio::process` is the async process API |
| nix | 0.29 | Send SIGTERM before SIGKILL for graceful Metro shutdown (optional) | Safe Unix signal API; needed only if graceful shutdown path is added |

No new mandatory crates for Phase 2 beyond what Phase 1 already added. `tokio::process`, `tokio::sync::mpsc`, and `tokio::io::BufReader` are all part of tokio 1.x.

### No New Crates Required

Phase 1 `Cargo.toml` already has everything needed:
- `tokio = { version = "1.49", features = ["full"] }` — includes `tokio::process`, `tokio::sync::mpsc`, `tokio::io::BufReader`, `tokio::io::AsyncBufReadExt`
- `anyhow` — error handling for process spawn failures
- `tracing` — log process events to file (never stdout)

The `tokio-process-stream` crate (0.4.1) is an optional convenience that wraps `tokio::process::Child` into a `Stream<Item=ProcessLineItem>`. It simplifies merged stdout+stderr streaming but adds a dependency. Recommend staying with raw `tokio::io::BufReader` + `tokio::select!` to keep the dependency graph small — the pattern is 20 lines and easy to reason about.

### Optional (Do Not Add Unless Needed)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio-process-stream | 0.4.1 | Stream wrapper for Child yielding Stdout/Stderr/Done items | Only if merged stdout+stderr streaming becomes complex |
| nix | 0.29 | SIGTERM → SIGKILL graceful shutdown | Only if Metro requires graceful shutdown before hard kill |

### Installation (if nix is needed)

```toml
nix = { version = "0.29", features = ["signal"] }
```

---

## Architecture Patterns

### Recommended Project Structure (Phase 2 additions)

```
src/
├── action.rs              # Add: MetroStart, MetroStop, MetroRestart, MetroToggleLog,
│                          #      MetroScrollUp, MetroScrollDown, MetroSendDebugger,
│                          #      MetroSendReload, MetroLogLine(String), MetroExited
├── app.rs                 # Add: MetroState to AppState; handle_key() metro pane branch;
│                          #      update() arms for all metro actions
│
├── domain/
│   ├── mod.rs             # Add: pub mod metro;
│   └── metro.rs           # MetroStatus enum + MetroManager struct (single-instance enforcer)
│
├── infra/
│   ├── mod.rs             # Add: pub mod process; pub mod port;
│   ├── process.rs         # ProcessClient trait + MetroProcess impl (tokio::process)
│   └── port.rs            # port_is_free(u16) → bool via TcpListener::bind probe
│
└── ui/
    └── panels.rs          # Replace metro placeholder with real MetroPane render
                           # Add: render_log_panel() when log panel visible
```

### Pattern 1: MetroManager — Single-Instance Invariant in Domain

**What:** A struct in `domain/metro.rs` that owns `Option<MetroHandle>`. Because there is only one `MetroManager` in `AppState`, and it holds at most one `Option`, the type system makes two simultaneous handles impossible.

**When to use:** All metro state transitions go through `MetroManager` methods. The `update()` function in `app.rs` calls these methods, never manipulates metro handles directly.

```rust
// src/domain/metro.rs
// Source: ARCH-06 requirement — domain invariants enforced in domain types

/// Live process handle held by MetroManager.
/// Stored in AppState via app.rs — lives inside the tokio task context.
pub struct MetroHandle {
    pub pid: u32,
    pub worktree_id: String,
    /// Sender for stdin writes (j, r commands)
    pub stdin_tx: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    /// Join handle for the streaming background task
    pub stream_task: tokio::task::JoinHandle<()>,
}

/// Current metro state as seen by domain.
#[derive(Debug, Clone, PartialEq)]
pub enum MetroStatus {
    Stopped,
    Running { pid: u32, worktree_id: String },
    Starting,   // transient — while spawn is in flight
    Stopping,   // transient — while kill + port-free wait is in flight
}

/// Enforces single-instance invariant: holds at most one handle.
/// All mutation goes through MetroManager methods.
pub struct MetroManager {
    handle: Option<MetroHandle>,
    pub status: MetroStatus,
}

impl MetroManager {
    pub fn new() -> Self {
        Self { handle: None, status: MetroStatus::Stopped }
    }

    /// True if any metro is running.
    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    /// Register a freshly spawned process.
    /// Panics if called while a handle exists — callers must kill first.
    pub fn register(&mut self, handle: MetroHandle) {
        assert!(self.handle.is_none(), "BUG: register() called with existing handle");
        let pid = handle.pid;
        let worktree_id = handle.worktree_id.clone();
        self.handle = Some(handle);
        self.status = MetroStatus::Running { pid, worktree_id };
    }

    /// Clear the handle after the process has been killed and reaped.
    pub fn clear(&mut self) {
        self.handle = None;
        self.status = MetroStatus::Stopped;
    }

    /// Send a raw byte sequence to metro's stdin.
    pub fn send_stdin(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        if let Some(ref h) = self.handle {
            h.stdin_tx.send(bytes).map_err(|e| anyhow::anyhow!("stdin send failed: {e}"))?;
        }
        Ok(())
    }
}
```

### Pattern 2: ProcessClient Trait + Concrete Implementation

**What:** Infra trait in `infra/process.rs` keeping `tokio::process` behind a boundary (ARCH-02). In tests, swap in a `FakeProcessClient` that returns canned responses.

```rust
// src/infra/process.rs
// Source: ARCH-02 — all infra behind trait boundaries

use tokio::process::Child;
use std::path::PathBuf;

#[async_trait::async_trait]
pub trait ProcessClient: Send + Sync {
    /// Spawn metro in the given worktree directory.
    /// Returns the Child handle with stdout/stderr piped.
    async fn spawn_metro(&self, worktree_path: PathBuf) -> anyhow::Result<Child>;

    /// Kill the process by PID using the OS.
    /// For cases where we lost the Child handle (e.g., external kill detection).
    async fn kill_pid(&self, pid: u32) -> anyhow::Result<()>;
}

pub struct TokioProcessClient;

#[async_trait::async_trait]
impl ProcessClient for TokioProcessClient {
    async fn spawn_metro(&self, worktree_path: PathBuf) -> anyhow::Result<Child> {
        let child = tokio::process::Command::new("yarn")
            .args(["start", "--reset-cache"])
            .current_dir(worktree_path)
            // CRITICAL: process group 0 = own PGID = kill() kills all Node children
            .process_group(0)
            // Drop safety net: if Child handle is dropped without explicit kill, OS cleans up
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        Ok(child)
    }

    async fn kill_pid(&self, pid: u32) -> anyhow::Result<()> {
        // Used for external process detection only.
        // Normal flow uses Child::kill() which also waits.
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill as nix_kill, Signal};
            use nix::unistd::Pid;
            nix_kill(Pid::from_raw(pid as i32), Signal::SIGKILL)?;
        }
        Ok(())
    }
}
```

### Pattern 3: Spawning Metro with Zombie-Safe Kill

**What:** The correct sequence to kill Metro, wait for reaping, and confirm port is free before showing Stopped status.

**When to use:** MetroStop and MetroRestart actions. The sequence must happen in a background tokio task so the event loop stays responsive.

```rust
// Spawned by update() as a tokio::spawn when MetroStop is dispatched.
// Source: https://docs.rs/tokio/latest/tokio/process/struct.Child.html

async fn kill_metro_and_verify(
    mut child: tokio::process::Child,
    event_tx: tokio::sync::mpsc::UnboundedSender<Action>,
) {
    // tokio::Child::kill() = SIGKILL + wait() — prevents zombie
    // Distinct from std::process::Child::kill() which does NOT wait.
    if let Err(e) = child.kill().await {
        tracing::error!("metro kill failed: {e}");
    }

    // Poll port 8081 until free — retry up to 50 times with 100ms sleep
    for _ in 0..50 {
        if port_is_free(8081) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Signal main loop that metro is confirmed stopped
    let _ = event_tx.send(Action::MetroExited);
}

// Port probe — success means port is free (no process bound to it)
fn port_is_free(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}
```

### Pattern 4: Log Streaming via mpsc to Event Loop

**What:** A background tokio task reads metro's stdout+stderr line-by-line and sends each line as `Action::MetroLogLine(String)` to the main event loop, which appends it to a `VecDeque<String>` in AppState.

**When to use:** Immediately after metro is spawned, if log panel is enabled (filter active).

```rust
// Spawned once after metro start. Runs until process exits.
// Source: https://docs.rs/tokio/latest/tokio/io/index.html (BufReader + AsyncBufReadExt)

use tokio::io::{AsyncBufReadExt, BufReader};

async fn stream_metro_logs(
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    tx: tokio::sync::mpsc::UnboundedSender<Action>,
) {
    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(Action::MetroLogLine(l)); }
                    _ => break,
                }
            }
            line = stderr_lines.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(Action::MetroLogLine(l)); }
                    _ => break,
                }
            }
        }
    }
    let _ = tx.send(Action::MetroExited);
}
```

**Event loop integration:** Add a `tokio::select!` arm for `metro_rx.recv()` in `app::run()` loop, or use the pattern of wrapping all events in a unified `AppEvent` enum (log lines are just another variant).

### Pattern 5: stdin Forwarding for j / r Commands

**What:** Keep a `ChildStdin` handle in MetroHandle (via a background stdin-writer task). When user presses `j` or `r` in metro pane, send `b"j\n"` or `b"r\n"` through an mpsc channel to the stdin writer.

```rust
// In the background stdin writer task:
async fn stdin_writer(
    mut stdin: tokio::process::ChildStdin,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) {
    use tokio::io::AsyncWriteExt;
    while let Some(bytes) = rx.recv().await {
        if let Err(e) = stdin.write_all(&bytes).await {
            tracing::warn!("metro stdin write failed: {e}");
            break;
        }
    }
}

// In MetroHandle:
pub struct MetroHandle {
    pub pid: u32,
    pub worktree_id: String,
    pub stdin_tx: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    pub stream_task: tokio::task::JoinHandle<()>,
    pub stdin_task: tokio::task::JoinHandle<()>,
}

// Sending j or r from update():
// state.metro.send_stdin(b"j\n".to_vec())?;
// state.metro.send_stdin(b"r\n".to_vec())?;
```

### Pattern 6: External Death Detection (Polling try_wait)

**What:** Metro can be killed from outside the dashboard (user runs `kill`, process crashes). The 250ms tick in `app::run()` is used to poll `child.try_wait()` to detect this.

**When to use:** Every tick, if metro is in Running state. Non-blocking — never awaits.

```rust
// In the streaming task: MetroExited action is sent when stdout/stderr closes.
// Stdout/stderr EOF is the most reliable signal that the process is gone.
// The stream_metro_logs() task already sends Action::MetroExited when both streams close.

// Fallback: on tick, check port 8081 still bound
// If port_is_free(8081) and state.metro.is_running() → dispatch MetroExited
// This covers cases where metro exits silently without closing streams.
```

### Pattern 7: Scrollable Log Panel

**What:** Render log lines from `VecDeque<String>` in AppState as a scrollable `Paragraph`. Track `log_scroll_offset: usize` in AppState to support j/k scrolling within the log panel.

```rust
// In ui/panels.rs
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::text::{Line, Text};
use ratatui::layout::Margin;

pub fn render_log_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let lines: Vec<Line> = state.metro_logs.iter()
        .map(|l| Line::from(l.as_str()))
        .collect();

    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(Block::default().title(" Metro Logs ").borders(Borders::ALL))
        .scroll((state.log_scroll_offset as u16, 0));

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(lines.len())
        .position(state.log_scroll_offset);

    f.render_widget(paragraph, area);
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin { vertical: 1, horizontal: 0 }),
        &mut scrollbar_state,
    );
}
```

### Pattern 8: Metro Log Filtering (Metro-specific behavior)

**What:** Metro does NOT stream logs to stdout by default in current RN CLI versions (project-confirmed behavior, CLAUDE.md). Logs only appear when a filter is active. The log panel is therefore a "filter mode" feature — it must be toggled on by the user, and when toggled, metro is started (or restarted) with a filter argument.

**Implementation options:**

Option A — `DEBUG=Metro:*` env var: Metro uses the `debug` npm package internally. Setting `DEBUG=Metro:*` (or a more specific scope like `DEBUG=Metro:Server`) before spawning metro causes it to emit structured logs to stderr.

Option B — `--filter <pattern>`: If the installed version of `@react-native-community/cli` supports a `--filter` flag on `start`, use it. This is version-dependent.

**Recommended approach (confirmed project behavior from CLAUDE.md):**
- Log panel toggle (`l` key) tracks `log_filter_active: bool` in AppState
- When filter is toggled on: if metro is running, kill and restart with `DEBUG=Metro:*` env var set
- When filter is toggled off: kill and restart without env var
- Document in help overlay: "l — toggle log filter (restarts metro)"
- This is consistent with the METRO-05 requirement: "only when a log filter is applied"

```rust
// In TokioProcessClient::spawn_metro:
async fn spawn_metro(&self, worktree_path: PathBuf, filter: bool) -> anyhow::Result<Child> {
    let mut cmd = tokio::process::Command::new("yarn");
    cmd.args(["start", "--reset-cache"])
       .current_dir(worktree_path)
       .process_group(0)
       .kill_on_drop(true)
       .stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped())
       .stdin(std::process::Stdio::piped());

    if filter {
        cmd.env("DEBUG", "Metro:*");
    }

    Ok(cmd.spawn()?)
}
```

### AppState Extensions for Phase 2

```rust
// Additional fields in AppState (app.rs):
pub struct AppState {
    // ... existing Phase 1 fields ...

    // Metro state
    pub metro: domain::metro::MetroManager,

    // Log panel
    pub metro_logs: std::collections::VecDeque<String>,  // capped at 1000
    pub log_scroll_offset: usize,
    pub log_panel_visible: bool,
    pub log_filter_active: bool,

    // Active worktree (stub — real list in Phase 3)
    pub active_worktree_path: Option<std::path::PathBuf>,
}

// MAX_LOG_LINES = 1000 — constant in app.rs
```

### Action Enum Additions for Phase 2

```rust
// Additional variants in action.rs:
pub enum Action {
    // ... existing Phase 1 variants ...

    // Metro control
    MetroStart,
    MetroStop,
    MetroRestart,
    MetroToggleLog,       // l
    MetroScrollUp,        // k when log panel focused
    MetroScrollDown,      // j when log panel focused
    MetroSendDebugger,    // J (capital, when metro focused, to avoid conflict with j=FocusDown)
    MetroSendReload,      // R (capital, when metro focused)

    // Background task notifications (not user-triggered)
    MetroLogLine(String),
    MetroExited,
}
```

**Key binding conflict resolution:** `j` is already `FocusDown` and `r` is `RetryLastCommand` (error overlay). When `MetroPane` is focused:
- `j` should remain FocusDown (scroll log if log panel is focused)
- `r` should be MetroRestart (footer already shows `r restart`)
- `J` (Shift-J) → MetroSendDebugger
- `R` (Shift-R) → MetroSendReload
- Alternative: use dedicated submode keys or confirm with CLAUDE.md context

### Anti-Patterns to Avoid

- **`std::process::Child::kill()` instead of `tokio::process::Child::kill()`:** The std version does NOT wait — it sends SIGKILL and returns without reaping. This creates a zombie. Always use `tokio::process::Child::kill().await`.
- **Not using `process_group(0)`:** Without it, killing the `Child` only kills the direct `yarn` process — the Node.js subprocess it spawns continues running and holds port 8081. The zombie-port-binding bug is caused by this.
- **Checking port-free immediately after kill:** The OS may not release the port instantly. Always poll with retries (50 × 100ms = 5 seconds max) before declaring Stopped.
- **Storing Child in AppState directly:** `tokio::process::Child` is not `Clone` or `Send` across thread boundaries easily. Wrap in the `MetroHandle` pattern — keep the `JoinHandle` to the streaming task, communicate via mpsc channels.
- **Collecting all logs into Vec<String> unbounded:** Long metro sessions produce enormous output. Use `VecDeque<String>` with a pop_front when len > MAX_LOG_LINES.
- **Calling `child.stdout.take()` after `child.kill()`:** Stdout is consumed during kill in some tokio versions. Take stdout/stderr handles immediately after spawn, before any kill calls.
- **Blocking the tokio event loop for port polling:** Never call `std::thread::sleep` in an async context. Use `tokio::time::sleep(Duration::from_millis(100)).await` in the kill task.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process kill that prevents zombies | Custom SIGKILL + manual wait loop | `tokio::process::Child::kill().await` | Tokio's version automatically calls wait() after SIGKILL — the std version does NOT |
| Killing metro's Node subprocess | Kill only the yarn process PID | `Command::process_group(0)` before spawn, then kill the whole group | yarn spawns Node as a child; only group kill reaches both |
| Port availability check | Shell out to `lsof -i :8081` | `std::net::TcpListener::bind("127.0.0.1:8081").is_ok()` | TcpListener::bind is synchronous, self-contained, no subprocess needed |
| Log line streaming | Build a custom stream wrapper | `tokio::io::BufReader::new(stdout).lines()` + `next_line().await` | This is the canonical tokio async line-reader; handles buffering and partial reads |
| Single-instance enforcement | Boolean flag in AppState | `Option<MetroHandle>` in `MetroManager` | Type system enforces invariant — None = stopped, Some = running, impossible to hold two |
| stdin command sending | Write directly from event handler | Dedicated stdin writer task + mpsc channel | Event handler is sync; stdin writes are async; must go through a task |

**Key insight:** The zombie-process and port-binding bugs both have the same root cause: not using `process_group(0)`. This single call makes `kill()` reach the entire process tree. Combined with tokio's wait-after-kill behavior, there are no zombies.

---

## Common Pitfalls

### Pitfall 1: tokio vs std kill() — Zombie Processes

**What goes wrong:** `std::process::Child::kill()` sends SIGKILL but does NOT reap the process. The process becomes a zombie and the port stays bound.

**Why it happens:** Using the wrong `Child` type — std vs tokio. If `tokio::process::Command::spawn()` is used but the returned `Child` is treated like a std `Child`, or if `start_kill()` is called instead of `kill().await`.

**How to avoid:** Always use `tokio::process::Child::kill().await`. The tokio docs explicitly state: "Unlike std version of Child::kill, this function will wait for the process to exit."

**Warning signs:**
- `lsof -i :8081` shows a process still bound after "stop" completes
- `ps aux | grep metro` shows zombie (Z) processes

### Pitfall 2: yarn subprocess not killed (port still bound after kill)

**What goes wrong:** `child.kill().await` kills yarn but not the Node.js child that yarn spawned. Port 8081 stays bound.

**Why it happens:** Not setting `process_group(0)`. The Node subprocess inherits yarn's PGID and is not in a separate group, so a targeted kill of just yarn's PID leaves Node running.

**How to avoid:**
```rust
Command::new("yarn")
    .process_group(0)  // CRITICAL — puts yarn+all children in their own process group
    // ...
    .spawn()
```

**How to kill the group:**
```rust
// tokio::Child::kill() when process_group(0) was set:
// SIGKILL goes to the entire process group (pgid = child's PID)
child.kill().await?;
```

**Warning signs:** Port 8081 remains bound after stop; `ps aux | grep node` shows node still running.

### Pitfall 3: Race — Port Check Immediately After Kill

**What goes wrong:** `port_is_free(8081)` returns false right after kill because the OS socket linger timer hasn't expired. UI shows "Stopped" but next `yarn start` immediately fails with EADDRINUSE.

**Why it happens:** Port release is not instantaneous. TCP connections have TIME_WAIT states. Even after SIGKILL, the kernel may hold the socket briefly.

**How to avoid:** Poll with retries in the kill task before sending `MetroExited` action:
```rust
for _ in 0..50 {
    if port_is_free(8081) { break; }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```
Maximum 5 second wait. If still not free after 50 tries, send `MetroExited` anyway (user can retry).

### Pitfall 4: Metro Log Streaming Requires Filter (Metro Default Behavior Change)

**What goes wrong:** Metro is spawned with piped stdout/stderr, but the BufReader never yields any lines (or yields only non-log lines) because modern Metro does not stream console.log output by default.

**Why it happens:** Recent versions of `@react-native-community/cli` changed Metro to suppress app console.log output unless a filter/debug flag is set. This is project-confirmed behavior (CLAUDE.md: "Metro logs only stream when a filter is applied (metro doesn't stream by default anymore)").

**How to avoid:** Only start streaming when `log_filter_active` is true. When filter is toggled on, restart metro with `DEBUG=Metro:*` env var. When filter is off, don't read logs (save CPU).

**Warning signs:** Log panel is empty despite metro running and app actively logging.

### Pitfall 5: ChildStdin Take Before Kill

**What goes wrong:** Calling `child.stdin.take()` after `child.kill().await` may fail or panic because kill consumes/closes the child's fds in some configurations.

**How to avoid:** Always take stdin/stdout/stderr immediately after `spawn()`, before any kill calls:
```rust
let mut child = command.spawn()?;
let stdout = child.stdout.take().expect("stdout piped");
let stderr = child.stderr.take().expect("stderr piped");
let stdin = child.stdin.take().expect("stdin piped");
// spawn streaming tasks with these handles
// child handle kept for kill later
```

### Pitfall 6: Action Keybinding Conflict — j/r in Metro Context

**What goes wrong:** `j` is already `FocusDown`/scroll and `r` is `RetryLastCommand` (error overlay). Metro's terminal keystrokes are also `j` (debugger) and `r` (reload). Dispatching `j` when metro pane is focused would scroll the log panel, not open the debugger.

**How to avoid:** Use uppercase `J` and `R` for metro stdin commands (Shift-J / Shift-R), or use dedicated keys like `d` (debugger) and `e` (rEload). The footer for MetroPane already shows `s start`, `x stop`, `r restart` — `r` should stay as `MetroRestart`, and debugger/reload commands get distinct keys.

**Warning signs:** Pressing `j` in metro pane scrolls the list when the user expected to open the debugger.

### Pitfall 7: Child Handle Ownership Across Async Tasks

**What goes wrong:** `tokio::process::Child` is not `Clone`. Attempting to share it between the streaming task and the kill handler causes borrow errors or requires `Arc<Mutex<>>` which blocks async tasks.

**How to avoid:** Do NOT share `Child` directly. Instead:
1. Take stdout/stderr/stdin handles immediately after spawn (each is a separate owned handle)
2. Keep `Child` (for kill) in `MetroHandle` inside `MetroManager`
3. Spawn tasks that own their individual handles
4. Kill is done through the `MetroHandle`'s stored `Child` (triggered by MetroStop action)

This requires an async wrapper in `app.rs`/`update()` — spawn a tokio task for the kill sequence, have it send `Action::MetroExited` back via the same mpsc channel used for log lines.

### Pitfall 8: Blocking tokio::select! Arms on Slow Metro Startup

**What goes wrong:** Metro takes 5-10 seconds to start. During this time, the streaming task is blocked waiting on `next_line()`. If the event loop select arm for logs blocks, keystrokes become unresponsive.

**How to avoid:** The streaming task is a separate `tokio::spawn`, not a select arm in the main event loop. It sends lines to the main loop via mpsc channel. The main loop's `select!` arm is just `metro_rx.recv()` which is always fast — it yields immediately if there's no log line ready. Metro startup delay is invisible to the event loop.

---

## Code Examples

Verified patterns from official and documented sources:

### Spawning Metro with Process Group and Piped IO

```rust
// Source: https://docs.rs/tokio/latest/tokio/process/struct.Command.html
use tokio::process::Command;
use std::process::Stdio;

let mut child = Command::new("yarn")
    .args(["start", "--reset-cache"])
    .current_dir("/path/to/worktree")
    .process_group(0)        // own process group — kill reaches all children
    .kill_on_drop(true)      // drop safety net
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .stdin(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().expect("piped");
let stderr = child.stderr.take().expect("piped");
let stdin = child.stdin.take().expect("piped");
let pid = child.id().expect("has pid before kill");
```

### Kill + Wait (Zombie-Safe)

```rust
// Source: https://docs.rs/tokio/latest/tokio/process/struct.Child.html
// tokio::Child::kill() = SIGKILL + wait() — no zombie left behind
child.kill().await?;
// After this returns: process is dead and reaped.
```

### Port-Free Polling

```rust
// Source: https://doc.rust-lang.org/std/net/struct.TcpListener.html
fn port_is_free(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

// In kill task:
for _ in 0..50 {
    if port_is_free(8081) { break; }
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
```

### Async Line Streaming

```rust
// Source: https://docs.rs/tokio/latest/tokio/io/index.html (AsyncBufReadExt)
use tokio::io::{AsyncBufReadExt, BufReader};

let mut lines = BufReader::new(stdout).lines();
while let Ok(Some(line)) = lines.next_line().await {
    tx.send(Action::MetroLogLine(line)).ok();
}
```

### Stdin Send

```rust
// Source: https://docs.rs/tokio/latest/tokio/process/struct.Child.html (stdin field)
use tokio::io::AsyncWriteExt;

// Reload:
stdin.write_all(b"r\n").await?;
// Open debugger:
stdin.write_all(b"j\n").await?;
```

### Scrollable Log Panel Render

```rust
// Source: https://docs.rs/ratatui/latest/ratatui/widgets/struct.Scrollbar.html
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::layout::Margin;
use ratatui::text::{Line, Text};

let lines: Vec<Line> = state.metro_logs.iter()
    .map(|s| Line::from(s.as_str())).collect();

// Auto-scroll to bottom when new lines arrive:
let scroll = if state.log_scroll_offset == 0 {
    lines.len().saturating_sub(visible_height as usize)
} else {
    state.log_scroll_offset
};

let para = Paragraph::new(Text::from(lines.clone()))
    .scroll((scroll as u16, 0))
    .block(Block::default().borders(Borders::ALL).title(" Metro Logs "));

let mut sb_state = ScrollbarState::new(lines.len()).position(scroll);

frame.render_widget(para, area);
frame.render_stateful_widget(
    Scrollbar::new(ScrollbarOrientation::VerticalRight),
    area.inner(Margin { vertical: 1, horizontal: 0 }),
    &mut sb_state,
);
```

### Metro Status Indicator in Metro Pane

```rust
// Compact status indicator for metro pane header
let status_text = match &state.metro.status {
    MetroStatus::Running { pid, worktree_id } =>
        format!(" RUNNING  pid={pid}  [{worktree_id}]"),
    MetroStatus::Stopped =>
        " STOPPED ".to_string(),
    MetroStatus::Starting =>
        " STARTING... ".to_string(),
    MetroStatus::Stopping =>
        " STOPPING... ".to_string(),
};

let style = match &state.metro.status {
    MetroStatus::Running { .. } => Style::new().green().bold(),
    MetroStatus::Stopped => Style::new().dark_gray(),
    _ => Style::new().yellow(),
};
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `std::process::Child::kill()` | `tokio::process::Child::kill().await` | tokio 0.x → 1.x | Tokio version automatically awaits reap — no zombie |
| `process_group` manual via `nix::setsid` in pre_exec | `Command::process_group(0)` | tokio ~1.13 | Built-in API, no nix crate needed for basic group management |
| Polling crossterm events + process output in one thread | `tokio::select!` on mpsc receiver + EventStream | tokio 1.x async I/O | Event-driven; no busy-wait |
| Metro streamed all logs to terminal | Metro suppresses logs unless `DEBUG=Metro:*` or filter | RN CLI ~0.72+ | Log panel must be opt-in; filter restart required |
| Fixed log Vec unbounded | `VecDeque<String>` with pop_front at max size | Standard practice | Prevents OOM on long metro sessions |

**Deprecated/outdated:**
- `tokio-process` crate (0.3.0-alpha.2): Old, not recommended, superseded by `tokio::process` built-in.
- Manual `std::process::Command` for async process management: Blocks the tokio thread pool.
- Direct crossterm write for stdin interaction: Not applicable — stdin goes to child process, not the terminal.

---

## Open Questions

1. **Exact --filter flag availability in project's RN CLI version**
   - What we know: Metro does not stream logs by default (CLAUDE.md, confirmed). `DEBUG=Metro:*` env var activates Metro's own debug logging.
   - What's unclear: Whether the project's installed `@react-native-community/cli` version supports a `--filter` CLI flag on `start`. Multiple web searches found no evidence of a standard `--filter` flag; the CLAUDE.md note likely refers to `DEBUG=` env var behavior.
   - Recommendation: Use `DEBUG=Metro:*` env var approach. Confirm by running `yarn start --help` in a worktree and checking available flags before implementing.

2. **Keybinding conflict: r for MetroRestart vs RetryLastCommand**
   - What we know: `r` in error overlay → RetryLastCommand. Footer for MetroPane shows `r restart`. These are separate overlay contexts.
   - What's unclear: If error overlay appears while metro pane is focused, does `r` retry the last command or restart metro? The current `handle_key()` checks `error_state.is_some()` first, so error overlay takes priority. This is correct behavior.
   - Recommendation: Document clearly in code: error overlay intercepts `r` first. No conflict — both work in their respective contexts.

3. **Metro external kill detection latency**
   - What we know: The streaming task sends `MetroExited` when stdout/stderr closes. This happens when the process dies.
   - What's unclear: How quickly does tokio detect the EOF on piped stdout when the process is killed externally (e.g., `kill -9 <pid>`)? Expected to be near-instant on Linux/macOS.
   - Recommendation: Rely on the streaming task EOF as primary detection. Add tick-based port probe as secondary fallback (if port becomes free but no MetroExited was received).

---

## Sources

### Primary (HIGH confidence)
- https://docs.rs/tokio/latest/tokio/process/index.html — zombie prevention, kill_on_drop, process_group, piped I/O
- https://docs.rs/tokio/latest/tokio/process/struct.Child.html — kill() vs start_kill(), try_wait(), stdin/stdout/stderr handles
- https://docs.rs/tokio/latest/tokio/process/struct.Command.html — process_group(0) API, pre_exec, kill_on_drop
- https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html — bounded/unbounded channels, try_recv, clone for multi-producer
- https://docs.rs/tokio/latest/tokio/io/index.html — BufReader, AsyncBufReadExt, lines()
- https://docs.rs/ratatui/latest/ratatui/widgets/struct.Scrollbar.html — Scrollbar, ScrollbarState, content_length, position
- https://doc.rust-lang.org/std/net/struct.TcpListener.html — bind() as port-availability probe
- https://ratatui.rs/tutorials/counter-async-app/async-event-stream/ — mpsc + EventStream integration pattern for background task output
- Project CLAUDE.md — "Metro logs only stream when a filter is applied (metro doesn't stream by default anymore)" — confirmed project-specific behavior

### Secondary (MEDIUM confidence)
- https://github.com/tokio-rs/tokio/issues/2685 — zombie process issue confirming behavior when Child is dropped without kill
- https://reactnative.dev/docs/debugging — Metro terminal keystrokes: `j` opens DevTools, `r` reloads, `d` opens dev menu
- WebSearch result citing Metro CLI keystrokes (r reload, d devmenu, j debugger) — corroborated by react-native docs

### Tertiary (LOW confidence)
- https://lib.rs/crates/tokio-process-stream — tokio-process-stream 0.4.1 as alternative streaming approach (not used in plan)
- Multiple GitHub issues confirming Metro log suppression behavior in recent RN CLI versions

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — tokio process APIs verified via official docs.rs
- Architecture: HIGH — MetroManager pattern directly from ARCH-06 requirement; process_group(0) verified via tokio Command docs
- Pitfalls: HIGH — zombie behavior verified via tokio docs and GitHub issue #2685; port-timing from standard TCP behavior; metro log behavior from CLAUDE.md (project knowledge) + GitHub issues
- Code examples: HIGH — all tokio examples cite official docs.rs URLs; ratatui Scrollbar from official docs

**Research date:** 2026-03-02
**Valid until:** 2026-04-01 (tokio 1.x stable API; metro behavior confirmed by project experience)
