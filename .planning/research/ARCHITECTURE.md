# Architecture Research — v1.3 Per-Worktree Tasks + Architecture Audit

**Domain:** Rust TUI dashboard — adding per-worktree task ownership, parallel execution, cancellation, spinner animation
**Researched:** 2026-04-13
**Confidence:** HIGH (derived from direct source reading of all existing modules; no external research needed for integration design)

---

## Existing Architecture Summary

The app is a strict TEA implementation with four layers. Every layer boundary is already enforced by import discipline:

```
domain/      — pure Rust types. Zero IO, no tokio, no ratatui. Tested in isolation.
infra/       — async I/O: process spawning, git, JIRA, config, multiplexer.
app.rs       — TEA brain: AppState struct + update() + handle_key() + run() event loop.
ui/          — ratatui widgets that read &AppState. No logic, no async.
```

The event loop in `run()` is a `tokio::select!` over four sources:

1. `tick` — 250ms `tokio::time::interval` (redraws for time-based UI)
2. `refresh_interval` — 60s `tokio::time::interval` (periodic worktree refresh)
3. `events.next()` — crossterm `EventStream` (keyboard input)
4. `metro_rx.recv()` — unbounded `mpsc` channel (all background Action senders)

Plus a side channel: `handle_rx` delivers `MetroHandle` objects from the metro spawn task.

### Current Command System (What Must Change)

The current command system has three problems the new milestone must fix:

**Problem 1: Command is globally anonymous.**
`AppState.running_command: Option<CommandSpec>` has no worktree binding. When a command runs, the system knows *what* is running but not *for which worktree*. `dispatch_command()` captures the currently selected worktree's path at dispatch time, but this is not stored as part of the running task record. The `CommandOutputLine` action routes to whichever worktree is selected at the time the line arrives — which may differ from the worktree that dispatched the command.

**Problem 2: Only one command can run at a time (global serialization).**
`AppState.command_task: Option<JoinHandle<()>>` is a single slot. When a new command starts (or `dispatch_command` is called), any existing task handle is aborted first. There is no mechanism for running a yarn command in worktree A while a git pull runs in worktree B.

**Problem 3: Cancellation is task-abort only, with no SIGTERM.**
`Action::CommandCancel` calls `task.abort()` on the outer `tokio::spawn` wrapper. This sends a Tokio task cancellation signal (cooperative future poll cancellation) but does NOT send SIGTERM or SIGKILL to the child process. The child process continues running until it exits naturally; only the stdout-streaming task is dropped. For long commands (yarn install, jest, gradlew), this leaves orphaned processes.

**Problem 4: No elapsed-time tracking.**
`running_command` is an `Option<CommandSpec>` with no start timestamp. The UI cannot show elapsed time for the current command.

**Problem 5: No spinner state.**
Tick-driven animation requires a frame counter in `AppState`. The current tick fires at 250ms but there is no frame counter consumed by the UI. Spinner state is missing.

---

## Proposed Changes by Layer

### Layer 1: domain/ — New Types

Add to `domain/command.rs` (new section, no existing types removed):

