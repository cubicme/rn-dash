# Architecture Research

**Domain:** Rust TUI dashboard — process management, worktree browser, tmux integration
**Researched:** 2026-03-02
**Confidence:** HIGH (ratatui official docs, tokio official docs, gitui source analysis, lazygit architecture review)

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Presentation Layer                           │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐    │
│  │  MetroPane    │  │ WorktreePane  │  │    CommandPalette     │    │
│  │  (Widget)     │  │  (Widget)     │  │      (Widget)         │    │
│  └───────┬───────┘  └───────┬───────┘  └───────────┬───────────┘    │
│          │                  │                      │                 │
│          └──────────────────┴──────────────────────┘                │
│                             │                                        │
│                      view(&AppState)                                 │
├─────────────────────────────────────────────────────────────────────┤
│                        Application Layer                             │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                       AppState                               │   │
│  │  active_worktree | worktrees | metro_status | ui_focus       │   │
│  └──────────────────────────────┬───────────────────────────────┘   │
│                                 │                                    │
│  ┌──────────────────────────────┴───────────────────────────────┐   │
│  │               Event Loop (tokio::select!)                    │   │
│  │   terminal events ── app events ── process events ── tick   │   │
│  └──────────────────────────────┬───────────────────────────────┘   │
│                                 │                                    │
│  ┌──────────────────────────────┴───────────────────────────────┐   │
│  │                     Action enum                              │   │
│  │  SwitchWorktree | KillMetro | RunCommand | FetchJira | ...  │   │
│  └──────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                         Domain Layer                                 │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────────────┐   │
│  │  Worktree      │  │  Metro         │  │   Command            │   │
│  │  (pure data)   │  │  (pure data)   │  │   (pure data)        │   │
│  └────────────────┘  └────────────────┘  └──────────────────────┘   │
│  ┌────────────────┐  ┌──────────────────────────────────────────┐   │
│  │  JiraTicket    │  │  WorktreeManager (business rules)        │   │
│  │  (pure data)   │  │  "only one metro at a time"              │   │
│  └────────────────┘  └──────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                       Infrastructure Layer                           │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────────────┐   │
│  │ ProcessManager │  │  JiraClient    │  │   TmuxClient         │   │
│  │ (tokio::Child) │  │  (reqwest)     │  │  (tmux_interface)    │   │
│  └────────────────┘  └────────────────┘  └──────────────────────┘   │
│  ┌────────────────┐  ┌────────────────┐                             │
│  │  GitClient     │  │  ConfigStore   │                             │
│  │  (git2 / cmd)  │  │  (toml + XDG)  │                             │
│  └────────────────┘  └────────────────┘                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| AppState | Single source of truth for all UI-visible data | Plain struct, cloned into render; mutated by update() |
| Event Loop | Multiplex terminal input, process events, ticks | `tokio::select!` over mpsc channels |
| Action enum | Named, typed user/system intents | `enum Action { SwitchWorktree(usize), KillMetro, ... }` |
| Presentation Widgets | Render AppState into ratatui Buffers | Implement `Widget` or `StatefulWidget` trait; no logic |
| WorktreeManager | Enforce "one metro at a time" business rule | Pure Rust, no async, no IO; tested in isolation |
| ProcessManager | Spawn, capture, kill tokio child processes | Owns `tokio::process::Child` + stdout reader task |
| JiraClient | Fetch ticket title from branch name | HTTP via reqwest; returns domain type `JiraTicket` |
| TmuxClient | Create windows, send keys to panes | Wraps `tmux_interface` crate; fire-and-forget |
| GitClient | Run git operations per worktree | Calls `git` subprocess via `tokio::process::Command` |
| ConfigStore | Read/write XDG config file | `~/.config/ump-dash/*.toml`; deserializes to typed structs |

## Recommended Project Structure

