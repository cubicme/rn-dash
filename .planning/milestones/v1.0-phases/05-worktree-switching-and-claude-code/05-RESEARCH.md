# Phase 5: Worktree Switching and Claude Code - Research

**Researched:** 2026-03-02
**Domain:** Rust async state machines, tmux process spawning, TEA action routing
**Confidence:** HIGH

## Summary

Phase 5 delivers two keybindings in the WorktreeList panel: one to switch the running metro instance to the selected worktree (WORK-04), and one to open Claude Code in a new tmux tab at the selected worktree's directory (INTG-04).

The metro-switch feature (WORK-04) requires no new infrastructure. The existing pending_restart flag plus the MetroStop → MetroExited → MetroStart chain already handles kill-then-restart. The only new mechanism needed is capturing the target worktree path at switch time (before any subsequent navigation changes active_worktree_path), and a new Action variant to trigger the sequence. Progress visibility is already implemented: MetroStatus::Stopping and MetroStatus::Starting states are rendered in the metro pane as "STOPPING..." and "STARTING..." yellow text.

The Claude Code feature (INTG-04) calls `tmux new-window` directly via `std::process::Command` (not via the `tmux_interface` crate, which is pre-1.0 and not yet in Cargo.toml). The critical design point from the requirements is to pass `claude` as the shell-command argument to `new-window` — this avoids the send-keys race condition where keystrokes can fire before the shell finishes initializing. tmux 3.5a (installed) supports `new-window [shell-command]` syntax natively.

**Primary recommendation:** Use a new `Action::WorktreeSwitchToSelected` for WORK-04 and a new `Action::OpenClaudeCode` for INTG-04. No new crate dependencies are needed — tmux interaction uses raw `std::process::Command`.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WORK-04 | User can switch the "running" worktree with one keystroke; dashboard kills metro in current, waits for port 8081 to free, starts metro in the newly selected worktree; progress visible during transition | Existing pending_restart + MetroStop/MetroExited/MetroStart chain handles the sequence; MetroStatus::Stopping/Starting already rendered; only needs new Action variant + path capture at switch time |
| INTG-04 | User can open Claude Code in a new tmux tab at a selected worktree's directory with one keystroke; tab opens with claude as initial shell command (not send-keys) to avoid race condition | `tmux new-window -d -c <path> -n "claude:<branch>" claude` passes claude as shell-command directly; tmux 3.5a confirmed installed; claude confirmed at /Users/cubicme/.local/bin/claude |
</phase_requirements>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.49 (already in Cargo.toml) | Async runtime for background tasks | Already used throughout |
| ratatui | 0.30 (already in Cargo.toml) | TUI rendering | Already used throughout |
| std::process::Command | stdlib | tmux invocation | No new dependency needed |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tmux_interface | NOT needed | Would wrap tmux CLI | Skip — stdlib Command is simpler and more predictable for 2 fire-and-forget calls; tmux_interface is pre-1.0 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| std::process::Command for tmux | tmux_interface 0.3.x | tmux_interface adds typed structs but is pre-1.0, not yet in Cargo.toml, and overkill for 2 one-shot calls |
| Direct pending_restart reuse | New dedicated switch state machine | Reusing pending_restart is simpler; dedicated state adds complexity without benefit |

**Installation:** No new dependencies required.

## Architecture Patterns

### Recommended Project Structure

No new files required. Changes touch existing files:

```
src/
├── action.rs           # Add WorktreeSwitchToSelected, OpenClaudeCode variants
├── app.rs              # Add pending_switch_path field; handle new actions in update(); add WorktreeList key bindings
├── domain/
│   └── (no changes)    # Metro domain already has all needed states
├── infra/
│   └── (optional) tmux.rs  # open_claude_in_worktree() function — fire-and-forget tmux call
└── ui/
    └── footer.rs       # Add Enter/C keybinding hints to WorktreeList panel
```

