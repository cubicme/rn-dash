# Pitfalls Research

**Domain:** Rust TUI process management dashboard (Ratatui + Tokio + tmux + git + JIRA API)
**Researched:** 2026-03-02
**Confidence:** HIGH (multiple verified sources including official Ratatui docs, Tokio docs, Atlassian developer docs)

---

## Critical Pitfalls

### Pitfall 1: Terminal Left in Raw Mode on Panic or Error Exit

**What goes wrong:**
If the app crashes (panic) or exits through an error path without running cleanup code, the terminal remains in raw mode and/or the alternate screen. The user is left with a broken shell where input is invisible and echo is disabled. The only recovery is typing `reset` blindly.

**Why it happens:**
Ratatui requires enabling raw mode and switching to the alternate screen on startup. If any code path between startup and shutdown panics or returns an error without calling `disable_raw_mode()` and `LeaveAlternateScreen`, the terminal is never restored. This includes panics in widget rendering, child process handlers, or async tasks.

**How to avoid:**
Use `ratatui::init()` and `ratatui::restore()` (introduced in Ratatui 0.28.1). These convenience functions automatically install a panic hook that calls restore before the panic message is printed. For manual setup, install a panic hook at startup:
```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    ratatui::restore(); // disable raw mode, leave alternate screen
    original_hook(info);
}));
```
Also wrap the main run loop in a result handler that calls `restore()` before propagating errors.

**Warning signs:**
- Terminal appears broken after a test run crashes
- Debugging with `println!` causes display artifacts
- Pressing Enter after crash shows no prompt
- Any code path that can return early (e.g., config load failure) before the main loop is entered

**Phase to address:**
Phase 1 (Core TUI shell setup) — establish the terminal init/restore pattern before writing any feature code. Every subsequent phase inherits the correct pattern.

---

### Pitfall 2: Zombie Metro Process After Kill (Port 8081 Stays Occupied)

**What goes wrong:**
When the dashboard kills the metro process to switch worktrees, the process is sent SIGKILL (or SIGTERM) but never waited on. The process lingers as a zombie, port 8081 remains bound, and the new metro in the target worktree fails to start with "port already in use."

**Why it happens:**
On Unix, a killed child process that has not been reaped by its parent becomes a zombie. `tokio::process::Child::kill()` sends the signal but does not wait for the process to exit. If the `Child` handle is dropped before `.wait().await` is called, the Tokio runtime will attempt cleanup on a best-effort basis only — there are no timing guarantees. The port is not released until the process fully exits and is reaped.

**How to avoid:**
Always await the child process after killing it:
```rust
child.kill().await?;
child.wait().await?; // reap the process
```
Also set `kill_on_drop(true)` on the `Command` so that if the `Child` handle is unexpectedly dropped (e.g., during error handling), the process is sent SIGKILL. Store the `Child` handle in application state for the lifetime of the metro process, not as a local variable.

**Warning signs:**
- `lsof -i :8081` shows the port still bound after a kill
- Metro fails to start in a different worktree with "EADDRINUSE"
- Process appears in `ps aux | grep metro` after the dashboard kills it

**Phase to address:**
Phase 2 (Metro process management) — design the ProcessManager component with correct kill+wait semantics from day one. Do not prototype with fire-and-forget kills.

---

### Pitfall 3: Blocking I/O on the Async Runtime (UI Freeze)

**What goes wrong:**
The TUI stops responding to key input, the render loop stalls, and the application appears frozen. This happens when a synchronous blocking call is made directly on a Tokio async worker thread — for example, reading stdout from a child process using `std::io::BufRead`, making a synchronous HTTP request, or calling `std::process::Command::output()` (which blocks until the process exits).

**Why it happens:**
Tokio is a cooperative async runtime. If a task on a worker thread calls a blocking system call without yielding, all other tasks scheduled on that thread starve. A `metro start` process can run indefinitely, so reading its stdout synchronously blocks forever. JIRA API calls over the network are also blocking if done with a synchronous HTTP client.

