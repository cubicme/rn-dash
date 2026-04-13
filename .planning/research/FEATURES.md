# Feature Research

**Domain:** TUI developer workspace/process manager (Rust/Ratatui, React Native worktrees) — v1.3 per-worktree task system + architecture audit
**Researched:** 2026-04-13
**Confidence:** HIGH (task cancellation patterns — verified against tokio-util docs and canonical blog post), HIGH (UI indicator patterns — verified against throbber-widgets-tui and Ratatui ecosystem), HIGH (architecture audit patterns — verified against cargo-modules and Ousterhout literature), MEDIUM (elapsed-time tick pattern — standard Instant::now() but no ratatui-specific prior art for per-row display found)

---

## Context: What Is Already Built

The v1.0–v1.2 foundation is complete. Existing state relevant to v1.3:

- `AppState.command_queue`: `VecDeque<CommandSpec>` — single global FIFO queue
- `AppState.running_command`: `Option<CommandSpec>` — single global running slot
- `AppState.command_task`: `Option<JoinHandle<()>>` — single global task handle
- `AppState.command_output_by_worktree`: `HashMap<WorktreeId, VecDeque<String>>` — output is already per-worktree (good foundation)
- `spawn_command_task`: spawns a child process with `kill_on_drop(true)`, streams stdout/stderr via `Action::CommandOutputLine`, sends `Action::CommandExited` on completion
- `CommandSpec.is_destructive()` and `CommandSpec.needs_metro()` — classification predicates already exist
- `Action::CommandCancel` — already defined, but currently aborts via `JoinHandle::abort()` on the single global handle
- Y/P letters in the worktree table icon column — currently static green/red staleness indicators
- No elapsed time tracking, no per-worktree task state, no spinner frames

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features the v1.3 milestone must deliver for the product to feel complete at this level of maturity. Missing any of these makes the feature set feel half-built.

| Feature | Why Expected | Complexity | Existing Dependency |
|---------|--------------|------------|---------------------|
| Per-worktree task ownership | Commands dispatched in a worktree context must remain bound to that worktree — not a global slot. A user dispatching `yarn install` on worktree A while worktree B is already running a test must produce independent, attributed output. | MEDIUM | `command_output_by_worktree` already keyed — need to extend `running_command` and `command_task` to the same HashMap pattern |
| Parallel execution across worktrees | If worktree A is running `yarn install` and the user switches to worktree B and dispatches `run-android`, both should run concurrently. Serializing across worktrees is a regression: users with multiple worktrees open expect independent task lanes. | HIGH | Requires replacing the single `command_task: Option<JoinHandle>` with `HashMap<WorktreeId, JoinHandle>` |
| Individual task cancellation for yarn/clean/install/run/test | Accidental dispatches happen. A 10-minute `yarn install` on the wrong worktree must be cancellable without killing everything else. This is table stakes because `Action::CommandCancel` already exists in the codebase — users already expect it to work. | MEDIUM | `JoinHandle::abort()` is already used; needs to become per-worktree abort rather than global. Child process has `kill_on_drop(true)` so task abort cascades to process kill. |
| Git operations remain non-cancellable and globally serialized | Git ops (reset, pull, push, rebase, fetch) must not be interrupted mid-operation — a half-applied rebase or a partial push leaves the repo in a broken state. This is not a missing feature; it is a deliberate constraint that must be surfaced in the UI. | LOW | Enforce via `CommandSpec::is_cancellable()` predicate — Git variants return false; all others return true |
| Metro stays single-instance globally | v1.0–v1.2 already enforces this. v1.3 must not regress: parallelism across worktrees for tasks must not extend to metro. Metro is still one instance at a time, managed globally by `MetroManager`. | LOW | No new work — preserve existing constraint, explicitly document that per-worktree task parallelism excludes metro |
| Output panel shows correct worktree's output | When the user navigates to worktree B in the table, the output panel must show B's output stream, not A's. Already partially true via `command_output_by_worktree` but must be fully consistent with the per-worktree task model. | LOW | Already built in `active_output(state)` — keep consistent |

### Differentiators (Competitive Advantage)

Features that distinguish v1.3 from a basic task runner. These map to the core value of "one place to see and control everything about your worktrees."