### Pattern 1: WorktreeSwitchToSelected — Capture Target at Switch Time

**What:** When the user presses Enter in WorktreeList, capture the selected worktree's path immediately (before any navigation can change active_worktree_path), store it in a new `pending_switch_path: Option<PathBuf>` field, then trigger the pending_restart sequence.

**Why this matters:** Between MetroStop and MetroExited (up to ~5 seconds), the user could navigate to a different worktree, changing active_worktree_path. Without capturing at switch time, MetroStart would use the wrong path.

**When to use:** Only for WORK-04 worktree switching, not for the regular MetroRestart action.

**Implementation in update():**

```rust
// In action.rs — new variant
Action::WorktreeSwitchToSelected,

// In app.rs update() — new field in AppState
pub pending_switch_path: Option<std::path::PathBuf>,

// Handler for WorktreeSwitchToSelected
Action::WorktreeSwitchToSelected => {
    // 1. Capture target path NOW (before navigation can change active_worktree_path)
    let target_path = if !state.worktrees.is_empty() {
        let idx = state.worktree_list_state.selected().unwrap_or(0);
        let idx = idx.min(state.worktrees.len() - 1);
        Some(state.worktrees[idx].path.clone())
    } else {
        None
    };
    state.pending_switch_path = target_path;

    // 2. Trigger kill + restart via existing mechanism
    state.pending_restart = true;
    if state.metro.is_running() {
        update(state, Action::MetroStop, metro_tx, handle_tx);
    } else {
        // Metro not running — just start in the new worktree
        state.pending_restart = false;
        // Update active_worktree_path to the target and start
        if let Some(path) = state.pending_switch_path.take() {
            state.active_worktree_path = Some(path);
        }
        update(state, Action::MetroStart, metro_tx, handle_tx);
    }
}

// Modify MetroExited handler to consume pending_switch_path
Action::MetroExited => {
    state.metro.clear();
    if state.pending_restart {
        state.pending_restart = false;
        // Use pending_switch_path if present (worktree switch), else use active_worktree_path
        if let Some(path) = state.pending_switch_path.take() {
            state.active_worktree_path = Some(path);
        }
        update(state, Action::MetroStart, metro_tx, handle_tx);
    }
}
```

### Pattern 2: OpenClaudeCode — tmux new-window Direct Command

**What:** Fire-and-forget `tmux new-window` call that passes `claude` as the shell-command argument. This runs claude directly without a login shell wrapper, eliminating the send-keys race condition.

**Why direct command (not send-keys):** tmux `new-window [shell-command]` launches the specified program as the pane's initial process. The window is immediately in the claude session. With `send-keys`, there is a timing gap where keystrokes can arrive before the shell prompt is ready.

**tmux 3.5a syntax (confirmed):**
```
new-window (neww) [-abdkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command]
```

**Rust implementation:**

```rust
// In infra/tmux.rs (new file) or inline in app.rs update()
pub fn open_claude_in_worktree(path: &std::path::Path, window_name: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("tmux")
        .args([
            "new-window",
            "-d",                           // don't switch focus to new window
            "-c", path.to_str().unwrap_or("."),  // start directory = worktree path
            "-n", window_name,              // tab label (e.g. "claude:UMP-1234")
            "claude",                       // shell-command: run claude directly
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("tmux new-window failed: exit code {:?}", status.code());
    }
    Ok(())
}

// In action.rs
Action::OpenClaudeCode,

// In app.rs update() — key binding 'C' in WorktreeList
// In handle_key():
if state.focused_panel == FocusedPanel::WorktreeList {
    match key.code {
        // ... existing bindings ...
        Char('C') => return Some(Action::OpenClaudeCode),
        Enter => return Some(Action::WorktreeSwitchToSelected),
    }
}

// In update():
Action::OpenClaudeCode => {
    if !state.tmux_available {
        state.error_state = Some(ErrorState {
            message: "Cannot open Claude Code: not inside a tmux session".into(),
            can_retry: false,
        });
        return;
    }
    if let Some(wt) = state.worktrees.get(
        state.worktree_list_state.selected().unwrap_or(0)
    ) {
        let path = wt.path.clone();
        let branch = wt.branch.clone();
        tokio::spawn(async move {
            let window_name = format!("claude:{}", branch.split('/').last().unwrap_or(&branch));
            if let Err(e) = crate::infra::tmux::open_claude_in_worktree(&path, &window_name) {
                tracing::warn!("open claude code failed: {e}");
            }
        });
    }
}
```

