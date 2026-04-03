---
phase: quick
plan: 260403-dmz
type: execute
wave: 1
depends_on: []
files_modified:
  - src/action.rs
  - src/infra/worktrees.rs
  - src/app.rs
  - src/ui/footer.rs
  - src/ui/help_overlay.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "User can press g>W to create a new worktree from git palette"
    - "User is prompted for a branch name via TextInput modal"
    - "Worktree is created as a sibling directory of repo_root using git worktree add"
    - "Worktree list refreshes after creation"
    - "Errors are shown in error overlay"
  artifacts:
    - path: "src/infra/worktrees.rs"
      provides: "add_worktree async function"
      contains: "pub async fn add_worktree"
    - path: "src/action.rs"
      provides: "WorktreeAdd, WorktreeAdded, WorktreeAddFailed action variants"
    - path: "src/app.rs"
      provides: "Keybinding g>W, TextInput modal, async spawn, result handling"
  key_links:
    - from: "src/app.rs"
      to: "src/infra/worktrees.rs"
      via: "tokio::spawn calling add_worktree"
      pattern: "add_worktree"
    - from: "src/app.rs"
      to: "Action::WorktreesLoaded"
      via: "list_worktrees refresh after WorktreeAdded"
---

<objective>
Add a "create worktree" command (g>W) to the git palette, mirroring the existing "remove worktree" (g>D) pattern. User types a branch name in a TextInput modal, the dashboard runs `git worktree add` to create a new worktree as a sibling directory, then refreshes the worktree list.

Purpose: Allow creating new worktrees without leaving the dashboard.
Output: Working g>W command with TextInput prompt, async creation, and list refresh.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/action.rs
@src/infra/worktrees.rs
@src/app.rs
@src/ui/footer.rs
@src/ui/help_overlay.rs
@src/domain/command.rs (ModalState::TextInput pattern)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add add_worktree infra function and Action variants</name>
  <files>src/infra/worktrees.rs, src/action.rs</files>
  <action>
1. In src/infra/worktrees.rs, add `pub async fn add_worktree(repo_root: &Path, branch_name: &str) -> anyhow::Result<PathBuf>`:
   - Compute worktree path as `repo_root.parent().unwrap().join(&branch_name)` (sibling directory, same pattern git uses by default)
   - If the path already exists, bail with a clear error message
   - Run `git worktree add <path> -b <branch_name>` with current_dir(repo_root)
   - If the command fails, check stderr — if it says the branch already exists, retry with `git worktree add <path> <branch_name>` (without -b, to checkout existing branch)
   - Return the created worktree path on success
   - Follow the same error handling pattern as remove_worktree (bail with stderr on failure)

2. In src/action.rs, add three new variants after the WorktreeRemove group (in a "Quick: Worktree creation" comment block):
   - `WorktreeAdd` — user-triggered via g>W, shows TextInput modal
   - `WorktreeAdded(String)` — background: creation succeeded (carries path string)
   - `WorktreeAddFailed(String)` — background: creation failed (carries error message)
  </action>
  <verify>cargo check 2>&1 | tail -5 (should show warnings only for unused variants, no errors)</verify>
  <done>add_worktree function exists and compiles; three new Action variants defined</done>
</task>

<task type="auto">
  <name>Task 2: Wire keybinding, modal flow, async spawn, and result handling in app.rs</name>
  <files>src/app.rs, src/ui/footer.rs, src/ui/help_overlay.rs</files>
  <action>
1. In handle_key (PaletteMode::Git match arm), add `Char('W') => Some(Action::WorktreeAdd)` — capital W to parallel capital D for remove.

2. In update() function, add handler for Action::WorktreeAdd:
   - Open a TextInput modal with prompt "New worktree branch name:", empty buffer, and a sentinel pending_template (use Box::new(CommandSpec::GitPull) as sentinel, same pattern as remove's confirm modal).
   - Set a new field `pending_worktree_add: bool` on AppState (default false) to true — this distinguishes the TextInput submit from other TextInput uses (same pattern as pending_label_branch, pending_claude_open, pending_android_mode).
   - Clear palette_mode.

3. In ModalInputSubmit handler (the TextInput match arm), add a branch for `state.pending_worktree_add`:
   - Insert it AFTER the pending_android_mode check and BEFORE the else branch that dispatches pending_template as a command.
   - Set `state.pending_worktree_add = false`.
   - Trim the buffer. If empty, return early (don't create).
   - Clone repo_root and metro_tx, then tokio::spawn an async block that:
     - Calls `crate::infra::worktrees::add_worktree(&repo_root, &branch_name).await`
     - On Ok(path): sends `Action::WorktreeAdded(path.to_string_lossy().to_string())`
     - On Err(e): sends `Action::WorktreeAddFailed(e.to_string())`

4. In ModalCancel handler, add `state.pending_worktree_add = false;` alongside the other pending state resets.

5. In update(), add handler for Action::WorktreeAdded(path_str):
   - Log with tracing::info!
   - Refresh worktree list (clone repo_root, spawn async list_worktrees, send WorktreesLoaded) — exact same pattern as WorktreeRemoved handler.

6. In update(), add handler for Action::WorktreeAddFailed(err):
   - Set error_state with message "Failed to create worktree: {err}", can_retry: false — same pattern as WorktreeRemoveFailed.

7. In AppState struct, add `pub pending_worktree_add: bool` field (default false in Default impl).

8. In src/ui/footer.rs, add ("W", "add wt") to the PaletteMode::Git hints vec, before ("D", "remove wt").

9. In src/ui/help_overlay.rs, add a row `Row::new(vec!["W", "Add new worktree"])` in the Git submenu section, before the D/Remove row.
  </action>
  <verify>cargo check 2>&1 | tail -5 (no errors); cargo build 2>&1 | tail -3 (builds successfully)</verify>
  <done>g>W opens TextInput for branch name; submitting creates worktree via git worktree add; list refreshes on success; errors shown in overlay; footer and help updated</done>
</task>

</tasks>

<verification>
cargo build 2>&1 | tail -5
</verification>

<success_criteria>
- g>W keybinding opens TextInput modal prompting for branch name
- Submitting a branch name spawns async `git worktree add` in sibling directory
- On success, worktree list refreshes automatically
- On failure, error overlay shows the error message
- Footer shows "W add wt" in git palette hints
- Help overlay documents the new keybinding
- Code compiles with no errors
</success_criteria>

<output>
After completion, create `.planning/quick/260403-dmz-add-command-to-create-worktree-like-the-/260403-dmz-SUMMARY.md`
</output>