**How to avoid:**
- Use `tokio::process::Command` (not `std::process::Command`) for child processes
- Use `tokio::io::BufReader` to read stdout/stderr line-by-line asynchronously
- Use `reqwest` with the async feature enabled for JIRA API calls
- For any unavoidable blocking work, wrap in `tokio::task::spawn_blocking`
- Read child process stdout in a dedicated `tokio::spawn` task that sends lines through a `tokio::sync::mpsc` channel to the app state

**Warning signs:**
- Key presses lag behind or are processed in bursts
- The render loop stops updating during a long-running command
- `tokio-console` shows tasks stuck in "poll" state for hundreds of milliseconds
- Using `std::process::Command` anywhere in async code

**Phase to address:**
Phase 2 (Metro process management) and Phase 3 (Git/RN command execution) — establish the async I/O pattern in Phase 2 so Phase 3 inherits it. Never use sync I/O in async contexts.

---

### Pitfall 4: tmux send-keys Race Condition with Shell Initialization

**What goes wrong:**
The dashboard creates a new tmux window or pane, then immediately sends a command via `tmux send-keys`. The command text appears in the pane but is never executed because the shell (zsh) has not finished initializing when the keys are sent. The shell becomes interactive after the keys were already flushed, and the terminal shows the keys as typed text that was never submitted.

**Why it happens:**
`tmux send-keys` is fire-and-forget from tmux's perspective. The shell in a new pane goes through `.zshrc`/`.zprofile` initialization before it displays a prompt and is ready to accept input. There is a race between pane creation and shell readiness. This is a confirmed issue affecting even Claude Code's own agent spawning in tmux.

**How to avoid:**
Use `tmux split-window "command"` or `tmux new-window "command"` to pass the command as the initial shell command rather than sending keys after creation. For cases where send-keys is necessary (e.g., sending `j` to an already-running metro for "open debugger"), add a brief delay or poll for shell readiness by checking if the pane content contains a prompt string before sending keys.

**Warning signs:**
- Commands typed into a new tmux pane appear but don't execute
- Claude Code launch opens a pane but doesn't start the agent
- Intermittent failures where sometimes the command runs and sometimes it doesn't (timing-dependent)

**Phase to address:**
Phase 5 (Claude Code integration and tmux window management) — design the tmux integration to use `new-window "command"` style rather than send-keys where possible. Test shell initialization timing explicitly.

---

### Pitfall 5: Crossterm/Ratatui Version Mismatch Causes Invisible Bugs

**What goes wrong:**
Adding `crossterm` as a direct dependency (alongside `ratatui`) pulls in a different major version of crossterm than the one ratatui uses internally. The result is two copies of crossterm in the binary. Static global state (e.g., stored termios settings for raw mode) is duplicated, causing raw mode to be enabled by one copy and disabled by another, or key events to come from a different type than expected. The bug manifests as garbled rendering, key events that never fire, or raw mode not being properly restored.

**Why it happens:**
Ratatui bundles crossterm as a dependency and re-exports it. If `Cargo.toml` lists `crossterm = "0.28"` while ratatui uses `crossterm = "0.29"` (or vice versa), cargo resolves both versions independently and includes both. Type system conflicts may not surface at compile time since both versions share the same crate name.

**How to avoid:**
Do not add `crossterm` as a direct dependency. Import crossterm through ratatui's re-export: `use ratatui::crossterm::...`. If crossterm-specific features are needed, use ratatui's `crossterm_{version}` feature flags to select the version. Remove any standalone `crossterm` entry from `Cargo.toml`.

**Warning signs:**
- `cargo tree | grep crossterm` shows two different versions
- Raw mode cleanup behavior is inconsistent between runs
- Key events are delivered as the wrong enum variant
- Compile errors mentioning "expected type crossterm::X, found crossterm::X"

**Phase to address:**
Phase 1 (Project scaffold) — set up `Cargo.toml` correctly before writing any feature code. Audit with `cargo tree` before first commit.

---

## Moderate Pitfalls