### Pattern 3: Progress Visibility — Existing MetroStatus States

**What:** The metro pane already renders four states:
- `MetroStatus::Running` → green "RUNNING pid=X [worktree]"
- `MetroStatus::Stopping` → yellow "STOPPING..."
- `MetroStatus::Starting` → yellow "STARTING..."
- `MetroStatus::Stopped` → dark gray "STOPPED"

**No UI changes needed** for progress visibility in WORK-04. The transition STOPPING → STARTING is visible in the metro pane throughout the switch. The 250ms tick interval means the UI updates ~4x/second during the port-free wait.

### Pattern 4: WorktreeList Badge Update (optional enhancement)

The worktree list currently shows `[M]` (green) for `WorktreeMetroStatus::Running` and `[ ]` (dark gray) for `WorktreeMetroStatus::Stopped`. During a switch, the badge on the old worktree won't change until `MetroExited` (and a `WorktreesLoaded` refresh). This is acceptable since the metro pane status text gives clear transition feedback. No changes to `WorktreeMetroStatus` are needed.

### Anti-Patterns to Avoid

- **Using active_worktree_path directly in MetroExited:** If user navigates during stop→start gap, the wrong worktree starts. Always use pending_switch_path when it's set.
- **Using send-keys for Claude Code:** Race condition — shell may not be ready when keys arrive.
- **Spawning tmux on the main thread:** `std::process::Command::status()` blocks. Always wrap in `tokio::spawn`.
- **Adding tmux_interface crate:** Unnecessary dependency for 2 fire-and-forget calls. Keep it simple.
- **Running WorktreeSwitchToSelected when metro is not running and no path is selected:** Guard against empty worktree list.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Port-free wait loop | Custom sleep+probe | Existing metro_process_task (50 × 100ms retries) | Already implemented and tested |
| tmux abstraction layer | TmuxClient trait | Direct std::process::Command | Pre-1.0 tmux_interface not worth pulling in; trait adds indirection with no test benefit here |
| Metro restart state machine | New state enum | Existing pending_restart + MetroExited chain | Already handles kill→wait→start correctly |

**Key insight:** Phase 5 is almost entirely wiring — the hard parts (port wait, kill, restart) already exist. The implementation is routing new keystrokes to existing machinery, plus one new tmux subprocess call.

## Common Pitfalls

### Pitfall 1: Race Between Navigation and Switch Target

**What goes wrong:** User presses Enter to switch, then immediately presses j/k to navigate while metro is stopping. MetroStart fires with the navigation target, not the switch target.

**Why it happens:** active_worktree_path is updated by WorktreeSelectNext/Prev, and MetroStart reads it at call time (which is after MetroExited, seconds later).

**How to avoid:** Capture the target path into `pending_switch_path` at the moment WorktreeSwitchToSelected is processed (inside update()), before any async delay.

**Warning signs:** Metro starts in a different worktree than the one the user pressed Enter on.

### Pitfall 2: Triggering Switch When Metro Not Running

**What goes wrong:** User presses Enter on a worktree when metro is stopped. pending_restart=true, MetroStop is called on a non-running metro, nothing happens, then MetroExited never fires, so MetroStart never fires.