```
src/
├── main.rs                 # Entry point — parse args, init terminal, run event loop
├── app.rs                  # AppState struct + update() function (the brain)
├── event.rs                # Event enum (terminal + process + tick events)
├── action.rs               # Action enum (typed user/system intents)
├── tui.rs                  # Terminal setup/teardown (crossterm boilerplate)
│
├── domain/                 # Pure business data and rules — zero IO, fully testable
│   ├── mod.rs
│   ├── worktree.rs         # Worktree struct, WorktreeId, branch name parsing
│   ├── metro.rs            # MetroStatus enum (Running, Stopped, Starting, Crashing)
│   ├── command.rs          # RnCommand enum (Clean, Install, RunAndroid, ...) + arg structs
│   └── jira.rs             # JiraTicket struct, ticket ID extraction from branch name
│
├── ui/                     # Rendering only — reads AppState, produces ratatui output
│   ├── mod.rs              # Root view() function — assembles layout
│   ├── metro_pane.rs       # Metro status, log toggle, controls
│   ├── worktree_pane.rs    # Worktree list with JIRA titles and labels
│   ├── command_palette.rs  # Command options overlay when executing
│   ├── key_hints.rs        # On-screen key bindings bar
│   └── theme.rs            # Colors, styles (no logic)
│
└── infra/                  # IO and system concerns — implements domain contracts
    ├── mod.rs
    ├── process_manager.rs  # Spawn/kill/capture metro and RN commands
    ├── git_client.rs       # Executes git commands per worktree
    ├── jira_client.rs      # HTTP calls to JIRA API
    ├── tmux_client.rs      # Creates windows, sends keys for Claude Code
    └── config.rs           # Read/write ~/.config/ump-dash/ (serde + toml)
```

### Structure Rationale

- **domain/:** Zero imports from infra/ or ui/. Every type and function here can be unit tested with no mocking. This is Ousterhout's "deep module" principle: hide complexity behind a simple interface that owns the business rules.
- **ui/:** Only imports from domain/ and the ratatui crate. No async, no IO, no `tokio`. The `view()` function is a pure function of `&AppState` — same state always produces identical frames. This makes rendering trivially correct and easy to reason about.
- **infra/:** The only layer that touches the filesystem, network, child processes, and tmux. Each client type exposes a small async interface. Domain types flow in; domain types flow out. No ratatui imports here.
- **app.rs:** The orchestrator. `AppState` owns all the data the UI needs. `update(state, action) -> Vec<Effect>` maps actions to state mutations and side-effect requests. This is the only place that talks to both domain/ and triggers infra/ work.

## Architectural Patterns

### Pattern 1: Elm Architecture (TEA) for the Main Loop

**What:** The app loop runs as Model → Update → View with a typed Message/Action enum. Every user keypress and process event maps to an Action. The `update()` function mutates AppState and returns a list of Effects (async work to kick off). The `view()` function renders the current AppState.

**When to use:** Always. This is the dominant pattern in ratatui apps (confirmed by ratatui official docs). It eliminates callback hell and makes state changes auditable.

**Trade-offs:** Requires defining the Action enum upfront. Adding a new interaction means adding to Action, updating update(), and possibly adding a new Effect — three predictable edits in three places.

**Example:**
```rust
// action.rs
pub enum Action {
    SwitchWorktree(WorktreeId),
    KillMetro,
    StartMetro(WorktreeId),
    RunGitCommand(WorktreeId, GitCommand),
    RunRnCommand(WorktreeId, RnCommand),
    LaunchClaudeCode(WorktreeId),
    FetchJiraTitle(WorktreeId),
    ToggleMetroLog,
    Quit,
}

// app.rs
pub fn update(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::SwitchWorktree(id) => {
            // Domain rule: WorktreeManager validates the switch
            let switch = state.worktree_manager.begin_switch(id);
            vec![Effect::KillMetroThenStart(switch)]
        }
        Action::KillMetro => {
            state.metro_status = MetroStatus::Stopping;
            vec![Effect::KillMetroProcess]
        }
        // ...
    }
}
```

### Pattern 2: Channel-Based Event Multiplex

**What:** A single `tokio::select!` loop combines four event sources: terminal key events (from crossterm), process output events (from ProcessManager), background task results (JiraClient, GitClient), and tick events for animations/refresh. All sources send into typed channels. The main loop receives from all channels and dispatches to `update()`.

**When to use:** Any time you have both interactive UI and background async work. This is required for this project because metro log streaming and JIRA fetching run concurrently with user input.

**Trade-offs:** All concurrent state ends up in one AppState — simpler than distributed state but requires discipline to not make AppState a grab-bag.