### Pitfall 6: Stdout/Stderr Pipe Deadlock When Reading Child Output

**What goes wrong:**
The dashboard spawns a long-running command (e.g., `yarn install`) and tries to capture its stdout while also capturing stderr. If stdout is consumed but stderr is not (or vice versa), and the child process fills the stderr pipe buffer, the child blocks waiting for the buffer to drain. The parent is also blocked waiting for the child. Both are stuck — classic deadlock.

**Why it happens:**
OS pipe buffers are finite (typically 64KB on macOS). Any command that produces stderr output (warnings, deprecations) while the parent only reads stdout will deadlock when the stderr buffer fills. `wait_with_output()` from Tokio handles this correctly by reading both streams concurrently, but manual implementations frequently get this wrong.

**How to avoid:**
Use `tokio::process::Child`'s `wait_with_output()` for commands where you want captured output and don't need streaming. For streaming (metro log tailing), spawn separate `tokio::spawn` tasks for stdout and stderr, each reading independently via `BufReader::lines()` and forwarding to channels.

**Warning signs:**
- Commands hang indefinitely past expected completion time
- Works with small output but hangs on verbose commands like `yarn install`
- Only occurs when both stdout and stderr are piped

**Phase to address:**
Phase 3 (Git and RN command execution) — all command execution should use a single shared helper that handles stdout/stderr correctly. Don't reinvent this per-command.

---

### Pitfall 7: Fixed-Rate Render Loop Wastes CPU When Nothing Changes

**What goes wrong:**
The app polls at 60fps unconditionally, burning CPU even when the dashboard is idle and no data has changed. On a development machine this is annoying noise; it can also interfere with battery life and thermal throttling during long builds.

**Why it happens:**
Tutorial-style examples often show a simple `loop { terminal.draw(...); sleep(16ms) }` to get started. This pattern is copied without considering that most of the time the dashboard is waiting for user input or a background task result, not actively animating.

**How to avoid:**
Render on demand. Use `tokio::select!` to wait on either a key event, a channel message from a background task, or a tick timer. Only call `terminal.draw()` when something has changed (dirty flag in app state) or on a slower background tick (e.g., 4fps) for the log tail. Metro log streaming is the one exception — that warrants more frequent renders but still should be throttled to avoid thrashing.

**Warning signs:**
- `top` shows the dashboard process using 5-15% CPU while idle
- Fan noise increases when the dashboard is open but inactive
- Render loop runs at max speed when polling with `poll(Duration::ZERO)`

**Phase to address:**
Phase 1 (Core event loop architecture) — establish the event-driven render trigger before adding any features. This is architectural and hard to retrofit.

---

### Pitfall 8: Metro Port Detection by PID Lookup Instead of Port Check

**What goes wrong:**
The dashboard tracks the metro process by PID and assumes "metro is running if this PID is alive." When metro crashes and restarts itself (it has auto-restart behavior), the PID changes. The dashboard thinks metro died, shows it as stopped, and may attempt a double-start. Alternatively, a stale PID from a previous session is stored in state, and the dashboard incorrectly reports metro as running.

**Why it happens:**
Storing a PID is the obvious approach. But metro is a Node.js process that can fork child processes, and the actual bundler may run in a child. PIDs are also ephemeral — they are reused by the OS.

**How to avoid:**
Verify metro status by checking port 8081 is listening (via `TcpStream::connect("127.0.0.1:8081")`) rather than relying solely on PID liveness. Keep the `Child` handle in Rust state to detect process exit via `try_wait()`, but cross-validate with the port check. On startup, probe port 8081 first to detect a metro that was left running from a previous session.

**Warning signs:**
- Dashboard shows metro as stopped but it's actually running (browser can connect)
- Attempting to start metro fails with "port already in use"
- After dashboard restart, metro status is wrong

**Phase to address:**
Phase 2 (Metro process management) — define the status detection contract at design time. Do not shortcut to PID-only tracking.

---

### Pitfall 9: JIRA API Token Stored in Plaintext in Config Without Scoping