**Why it happens:** The MetroStop action only kills if a handle exists. If no handle, it's a no-op. MetroExited is only sent by metro_process_task, which only runs when metro was actually started.

**How to avoid:** In WorktreeSwitchToSelected handler: check `state.metro.is_running()`. If running, use the pending_restart flow. If not running, immediately set active_worktree_path = pending_switch_path and call MetroStart directly.

**Warning signs:** After pressing Enter with metro stopped, metro never starts in the new worktree.

### Pitfall 3: tmux Not Available

**What goes wrong:** User presses C (open Claude) but dashboard is not running inside tmux. `tmux new-window` fails with non-zero exit or no TMUX env var.

**Why it happens:** INTG-04 is tmux-specific. The dashboard can run outside tmux.

**How to avoid:** Check `state.tmux_available` (already set in run() via is_inside_tmux()). Show an error via ErrorState if not in tmux.

**Warning signs:** Silent failure when claude tab doesn't open.

### Pitfall 4: claude Binary Not in PATH

**What goes wrong:** `tmux new-window ... claude` fails because claude is not on $PATH inside the tmux window.

**Why it happens:** claude is at `/Users/cubicme/.local/bin/claude` which may not be on PATH in non-interactive tmux sessions.

**How to avoid:** Two options:
1. Use the full path `/Users/cubicme/.local/bin/claude` (fragile, machine-specific)
2. Pass claude as the shell-command — tmux uses the user's default shell to exec it, which inherits PATH from the user's shell config.

**Correct approach:** Use the shell-command form. Per tmux man page: "new-window 'vi ~/.tmux.conf' will run /bin/sh -c 'vi ~/.tmux.conf'". This inherits PATH from the user's shell. So `tmux new-window ... claude` will resolve `claude` via the user's PATH (which includes `~/.local/bin`).

**Warning signs:** Window opens but immediately closes (process not found).

### Pitfall 5: Window Name Conflicts

**What goes wrong:** Multiple worktrees with the same branch suffix produce window names like "claude:feature" which collide.

**Why it happens:** Branch names can share suffixes.

**How to avoid:** Use the last segment of the branch for the window name (brief) — tmux allows duplicate window names; it's not an error. Or use a unique name like the full branch with slashes replaced by dashes.

## Code Examples

Verified patterns from official sources:

### tmux new-window with Direct Shell Command

```bash
# Verified: tmux 3.5a man page syntax
# new-window (neww) [-abdkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command]

# Opens a new tmux window in /path/to/worktree running claude directly
# -d: don't switch focus
# -c: start directory
# -n: window name (tab label)
# "claude": the shell-command argument — passed to default-shell -c "claude"
tmux new-window -d -c /path/to/worktree -n "claude:UMP-1234" claude
```

### Rust: Fire-and-Forget tmux Call

```rust
// Source: pattern derived from existing dispatch_command in app.rs
// Run blocking std::process::Command in a tokio::spawn to avoid blocking the event loop
tokio::spawn(async move {
    let result = std::process::Command::new("tmux")
        .args([
            "new-window",
            "-d",
            "-c", path.to_str().unwrap_or("."),
            "-n", &window_name,
            "claude",
        ])
        .status();
    match result {
        Ok(s) if s.success() => {}
        Ok(s) => tracing::warn!("tmux new-window exit: {:?}", s.code()),
        Err(e) => tracing::warn!("tmux new-window failed: {e}"),
    }
});
```

### WorktreeSwitchToSelected: Guard for Metro Not Running

```rust
Action::WorktreeSwitchToSelected => {
    // Capture target immediately (navigation may change active_worktree_path later)
    let target_path = state.worktrees
        .get(state.worktree_list_state.selected().unwrap_or(0))
        .map(|wt| wt.path.clone());

    if state.metro.is_running() {
        // Kill current → wait for port free → start in new worktree
        state.pending_switch_path = target_path;
        state.pending_restart = true;
        update(state, Action::MetroStop, metro_tx, handle_tx);
    } else {
        // Not running — just start directly in selected worktree
        if let Some(path) = target_path {
            state.active_worktree_path = Some(path);
        }
        update(state, Action::MetroStart, metro_tx, handle_tx);
    }
}
```