**Example:**
```rust
// event.rs
pub enum Event {
    Key(crossterm::event::KeyEvent),
    ProcessOutput(WorktreeId, String),   // metro log lines
    ProcessExited(WorktreeId, ExitStatus),
    JiraFetched(WorktreeId, JiraTicket),
    Tick,
    Resize(u16, u16),
}

// main loop (simplified)
loop {
    tokio::select! {
        event = terminal_rx.recv() => { /* crossterm key events */ }
        event = process_rx.recv() => { /* metro output lines */ }
        event = bg_rx.recv()      => { /* jira, git results */ }
        _ = tick_interval.tick()  => { /* refresh animations */ }
    }
    let action = map_event_to_action(&state, event);
    let effects = update(&mut state, action);
    spawn_effects(effects, &process_tx, &bg_tx);
    terminal.draw(|f| view(f, &state))?;
}
```

### Pattern 3: Command Pattern for Process Execution

**What:** The `RnCommand` and `GitCommand` enums are rich types that carry all arguments needed to run a command. The ProcessManager receives these types and translates them into `tokio::process::Command` invocations. This decouples "what to run" (domain) from "how to run it" (infra).

**When to use:** Any time you have a set of typed operations with parameters that the UI needs to present and the user needs to configure. In this project, commands like `run-android` have device selection that must flow from the UI through to the process.

**Trade-offs:** More types to define upfront. The payoff is that the UI layer can display command options without knowing about process spawning, and the infra layer can run commands without knowing about display.

**Example:**
```rust
// domain/command.rs
pub enum RnCommand {
    Clean(CleanTarget),          // android | cocoapods | both
    Install,
    RunAndroid(AndroidOptions),  // device: Option<String>, variant: BuildVariant
    RunIos(IosOptions),          // device: Option<DeviceId>
    YarnTest(TestFilter),
    YarnLint,
    CheckTypes,
}

// infra/process_manager.rs — only layer that calls tokio::process
impl ProcessManager {
    pub async fn spawn(&self, cmd: RnCommand, cwd: &Path) -> Result<ChildHandle> {
        let mut command = tokio::process::Command::new("yarn");
        match cmd {
            RnCommand::RunAndroid(opts) => {
                command.args(["react-native", "run-android"]);
                if let Some(device) = opts.device {
                    command.args(["--deviceId", &device]);
                }
            }
            // ...
        }
        command.current_dir(cwd).stdout(Stdio::piped());
        // spawn, attach stdout reader task that sends lines to process_tx channel
    }
}
```

### Pattern 4: Deep Module for Metro Management (Ousterhout)

**What:** The `ProcessManager` hides all complexity of PTY/pipe management, zombie process prevention, graceful vs. forced kill, and output buffering behind a minimal interface: `spawn(cmd, cwd)`, `kill(id)`, `is_running(id)`. Callers never touch `tokio::process::Child` directly.

**When to use:** Any subsystem with significant implementation complexity that has a clean, stable purpose. Metro management has exactly this profile: the internals are tricky (kill ordering, SIGTERM/SIGKILL escalation, log streaming) but the external contract is simple.

**Trade-offs:** More code inside the module. The benefit is that all the hard parts are in one place and the rest of the codebase stays clean.

## Data Flow

### User Keypress to Screen Update

```
User presses key
    ↓
crossterm sends KeyEvent → terminal_rx channel
    ↓
main loop receives Event::Key(k)
    ↓
map_event_to_action(&state, Event::Key(k)) → Action::SwitchWorktree(id)
    ↓
update(&mut state, action) → mutates state + returns Vec<Effect>
    ↓
spawn_effects(effects) → spawns tokio task to kill metro
    ↓
terminal.draw(|f| view(f, &state))  ← renders immediately with new state
    ↓
tokio task kills process → sends Event::ProcessExited(id, status)
    ↓
main loop receives it → update() mutates metro_status to Stopped
    ↓
terminal.draw() → next frame reflects Stopped state
```

### Metro Log Streaming

```
ProcessManager::spawn() called
    ↓
tokio::process::Child spawned with stdout: Stdio::piped()
    ↓
Background task: BufReader::new(child.stdout).lines()
    ↓
Each line → process_tx.send(Event::ProcessOutput(id, line))
    ↓
main loop → update() → state.metro_log.push(line)
    ↓
view() → MetroPane renders from state.metro_log (ring buffer)
```