```rust
/// A task record binding a command to its owning worktree.
/// Created by dispatch_command(), lives in AppState.tasks.
#[derive(Debug, Clone)]
pub struct TaskRecord {
    pub id: TaskId,
    pub worktree_id: WorktreeId,
    pub spec: CommandSpec,
    pub started_at: std::time::Instant,
}

/// Stable identifier for an in-flight or recently-completed task.
/// Newtype prevents mixing with WorktreeId and CommandSpec indices.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

/// Categorizes a command for the UI animation indicator.
/// Used by ui/ to decide which spinner style to render.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskCategory {
    /// Yarn-like (yarn install, pod-install, clean): renders Y/P spinner
    Yarn,
    /// Run commands (run-android, run-ios, release build): renders R spinner
    Run,
    /// Test/lint/types: renders T spinner
    Test,
    /// Git operations: no spinner (non-cancellable, fast enough)
    Git,
    /// Shell: generic spinner
    Shell,
}

impl CommandSpec {
    /// Returns the animation category for this command.
    pub fn task_category(&self) -> TaskCategory {
        match self {
            CommandSpec::YarnInstall | CommandSpec::YarnPodInstall
            | CommandSpec::RnCleanAndroid | CommandSpec::RnCleanCocoapods
            | CommandSpec::RmNodeModules => TaskCategory::Yarn,

            CommandSpec::RnRunAndroid { .. } | CommandSpec::RnRunIos { .. }
            | CommandSpec::RnRunIosDevice | CommandSpec::RnReleaseBuild
            | CommandSpec::AdbInstallApk => TaskCategory::Run,

            CommandSpec::YarnUnitTests | CommandSpec::YarnJest { .. }
            | CommandSpec::YarnLint | CommandSpec::YarnCheckTypes => TaskCategory::Test,

            CommandSpec::GitResetHard | CommandSpec::GitPull | CommandSpec::GitPush
            | CommandSpec::GitRebase { .. } | CommandSpec::GitCheckout { .. }
            | CommandSpec::GitCheckoutNew { .. } | CommandSpec::GitFetch
            | CommandSpec::GitResetHardFetch => TaskCategory::Git,

            CommandSpec::ShellCommand { .. } => TaskCategory::Shell,
        }
    }

    /// True for commands that support cancellation (SIGTERM to process group).
    /// Git operations are NOT cancellable — mid-operation kill risks index corruption.
    pub fn is_cancellable(&self) -> bool {
        !matches!(self.task_category(), TaskCategory::Git)
    }
}
```

Add `domain/task.rs` as a new file (or add inline to `command.rs` if small enough). The key insight: `TaskRecord` is pure data — `std::time::Instant` is in std, no tokio dependency needed in domain.

Add to `domain/worktree.rs`:

```rust
/// Live task indicator for a worktree row — what task (if any) is running on it.
/// Populated in AppState from the tasks map; read by ui/panels.rs for the spinner column.
#[derive(Debug, Clone, PartialEq)]
pub enum WorktreeTaskIndicator {
    Idle,
    Running {
        label: &'static str,      // spec.label() — static str from CommandSpec
        category: TaskCategory,
        elapsed_secs: u64,
    },
}
```

### Layer 2: infra/command_runner.rs — Process Group + Cancellation

The infra change is surgical: `spawn_command_task` needs to return a type that carries both the JoinHandle and a kill mechanism that sends SIGTERM to the child process group.

**Current signature:**
```rust
pub async fn spawn_command_task(
    spec: CommandSpec,
    worktree_path: PathBuf,
    current_branch: String,
    action_tx: UnboundedSender<Action>,
) -> JoinHandle<()>
```

**New return type:**

```rust
/// Handle to a running command task.
/// abort() cancels Tokio streaming. kill() sends SIGTERM to the process group.
pub struct CommandHandle {
    /// Task join handle — abort() stops stdout/stderr streaming.
    pub join_handle: tokio::task::JoinHandle<()>,
    /// Cancellation channel — send () to trigger SIGTERM on the child process group.
    /// Using a oneshot so it can be consumed exactly once.
    pub cancel_tx: tokio::sync::oneshot::Sender<()>,
    /// OS pid of the spawned child — used for process group kill.
    pub pid: u32,
}
```

Inside `spawn_command_task`, use `tokio::sync::oneshot::channel()` to create a kill signal. The spawned task selects between the stdout/stderr streams and the kill receiver:

```rust
tokio::select! {
    _ = stream_command_output(stdout, stderr, action_tx.clone()) => {}
    _ = kill_rx => {
        // SIGTERM to process group: -pid kills all children spawned by the command
        unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGTERM); }
        // Brief wait then SIGKILL if still alive
        tokio::time::sleep(Duration::from_millis(200)).await;
        let _ = child.kill().await;
    }
}
let _ = child.wait().await;
let _ = action_tx.send(Action::CommandExited);
```

The `libc` crate is already in `Cargo.toml` — no new dependencies.

**Process group setup:** The child must be spawned with `process_group(0)` so it gets its own PGID equal to its PID. Add `.process_group(0)` to the `tokio::process::Command` builder in `spawn_command_task`. This matches what the existing metro process does (same pattern in `infra/process.rs`).