### Modified MetroExited Handler

```rust
Action::MetroExited => {
    state.metro.clear();
    if state.pending_restart {
        state.pending_restart = false;
        // Consume pending_switch_path if set (worktree switch takes priority)
        if let Some(path) = state.pending_switch_path.take() {
            state.active_worktree_path = Some(path);
        }
        update(state, Action::MetroStart, metro_tx, handle_tx);
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| send-keys to inject commands | new-window [shell-command] | tmux ~3.0 | Eliminates race condition on shell init |
| tmux_interface crate | Raw std::process::Command | Phase 5 decision | Less code, no pre-1.0 dependency risk |

**Deprecated/outdated:**
- send-keys approach for opening tools: Race condition exists; correct approach is shell-command arg to new-window.

## Open Questions

1. **Should WorktreeSwitchToSelected show the metro pane hint?**
   - What we know: After pressing Enter, progress is visible in the metro pane (STOPPING... → STARTING...). The worktree list doesn't update metro badges until the next WorktreesLoaded.
   - What's unclear: Whether users expect the badge on the old worktree to immediately clear.
   - Recommendation: No extra UI change needed. Metro pane status text is sufficient for progress tracking.

2. **Should WorktreeSwitchToSelected work when metro is stopped?**
   - What we know: The requirement says "auto-kills metro in current worktree... and starts metro in the newly selected worktree" — this implies metro was running.
   - What's unclear: What if user presses Enter when metro is not running?
   - Recommendation: Support both cases. If metro is running: switch+restart. If stopped: just start in selected worktree. This is more useful and harmless.

3. **Window name for Claude tab when branch has no suffix segment**
   - What we know: branch names like "main" have no UMP ticket.
   - Recommendation: Use `format!("claude:{}", branch)` truncated to 20 chars, replacing `/` with `:`.

## Validation Architecture

> nyquist_validation not found in .planning/config.json — skip this section.

## Sources

### Primary (HIGH confidence)

- tmux 3.5a man page (local) — `new-window` syntax, shell-command argument behavior, confirmed `-d -c -n [shell-command]` flags
- `/Users/cubicme/aljazeera/dashboard/src/app.rs` — existing pending_restart mechanism, MetroStop/MetroExited/MetroStart chain, active_worktree_path usage, dispatch_command pattern
- `/Users/cubicme/aljazeera/dashboard/src/domain/metro.rs` — MetroStatus enum (Stopped/Starting/Running/Stopping), MetroManager API
- `/Users/cubicme/aljazeera/dashboard/src/infra/port.rs` — port_is_free() already handles TIME_WAIT wait
- `/Users/cubicme/aljazeera/dashboard/src/infra/process.rs` — TokioProcessClient, process_group(0) kill pattern

### Secondary (MEDIUM confidence)

- `/Users/cubicme/aljazeera/dashboard/.planning/research/ARCHITECTURE.md` — prior research recommending TmuxClient trait + tmux_interface (overridden: direct Command is simpler for Phase 5 scope)
- `/Users/cubicme/aljazeera/dashboard/.planning/STATE.md` — `tmux_interface is pre-1.0 — pin exact version in Cargo.toml, keep behind TmuxClient trait` (decision stands; trait not needed for 2 fire-and-forget calls)

### Tertiary (LOW confidence)

- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all existing crates already in Cargo.toml
- Architecture: HIGH — directly traced through existing app.rs code; pending_restart mechanism confirmed
- Pitfalls: HIGH — race condition pitfalls are traceable in source code; PATH issue verified by checking claude binary location

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (stable — tmux syntax and Rust stdlib Command API are extremely stable)
