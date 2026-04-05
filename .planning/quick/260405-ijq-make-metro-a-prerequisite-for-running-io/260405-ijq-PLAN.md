---
phase: quick-260405-ijq
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/app.rs
  - src/domain/command.rs
autonomous: true
must_haves:
  truths:
    - "When user triggers i>e, i>d, a>e, or a>d and metro is not running, metro starts automatically before the build begins"
    - "The RN run command dispatches only after metro reports Ready activity"
    - "If metro is already running, RN run commands dispatch immediately (no change in behavior)"
  artifacts:
    - path: "src/app.rs"
      provides: "pending_metro_run field on AppState, metro prerequisite check in CommandRun pipeline, auto-dispatch in MetroActivityUpdate"
    - path: "src/domain/command.rs"
      provides: "needs_metro() method on CommandSpec"
  key_links:
    - from: "Action::CommandRun pre-processing"
      to: "Action::MetroStart"
      via: "stores command in pending_metro_run, triggers MetroStart"
    - from: "Action::MetroActivityUpdate(Ready)"
      to: "dispatch_command"
      via: "takes pending_metro_run and dispatches it"
---

<objective>
Make metro a prerequisite for RN run commands (run-ios, run-android, run-ios-device, release-build). When the user triggers an RN run command and metro is not running, the dashboard auto-starts metro first and dispatches the run command only after metro reports Ready. This prevents React Native CLI from spawning its own unmanaged metro instance.

Purpose: Eliminate the race where RN CLI opens a separate metro terminal that the dashboard cannot control.
Output: Modified app.rs with metro prerequisite gate in the CommandRun pipeline.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@src/app.rs
@src/action.rs
@src/domain/command.rs
@src/domain/metro.rs

<interfaces>
From src/domain/metro.rs:
```rust
pub enum MetroStatus { Stopped, Starting, Running { pid: u32, worktree_id: WorktreeId }, Stopping }
pub enum MetroActivity { Starting, Ready, Bundling, DeviceConnected(String) }
impl MetroManager {
    pub fn is_running(&self) -> bool;  // true if handle is Some
    pub status: MetroStatus;
    pub activity: Option<MetroActivity>;
}
```

From src/domain/command.rs:
```rust
pub enum CommandSpec {
    RnRunAndroid { device_id: String, mode: Option<String> },
    RnRunIos { device_id: String },
    RnRunIosDevice,
    RnReleaseBuild,
    // ... other variants
}
pub fn needs_device_selection(&self) -> bool;
pub fn needs_text_input(&self) -> bool;
pub fn is_destructive(&self) -> bool;
```

From src/app.rs:
```rust
pub struct AppState {
    pub metro: MetroManager,
    pub active_worktree_path: Option<PathBuf>,
    pub pending_restart: bool,
    pub command_queue: VecDeque<CommandSpec>,
    // ...
}
fn dispatch_command(state: &mut AppState, spec: CommandSpec, metro_tx: &UnboundedSender<Action>);
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add needs_metro() to CommandSpec and pending_metro_run to AppState</name>
  <files>src/domain/command.rs, src/app.rs</files>
  <action>
1. In `src/domain/command.rs`, add a `needs_metro()` method to `CommandSpec`:
   ```rust
   /// Returns true for commands that require metro to be running before dispatch.
   pub fn needs_metro(&self) -> bool {
       matches!(self,
           CommandSpec::RnRunAndroid { .. }
           | CommandSpec::RnRunIos { .. }
           | CommandSpec::RnRunIosDevice
           | CommandSpec::RnReleaseBuild
       )
   }
   ```

2. In `src/app.rs`, add a field to `AppState`:
   ```rust
   /// RN run command waiting for metro to become Ready before dispatch.
   pub pending_metro_run: Option<crate::domain::command::CommandSpec>,
   ```
   Initialize to `None` in `AppState::new()`.
  </action>
  <verify>
    <automated>cd /Users/cubicme/aljazeera/dashboard && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>needs_metro() returns true for all four RN run variants, pending_metro_run field exists on AppState initialized to None, cargo check passes</done>
</task>

<task type="auto">
  <name>Task 2: Wire metro prerequisite gate in CommandRun pipeline and auto-dispatch on Ready</name>
  <files>src/app.rs</files>
  <action>
1. In the `Action::CommandRun(spec)` handler in `update()`, add a metro prerequisite check **after** the palette_mode clear (line ~785) and **before** the sync-before-run check (line ~796). The check should be:

   ```rust
   // Metro prerequisite: RN run commands need metro running first
   if spec.needs_metro() && !state.metro.is_running() {
       // Store the run command — will be dispatched when metro reports Ready
       state.pending_metro_run = Some(spec);
       update(state, Action::MetroStart, metro_tx, handle_tx);
       return;
   }
   ```

   This placement means: if metro is not running, we start it and stash the command. The rest of the pipeline (sync-before-run, destructive confirm, device selection) runs AFTER metro is confirmed ready, because the command re-enters CommandRun from step 2 below.

2. In the `Action::MetroActivityUpdate(activity)` handler (currently just sets `state.metro.activity = Some(activity)`), add after the existing line:

   ```rust
   // Auto-dispatch pending RN run command when metro becomes Ready
   if matches!(activity, crate::domain::metro::MetroActivity::Ready) {
       if let Some(run_spec) = state.pending_metro_run.take() {
           // Re-enter the full CommandRun pipeline (sync check, device selection, etc.)
           update(state, Action::CommandRun(run_spec), metro_tx, handle_tx);
       }
   }
   ```

   Using `Action::CommandRun` (not `dispatch_command` directly) ensures the stashed command goes through the full pre-processing pipeline (sync-before-run, device picker, etc.) which it had not yet traversed.

3. In `Action::MetroExited` handler, clear any pending metro run to avoid stale commands if metro fails to start:
   ```rust
   // Clear pending run command if metro exited unexpectedly
   state.pending_metro_run = None;
   ```
   Add this near the top of the MetroExited handler, before the `pending_restart` check.

4. Similarly in `Action::MetroSpawnFailed`, clear pending_metro_run:
   ```rust
   state.pending_metro_run = None;
   ```
  </action>
  <verify>
    <automated>cd /Users/cubicme/aljazeera/dashboard && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>
    - Triggering a>e or i>e when metro is stopped causes metro to start, then the run command dispatches after metro logs "Welcome to Metro"
    - If metro is already running, run commands dispatch immediately with no behavioral change
    - If metro fails to start (MetroSpawnFailed/MetroExited), the pending command is cleared (no stale dispatch)
    - The pending command re-enters the full CommandRun pipeline on Ready, so sync-before-run and device picker still work correctly
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

No new trust boundaries introduced. All changes are internal state management within the existing TEA update loop.

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-quick-01 | D (Denial of Service) | pending_metro_run | accept | Single Option field — at most one pending command. If metro never reaches Ready, MetroExited clears it. No resource leak. |
</threat_model>

<verification>
1. `cargo check` passes with no errors
2. `cargo clippy` passes with no new warnings
3. Manual test: with metro stopped, press `a>e` — metro should start, then device picker should appear after metro logs Ready
</verification>

<success_criteria>
- RN run commands (i>e, i>d, a>e, a>d, a>r) auto-start metro when it is not running
- The run command dispatches only after metro reports Ready activity
- Existing behavior unchanged when metro is already running
- No stale pending commands after metro failure
</success_criteria>

<output>
After completion, create `.planning/quick/260405-ijq-make-metro-a-prerequisite-for-running-io/260405-ijq-SUMMARY.md`
</output>