**New signature:**
```rust
pub async fn spawn_command_task(
    spec: CommandSpec,
    worktree_id: WorktreeId,        // NEW — passed through for TaskRecord creation
    worktree_path: PathBuf,
    current_branch: String,
    action_tx: UnboundedSender<Action>,
) -> CommandHandle
```

The `action_tx` sends `Action::CommandOutputLine` and `Action::CommandExited` exactly as before, but `CommandExited` needs to carry the `TaskId` so `update()` can look up the right record:

```rust
// action.rs changes:
CommandExited,                  // CHANGE to:
CommandExited { task_id: TaskId },  // so update() can find the right record
CommandOutputLine(String),      // CHANGE to:
CommandOutputLine { task_id: TaskId, line: String },  // route to correct worktree
```

This is the critical routing fix: output lines carry their `TaskId`, and `update()` resolves `task_id → worktree_id → command_output_by_worktree[worktree_id]`. The currently selected worktree is no longer involved in routing.

### Layer 3: app.rs — AppState and update()

**AppState changes (additions, no removals):**

```rust
// REPLACE:
pub running_command: Option<CommandSpec>,
pub command_task: Option<JoinHandle<()>>,
pub command_queue: VecDeque<CommandSpec>,

// WITH:
/// All in-flight command tasks, keyed by TaskId.
/// Allows multiple worktrees to run commands concurrently.
pub tasks: HashMap<TaskId, (TaskRecord, CommandHandle)>,

/// Per-worktree command queue. Each worktree has its own FIFO queue.
/// Drained on CommandExited for that worktree's task.
pub command_queue_by_worktree: HashMap<WorktreeId, VecDeque<CommandSpec>>,

/// Task ID counter — monotonically increasing, wraps at u64::MAX (safe in practice).
pub next_task_id: u64,

/// Animation tick counter — incremented on every 250ms tick.
/// Spinner frame = tick_count % SPINNER_FRAMES.len()
pub tick_count: u64,
```

The old `command_queue` (global FIFO) becomes `command_queue_by_worktree` (per-worktree FIFO). The queue drain logic in `CommandExited` now only drains the queue for the worktree that just finished, not the global queue.

**Metro single-instance constraint stays intact.** Metro lives in `AppState.metro: MetroManager` which is unchanged. The "only one metro at a time" rule is at the domain layer in `MetroManager::register()` which panics on double-register. This is independent of the command task system.

**dispatch_command() changes:**

```rust
fn dispatch_command(
    state: &mut AppState,
    worktree_id: WorktreeId,   // explicit — no longer derived from selected row
    spec: CommandSpec,
    metro_tx: &UnboundedSender<Action>,
) {
    let task_id = TaskId(state.next_task_id);
    state.next_task_id += 1;

    // ... output routing, separator line (unchanged logic, uses worktree_id directly) ...

    let record = TaskRecord {
        id: task_id.clone(),
        worktree_id: worktree_id.clone(),
        spec: spec.clone(),
        started_at: std::time::Instant::now(),
    };

    let tx = metro_tx.clone();
    let path = /* looked up from state.worktrees */;
    let branch = /* looked up from state.worktrees */;
    let tid = task_id.clone();

    let handle = tokio::spawn(async move {
        let cmd_handle = spawn_command_task(spec, worktree_id, path, branch, tx, tid).await;
        // drive the handle
    });
    // store (record, handle) in state.tasks[task_id]
}
```

**Action::CommandExited { task_id } changes:**

```rust
Action::CommandExited { task_id } => {
    let Some((record, _handle)) = state.tasks.remove(&task_id) else { return; };
    let worktree_id = record.worktree_id.clone();

    // post-command staleness refresh (existing logic, unchanged)
    // ...

    // Drain THIS WORKTREE'S queue only
    if let Some(queue) = state.command_queue_by_worktree.get_mut(&worktree_id) {
        if let Some(next_spec) = queue.pop_front() {
            dispatch_command(state, worktree_id, next_spec, metro_tx);
        }
    }
}
```

**Action::CommandCancel changes:**

