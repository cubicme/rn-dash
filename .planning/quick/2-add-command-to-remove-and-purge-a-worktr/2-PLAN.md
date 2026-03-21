---
phase: quick-2
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - src/action.rs
  - src/app.rs
  - src/infra/worktrees.rs
  - src/ui/help_overlay.rs
  - src/ui/footer.rs
autonomous: true
requirements: ["QUICK-2"]
must_haves:
  truths:
    - "User can press g>D on a selected worktree to trigger removal"
    - "Confirm modal appears before destructive removal"
    - "Worktree is removed from git, directory is deleted, and worktree list refreshes"
    - "Cannot remove the main worktree (bare repo / primary checkout)"
    - "Metro is stopped if running on the worktree being removed"
    - "Per-worktree state (command output, scroll) is cleaned up after removal"
  artifacts:
    - path: "src/infra/worktrees.rs"
      provides: "remove_worktree async function"
      contains: "pub async fn remove_worktree"
    - path: "src/action.rs"
      provides: "WorktreeRemove and WorktreeRemoved action variants"
    - path: "src/app.rs"
      provides: "Confirm modal flow + post-removal cleanup in update()"
  key_links:
    - from: "src/app.rs (Git palette)"
      to: "Action::WorktreeRemove"
      via: "g>D keybinding dispatches confirm modal"
    - from: "src/app.rs (update)"
      to: "src/infra/worktrees.rs"
      via: "tokio::spawn calls remove_worktree then RefreshWorktrees"
---

<objective>
Add a worktree removal command accessible via the git palette (g>D) that removes a worktree from git, deletes its directory, cleans up LSP-affecting artifacts, and refreshes the worktree list.

Purpose: Users need to clean up worktrees they no longer need without leaving the dashboard. The command must be safe (confirmation required, cannot remove main worktree) and thorough (git metadata + directory + per-worktree dashboard state all cleaned up).

Output: Working g>D command with confirm modal, worktree removal, and automatic refresh.
</objective>

<execution_context>
@/Users/cubicme/.claude/get-shit-done/workflows/execute-plan.md
@/Users/cubicme/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/action.rs
@src/app.rs
@src/infra/worktrees.rs
@src/domain/worktree.rs
@src/domain/command.rs
@src/ui/help_overlay.rs
@src/ui/footer.rs

<interfaces>
<!-- Key types and contracts the executor needs -->

From src/action.rs:
```rust
pub enum Action {
    // ... existing variants
    // Git palette dispatches CommandRun(CommandSpec) for most commands
    // But worktree removal is NOT a CommandSpec — it's a repo-level operation
    // that needs custom handling (runs in repo_root, not in worktree dir)
}
```

From src/app.rs:
```rust
pub struct AppState {
    pub repo_root: std::path::PathBuf,
    pub worktrees: Vec<Worktree>,
    pub worktree_table_state: ratatui::widgets::TableState,
    pub selected_worktree_id: Option<WorktreeId>,
    pub active_worktree_path: Option<std::path::PathBuf>,
    pub metro: MetroManager,
    pub command_output_by_worktree: HashMap<WorktreeId, VecDeque<String>>,
    pub command_output_scroll_by_worktree: HashMap<WorktreeId, usize>,
    // ...
}

pub fn handle_key(state: &AppState, key: KeyEvent) -> Option<Action>;
pub fn update(state: &mut AppState, action: Action, metro_tx: &..., handle_tx: &...);
fn dispatch_command(state: &mut AppState, spec: CommandSpec, metro_tx: &...);

// Git palette keybindings (in handle_key):
PaletteMode::Git => match key.code {
    Char('f') => fetch, Char('p') => pull, Char('P') => push,
    Char('X') => reset hard, Char('b') => checkout, Char('c') => checkout -b,
    Char('r') => rebase, Esc => cancel, _ => cancel,
}
```

From src/infra/worktrees.rs:
```rust
pub fn parse_worktree_porcelain(text: &str) -> anyhow::Result<Vec<Worktree>>;
pub async fn list_worktrees(repo_root: &Path) -> anyhow::Result<Vec<Worktree>>;
```

