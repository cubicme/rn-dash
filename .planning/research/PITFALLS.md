# Domain Pitfalls

**Domain:** Adding per-worktree parallel task execution + cancellation + animated UI indicators to existing Rust/tokio/Ratatui TUI (rn-dash v1.3)
**Researched:** 2026-04-13
**Confidence:** HIGH (tokio official docs, ratatui official docs, yarn issue tracker, verified sources)

> This file supersedes the v1.0-era PITFALLS.md for the v1.3 milestone. It focuses exclusively on pitfalls specific to the features being added: per-worktree task ownership, tokio task cancellation, process kill semantics, spinner animation, parallel yarn installs, architecture audit regressions, and TEA purity.

---

## Critical Pitfalls

### Pitfall 1: Dropping a JoinHandle Does NOT Cancel the Task

**Category:** Concurrency

**What goes wrong:**
A `tokio::spawn` task is launched for a worktree command. The JoinHandle is stored in AppState per-worktree. When the user cancels, the code drops the JoinHandle. The task continues running silently in the background — the child process is still alive, still writing output to shared state, still potentially mutating the worktree's status. The UI shows "no task running" while the process is actually running.

**Why it happens:**
This is a well-documented tokio trap: "Dropping the JoinHandle does NOT cancel the running task." Dropping only detaches the handle; the spawned future keeps executing until it completes or is explicitly aborted. Newcomers (and code reviewers) assume Rust's Drop = cleanup, but for `JoinHandle` it means "I'm done watching the task," not "the task is done."

**Consequences:**
- Zombie tokio tasks accumulate across cancellations
- Two tasks for the same worktree run concurrently after the second launch
- Background task writes to per-worktree output after new task has started, corrupting log display
- Child process (yarn, jest, etc.) outlives the "cancelled" task handle