The existing `CommandCancel` maps to `Char('X')` in `CommandOutput` focus. Under the new system, this should cancel the task for the *selected worktree* (the one whose output is visible). The implementation:

```rust
Action::CommandCancel => {
    // Find task for the selected worktree
    let wt_id = active_worktree_id(state);
    if let Some(id) = wt_id {
        // Find task belonging to this worktree
        let task_id = state.tasks.iter()
            .find(|(_, (rec, _))| rec.worktree_id == id)
            .map(|(tid, _)| tid.clone());
        if let Some(tid) = task_id {
            if let Some((rec, handle)) = state.tasks.remove(&tid) {
                if rec.spec.is_cancellable() {
                    let _ = handle.cancel_tx.send(());
                    // handle.join_handle will complete via CommandExited action
                }
                // For non-cancellable (git): just drop the streaming, process finishes naturally
                handle.join_handle.abort();
            }
        }
    }
    // Clear this worktree's queue too
    if let Some(id) = active_worktree_id(state) {
        state.command_queue_by_worktree.remove(&id);
    }
}
```

**Tick counter increment:** In the `run()` event loop tick arm:

```rust
_ = tick.tick() => {
    state.tick_count = state.tick_count.wrapping_add(1);
    // existing behavior: triggers redraw
}
```

**Per-worktree task lookup helper** (for ui/ consumption):

```rust
/// Returns the running task for a given worktree, if any.
pub fn task_for_worktree<'a>(
    state: &'a AppState,
    id: &WorktreeId,
) -> Option<&'a TaskRecord> {
    state.tasks.values()
        .find(|(rec, _)| &rec.worktree_id == id)
        .map(|(rec, _)| rec)
}
```

### Layer 4: ui/ — Spinner and Elapsed Time

**Spinner frames constant (in `ui/theme.rs` or a new `ui/spinner.rs`):**

```rust
pub const SPINNER_FRAMES: [char; 6] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴'];

pub fn spinner_frame(tick_count: u64) -> char {
    SPINNER_FRAMES[(tick_count as usize) % SPINNER_FRAMES.len()]
}
```

At 250ms tick, 6 frames = 1.5s per rotation. That is visually appropriate for a terminal spinner (not too fast, clearly animated).

**Worktree table row rendering in `ui/panels.rs`:**

The worktree table currently renders Y/P indicators as static styled cells. Under the new design:

- If `task_for_worktree(state, &wt.id)` returns `Some(record)`:
  - Render `spinner_frame(state.tick_count)` in yellow bold
  - Render `elapsed: {record.started_at.elapsed().as_secs()}s` in a dim style
  - The spinner replaces the Y/P/T static character for the active task category
- If `None`: render the static Y/P/T as today

The elapsed time renders in the same column as the task indicator, after the spinner. The column width may need to be widened slightly (from fixed `2` to `8–10` characters) to fit `spinner + elapsed`.

**No new async in ui/.** The `tick_count` is plain `u64` in `AppState`. `spinner_frame()` is a pure function. `task_for_worktree()` returns a reference. The render function remains a pure function of `&AppState`.

---

## New Types Summary

| Type | Layer | File | Purpose |
|------|-------|------|---------|
| `TaskId` | domain | `domain/command.rs` | Stable opaque handle for an in-flight task |
| `TaskRecord` | domain | `domain/command.rs` | Binds CommandSpec + WorktreeId + start time |
| `TaskCategory` | domain | `domain/command.rs` | Drives spinner style in UI |
| `WorktreeTaskIndicator` | domain | `domain/worktree.rs` | UI-facing task state per worktree row |
| `CommandHandle` | infra | `infra/command_runner.rs` | cancel_tx + join_handle + pid for a running task |

No new files strictly required. All types fit naturally in existing files. The only potential new file would be `ui/spinner.rs` if spinner logic grows beyond two functions — optional.

---

## New and Modified Modules