| Feature | Value Proposition | Complexity | Existing Dependency |
|---------|-------------------|------------|---------------------|
| Live task name + elapsed time in worktree table row | Each worktree row shows the currently running task label and a live HH:MM:SS or MM:SS elapsed counter. This eliminates the need to focus the output panel to know what's happening across multiple worktrees. No comparable TUI worktree manager does this. | MEDIUM | Requires `task_started_at: Option<Instant>` per worktree in `AppState`; tick events (already present in the event loop via `tokio::select!`) drive re-render. `std::time::Instant::elapsed()` is sufficient — no new dependencies. |
| 6-frame rotating yellow spinner replacing Y/P during active tasks | The Y/P staleness letters are replaced by an animated spinner (frames: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` or `◐◓◑◒` or a 6-char custom set) rendered in yellow when a yarn-like or pod-like task is running on that worktree. Run/test tasks get the same treatment in the run column. Returns to Y/P (with staleness colors) when idle. | MEDIUM | `throbber-widgets-tui` exists in the Ratatui ecosystem (listed in official third-party widgets showcase). However, the custom 6-frame set is simple enough to implement inline with a tick-counter modulo 6 — no external dep needed. Frame advancement driven by the existing tick event. |
| Per-worktree task queue (not global) | Commands queued for worktree A wait for A's current task to finish, independently of B's queue state. This is a natural progression of the existing global `VecDeque<CommandSpec>` — just move it into the per-worktree task record. | MEDIUM | Existing `command_queue: VecDeque<CommandSpec>` becomes `HashMap<WorktreeId, VecDeque<CommandSpec>>` |
| Architecture audit surfacing real violations | A structured audit against Ousterhout deep-module criteria and domain/infra/app/ui boundary rules produces a prioritized list of actual regressions, not a theoretical checklist. Refactoring phases address only what the audit finds — not speculative cleanup. | MEDIUM | `cargo-modules` (visualize/analyze crate internal structure, detect cycles, report visibility) and manual Ousterhout checklist; no new runtime dependencies |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why It Seems Appealing | Why It Is Problematic | Better Approach |
|---------|------------------------|----------------------|-----------------|
| CancellationToken (tokio-util) instead of JoinHandle::abort() for task cancellation | `CancellationToken` is the recommended graceful-shutdown pattern in the tokio ecosystem and allows cooperative cleanup. | For shell commands (`yarn install`, `npx react-native run-android`), cooperative cancellation requires the child process to respond to a signal — which most yarn/RN processes do not implement. The existing `kill_on_drop(true)` on `tokio::process::Command` already correctly propagates SIGKILL to the child process when the task is aborted. `CancellationToken` adds complexity without benefit here because the "cleanup" we want is process kill, not async teardown logic. | Keep `JoinHandle::abort()` for task cancellation. The child process is killed by `kill_on_drop(true)`. Add `Action::CommandCancelForWorktree(WorktreeId)` that calls `abort()` on the per-worktree handle. |
| Task parallelism inside a single worktree | Allowing two tasks to run simultaneously in the same worktree (e.g., `yarn install` and `yarn lint` both running in worktree A) seems like more power. | Two concurrent processes writing to the same `node_modules` directory will corrupt each other. RN build tools are not parallelism-safe within a single worktree. The FIFO queue per worktree is the correct model. | Enforce one active task per worktree via the existing queue model — just move the queue to per-worktree scope. |
| Persisting task history across restarts | Showing a log of past tasks and their outcomes (succeeded/failed/cancelled) could be useful for audit trail purposes. | Adds persistent storage, file I/O in the hot path, migration concerns, and UI surface area with no direct benefit to the current workflow. The output buffer per worktree (`VecDeque<String>`) already provides in-session history. | Keep the existing in-memory `VecDeque<String>` per worktree. No disk persistence of task history in v1.3. |
| Global "cancel all" action | A single keystroke to cancel all running tasks across all worktrees seems convenient. | Dangerous: cancelling a nearly-finished `yarn install` or a `run-android` that is mid-deployment could leave devices or build artifacts in a broken state. The per-worktree cancel is deliberate and targeted. | Only expose per-worktree cancel. If the user needs to stop everything, they can cancel each worktree individually or quit the dashboard. |
| Parallel architecture refactor phases (do audit and refactor simultaneously) | Seems efficient — fix things as you find them. | The audit phase must complete before refactor phases begin. Running them in parallel means refactoring against a moving target, invalidating discoveries, and producing incomplete diffs. | Strict two-phase approach: Phase 1 = audit only (read + document), Phase 2+ = refactor phases derived from audit findings. |
| Metric-based fitness functions for architecture enforcement (e.g., ArchUnit-style) | Automated enforcement of architecture rules (no `ui` importing `infra`, no domain depending on tokio) sounds rigorous. | Rust's visibility system (`pub(crate)`, `pub(super)`) already enforces module boundaries at compile time. Adding a runtime or CI fitness-function framework for a ~6K LOC single-crate project is over-engineering. The right tool is `cargo-modules` for visualization and manual Ousterhout checklist review, not a rules engine. | Use `cargo-modules graph` to visualize actual import edges; use Clippy lints for complexity signals; use manual review against the Ousterhout checklist for interface depth assessment. |
| Animated spinner as a separate widget crate dependency | `throbber-widgets-tui` is a well-maintained Ratatui ecosystem crate that provides spinner widgets. | Adding a new runtime dependency for 6 unicode characters and a modulo counter is not warranted. The crate adds compile time and a transitive Ratatui version coupling risk. | Inline implementation: a `tick_counter: u64` in `AppState`, incremented on each tick event. Spinner frame = `FRAMES[tick_counter % 6]` where `FRAMES` is a `[&str; 6]` const. Total implementation: ~5 lines. |

---

## Feature Dependencies

```
[Per-Worktree Task State]
    └──requires──> [WorktreeTaskRecord: task_handle, queue, started_at]
    └──requires──> [action_tx per-worktree routing]
    └──replaces──> [global running_command / command_task / command_queue in AppState]

[Task Cancellation]
    └──requires──> [Per-Worktree Task State] (need per-worktree JoinHandle)
    └──requires──> [CommandSpec::is_cancellable()] (new predicate, git=false, rest=true)
    └──uses──> [JoinHandle::abort() + kill_on_drop(true)] (already correct in command_runner)

[Parallel Execution]
    └──requires──> [Per-Worktree Task State] (independent task lanes)
    └──conflicts with──> [Metro single-instance] (parallelism excludes metro — preserve existing constraint)
    └──conflicts with──> [Serial within worktree] (queue enforces serial within one worktree)

[Live UI Indicators: task name + elapsed time]
    └──requires──> [Per-Worktree Task State] (task_started_at: Option<Instant>)
    └──requires──> [Tick event in event loop] (already present in app.rs tokio::select!)
    └──enhances──> [Worktree table row render] (adds task label + MM:SS column or inline span)

[Animated Spinner (Y/P replacement)]
    └──requires──> [Tick event] (already present)
    └──requires──> [tick_counter: u64 in AppState] (new field, trivial)
    └──requires──> [CommandSpec category predicates] (yarn-like vs run-like vs test-like)
    └──enhances──> [Worktree table icon column render] (replace Y/P spans with spinner frame spans)

[Architecture Audit]
    └──requires──> [cargo-modules CLI] (external tool, no crate dep)
    └──requires──> [Ousterhout checklist] (manual review)
    └──produces──> [Audit findings doc] (input to refactor phases)
    └──gates──> [Refactor phases] (must complete before any structural changes)

[Refactor Phases]
    └──requires──> [Architecture Audit findings] (must know what to refactor)
    └──requires──> [Full test suite green] (cargo test, cargo clippy -D warnings)
    └──conflicts with──> [Per-Worktree Task feature work] (do not run audit+refactor in parallel with task system changes — pick one per phase)
```

### Dependency Notes

- **Per-Worktree Task State is the foundation**: Every other v1.3 feature (cancellation, parallelism, live indicators, spinner) depends on replacing the three global task fields in `AppState` with a `HashMap<WorktreeId, WorktreeTaskRecord>`. This must be the first feature built.
- **Architecture audit gates all structural refactor work**: The audit must run against the current codebase before any structural changes. Running the per-worktree task feature work first is acceptable because it adds new fields/logic without restructuring existing module boundaries. The audit findings will then assess the post-task-feature state.
- **Spinner and elapsed time share the same tick signal**: Both features advance on the same tick event. The tick counter modulo 6 drives spinner frame; `Instant::elapsed()` drives the elapsed display. No additional timer infrastructure needed.
- **`kill_on_drop(true)` is the cancellation mechanism**: Already set in `command_runner.rs`. Task abort → JoinHandle dropped → `kill_on_drop` kills child process. No additional cleanup logic required.
- **`action_tx` routing does not change**: Background tasks already send `Action::CommandOutputLine` and `Action::CommandExited` via the shared `mpsc::UnboundedSender<Action>`. With per-worktree tasks, `CommandOutputLine` and `CommandExited` need to carry a `WorktreeId` so `update()` can route them to the correct worktree's record. This is a targeted change to the `Action` enum variants.

---

## Architecture Audit Feature — What "Good" Looks Like

This is a distinct category: the audit is a process, not a runtime feature. Here is what the audit must cover and what good output looks like.

### Audit Scope

| Area | What to Check | Tool / Method |
|------|---------------|---------------|
| Module import graph | Does any module in `domain/` import from `infra/` or `ui/`? Does `ui/` import domain types directly (allowed) or infra types (not allowed)? | `cargo-modules graph --package rn-dash` — visualize actual edges |
| Shallow vs deep modules | Does each public module's interface justify its implementation size? Are there modules with large `pub` surfaces but trivial logic (shallow)? Are there modules with complex logic hidden behind a narrow interface (deep — good)? | Manual checklist: count `pub fn` vs total `fn` per module; flag any module where `pub` items exceed 50% of items |
| Information leakage | Are implementation details exposed in public types? Do callers need to know things they shouldn't? | Manual review of `pub` fields in domain structs; flag any `pub` fields that could be `pub(crate)` or removed |
| Temporal coupling | Are there sequences of function calls that must happen in a specific order with no enforcement? | Manual review of `AppState` mutation patterns in `update()` — flag "must set X before Y" patterns with no type-level enforcement |
| Comment density vs code clarity | Ousterhout: good comments describe *what* and *why*, not *how*. Code should be self-documenting. | Grep for comments that restate the code; flag functions where the comment is longer than the body |
| `app.rs` size and God Object risk | At ~2,200+ lines, `app.rs` contains `AppState`, `update()`, `dispatch_command()`, and the event loop. This is a known risk area. | Count public items; check if `update()` could be split into domain-specific handler modules |

### Audit Output Format

The audit produces a prioritized findings doc, not a fix-everything list. Each finding has:
- **Severity**: Critical (violates a boundary), Major (makes future changes harder), Minor (style/polish)
- **Location**: file, line range
- **Description**: what the problem is
- **Ousterhout principle violated**: which one (e.g., "shallow module", "information leakage", "temporal coupling")
- **Recommended fix**: concrete action

### What the Audit Must NOT Do

- It must not propose speculative refactors ("this could be cleaner if...") without a concrete problem statement
- It must not flag working code as broken just because it is long
- It must not recommend splitting modules for splitting's sake — Ousterhout explicitly warns against over-decomposition
- It must not change behavior during the audit phase — audit is read-only

---

## MVP Definition for v1.3

This milestone's MVP is not about minimum viable product — the product already ships. It is about what v1.3 must deliver to be releasable as a complete increment.

### Must Deliver (v1.3 complete)

- [ ] Architecture audit complete — findings doc exists, prioritized by severity
- [ ] Refactor phases derived from audit — each critical/major finding addressed
- [ ] Per-worktree task ownership — `running_command`, `command_task`, `command_queue` are per-worktree not global
- [ ] `Action::CommandOutputLine` and `Action::CommandExited` carry `WorktreeId` for correct routing
- [ ] Parallel execution — dispatching a command on worktree B while A is running works correctly
- [ ] `CommandSpec::is_cancellable()` predicate — git ops return false, all others return true
- [ ] `Action::CommandCancelForWorktree(WorktreeId)` — cancels the task handle for that worktree only
- [ ] Live task name visible in worktree table row during active task
- [ ] Live elapsed time visible in worktree table row (MM:SS, updated by tick)
- [ ] Spinner frame replaces Y/P during active yarn-like or pod-like task; returns to Y/P when idle
- [ ] Run/test tasks show equivalent spinner in a separate column or the same icon column with distinct color

### Defer to v1.4+

- [ ] Persistent task history (succeeded/failed/cancelled log)
- [ ] Global cancel-all action
- [ ] Per-worktree task queue priority (priority lanes, not just FIFO)
- [ ] Configurable spinner style

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Architecture audit | HIGH (foundational) | LOW | P1 — do first, gates refactors |
| Refactor phases (per audit findings) | HIGH | MEDIUM–HIGH | P1 — do before task system work |
| Per-worktree task state (foundation) | HIGH | MEDIUM | P1 — all other task features depend on this |
| `CommandOutputLine`/`CommandExited` carry `WorktreeId` | HIGH | LOW | P1 — must accompany per-worktree state |
| Parallel execution across worktrees | HIGH | LOW (once per-worktree state exists) | P1 |
| Individual task cancellation | HIGH | LOW (abort on per-worktree handle) | P1 |
| `CommandSpec::is_cancellable()` + git non-cancellable | MEDIUM | LOW | P1 |
| Live task name in worktree row | MEDIUM | LOW | P2 |
| Live elapsed time in worktree row | MEDIUM | LOW | P2 |
| Animated spinner (Y/P replacement) | MEDIUM | LOW | P2 |
| Run/test task spinner indicator | LOW | LOW | P2 |

**Priority key:**
- P1: Must have for v1.3 to be complete
- P2: Should have in v1.3, can be phase 2 of the milestone
- P3: Nice to have, defer to v1.4+

---

## Implementation Complexity Notes

### WorktreeTaskRecord struct (new domain type)

The core structural change. Replaces three global `AppState` fields:

```
struct WorktreeTaskRecord {
    running_command: Option<CommandSpec>,   // replaces AppState.running_command
    task_handle:     Option<JoinHandle<()>>, // replaces AppState.command_task
    queue:           VecDeque<CommandSpec>, // replaces AppState.command_queue
    started_at:      Option<std::time::Instant>, // new — for elapsed time
}
```

Stored as `HashMap<WorktreeId, WorktreeTaskRecord>` in `AppState`. The three existing global fields remain during migration, removed once all call sites are updated. This is a targeted, mechanical refactor — no logic changes in `spawn_command_task` or `command_runner.rs`.

### Action enum changes

`CommandOutputLine(String)` becomes `CommandOutputLine { worktree_id: WorktreeId, line: String }`.
`CommandExited` becomes `CommandExited { worktree_id: WorktreeId }`.
`CommandCancel` (existing global) is supplemented by `CommandCancelForWorktree(WorktreeId)`.

Impact: all match arms in `update()` that handle these variants must be updated. This is the highest-touch change in the milestone — estimate 40–80 lines of `update()` to update.

### Tick counter for spinner

```
// In AppState:
pub tick_counter: u64,  // incremented on each Tick event

// In update():
Action::Tick => { state.tick_counter = state.tick_counter.wrapping_add(1); }

// In render:
const SPINNER_FRAMES: [&str; 6] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴"];
let frame = SPINNER_FRAMES[(state.tick_counter % 6) as usize];
```

Confidence: HIGH — this is a standard pattern in all TUI apps with animation. No external dependencies.

### Elapsed time display

```
let elapsed = task_record.started_at
    .map(|t| t.elapsed())
    .unwrap_or_default();
let secs = elapsed.as_secs();
let display = format!("{:02}:{:02}", secs / 60, secs % 60);
```

Confidence: HIGH — `std::time::Instant` is stable, this is a direct computation. Render on every tick.

---

## Sources

- [tokio-util CancellationToken docs](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html) — confirmed CancellationToken pattern; determined it is not the right fit here (cooperative, child processes do not cooperate)
- [Rust tokio task cancellation patterns — Cybernetist (2024)](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/) — confirmed JoinHandle::abort() vs CancellationToken tradeoffs
- [tokio JoinHandle docs](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html) — abort() semantics, drop-does-not-cancel behavior
- [throbber-widgets-tui crate](https://docs.rs/throbber-widgets-tui/latest/throbber_widgets_tui/) — confirmed spinner widget exists; decided to implement inline to avoid dependency
- [Ratatui third-party widgets showcase](https://ratatui.rs/showcase/third-party-widgets/) — throbber-widgets-tui listed as official ecosystem widget
- [cargo-modules GitHub](https://github.com/regexident/cargo-modules) — visualize/analyze Rust crate internal structure, detect cycles; confirmed as the right audit tool
- [A Philosophy of Software Design — Ousterhout](https://www.amazon.com/Philosophy-Software-Design-John-Ousterhout/dp/1732102201) — deep module / shallow module criteria for audit checklist
- Existing codebase: `/Users/cubicme/aljazeera/dashboard/src/` — direct reading of `app.rs`, `action.rs`, `domain/command.rs`, `infra/command_runner.rs`, `ui/panels.rs`

---
*Feature research for: v1.3 Per-Worktree Task System + Architecture Audit (rn-dash)*
*Researched: 2026-04-13*