**What goes wrong:**
The JIRA API token is stored in `~/.config/ump-dash/config.toml` in plaintext. If the config file is accidentally committed to git, shared via dotfiles, or readable by other users, the token is exposed. Additionally, if the token is scoped too broadly (e.g., to write permissions), a leak enables JIRA actions beyond reading ticket titles.

**Why it happens:**
It's the simplest approach: put the token in the config. The risk is dismissed because it's a "personal tool."

**How to avoid:**
Document clearly in the config template that the token should not be committed. Set `~/.config/ump-dash/` file permissions to `0600` (user read/write only) on first write. Request a JIRA API token with read-only scope (Atlassian supports scoped tokens). Consider supporting `JIRA_API_TOKEN` environment variable as an alternative to file storage (avoids the file entirely).

**Warning signs:**
- Config file is world-readable (`ls -la ~/.config/ump-dash/`)
- Config directory is inside a git repo
- Token has write permissions on JIRA

**Phase to address:**
Phase 4 (JIRA integration) — bake in the permission hardening when writing config for the first time, not as a post-launch security fix.

---

### Pitfall 10: JIRA API Calls Block Dashboard Startup

**What goes wrong:**
On startup, the dashboard fetches JIRA ticket titles for all visible worktrees before displaying anything. If the JIRA API is slow (>500ms is common), the dashboard appears frozen during boot. If the API is unreachable (offline, VPN required), the dashboard fails to start entirely.

**Why it happens:**
Synchronous "fetch everything before showing" is the naive approach. JIRA Cloud API latency is 200-800ms per request. With 5+ worktrees that's multiple seconds of blank screen.

**How to avoid:**
Show the dashboard immediately with placeholder titles ("Loading..."). Fetch JIRA data in background tasks after the first render. Update the UI as results arrive. Handle failure gracefully — if the JIRA API is unreachable, display the branch name without a ticket title rather than erroring. Cache ticket titles to disk (in `~/.config/ump-dash/cache/`) with a TTL so repeat opens are instant even without connectivity.

**Warning signs:**
- Dashboard takes >1s to show anything on startup
- Working offline/without VPN causes a crash or error screen
- All JIRA fetches happen before the first `terminal.draw()` call

**Phase to address:**
Phase 4 (JIRA integration) — design the JIRA layer as always-async and always-optional from the start.

---

### Pitfall 11: Git Operations Run on the Main Async Thread

**What goes wrong:**
`git reset --hard`, `git pull`, and `git rebase` can take seconds. Running them as `tokio::process::Command` is fine (non-blocking), but if they are run using `std::process::Command::output()` or the `git2` crate's synchronous bindings, the render loop stalls for the entire duration of the operation.

**Why it happens:**
The `git2` crate is a popular Rust git library but is entirely synchronous. Using it directly in async code blocks the runtime. This is easy to overlook because `git2` feels "native Rust" compared to shelling out to git.

**How to avoid:**
Shell out to the `git` command using `tokio::process::Command` for all git operations. Do not use the `git2` crate in async contexts unless wrapped in `spawn_blocking`. For `git worktree list` (listing worktrees), parse the output of the CLI command rather than using `git2` bindings.

**Warning signs:**
- UI freezes during git operations
- `git2` appears as a dependency in `Cargo.toml`
- A git operation takes 3+ seconds and the clock display doesn't update during that time

**Phase to address:**
Phase 3 (Git operations) — establish the "shell out via tokio" pattern as the only approach. Explicitly exclude `git2` from the dependency list.

---

## Minor Pitfalls

### Pitfall 12: Multiple terminal.draw() Calls in the Same Loop Iteration

**What goes wrong:**
If two different code paths in the event loop call `terminal.draw()`, only the second render is visible due to Ratatui's double-buffer diffing. The first draw's output is overwritten without being displayed, and the double-buffer gets out of sync.

**Why it happens:**
As the app grows, it becomes tempting to add an "immediate refresh" draw call in response to a specific event. Ratatui explicitly documents this as incorrect, but it's easy to miss.