| Module | Change Type | What Changes |
|--------|-------------|-------------|
| `domain/command.rs` | MODIFY | Add TaskId, TaskRecord, TaskCategory; add task_category() and is_cancellable() to CommandSpec |
| `domain/worktree.rs` | MODIFY | Add WorktreeTaskIndicator |
| `infra/command_runner.rs` | MODIFY | spawn_command_task returns CommandHandle; add process_group(0); add SIGTERM cancellation path |
| `action.rs` | MODIFY | CommandExited gains task_id field; CommandOutputLine gains task_id field |
| `app.rs` | MODIFY (AppState + update()) | Replace running_command/command_task/command_queue with tasks/command_queue_by_worktree/tick_count/next_task_id; update dispatch_command; update CommandExited arm; update CommandCancel arm; add tick_count increment in run() |
| `ui/panels.rs` | MODIFY | Worktree table row rendering: spinner + elapsed replaces static Y/P when task active |
| `ui/theme.rs` | MODIFY (small) | Add SPINNER_FRAMES and spinner_frame() |

Modules that are NOT touched: `domain/metro.rs`, `domain/refresh.rs`, `infra/process.rs`, `infra/config.rs`, `infra/jira.rs`, `infra/multiplexer.rs`, `ui/modals.rs`, `ui/footer.rs`, `event.rs`, `tui.rs`.

---

## Data Flow After Changes

### Parallel Command Execution (New Flow)

```
User presses 'y' > 'i' (yarn install) on worktree A (selected)
    ↓
handle_key() → Action::CommandRun(YarnInstall)
    ↓
update() → dispatch_command(worktree_A_id, YarnInstall)
    ↓
Creates TaskId(1), TaskRecord { worktree_A_id, YarnInstall, now }
Spawns CommandHandle { cancel_tx, join_handle, pid }
Stores in state.tasks[TaskId(1)]
    ↓
User navigates to worktree B, presses 'g' > 'p' (git pull)
    ↓
update() → dispatch_command(worktree_B_id, GitPull)
    ↓
Creates TaskId(2), TaskRecord { worktree_B_id, GitPull, now }
state.tasks[TaskId(2)] = ...
    ↓
BOTH commands run concurrently in separate tokio tasks.
    ↓
Output lines arrive: Action::CommandOutputLine { task_id: 1, line: "..." }
    ↓
update() → tasks[1].record.worktree_id = A
    → command_output_by_worktree[A].push_back(line)
    ↓
Action::CommandExited { task_id: 1 }
    ↓
update() → remove tasks[1] → drain command_queue_by_worktree[A]
```

### Cancellation Flow (New)

```
User presses 'X' in CommandOutput pane (worktree A selected)
    ↓
Action::CommandCancel
    ↓
update() finds task with worktree_id == A in state.tasks
    ↓
spec.is_cancellable() == true (e.g. YarnInstall)
    ↓
cancel_tx.send(()) → spawned task's kill_rx fires
    ↓
libc::kill(-pid, SIGTERM) → process group receives SIGTERM
    ↓
200ms grace period
    ↓
child.kill().await (SIGKILL if still alive)
    ↓
child.wait().await → CommandExited { task_id } sent
    ↓
update() cleans up state.tasks entry, drains queue
```

### Spinner Animation Flow (New)

```
tokio::time::interval(250ms) fires
    ↓
run() tick arm: state.tick_count += 1
    ↓
terminal.draw() called
    ↓
ui::view() → render_worktree_table()
    ↓
For each worktree row:
    task_for_worktree(state, &wt.id)?
    → spinner_frame(state.tick_count)  ← 6-frame rotation, 1.5s cycle
    → elapsed = record.started_at.elapsed().as_secs()
    → render yellow spinner char + "Xs" elapsed
```

---

## Architecture Audit Notes

The following deviations from Ousterhout and TEA principles exist in the current codebase and should be surfaced in the audit phase before implementing the new features:

**AUDIT-01: dispatch_command() derives worktree from selected table row.**
Location: `app.rs:dispatch_command()`. The function reads `state.worktree_table_state.selected()` to find the worktree at dispatch time. This couples command dispatch to UI cursor position. If the user moves the cursor after dispatching but before `CommandExited`, `running_command` still refers to the original command but `active_worktree_id()` returns the new cursor position. Output is currently routed to the original worktree correctly (path is captured at dispatch), but the running_command display in the UI may show the wrong context. Fix: pass `worktree_id` explicitly into `dispatch_command()` (which the new design does).