From src/domain/worktree.rs:
```rust
pub struct WorktreeId(pub String); // derived from worktree path
pub struct Worktree {
    pub id: WorktreeId,
    pub path: std::path::PathBuf,
    pub branch: String,
    // ...
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add WorktreeRemove action, remove_worktree infra, and confirm modal flow</name>
  <files>src/action.rs, src/infra/worktrees.rs, src/app.rs</files>
  <action>
1. In `src/action.rs`, add two new Action variants:
   - `WorktreeRemove` — user-triggered, dispatched from git palette g>D
   - `WorktreeRemoved(String)` — background event sent after successful removal (carries the path string for cleanup). Also add `WorktreeRemoveFailed(String)` for error case.

2. In `src/infra/worktrees.rs`, add a new async function:
   ```rust
   pub async fn remove_worktree(repo_root: &Path, worktree_path: &Path) -> anyhow::Result<()>
   ```
   This function should:
   - Run `git worktree remove --force <worktree_path>` with `current_dir(repo_root)`
   - If that fails, try `git worktree remove <worktree_path>` without --force as fallback info in the error
   - After successful removal, run `git worktree prune` to clean up stale git metadata
   - If the directory still exists after git worktree remove (shouldn't happen with --force, but safety), log a warning

3. In `src/app.rs` handle_key, add `Char('D')` (capital D) to the `PaletteMode::Git` match arm:
   ```rust
   Char('D') => Some(Action::WorktreeRemove),
   ```

4. In `src/app.rs` update(), handle `Action::WorktreeRemove`:
   - Get the currently selected worktree from `state.worktrees` using table selection index
   - **Guard: if the selected worktree path == state.repo_root, show error "Cannot remove the main worktree" and return** (the main worktree is the primary git checkout, its path matches repo_root)
   - Show a confirm modal: `ModalState::Confirm { prompt: "Remove worktree '<branch>' and delete directory?", pending_command: ... }`
   - BUT: ModalConfirm dispatches via `dispatch_command()` which runs CommandSpec in the worktree dir. We need a DIFFERENT flow. Instead, use a new `CommandSpec::RemoveWorktree { path: String }` variant. This is cleaner than adding a second confirm modal type.
   - Actually, simpler approach: Do NOT use CommandSpec. Instead, store the removal target in a new `pending_worktree_removal: Option<(WorktreeId, PathBuf, String)>` field on AppState (id, path, branch). When WorktreeRemove fires, set the confirm modal and store the target. When ModalConfirm fires AND pending_worktree_removal is Some, handle the removal path instead of dispatch_command.

   Revised approach — add to AppState:
   ```rust
   pub pending_worktree_removal: Option<(crate::domain::worktree::WorktreeId, std::path::PathBuf, String)>,
   ```
   Default to None.

   Handle `Action::WorktreeRemove`:
   - Get selected worktree
   - Guard: if worktree.path == state.repo_root → set error_state "Cannot remove the main worktree", return
   - Guard: if metro is running AND active_worktree_path == Some(worktree.path) → stop metro first by dispatching MetroStop, then set pending removal. Actually, simpler: just refuse if metro is running on that worktree — show confirm that says "Stop metro and remove worktree '<branch>'?" The removal handler will stop metro.
   - Set `state.pending_worktree_removal = Some((wt.id.clone(), wt.path.clone(), wt.branch.clone()))`
   - Set `state.modal = Some(ModalState::Confirm { prompt, pending_command: CommandSpec::GitPull })` — use a dummy CommandSpec as sentinel (same pattern as label input uses YarnLint sentinel)
   - Clear palette_mode

   Modify `Action::ModalConfirm` handler: BEFORE the existing `if let Some(ModalState::Confirm { pending_command, .. }) = state.modal.take()` block, check if `pending_worktree_removal` is set. If so:
   - Take the modal (state.modal.take())
   - Take the pending removal info
   - If metro is running AND active_worktree_path matches the removal path, kill metro (use the existing metro stop logic)
   - Clean up per-worktree state: remove from command_output_by_worktree, command_output_scroll_by_worktree
   - Spawn async task: clone repo_root and worktree_path, call remove_worktree(), then send WorktreeRemoved or WorktreeRemoveFailed back via metro_tx
   - Return early (don't fall through to dispatch_command)

   Handle `Action::WorktreeRemoved(path_str)`:
   - Dispatch RefreshWorktrees by cloning repo_root and spawning the list_worktrees task (copy the pattern from RefreshWorktrees handler)
   - Log success message to command output if desired — or just let the refresh handle it

   Handle `Action::WorktreeRemoveFailed(err)`:
   - Set error_state with the error message, can_retry: false

5. Initialize `pending_worktree_removal: None` in AppState::default().
  </action>
  <verify>
    <automated>cd /Users/cubicme/aljazeera/dashboard && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>
    - g>D keybinding mapped in Git palette
    - Confirm modal shown before removal (with worktree branch name in prompt)
    - Main worktree (repo_root) cannot be removed (error shown)
    - remove_worktree runs `git worktree remove --force` + `git worktree prune`
    - Metro stopped if running on the removed worktree
    - Per-worktree command output and scroll state cleaned up
    - Worktree list refreshes after successful removal
    - Error overlay shown on failure
  </done>
</task>

<task type="auto">
  <name>Task 2: Update help overlay and footer hints for g>D</name>
  <files>src/ui/help_overlay.rs, src/ui/footer.rs</files>
  <action>
1. In `src/ui/help_overlay.rs`, in the "Git submenu section" (around line 84-94), add a new row AFTER the "git rebase" row and BEFORE the "Esc/Cancel" row:
   ```rust
   Row::new(vec!["D",               "Remove worktree (purge)"]),
   ```

2. In `src/ui/footer.rs`, in the `PaletteMode::Git` match arm of `key_hints_for()` (around line 90-99), add before the Esc entry:
   ```rust
   ("D", "remove wt"),
   ```
   Place it after ("r", "rebase") and before ("Esc", "cancel").
  </action>
  <verify>
    <automated>cd /Users/cubicme/aljazeera/dashboard && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>
    - Help overlay shows "D — Remove worktree (purge)" in Git submenu section
    - Footer shows "D remove wt" hint when in Git palette mode
  </done>
</task>

</tasks>

<verification>
1. `cargo check` passes with no errors
2. `cargo clippy` passes (or only pre-existing warnings)
3. Manual test: launch dashboard, press g then D on a non-main worktree — confirm modal appears, Y removes the worktree, list refreshes
4. Manual test: press g then D on the main worktree — error message shown, no removal
</verification>

<success_criteria>
- g>D in git palette shows confirm modal for selected worktree
- Confirming removes the worktree from git, deletes directory, prunes metadata
- Main worktree protected from removal
- Worktree list auto-refreshes after removal
- Per-worktree state (output, scroll) cleaned up
- Help overlay and footer updated with new keybinding
- `cargo check` compiles cleanly
</success_criteria>

<output>
After completion, create `.planning/quick/2-add-command-to-remove-and-purge-a-worktr/2-SUMMARY.md`
</output>