### JIRA Title Fetch

```
Worktree discovered during init or added
    ↓
domain/jira.rs::extract_ticket_id("UMP-1234-description") → Some("UMP-1234")
    ↓
bg_tx.send(Effect::FetchJira(worktree_id, ticket_id))
    ↓
Background task: JiraClient::fetch_title(token, ticket_id).await
    ↓
bg_rx.send(Event::JiraFetched(worktree_id, ticket))
    ↓
main loop → update() → state.worktrees[id].jira_title = Some(title)
    ↓
view() → WorktreePane shows the title
```

### Key Data Flows Summary

1. **Input → Action → State mutation → Render**: Synchronous, completes in one loop iteration, keeps UI responsive.
2. **Action → Effect → Background task → Event → State mutation → Render**: Async, completes over multiple loop iterations; state shows in-progress immediately.
3. **Process stdout → State → Render**: Continuous streaming; log lines flow through channels into a ring buffer.

## Build Order Implications

The layered architecture implies a build order where each layer depends only on the ones below it:

1. **Build first — domain/ types and rules.** WorktreeId, MetroStatus, RnCommand, JiraTicket, the "one metro at a time" invariant in WorktreeManager. No async, no IO. Write and test in isolation. This is the foundation everything else depends on.

2. **Build second — infra/ adapters.** ProcessManager, ConfigStore, GitClient, JiraClient, TmuxClient. Each one independently testable with integration tests. No ratatui imports. Domain types are input/output.

3. **Build third — app.rs coordination.** AppState struct and update() function. Wire domain types and infra effects together. No rendering. Can be tested by driving actions and asserting on state.

4. **Build fourth — tui.rs and event.rs scaffolding.** Terminal init/teardown, crossterm event capture, channel setup, main loop skeleton. Ratatui setup code.

5. **Build last — ui/ widgets.** Each widget gets AppState, renders, done. No logic to test — visual inspection suffices. Widgets are decoupled so they can be built and refined independently.

## Anti-Patterns

### Anti-Pattern 1: Putting Logic Inside Widget render()

**What people do:** Calculate "is this worktree active?" or "should this button be enabled?" inside the Widget render method.

**Why it's wrong:** The view() function is supposed to be a pure, deterministic translation of state to pixels. Logic in render() means the same AppState could produce different output depending on hidden factors. It also makes the logic untestable without a full terminal setup.

**Do this instead:** Compute derived values in `update()` and store them in AppState. The widget reads `state.is_active[id]`, not `state.active_worktree == id`. Or use a thin ViewModel struct that `view()` constructs from AppState before passing to widgets.

### Anti-Pattern 2: Blocking Tokio in the Event Loop

**What people do:** Call `std::process::Command::output()` inside the async event loop to run git or yarn commands.

**Why it's wrong:** `std::process::Command` blocks the Tokio thread. The entire UI freezes while the command runs. In a dashboard context, this means the metro log stops updating and keypresses are lost.

**Do this instead:** Always use `tokio::process::Command` for subprocess spawning. For truly blocking operations (e.g., libgit2 synchronous API), use `tokio::task::spawn_blocking` to run them on a dedicated thread pool thread. Send results back through channels.

### Anti-Pattern 3: Shared Mutable State for Process Handles

**What people do:** Store `Arc<Mutex<Option<Child>>>` in AppState so both the UI and the process manager can access it.

**Why it's wrong:** Process handles are not UI state. Mixing them into AppState creates implicit coupling between the rendering layer and the OS process layer. It also makes the mutex a coordination point that adds latency.

**Do this instead:** ProcessManager owns all `Child` handles internally. AppState contains only domain-level status: `MetroStatus::Running { pid: u32 }`. The UI knows the process is running and its PID for display; it never holds the raw handle.

### Anti-Pattern 4: Domain Logic in Action Handlers

**What people do:** Put the "only one metro at a time" check inside the Action::SwitchWorktree arm of update().

**Why it's wrong:** Business rules embedded in event handlers are hard to find, hard to test, and tend to be replicated when similar actions are added later.

**Do this instead:** Business rules live in domain/. The `WorktreeManager::begin_switch(id)` method encodes the invariant and returns either a `PendingSwitch` (describing what needs to happen) or an error. The update() function just calls it and converts the result to Effects.