**AUDIT-02: AppState contains tokio JoinHandle (process handle).**
`command_task: Option<JoinHandle<()>>` mixes process lifecycle management (infra concern) into AppState (app concern). This violates the boundary described in the original ARCHITECTURE.md. The new `CommandHandle` moves closer to the boundary by pairing the handle with its cancel mechanism, but it still lives in `state.tasks`. This is acceptable for a TUI — unlike a production server, there is no test-isolation requirement for AppState — but should be noted as a pragmatic compromise.

**AUDIT-03: update() calls tokio::spawn directly.**
Throughout `update()`, async work is dispatched with `tokio::spawn(...)`. This is an `async fn` side-effect from a nominally pure update function. It works correctly because `update()` is called from an async context in `run()`, but it breaks the TEA contract (update should be synchronous, return effects, let the loop spawn them). The existing codebase accepts this trade-off for simplicity; the new milestone should preserve this pattern rather than introducing an Effect enum, as the refactor cost is high and the benefit is theoretical at this scale.

**AUDIT-04: Global command_queue is not per-worktree.**
Location: `AppState.command_queue`. The current FIFO queue is global: when worktree A dispatches a clean+install+run sequence, all three commands queue together. If worktree B then dispatches a command, it enters the same global queue and runs after worktree A's entire sequence finishes. The per-worktree queue (`command_queue_by_worktree`) introduced in this milestone fixes this correctly.

**AUDIT-05: MetroHandle in domain/ uses tokio types.**
`domain/metro.rs` contains a comment acknowledging this as a deliberate pragmatic choice. The types are inert data. Leave as is — refactoring adds complexity for no practical benefit at this scale.

---

## Build Order

Dependencies must be built bottom-up. Each step must compile and pass existing tests before the next step starts.

### Step 1: Domain types (no behavioral change)

Add `TaskId`, `TaskRecord`, `TaskCategory` to `domain/command.rs`.
Add `task_category()` and `is_cancellable()` methods to `CommandSpec`.
Add `WorktreeTaskIndicator` to `domain/worktree.rs`.

These are pure additions. Nothing breaks. Compile check passes immediately.

**Dependency:** nothing (self-contained domain layer)

### Step 2: Action enum changes

Add `task_id: TaskId` to `CommandExited` and `CommandOutputLine`.

This is a breaking change to a heavily-used enum. All existing match arms on these variants must be updated. The compiler will flag every location. Expected sites: `update()` (two arms in `app.rs`), `command_runner.rs` (the `action_tx.send()` calls).

Change `CommandQueuePush` to `CommandQueuePush { worktree_id: WorktreeId, spec: CommandSpec }` to prepare for per-worktree queuing.

**Dependency:** Step 1 (TaskId must exist)

### Step 3: infra/command_runner.rs — CommandHandle + process group

Introduce `CommandHandle`. Update `spawn_command_task` to:
- Accept `worktree_id: WorktreeId` and `task_id: TaskId` parameters
- Call `.process_group(0)` on the child
- Create a `oneshot::channel` for kill signaling
- Return `CommandHandle` instead of `JoinHandle<()>`
- Pass `task_id` in `CommandOutputLine` and `CommandExited` sends

**Dependency:** Steps 1 and 2 (TaskId and updated Action variants must exist)

### Step 4: AppState restructure

Replace `running_command`, `command_task`, `command_queue` with `tasks`, `command_queue_by_worktree`, `next_task_id`, `tick_count`.

Update `Default` impl accordingly (all new fields have obvious defaults).

Update `dispatch_command()` to accept explicit `worktree_id`, create `TaskRecord`, store `CommandHandle` in `state.tasks`.

**Dependency:** Steps 1–3 (TaskRecord and CommandHandle must exist)

### Step 5: update() arm updates

Update `Action::CommandExited { task_id }` to remove from `state.tasks`, drain per-worktree queue.
Update `Action::CommandOutputLine { task_id, line }` to route to correct worktree via `tasks[task_id].worktree_id`.
Update `Action::CommandCancel` to find task by active worktree, call cancel semantics.
Update `Action::CommandQueuePush` to push to `command_queue_by_worktree`.
Update `run()` tick arm to increment `state.tick_count`.
Update `run()` cleanup to abort all tasks in `state.tasks`.