**How to avoid:**
Enforce a single draw call per loop iteration. Combine all UI state into a single `render()` function called once per tick. Use a dirty flag to skip the draw call when nothing has changed rather than adding extra draw calls for urgency.

**Phase to address:**
Phase 1 (TUI shell) — establish the single-draw-per-tick pattern in the event loop skeleton.

---

### Pitfall 13: Config Struct Breaking Changes Crash Existing Installations

**What goes wrong:**
Adding a required field to the TOML config struct without a `#[serde(default)]` annotation causes a deserialization error for existing users whose config files predate the field. The dashboard fails to start with a cryptic serde error.

**Why it happens:**
Serde TOML deserialization fails on unknown or missing required fields. Config files persist on disk indefinitely — a field added in v0.2 will be absent in v0.1 config files.

**How to avoid:**
Every field in the config struct must have `#[serde(default)]` unless the field was present in the initial release. Use `Option<T>` for optional fields. Never remove or rename a field — add a new field with the new name and deprecate the old one with a migration path. Version the config file with a `version = 1` field from the start.

**Phase to address:**
Phase 4 (Config and JIRA integration) — version the config schema from day one.

---

### Pitfall 14: Vim Keybinding Conflicts with Terminal/tmux Passthrough

**What goes wrong:**
Keys like `Ctrl+[` (Escape alias), `Ctrl+W`, `Ctrl+B` (tmux prefix), or `Ctrl+Z` (SIGTSTP) are intercepted by tmux or the terminal before reaching the Rust process. Keybindings using these sequences never fire in the dashboard.

**Why it happens:**
The dashboard lives inside tmux. tmux has its own key prefix (`Ctrl+B` by default). Some terminals also intercept certain key combos. Raw mode captures most keys but not all — signal-generating keys (Ctrl+C, Ctrl+Z) require explicit handling via `signal_hook` or similar, and tmux prefix keys are consumed by tmux before delivery.

**How to avoid:**
Design keybindings around tmux-safe keys. Test all keybindings inside tmux explicitly. For the tmux prefix conflict: document that users with non-standard tmux prefixes may need to configure alternatives. Handle `Ctrl+C` explicitly (graceful shutdown) rather than relying on SIGINT propagation through raw mode.