**Prevention:**
Use explicit abort: `handle.abort()` for immediate cancellation where cleanup is not needed (e.g., yarn install that hasn't written anything yet). Use `CancellationToken` where the task needs to observe cancellation and perform cleanup (e.g., flush partial output, update status to Cancelled).

Store per-worktree task state as:
```rust
struct WorktreeTask {
    cancel: CancellationToken,
    handle: JoinHandle<()>,
    kind: TaskKind,
    started_at: Instant,
}
```
On cancel: call `cancel.cancel()` first (gives task a chance to observe), then `handle.abort()` as backstop. Never rely on drop.

**Warning signs:**
- "Cancelled" task's output lines still appear in the output panel after cancellation
- `ps aux` shows yarn/jest child processes after the user cancelled
- Second task launch for same worktree shows double output streams
- `tokio-console` shows tasks in Running state for worktrees marked as idle

**Phase to address:** Phase that implements per-worktree task tracking (likely Phase 2 of v1.3). Establish the cancel+abort pattern in the TaskHandle abstraction before any command dispatch code is written.

---

### Pitfall 2: Child Process Survives Task Cancellation (Orphaned Process)

**Category:** Cancellation

**What goes wrong:**
A tokio task spawned for `yarn install` is aborted or cancelled. The task future drops, running destructors. But the `tokio::process::Child` inside the task was already spawned — unless `kill_on_drop(true)` was set, the child process continues running independently of the Rust task that spawned it. The user sees the task as cancelled in the UI, but the terminal (and filesystem) are still in the middle of a yarn install.

**Why it happens:**
Tokio's documentation is explicit: "a spawned process will, by default, continue to execute even after the Child handle has been dropped." When a task is aborted at an `.await` point, Rust runs destructors for values in scope, but if `Child` is inside an async block at a non-`.await` point, it may not be reachable for cleanup. Even with `kill_on_drop(true)`, SIGKILL is sent but wait is never called, leaving a zombie.

**Consequences:**
- Parallel yarn installs in multiple worktrees start, user cancels some; orphaned yarn processes continue competing for file locks and global cache
- `node_modules` left in partial state (yarn died mid-install)
- Port conflicts for run-android/run-ios processes that survived cancellation
- Zombie processes accumulate across a long session

**Prevention:**
Two-layer defence:
1. Set `kill_on_drop(true)` on every `Command` before `.spawn()` — catches task abort path
2. In the `CancellationToken` observed path, explicitly `child.kill().await` then `child.wait().await` before returning — ensures process is reaped

For the SIGTERM → SIGKILL grace period (desired for yarn/jest which write files): send SIGTERM via `nix::sys::signal::kill(pid, Signal::SIGTERM)`, then `tokio::time::timeout(Duration::from_secs(5), child.wait()).await`. If timeout fires, `child.kill().await` (SIGKILL), then `child.wait().await`.

Note: The existing process-group kill for metro is a different, already-implemented path — do not refactor it during the audit without regression tests.

**Warning signs:**
- `ps aux | grep yarn` shows running yarn processes after user cancelled
- `node_modules/.yarn-integrity` is absent or corrupt after cancellation
- lsof shows file handles held on `node_modules` after task cancelled
- Dashboard reports idle worktree but disk I/O is still active

**Phase to address:** Phase that adds cancellation support. Address kill semantics in the TaskRunner abstraction, not at each call site.

---

### Pitfall 3: Shared AppState Write Race from Parallel Worktree Tasks

**Category:** Concurrency

**What goes wrong:**
Multiple worktrees run concurrent tasks. Each task calls back into AppState (via mpsc channel or Arc<Mutex>) to update task status, append output lines, update elapsed time. If the channel is unbounded and two tasks flush large output bursts simultaneously, or if Mutex contention is high, the render loop lags. Worse: if the update logic is not idempotent (e.g., `output.push(line)` after a task is already cancelled), stale data appears in the UI.

**Why it happens:**
The existing architecture uses a single mpsc channel to feed Actions into the TEA event loop. This is correct. The pitfall is adding *direct* AppState mutation in task callbacks (bypassing the channel) for "performance" reasons, or using `Arc<Mutex<AppState>>` shared between tasks. Either breaks the single-writer guarantee of the TEA model.

**Consequences:**
- Output lines from worktree A appear under worktree B's output panel (wrong routing key)
- Status shows "Running" after "Cancelled" because a late-arriving channel message overwrites the cancel
- Mutex deadlock if a task holds the AppState lock and tries to send on the channel while the render loop holds the channel and tries to acquire the AppState lock

**Prevention:**
Keep the single mpsc channel as the only write path into AppState. All task callbacks send typed Actions: `TaskOutput { worktree_id, line }`, `TaskStatusChanged { worktree_id, status }`, `TaskCompleted { worktree_id, elapsed }`. The update() function handles ordering and deduplication.

Use worktree_id as the routing key on every action. In update(), guard status transitions: only accept `TaskCompleted` if the task was still in Running state for that worktree_id. Discard late-arriving output from cancelled tasks.

Tag each launched task with a `task_generation: u64` counter per worktree. Actions carry the generation. In update(), discard actions whose generation doesn't match the current one.

**Warning signs:**
- Output from worktree A visible in worktree B's output panel
- Status transitions go Running → Cancelled → Running (out-of-order channel delivery)
- `Arc<Mutex<AppState>>` appears anywhere in the task spawning code
- Direct field mutation on AppState from outside the update() function

**Phase to address:** Architecture audit phase. Establish the routing-key-on-every-action pattern before adding parallel task dispatch.

---

### Pitfall 4: Spinner Animation Blocks or Starves the Event Loop

**Category:** UI Animation

**What goes wrong:**
A spinner tick timer is added with `tokio::time::interval(Duration::from_millis(100))`. The select! loop handles tick events by advancing the frame counter and immediately re-rendering. Under load (output streaming from 3 parallel tasks + key events + JIRA callbacks), the tick arm fires faster than the render loop can complete. The loop becomes CPU-bound spinning between tick and render, starving key event processing. Alternatively, if spinner ticks are sent as mpsc Actions, the channel fills with tick messages, delaying user input by hundreds of milliseconds.

**Why it happens:**
`tokio::time::interval` does not drop missed ticks (by default uses `MissedTickBehavior::Burst`). If one interval fires and the handler takes >100ms (e.g., rendering a complex layout with 6 worktree rows), the next tick fires immediately. Two fast ticks in a row can crowd out crossterm events in the select! arms.

**Consequences:**
- Key presses are delayed or dropped during active parallel builds
- Spinner advances at inconsistent speed (burst vs. stall)
- CPU usage spikes during builds when spinner + output streaming compete
- Flicker: spinner frame advances but layout hasn't been re-rendered, so display jumps

**Prevention:**
Set `MissedTickBehavior::Skip` on the spinner interval so missed ticks are discarded rather than bursted:
```rust
let mut spinner_tick = tokio::time::interval(Duration::from_millis(150));
spinner_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
```

Do NOT send spinner ticks through the mpsc Action channel. Instead, drive spinner frame advancement from a field in AppState that is updated directly in the select! tick arm, before the render call. The spinner frame is UI state, not domain state — it does not need to go through TEA update().

Rate-limit renders: use a dirty flag. Only call `terminal.draw()` if `app_state.dirty` is true. Mark dirty on any Action received OR on spinner tick. This caps render calls at the interval rate even if multiple actions arrive in the same tick.

Use a single unified tick (e.g., 100ms) that drives both spinner advancement and "should I render" decisions, rather than separate intervals for spinner and render.

**Warning signs:**
- CPU > 10% when builds are running in multiple worktrees
- Key presses appear with >200ms lag during active tasks
- Spinner visually stutters (skips frames or shows same frame twice)
- `tokio-console` shows the render task as the hottest by poll count

**Phase to address:** Phase that introduces spinner animation. Establish the tick architecture (MissedTickBehavior::Skip, dirty flag) before adding any animated indicators.

---

### Pitfall 5: Per-Worktree Parallel Yarn Installs Collide on Global Cache or node_modules

**Category:** Yarn/Node ecosystem

**What goes wrong:**
Two worktrees are both enqueued for yarn install (e.g., user has 3 feature branches, all stale). The dashboard dispatches concurrent yarn install processes. Both processes write to `node_modules` in their respective worktree directories. However, they share the same `yarn.lock` file (at the monorepo root, symlinked or via `--cwd`). Yarn Classic (v1) has a confirmed issue: "concurrent yarn calls in parallel directories causes cache failure" (yarnpkg/yarn#8854). Yarn Berry global cache is designed to be safe for concurrent access, but has known edge cases in CI where one instance removes files another was about to read.

**Specific collision vectors:**
1. **yarn.lock file lock**: Yarn acquires an exclusive lock on `yarn.lock` during install. Second concurrent install blocks waiting, then fails or produces inconsistent output
2. **Global cache mutex**: Yarn Berry has a global cache lock, but race conditions exist when two instances unpack the same package simultaneously  
3. **`node_modules` post-install scripts**: `postinstall` scripts (CocoaPods, native modules) that modify shared global state (gem cache, pod cache) can race
4. **`.yarn-integrity` sentinel**: The existing staleness detection reads `.yarn-integrity`; if two installs write it simultaneously, the sentinel becomes unreliable

**Prevention:**
Do NOT dispatch concurrent yarn installs for the same repo root. Serialize them: maintain a per-repo-root semaphore (a `tokio::sync::Semaphore` with permits=1) that yarn install tasks must acquire before spawning. Allow concurrent execution only across entirely separate repos.

For the common case (all worktrees in same monorepo): queue yarn installs as sequential within a worktree but allow the user to run installs in parallel across worktrees by acquiring the semaphore. Show a "waiting for yarn lock" status in the UI so the user understands the serialization.

Alternatively: detect that all worktrees share the same repo root and offer "install all worktrees sequentially" as a single action rather than dispatching N parallel installs.

**Warning signs:**
- yarn exits with non-zero code with "lockfile conflict" or "cache write failed" error
- `node_modules` in one worktree has missing packages after parallel install
- `.yarn-integrity` is corrupt (non-JSON content) after parallel install
- One worktree's install succeeds while another's fails with no user-visible queuing

**Phase to address:** Phase that implements parallel task dispatch. The semaphore must be established in the TaskRunner before any yarn install can be dispatched in parallel.

---

## Moderate Pitfalls

### Pitfall 6: AbortHandle Fires at the Wrong Await Point (Cancel Safety)

**Category:** Cancellation

**What goes wrong:**
A task is aborted via `handle.abort()`. The task was in the middle of a multi-step async operation (e.g., between `child.kill().await` and `child.wait().await`, or between writing output to the channel and updating status). The abort fires at an arbitrary `.await` point, leaving the task half-done: the child is killed but not reaped (zombie), or the output channel has partial data with no corresponding status update.

**Why it happens:**
`abort()` causes the task to be cancelled at the next `.await` point. Code between two `.await` calls runs atomically from the cancellation perspective, but code across multiple `.await` calls does not. This is Rust's "cancel safety" problem. The tokio docs note: "It is very difficult to write code that is cancel-safe at any arbitrary await point."

**Prevention:**
Prefer `CancellationToken` over raw `abort()` for tasks that have cleanup steps. Structure tasks as:
```rust
tokio::select! {
    _ = token.cancelled() => {
        // explicit cleanup: kill child, wait, update status
    }
    result = run_command(&mut child) => {
        // normal completion
    }
}
```
Use `abort()` only as a final backstop after the cancellation token has been given time to fire (e.g., 1-second timeout after token.cancel() before handle.abort()).

Reserve `abort()` for tasks with no cleanup requirements (e.g., a pure computation task with no child processes or file handles).

**Warning signs:**
- Tasks intermittently leave child processes alive after cancellation (non-deterministic)
- Status shows "Cancelled" but output panel keeps receiving lines for a few seconds after
- `child.wait()` is never called on the Cancelled path (easy to miss in abort path)

**Phase to address:** Same phase as Pitfall 1 (per-worktree task tracking). Document the cancel-then-abort sequence in the TaskHandle abstraction.

---

### Pitfall 7: Metro Single-Instance Invariant Broken During Architecture Audit Refactor

**Category:** Refactor Regressions

**What goes wrong:**
The audit identifies that metro management code has leaked into the wrong layer (e.g., UI layer calls MetroManager directly). During refactor to move it to the domain layer, a new code path is introduced that can start metro without going through the single-instance check. Or: the refactor introduces a new `MetroStarted` action path that doesn't go through the existing kill-before-start guard, allowing two metro instances to run simultaneously across worktrees.

**Why it happens:**
The single-instance invariant is enforced by a state check in the existing code. During refactor, this check may be copy-pasted into the new location but not the old location (leaving two enforcement points that can diverge), or it may be omitted from a new fast-path added during the refactor.

**Consequences:**
- Two metro bundlers running simultaneously on the same port → second one crashes immediately
- Both metros attempt to bind port 8081, one wins, the other's logs appear in the wrong worktree panel
- Existing external conflict detection (lsof-based) fires against our own metro process

**Prevention:**
Before any metro-related refactor: write a characterization test that asserts the invariant. Even a simple integration test that verifies `MetroManager::start()` called twice returns an error on the second call. The test must pass before refactor begins and must still pass after.

Make the invariant structurally enforced: if `MetroManager` is moved to domain layer, its `start()` function should return `Err(AlreadyRunning)` if a metro is already active — not rely on callers to check. This is the Ousterhout "make errors impossible" principle.

**Warning signs:**
- Two entries with "metro" visible in `ps aux` after a worktree switch
- Port 8081 conflict detected by lsof check against our own process
- The single-instance check appears in more than one place in the codebase after refactor

**Phase to address:** Architecture audit phase. Write the invariant test before touching any metro-related code.

---

### Pitfall 8: Modal Flow Broken by New Action Variants During Refactor

**Category:** Refactor Regressions

**What goes wrong:**
New action variants added for per-worktree task cancellation (e.g., `CancelTask(WorktreeId)`, `TaskCancelled(WorktreeId)`) are not handled in the modal routing section of `handle_key()`. When a modal is open (e.g., `SyncBeforeMetro` confirmation dialog) and a background task completes or is cancelled, the incoming Action is dispatched through the modal path, hits an unhandled arm, and either panics (if the match is exhaustive) or silently does nothing (if there's a catch-all). Either way the modal gets stuck open or the task status is never updated.

**Why it happens:**
TEA's `update()` and `handle_key()` are large match expressions. Adding new Action variants requires adding arms in multiple places (modal handler, normal handler, render logic). The Rust compiler enforces exhaustive matching on enums, so panics are possible if `_` fallthrough arms hide the gap. Silent failures occur when `_` arms exist in modal branches.

**Prevention:**
Remove all `_ => {}` catch-all arms from modal match expressions. Force the compiler to surface every unhandled Action in every modal state. This is a safe refactor to do as part of the architecture audit — exhaustive matches are a free static invariant checker.

When adding new Action variants, run `cargo check` and use compiler errors as the checklist of "places that need updating."

Structure the modal update logic as: background Actions (TaskOutput, TaskStatusChanged, TaskCompleted, TaskCancelled) are always processed regardless of modal state. Only user input Actions (key presses that drive modal transitions) are gated by modal state.

**Warning signs:**
- `_ => {}` arms exist in modal handling match expressions
- A modal can be opened and then the app becomes unresponsive to background events
- A new Action variant compiles cleanly but task status never updates when a modal is open

**Phase to address:** Architecture audit phase (exhaustive match cleanup), and any phase that adds new Action variants.

---

### Pitfall 9: TEA Purity Violation — tokio::spawn Inside update()

**Category:** TEA Purity

**What goes wrong:**
The `update()` function is the pure domain brain of the TEA loop. A developer adds `tokio::spawn(...)` directly inside an `update()` arm to launch a task when an Action arrives (e.g., on `Action::RunYarnInstall(id)`, immediately spawn the task). This works initially but creates problems: `update()` is no longer testable without a tokio runtime, side effects happen inside what should be a pure function, and the spawned task handle has nowhere to be stored (it's inside `update()` which returns nothing except the next state). The handle is dropped immediately, triggering the Pitfall 1 trap (but since it's not aborted, the task runs — the handle is just lost).

**Why it happens:**
The temptation is strong: the action arrives, the work should start, spawn it here. The TEA pattern says `update()` should return an effect descriptor instead, and the event loop should execute the effect. But this pattern is not enforced by the type system in a hand-rolled TEA implementation.

**Consequences:**
- Task handles are immediately dropped (no way to cancel later)
- Unit tests of `update()` require `#[tokio::test]` and a real runtime
- Side effects execute twice if `update()` is called twice for replay/debugging
- Ordering of spawns is unpredictable relative to state updates

**Prevention:**
`update()` must never call `tokio::spawn`, `tokio::fs`, `std::process::Command`, or any I/O function directly. Instead, it returns a `Vec<Effect>` (or pushes to an effect queue field on AppState):

```rust
enum Effect {
    SpawnTask { worktree_id: WorktreeId, spec: CommandSpec },
    CancelTask { worktree_id: WorktreeId },
    KillProcess { pid: u32 },
}
```

The event loop calls `update()`, receives effects, and dispatches them. This keeps `update()` pure, testable, and the single source of truth for state transitions.

**Warning signs:**
- `tokio::spawn` or `std::process::Command` appears inside `update()` function body
- `#[tokio::test]` required to test any update() arm
- Task handles are created as local variables inside update() with no way to store them

**Phase to address:** Architecture audit phase. Establish the Effect enum pattern before any new task dispatch code is written.

---

### Pitfall 10: Spinner Frame Counter in Domain State (Ousterhout Violation)

**Category:** TEA Purity / Architecture

**What goes wrong:**
A `spinner_frame: u8` field is added to `AppState` (the domain model) to track the current animation frame. The tick action updates it. This means the domain struct carries pure UI display state, violating the domain/UI separation. During the architecture audit, this will be correctly identified as a violation and refactored out — causing a second churn on the same code if it was put there initially.

**Why it happens:**
AppState is the convenient place to put things. The spinner frame needs to persist between renders. AppState persists between renders. Therefore: put it in AppState. The logic is locally correct but violates the Ousterhout principle that domain state should not contain UI rendering decisions.

**Prevention:**
Spinner frame state belongs in the UI layer, not the domain model. Options:
1. A separate `UiState` struct that lives alongside `AppState` in the event loop
2. Computed directly in the render function from `task.started_at.elapsed()` without storing frame state at all: `let frame = (elapsed.as_millis() / 150) % 6` gives a deterministic frame index from wall time

The second option is superior: no mutable state, no tick-driven counter, deterministic for the same elapsed time, naturally consistent across re-renders.

**Warning signs:**
- `spinner_frame` or `animation_tick` field in the domain AppState struct
- Spinner logic in the `update()` function body
- Tests for domain update arms accidentally verify animation frame values

**Phase to address:** Phase that introduces spinner animation. Use the elapsed-time formula from day one.

---

## Minor Pitfalls

### Pitfall 11: Task Elapsed Time Computed with Instant::now() in render()

**Category:** UI Animation

**What goes wrong:**
The worktree table render function calls `Instant::now()` to compute task elapsed time for display. This is called once per render frame. The output is correct but it means the render function has a side effect (reading the system clock) and the elapsed time displayed is subtly non-deterministic (depends on when in the frame the render fires). Not a bug in practice, but technically impure. More concretely: if snapshot-based UI tests are added later, time-based display strings will always differ.

**Prevention:**
Pass a `render_time: Instant` into the render function as a parameter, captured once per tick in the event loop. All elapsed time calculations use `render_time - task.started_at`. The render function remains pure. Tests can pass a fixed `render_time`.

---

### Pitfall 12: process-group kill Refactored Away During Audit

**Category:** Refactor Regressions

**What goes wrong:**
The architecture audit identifies that process-group kill logic is in the wrong layer. During refactor to move it, the `killpg(pgid, SIGKILL)` call is replaced with `child.kill()` (which only kills the direct child, not grandchildren). Metro's Node.js process spawns child processes (the bundler worker, the file watcher). If only the parent Node.js PID is killed, the workers survive and hold port 8081 open.

**Prevention:**
The process-group kill behavior is an intentional invariant documented in the v1.2 post-ship notes. Before touching any process kill code: write a test or document the expected behavior (kill pgid, not just pid) and add a comment in the code explaining why `killpg` is used instead of `child.kill()`. Treat this as a regression risk in the audit checklist.

---

### Pitfall 13: Per-Worktree Output VecDeque Grows Unbounded During Long Tasks

**Category:** Memory

**What goes wrong:**
A jest test run or long yarn install produces thousands of output lines. The existing per-worktree output VecDeque has a cap (already present from v1.0). But with parallel tasks, multiple worktrees accumulate output simultaneously. If the cap is per-worktree and set to 1000 lines, 6 concurrent worktrees can accumulate 6000 lines in memory. With the task status and elapsed time also stored, the AppState footprint grows.

**Prevention:**
Keep the existing per-worktree cap. Do not raise it for the parallel case. Make the cap a const with a comment explaining the footprint budget (e.g., `const MAX_OUTPUT_LINES: usize = 500; // ~200KB per worktree at avg 400 bytes/line`). The parallel case does not change the per-worktree cap — it just means multiple worktrees may be at their cap simultaneously.

---

### Pitfall 14: CancellationToken Child vs. Clone Confusion

**Category:** Cancellation

**What goes wrong:**
A `CancellationToken` is created for each worktree's task. A developer uses `.child_token()` thinking it's equivalent to `.clone()` for sharing the token between the spawning code and the task. Later, the spawning code calls `parent_token.cancel()` (the worktree-level token) but the child task's token is never cancelled — because cancelling a parent cancels children, but a separate cancel of the parent (owned by the per-worktree state) does not cancel a sibling. The confusion between `.clone()` (bidirectional) and `.child_token()` (parent → child only) causes cancellation to silently not propagate.

**Prevention:**
For the simple case (cancel a single task): use `.clone()` to share the same token between the task owner and the task. Use `.child_token()` only when you have a hierarchy (cancel all tasks for a worktree when the worktree is removed, while allowing individual task cancellation via the cloned leaf token).

Document the token topology in the `WorktreeTask` struct definition.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Per-worktree task tracking | JoinHandle drop doesn't cancel (Pitfall 1) | Establish cancel+abort pattern in TaskHandle abstraction first |
| Task cancellation | Child process orphan on abort (Pitfall 2) | `kill_on_drop(true)` + explicit kill→wait in cancel path |
| Parallel task dispatch | AppState write race via direct mutation (Pitfall 3) | Route all task callbacks through mpsc Action channel with worktree_id routing key |
| Spinner animation | Tick starvation of key events (Pitfall 4) | `MissedTickBehavior::Skip`, dirty flag, compute frame from elapsed time (Pitfall 10) |
| Parallel yarn installs | yarn.lock / global cache collision (Pitfall 5) | Per-repo-root `Semaphore(1)` before any yarn install spawn |
| Architecture audit: metro | Single-instance invariant regression (Pitfall 7) | Characterization test before touching MetroManager |
| Architecture audit: action enum | Modal deadlock from unhandled new variants (Pitfall 8) | Remove all `_ => {}` catch-all arms in modal match |
| Architecture audit: update() | tokio::spawn inside update() (Pitfall 9) | Effect enum pattern; update() returns effects, event loop executes them |
| Architecture audit: process kill | killpg replaced with child.kill() (Pitfall 12) | Comment documenting why killpg is required; treat as audit regression risk |
| Cancel safety | abort() at wrong await point (Pitfall 6) | CancellationToken for cleanup tasks, abort() only as backstop |

---

## Integration Gotchas Specific to v1.3

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| tokio JoinHandle | Drop to cancel | Call `handle.abort()` explicitly; drop only detaches |
| tokio CancellationToken | Use `.child_token()` to share between two peers | Use `.clone()` for peer sharing; `.child_token()` for hierarchy only |
| tokio::process::Child | Assume kill() prevents zombie | Always follow `kill().await` with `wait().await` |
| tokio::time::interval | Default burst on missed ticks starves events | `set_missed_tick_behavior(Skip)` for UI tick intervals |
| Yarn installs | Dispatch concurrently per-worktree | Serialize within same repo root via Semaphore |
| TEA update() | Add tokio::spawn for immediate task dispatch | Return Effect; let event loop spawn |
| Spinner frame | Store in AppState domain struct | Compute from `elapsed.as_millis() / interval % frames` in render |
| MetroManager refactor | Move code without preserving kill semantics | killpg is intentional; document before touching |

---

## "Looks Done But Isn't" Checklist for v1.3

- [ ] **Cancel actually stops the process:** After cancel, `ps aux | grep yarn` shows no running yarn in the cancelled worktree within 2 seconds
- [ ] **No orphaned tasks on abort:** `tokio-console` shows zero Running tasks for worktrees marked idle after cancellation
- [ ] **Metro single-instance after audit:** Switch worktrees 10 times; `ps aux | grep metro` shows exactly one metro process
- [ ] **Spinner CPU:** With 3 concurrent tasks running, CPU usage stays below 5%
- [ ] **Key lag during builds:** Key presses processed within 100ms even during active parallel yarn installs
- [ ] **Parallel yarn doesn't corrupt:** Run 2 simultaneous yarn installs; both complete with valid `node_modules` and non-corrupt `.yarn-integrity`
- [ ] **Modal stays responsive:** Open a confirmation modal, let a background task complete; modal still accepts input and closes correctly
- [ ] **update() purity:** `grep -n "tokio::spawn\|std::process" src/app/update.rs` returns zero results
- [ ] **process-group kill preserved:** After metro kill, `lsof -i :8081` shows port free; no metro child processes remain
- [ ] **Elapsed time display:** Spinner shows consistent frame rate with no stutter at 150ms intervals under load

---

## Sources

- [tokio::process::Child docs — drop, kill_on_drop, zombie warning](https://docs.rs/tokio/latest/tokio/process/struct.Child.html) (HIGH confidence)
- [tokio zombie processes issue #2685](https://github.com/tokio-rs/tokio/issues/2685) (HIGH confidence)
- [tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown) (HIGH confidence)
- [CancellationToken docs — child vs clone, cancellation atomicity](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html) (HIGH confidence)
- [Rust tokio task cancellation patterns (2024)](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/) (MEDIUM confidence)
- [Tokio abort subtasks — Rust forum](https://users.rust-lang.org/t/tokio-how-to-abort-one-task-and-all-its-subtasks/121153) (MEDIUM confidence)
- [Ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) (HIGH confidence)
- [Ratatui TEA pattern documentation](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/) (HIGH confidence)
- [Ratatui event handling concepts](https://ratatui.rs/concepts/event-handling/) (HIGH confidence)
- [Yarn concurrent parallel install cache failure #8854](https://github.com/yarnpkg/yarn/issues/8854) (HIGH confidence)
- [Yarn berry lock cache before changes #4102](https://github.com/yarnpkg/yarn/issues/4102) (MEDIUM confidence)
- [Git worktrees parallel AI agent execution — Augment Code](https://www.augmentcode.com/guides/git-worktrees-parallel-ai-agent-execution) (MEDIUM confidence)
- [Use git worktrees — Dave Schumaker (node_modules scale issues)](https://daveschumaker.net/use-git-worktrees-they-said-itll-be-fun-they-said/) (MEDIUM confidence)

---
*Pitfalls research for: rn-dash v1.3 Per-Worktree Tasks + Architecture Audit*
*Researched: 2026-04-13*