**Dependency:** Steps 1–4

### Step 6: UI — spinner and elapsed time

Add `SPINNER_FRAMES` and `spinner_frame()` to `ui/theme.rs`.
Update `render_worktree_table()` in `ui/panels.rs` to call `task_for_worktree()` per row and render spinner + elapsed.

**Dependency:** Steps 4 and 5 (tick_count and task_for_worktree must exist)

### Step 7: Architecture audit verification

After all behavioral changes compile and tests pass, audit the five AUDIT-0x items above. The ones that warrant a refactor (primarily AUDIT-01 which the new design already fixes, and AUDIT-04 which Step 5 fixes) will be resolved by the milestone work itself. AUDIT-02 and AUDIT-03 should be documented as accepted trade-offs in a code comment. AUDIT-05 already has its comment.

---

## Pitfalls Specific to This Milestone

**PITFALL-M1: SIGTERM without process_group(0) kills only the shell, not subprocesses.**
`yarn install` spawns node as a child of sh. `kill(-pid, SIGTERM)` only works if the child was spawned with `process_group(0)`. Without it, `-pid` references PGID 0 (the terminal's process group) which would kill the entire terminal session. Add `.process_group(0)` in `spawn_command_task` and verify this is not already set (metro's process.rs sets it there, not in command_runner.rs).

**PITFALL-M2: CommandExited variant change breaks existing match arms everywhere.**
The compiler catches all sites but the fix is mechanical. Do this change in one commit so the compiler guides all required edits.

**PITFALL-M3: Per-worktree queue + parallel execution creates ordering ambiguity for queue chains.**
A queue like `[YarnInstall, RnRunAndroid]` in worktree A relies on YarnInstall completing before the run command starts. This is preserved because the per-worktree queue still drains sequentially within a worktree. Parallel execution only applies across worktrees. This invariant must be maintained by ensuring `CommandExited { task_id }` only drains the queue for `tasks[task_id].worktree_id`.

**PITFALL-M4: Spinner tick rate drives all redraws, including when idle.**
The 250ms tick fires regardless of whether any task is running. This is already the case (tick exists today). It is not a new cost. Do not increase tick frequency for faster spinner animation — 250ms is already visually responsive and the existing rate is validated.

**PITFALL-M5: task_for_worktree() is O(n) over all tasks.**
Under normal conditions `state.tasks` has at most N entries where N is the number of worktrees (typically 5–15). An O(n) scan is negligible. Do not add a secondary index unless profiling shows a real cost.

---

## Sources

All findings are HIGH confidence — derived from direct code reading of the existing codebase (2026-04-13).

- `/Users/cubicme/aljazeera/dashboard/src/app.rs` — AppState, update(), dispatch_command(), run()
- `/Users/cubicme/aljazeera/dashboard/src/action.rs` — Action enum, all 50+ variants
- `/Users/cubicme/aljazeera/dashboard/src/domain/command.rs` — CommandSpec, ModalState, CleanOptions
- `/Users/cubicme/aljazeera/dashboard/src/domain/metro.rs` — MetroHandle, MetroManager
- `/Users/cubicme/aljazeera/dashboard/src/domain/worktree.rs` — Worktree, WorktreeId
- `/Users/cubicme/aljazeera/dashboard/src/infra/command_runner.rs` — spawn_command_task, stream_command_output
- `/Users/cubicme/aljazeera/dashboard/src/event.rs` — Event enum
- `/Users/cubicme/aljazeera/dashboard/Cargo.toml` — dependency versions (libc already present)
- `/Users/cubicme/aljazeera/dashboard/.planning/PROJECT.md` — v1.3 milestone requirements

---
*Architecture research for: v1.3 Per-Worktree Tasks + Architecture Audit*
*Researched: 2026-04-13*
*Supersedes prior ARCHITECTURE.md sections on command system (adds new sections, existing patterns remain valid)*