**Phase to address:**
Phase 1 (TUI shell) — test the keybinding layer inside tmux from the first prototype, not after all bindings are implemented.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `std::process::Command` instead of `tokio::process::Command` | Simpler API | Blocks render loop on any command | Never — use tokio from the start |
| PID-only metro status tracking | Easy to implement | Stale state, double-starts, port conflicts | Never for the primary check — use port probe |
| Fixed 60fps render loop | Simple event loop | CPU waste, battery drain | Acceptable in prototype only |
| Synchronous JIRA fetch on startup | No loading states needed | Frozen boot, offline failure | Never — fetch async after first render |
| git2 crate for git operations | Native Rust, no subprocess | Blocks async runtime | Never in async code — shell out instead |
| Config fields without `#[serde(default)]` | Simpler struct definition | Breaks existing installs on upgrade | Never for user-facing config |
| Storing JIRA token in world-readable file | No extra code | Token exposure risk | Acceptable temporarily in development only |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| tmux | Use `send-keys` to a new pane immediately after creation | Use `new-window "command"` or add shell-ready polling |
| tmux | Assume `send-keys` is synchronous | It is fire-and-forget; the pane may not be ready |
| Metro bundler | Kill by SIGTERM and assume port is free | Kill, then await exit, then probe port 8081 |
| JIRA API | Fetch all tickets synchronously on startup | Fetch in background after first render; cache results |
| JIRA API | Ignore HTTP 429 responses | Respect `Retry-After` header, implement exponential backoff |
| git worktree | Parse `git worktree list` output with hand-rolled regex | Use `--porcelain` flag for stable machine-readable output |
| git worktree | Assume worktree path == branch name | Worktree path is set at creation time; query actual branch with `git -C <path> branch --show-current` |
| crossterm | Add as direct dependency alongside ratatui | Import via `ratatui::crossterm` re-export only |
| Tokio child process | Drop `Child` handle without awaiting | Store handle in state; always `kill().await` then `wait().await` |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Unconditional 60fps render loop | 5-15% CPU idle, fan noise | Event-driven render with dirty flag | Immediately — always burning cycles |
| Allocating `String` per widget per frame | GC pressure at high fps | Pre-allocate, use `&str` slices where possible | Noticeable when metro log streams fast |
| Reading metro stdout on the render thread | Render skips when log line arrives | Dedicated `tokio::spawn` task with channel | First metro log line |
| JIRA API called on every worktree list refresh | API rate limit hit, slow UI | Cache with TTL, only refresh on explicit request | After ~100 refreshes per hour |
| `git status` called per-worktree on every tick | Disk I/O, slow renders during heavy git ops | Poll on a slow timer (5s) or on explicit trigger | When many worktrees exist or disk is busy |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| JIRA token in plaintext config with default permissions | Token readable by other processes/users | Set config file to `0600`, support env var alternative |
| Passing branch name directly as shell argument without quoting | Command injection if branch name contains special chars | Always use `tokio::process::Command` with explicit args (not shell string interpolation) |
| Logging JIRA API token in debug output | Token in log files | Sanitize log output; never log Authorization headers |
| Running `git` commands with user-provided input in shell mode | Arbitrary command execution | Never use `.shell(true)` or string interpolation for args; always pass args as separate elements |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No feedback during slow operations (yarn install, git pull) | User thinks app is frozen | Show spinner and live stdout stream in a log panel |
| Blocking keybindings during a running command | User can't switch worktrees while metro is starting | Use non-blocking async commands; allow input at all times |
| Showing raw branch name when JIRA fetch fails | Context is lost — why did metro fail to start? | Show branch name + "(JIRA unavailable)" rather than erroring |
| Confirmation dialog for destructive operations but wrong key | User presses `r` to reload but triggers reset --hard | Use capital letters or multi-key sequences for destructive ops |
| Vim-style but no escape hatch from modal states | User stuck in a subcommand modal with no visible way out | Always display `q`/`Esc` hint for every modal; never show a state with no exit |
| Long metro log in the UI overflows terminal height | Log content pushes worktree list off screen | Cap log display to a fixed height panel; offer toggle to expand |

---

## "Looks Done But Isn't" Checklist