### Anti-Pattern 5: Orphaned Metro Process on Crash/Exit

**What people do:** Spawn metro with `tokio::process::Command::spawn()` but drop the `Child` handle when switching worktrees, expecting the process to die.

**Why it's wrong:** Dropping a `Child` without `.kill()` does NOT kill the process on Unix. Metro keeps running in the background, consuming port 8081 and causing the next `yarn start` to fail with "address already in use."

**Do this instead:** Always call `child.kill().await` before dropping the handle. Use `kill_on_drop(true)` on the Command as a safety net, but do not rely on it exclusively. Maintain a handle registry in ProcessManager. On graceful shutdown and on panics (via a drop guard), kill all tracked children.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| JIRA REST API | HTTP GET with Bearer token (reqwest) | Token stored in ~/.config/ump-dash/config.toml; requests cached per session to avoid rate limits |
| git CLI | `tokio::process::Command` calling git binary | Prefer git binary over libgit2: simpler, no linking, matches user's git version |
| yarn / React Native CLI | `tokio::process::Command` in worktree cwd | Must capture stdout/stderr for display; must be killable on worktree switch |
| tmux | `tmux_interface` crate (wraps tmux CLI) | Use `NewWindow` + `SendKeys` to open Claude Code; fire-and-forget, no response needed |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| ui/ → app/ | Reads `&AppState` reference; no writes | One-way. UI never mutates state. |
| app/ → domain/ | Direct function calls (sync) | No async in domain/. update() calls domain functions synchronously. |
| app/ → infra/ | Via `Effect` enum + tokio tasks | update() returns Vec<Effect>. Caller spawns tasks to execute effects. Results come back as Events. |
| infra/ → app/ | mpsc channel sending Event variants | ProcessManager, JiraClient etc. hold Sender<Event>. Main loop holds Receiver<Event>. |
| domain/ → (nothing) | No outbound dependencies | Domain is a leaf node. Zero imports from infra/ or ui/. |

## Scaling Considerations

This is a single-user local tool. Scale is not relevant in the traditional sense. The equivalent concerns are:

| Concern | At 1 worktree | At 10 worktrees | At 50+ worktrees |
|---------|---------------|-----------------|------------------|
| JIRA fetching | Fire-and-forget single request | Batch requests on init, cache | Paginate or lazy-fetch on scroll |
| Metro log buffer | Unlimited growth is fine | Cap at 10k lines (ring buffer) | Same — log file on disk, stream tail |
| Git status refresh | Per-keystroke or per-tick | Debounced, one goroutine per worktree | Same, git is fast |
| Process tracking | One handle in ProcessManager | Same — still one metro, N process handles for git/yarn | Same |

## Sources

- [Ratatui: The Elm Architecture (TEA)](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/) — HIGH confidence, official docs
- [Ratatui: Event Handling](https://ratatui.rs/concepts/event-handling/) — HIGH confidence, official docs
- [Ratatui: Widget and StatefulWidget traits](https://ratatui.rs/concepts/widgets/) — HIGH confidence, official docs
- [Ratatui: Best Practices Discussion](https://github.com/ratatui/ratatui/discussions/220) — HIGH confidence, official repo discussion
- [Ratatui: Component Template](https://ratatui.rs/templates/component/) — HIGH confidence, official docs
- [tokio::process documentation](https://docs.rs/tokio/latest/tokio/process/index.html) — HIGH confidence, official docs
- [tmux_interface crate docs](https://docs.rs/tmux_interface/latest/tmux_interface/) — HIGH confidence, official docs
- [Codex TUI architecture analysis](https://zread.ai/openai/codex/18-tui-application-structure) — MEDIUM confidence, derived analysis
- [GitUI source code analysis (gitui-org/gitui)](https://github.com/gitui-org/gitui) — MEDIUM confidence, source code review
- [Lazygit architecture (DeepWiki)](https://deepwiki.com/jesseduffield/lazygit) — MEDIUM confidence, third-party analysis
- [Hexagonal Architecture in Rust](https://medium.com/@lucorset/hexagonal-architecture-in-rust-72f8958eb26d) — MEDIUM confidence, community article

---
*Architecture research for: Rust TUI dashboard — React Native worktree management*
*Researched: 2026-03-02*
