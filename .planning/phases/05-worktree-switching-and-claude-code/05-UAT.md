---
status: diagnosed
phase: 05-worktree-switching-and-claude-code
source: 05-01-SUMMARY.md
started: 2026-03-03T00:00:00Z
updated: 2026-03-03T00:10:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Switch worktree with Enter (metro running)
expected: With metro running in one worktree, navigate to a different worktree in the list and press Enter. Metro should stop in the current worktree (status shows STOPPING...), wait for port 8081 to free, then start in the newly selected worktree (status shows STARTING...). The transition should be visible in the metro pane.
result: issue
reported: "it runs and switches. but 1. if there's an error it just stops; which happened. 2. pid [running] is useless for that big output window. can't we just pipe the output from metro?"
severity: major

### 2. Switch worktree with Enter (metro stopped)
expected: With metro not running, navigate to a worktree in the list and press Enter. Metro should start directly in the selected worktree without any stop phase. Status shows STARTING... then transitions to running.
result: pass

### 3. Open Claude Code with C key (inside tmux)
expected: While running inside a tmux session, navigate to a worktree and press C (shift-c). A new tmux window should open with Claude Code running in it, named "claude:<branch>". The dashboard should remain in the current window (no focus switch away).
result: pass

### 4. Open Claude Code with C key (outside tmux)
expected: While running outside tmux, press C. An error message should appear instead of crashing. (Skip if you always run inside tmux.)
result: pass

### 5. Footer hints show new keybindings
expected: When the worktree list panel is focused, the footer bar at the bottom should show "Enter switch" and "C claude" among the available keybinding hints.
result: pass

### 6. Help overlay shows new keybindings
expected: Open the help overlay (? key). The Worktree List section should include "Enter — Switch metro to worktree" and "C (shift-c) — Open Claude Code (tmux)".
result: pass

## Summary

total: 6
passed: 5
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Metro switch handles errors gracefully and metro pane streams actual output"
  status: failed
  reason: "User reported: it runs and switches. but 1. if there's an error it just stops; which happened. 2. pid [running] is useless for that big output window. can't we just pipe the output from metro?"
  severity: major
  test: 1
  root_cause: "Two separate issues: (A) spawn_metro_task error at app.rs:1118-1121 only logs to tracing::error and sends MetroExited — never surfaces error to user via error_state. (B) render_metro_pane at panels.rs:99-140 only shows status text ('RUNNING pid=X [worktree]'); actual metro stdout/stderr is captured and stored in metro_logs VecDeque but only rendered in a separate log panel toggled by 'l' key. Metro logs also only stream when log_filter_active is true (requires metro restart)."
  artifacts:
    - path: "src/app.rs"
      issue: "Lines 1118-1121: spawn error silently sends MetroExited with no user feedback. Lines 456-461: log streaming coupled to filter toggle requiring metro restart."
    - path: "src/ui/panels.rs"
      issue: "Lines 99-140: render_metro_pane shows only status text, not metro_logs content. Lines 142-183: render_log_panel shows logs but is separate and hidden by default."
  missing:
    - "Surface spawn errors to error_state for user-visible feedback"
    - "Stream metro stdout/stderr into the metro pane directly (always, not just when filter toggled)"
    - "Render metro_logs in the metro pane instead of just status text"
