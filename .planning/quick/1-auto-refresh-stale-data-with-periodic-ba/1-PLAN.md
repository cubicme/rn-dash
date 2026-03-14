---
phase: quick
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - src/app.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "Worktree data (branches, staleness, JIRA titles) auto-refreshes every 60 seconds without user interaction"
    - "Labels reload from disk on each periodic refresh so external edits are picked up"
    - "Manual Shift-R refresh still works as before"
    - "Refresh does not fire if a command is currently running (avoids interference)"
  artifacts:
    - path: "src/app.rs"
      provides: "Periodic refresh timer in event loop + label reload on WorktreesLoaded"
  key_links:
    - from: "src/app.rs (event loop tick)"
      to: "Action::RefreshWorktrees"
      via: "60-second interval check in the existing select! loop"
      pattern: "last_refresh.*elapsed"
---

<objective>
Add periodic background polling (every 60 seconds) that auto-refreshes worktrees, staleness indicators, labels, and JIRA titles. This prevents stale data from accumulating when the user leaves the dashboard open without interacting.

Purpose: Data goes stale quickly (branch changes, installs, label edits). Currently only manual Shift-R or post-command hooks refresh data. A periodic timer ensures the dashboard stays current even when idle.
Output: Modified app.rs with periodic refresh logic.
</objective>

<execution_context>
@/Users/cubicme/.claude/get-shit-done/workflows/execute-plan.md
@/Users/cubicme/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/app.rs
@src/action.rs
@src/domain/refresh.rs
@src/infra/worktrees.rs
@src/infra/labels.rs
</context>

<interfaces>
<!-- Key types and contracts the executor needs. -->

From src/action.rs:
```rust
pub enum Action {
    RefreshWorktrees,     // manual refresh keybinding
    WorktreesLoaded(Vec<crate::domain::worktree::Worktree>), // background refresh done
    JiraTitlesFetched(Vec<(String, String)>),  // (ticket_key, title)
}
```

From src/app.rs (event loop structure):
```rust
// Existing 250ms tick for redraws
let mut tick = tokio::time::interval(std::time::Duration::from_millis(250));

// Event loop uses tokio::select! with branches: tick, events, metro_rx, handle_rx
```

From src/infra/labels.rs:
```rust
pub fn load_labels() -> anyhow::Result<HashMap<String, String>>;
```
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Add periodic auto-refresh timer to event loop</name>
  <files>src/app.rs</files>
  <action>
Add a 60-second periodic refresh to the existing event loop in `run()`. Implementation:

1. **Add `last_refresh` field to AppState:**
   ```rust
   pub last_refresh: std::time::Instant,
   ```
   Default it to `std::time::Instant::now()` in `Default` impl.

2. **Add a dedicated `tokio::time::interval` for refresh in `run()`:**
   Right after the existing `tick` interval (line ~1584), add:
   ```rust
   let mut refresh_interval = tokio::time::interval(std::time::Duration::from_secs(60));
   refresh_interval.tick().await; // consume the immediate first tick
   ```

3. **Add a new branch to the `tokio::select!` block (line ~1646):**
   ```rust
   _ = refresh_interval.tick() => {
       // Skip refresh if a command is running (avoid interfering with output)
       if state.running_command.is_none() {
           update(&mut state, Action::RefreshWorktrees, &metro_tx, &handle_tx);
       }
   }
   ```
   Place this branch after the existing tick branch and before `maybe_event`.

4. **Reload labels from disk inside the `Action::WorktreesLoaded` handler:**
   At the top of the `WorktreesLoaded` match arm (before the worktree processing), add:
   ```rust
   // Reload labels from disk — picks up external edits
   state.labels = crate::infra::labels::load_labels().unwrap_or_default();
   ```
   This ensures that both manual Shift-R refreshes AND periodic refreshes reload labels, since both paths go through WorktreesLoaded.

The 60-second interval is cheap because `git worktree list --porcelain` and `check_stale()` are fast filesystem operations. The existing `WorktreesLoaded` handler already re-derives metro status, checks staleness, and triggers JIRA title fetches when branch names change -- so the periodic refresh gets all of that for free.

Do NOT add a `last_refresh` field to AppState -- the `tokio::time::interval` handles timing internally. The `last_refresh` field mentioned above is unnecessary; remove it if you started adding it.
  </action>
  <verify>
    <automated>cd /Users/cubicme/aljazeera/dashboard && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>
    - 60-second refresh interval added to event loop select! block
    - Refresh skipped when a command is running
    - Labels reloaded from disk on every WorktreesLoaded (covers both periodic and manual refresh)
    - `cargo check` passes with no errors
  </done>
</task>

</tasks>

<verification>
- `cargo check` compiles without errors or new warnings
- Manual test: run the dashboard, wait 60+ seconds, observe worktree data refreshes (visible via tracing logs if RUST_LOG=debug)
- Manual test: Shift-R still triggers immediate refresh
- Manual test: while a command is running, the 60s tick does NOT trigger a refresh
</verification>

<success_criteria>
Dashboard auto-refreshes worktrees, staleness, labels, and JIRA titles every 60 seconds when idle. No refresh interference during active command execution.
</success_criteria>

<output>
After completion, create `.planning/quick/1-auto-refresh-stale-data-with-periodic-ba/1-SUMMARY.md`
</output>