- [ ] **Terminal cleanup:** Does the terminal restore correctly after a panic mid-render? (Test with `panic!("test")` in the main loop)
- [ ] **Metro kill:** After killing metro, does `lsof -i :8081` show the port as free within 1 second?
- [ ] **Metro zombie check:** After dashboard exits, does `ps aux | grep metro` still show a metro process?
- [ ] **Offline JIRA:** Does the dashboard start and display worktrees when network is unavailable?
- [ ] **JIRA rate limit:** Does the dashboard handle `HTTP 429` from JIRA without crashing?
- [ ] **tmux launch:** Does Claude Code launch correctly inside tmux when opened from the dashboard?
- [ ] **Config upgrade:** Does the dashboard start correctly with a config file missing a newly added optional field?
- [ ] **Git branch detection:** Does the branch display update correctly after a `git checkout` outside the dashboard?
- [ ] **Crossterm version:** Does `cargo tree | grep crossterm` show exactly one version?
- [ ] **CPU at idle:** Is CPU usage below 1% when the dashboard is open but no activity is happening?

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Terminal left in raw mode | LOW | User types `reset` blindly; add panic hook in next commit |
| Zombie metro process | LOW | `kill -9 $(lsof -ti :8081)` manually; fix kill+wait in next commit |
| Crossterm version conflict | MEDIUM | Remove direct `crossterm` dep from `Cargo.toml`; use ratatui re-export |
| Blocking I/O on render thread | HIGH | Refactor all I/O to tokio async; requires touching all command execution code |
| Config struct breaking change | MEDIUM | Add migration code that reads old config format and writes new; annoying but doable |
| JIRA fetch blocking startup | MEDIUM | Move fetch call to post-first-render background task; requires async JIRA layer |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Terminal not restored on panic/error | Phase 1: TUI shell | Run `panic!()` in main loop, verify terminal resets |
| Zombie metro / port not released | Phase 2: Metro process management | Kill metro, immediately run `lsof -i :8081` |
| Blocking I/O on async runtime | Phase 2: Metro process management | Check all spawn calls use tokio::process |
| tmux send-keys race condition | Phase 5: tmux/Claude Code integration | Test Claude Code launch 10x in tmux |
| Crossterm version mismatch | Phase 1: Project scaffold | `cargo tree \| grep crossterm` shows 1 version |
| Stdout/stderr pipe deadlock | Phase 3: Git/RN commands | Run `yarn install` (verbose), verify no hang |
| Fixed-rate CPU waste | Phase 1: Event loop architecture | `top` shows <1% CPU when idle |
| Metro status via PID only | Phase 2: Metro process management | Kill and restart metro; verify status correct |
| JIRA token exposure | Phase 4: JIRA integration | `stat ~/.config/ump-dash/config.toml` shows `0600` |
| JIRA fetch blocking startup | Phase 4: JIRA integration | Start with network disabled; dashboard must show in <500ms |
| Git ops on main async thread | Phase 3: Git operations | Run `git pull` during active renders; verify no frame skips |
| Config schema breaking change | Phase 4: Config management | Delete new field from config, restart dashboard |
| Vim key conflicts with tmux | Phase 1: TUI shell | Test all keybindings inside a live tmux session |

---

## Sources

- [Ratatui FAQ — panic hook and terminal cleanup](https://ratatui.rs/faq/) (HIGH confidence)
- [Ratatui Panic Hooks Recipe](https://ratatui.rs/recipes/apps/panic-hooks/) (HIGH confidence)
- [Ratatui Rendering Concepts](https://ratatui.rs/concepts/rendering/) (HIGH confidence)
- [Ratatui Async Event Stream Tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) (HIGH confidence)
- [Ratatui/Crossterm Version Incompatibility Advisory — GitHub Issue #1298](https://github.com/ratatui/ratatui/issues/1298) (HIGH confidence)
- [Ratatui CPU Usage Discussion — GitHub Discussion #89](https://github.com/ratatui/ratatui-website/discussions/89) (HIGH confidence)
- [Tokio process::Command documentation — zombie processes](https://docs.rs/tokio/latest/tokio/process/index.html) (HIGH confidence)
- [Tokio process::Child — kill handling](https://docs.rs/tokio/latest/tokio/process/struct.Child.html) (HIGH confidence)
- [tmux send-keys race condition — Claude Code issue #23513](https://github.com/anthropics/claude-code/issues/23513) (HIGH confidence — confirmed real-world issue)
- [tmux send-keys async execution issue #1517](https://github.com/tmux/tmux/issues/1517) (HIGH confidence)
- [Atlassian JIRA Cloud Rate Limiting — official docs](https://developer.atlassian.com/cloud/jira/platform/rate-limiting/) (HIGH confidence)
- [Atlassian API Token Rate Limiting announcement](https://community.developer.atlassian.com/t/api-token-rate-limiting/92292) (HIGH confidence)
- [Rust Forum: Catching panics with tui and crossterm — terminal state](https://users.rust-lang.org/t/catching-unwind-with-tui-and-crossterm-where-does-the-error-message-go/74955) (MEDIUM confidence)
- [Rust Forum: Using tokio child process across tasks](https://users.rust-lang.org/t/using-tokios-child-process-across-tasks/59017) (MEDIUM confidence)
- [Ratatui Rendering Best Practices Discussion #579](https://github.com/ratatui/ratatui/discussions/579) (MEDIUM confidence)

---
*Pitfalls research for: Rust TUI process management dashboard — UMP Dashboard*
*Researched: 2026-03-02*
