---
status: diagnosed
phase: 04-config-and-jira-integration
source: 04-01-SUMMARY.md, 04-02-SUMMARY.md
started: 2026-03-02T13:30:00Z
updated: 2026-03-02T13:42:00Z
---

## Current Test

[testing complete]

## Tests

### 1. App starts without JIRA config
expected: Launch the dashboard without ~/.config/ump-dash/config.json present. The app should start normally — no crash, no error message. Worktree list displays branch names as usual.
result: pass

### 2. Worktree display names show branch names
expected: In the worktree list panel, each entry shows its branch name as the display name (since no JIRA config is set, no JIRA titles should appear). Branch names render clearly without any missing-data artifacts.
result: pass

### 3. JIRA config file loads on startup
expected: Create ~/.config/ump-dash/config.json with your JIRA credentials (jira_url, jira_email, jira_token, auth_mode). Restart the app. No crash — the app silently loads the config. (If you don't have JIRA credentials, type "skip".)
result: pass

### 4. JIRA titles fetched for worktree branches
expected: With JIRA config set, worktrees whose branch names contain JIRA keys (e.g., feature/UMP-1234-login) should show the JIRA ticket title as the display name, with the branch name shown dimmed in parentheses. (Skip if no JIRA credentials.)
result: issue
reported: "the branch name should still be there first. with long titles the branch name is not there and for the others the color is barely visible. for state instead of [stale] and other stats, you can maybe show an icon or something"
severity: major

### 5. JIRA title cache persists across restarts
expected: After JIRA titles have been fetched once, quit and relaunch the app. The JIRA titles should appear immediately (loaded from cache at ~/.config/ump-dash/jira_cache.json) without waiting for a network fetch. (Skip if no JIRA credentials.)
result: pass

### 6. Tmux detection
expected: When running inside a tmux session, the app detects tmux is available (this gates Phase 5 features). When running outside tmux, it detects tmux is not available. No crash or error either way.
result: skipped
reason: No observable UI behavior yet — internal flag only, gates Phase 5

## Summary

total: 6
passed: 4
issues: 1
pending: 0
skipped: 1

## Gaps

- truth: "Worktree entries show branch name first, JIRA title second, with visible styling and icon-based status"
  status: failed
  reason: "User reported: the branch name should still be there first. with long titles the branch name is not there and for the others the color is barely visible. for state instead of [stale] and other stats, you can maybe show an icon or something"
  severity: major
  test: 4
  root_cause: "panels.rs renders display_name() (which returns jira_title) as primary text, branch name as DarkGray+DIM parenthetical that gets truncated on long titles. display_name() priority is label>jira_title>branch but user wants branch first. Status uses text '[stale]' instead of icons."
  artifacts:
    - path: "src/ui/panels.rs"
      issue: "Lines 52-55: display_name() used as primary span; lines 59-62: branch in DarkGray+DIM parenthetical truncated on long titles; lines 66-71: [stale] rendered as text"
    - path: "src/domain/worktree.rs"
      issue: "Lines 42-55: display_name() returns single string with jira_title priority over branch"
  missing:
    - "Render branch name first, JIRA title second (truncated if needed)"
    - "Use Color::Gray instead of DarkGray+DIM for better visibility"
    - "Replace '[stale]' text with Unicode icon (e.g., ⚠ or 󰁥)"
