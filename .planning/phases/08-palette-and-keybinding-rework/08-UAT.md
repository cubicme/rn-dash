---
status: diagnosed
phase: v1.1-combined
source: 07-01-SUMMARY.md, 08-01-SUMMARY.md, 08-03-SUMMARY.md, 09-01-SUMMARY.md, 09-02-SUMMARY.md
started: 2026-04-05T17:00:00Z
updated: 2026-04-06T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. App launches and loads worktrees
expected: App starts without errors, worktree list populates with your branches and JIRA titles. No "loading worktrees" hang.
result: pass

### 2. No label UI anywhere
expected: No "L" keybinding in footer or help overlay. No label column in worktree table. Pressing L does nothing.
result: pass

### 3. Yarn palette (y)
expected: Pressing 'y' opens a palette showing: install, pod-install, unit-tests, check-types, jest, lint, clean-android, clean-cocoapods, rm-node_modules.
result: pass

### 4. Worktree palette (w)
expected: Pressing 'w' opens a palette showing: W (create worktree), D (remove worktree), B (new branch worktree).
result: issue
reported: "worktree palette sub-keys are uppercase W/D/B — should be lowercase w/d/b like other palettes"
severity: minor

### 5. Git palette no longer has worktree commands
expected: Pressing 'g' opens git palette WITHOUT W or D options. Only git operations remain.
result: pass

### 6. Create worktree with new branch (w>B)
expected: Pressing w then B shows a branch picker with remote branches. Select base, enter name, creates worktree.
result: pass

### 7. Metro R/J only when running
expected: Metro not running: R refreshes worktrees, J does nothing. Metro running: R reloads, J sends debugger.
result: issue
reported: "the old m palette is still there. m should be removed completely"
severity: major

### 8. ESC stops metro
expected: Metro running: ESC stops metro. Metro not running: ESC does nothing or closes modal.
result: pass

### 9. No metro restart key
expected: No dedicated metro restart in help overlay. RET handles switching.
result: issue
reported: "help overlay still shows m metro. Enter on worktree with app-managed metro shows external conflict prompt instead of just switching"
severity: major

### 10. Dynamic footer hints
expected: Footer hints change by context. No stale legend.
result: pass

### 11. JIRA titles with UMP prefix
expected: UMP-XXXX branches show JIRA titles from Atlassian.
result: pass

### 12. App title shows "RN Dash"
expected: Title bar shows "RN Dash".
result: skipped
reason: No title bar ever existed — not a regression

### 13. LICENSE file exists
expected: MIT license in repo root.
result: pass

### 14. README exists with content
expected: README with description, build, usage, config reference.
result: pass

### 15. CI workflow file exists
expected: .github/workflows/ci.yml with macOS+Linux matrix.
result: pass

### 16. Release workflow file exists
expected: .github/workflows/release.yml with v* tag trigger and binary builds.
result: pass

## Summary

total: 16
passed: 13
issues: 3
pending: 0
skipped: 1
blocked: 0

## Gaps

- truth: "Worktree palette sub-keys should be lowercase like other palettes"
  status: failed
  reason: "User reported: worktree palette sub-keys are uppercase W/D/B — should be lowercase w/d/b"
  severity: minor
  test: 4
  root_cause: "PaletteMode::Worktree key matching uses uppercase chars in handle_key and footer/help render"
  artifacts:
    - path: "src/app.rs"
      issue: "Worktree palette key matching uses uppercase W/D/B"
    - path: "src/ui/footer.rs"
      issue: "Worktree palette hints show uppercase"
    - path: "src/ui/help_overlay.rs"
      issue: "Worktree palette help shows uppercase"
  missing:
    - "Change W/D/B to w/d/b in handle_key worktree palette arm"
    - "Update footer hints to lowercase"
    - "Update help overlay to lowercase"

- truth: "Metro m palette should not exist — metro controls are now R/J/Esc"
  status: failed
  reason: "User reported: the old m palette is still there. m should be removed completely"
  severity: major
  test: 7
  root_cause: "PaletteMode::Metro and EnterMetroPalette still exist in codebase — m key still dispatches to metro palette"
  artifacts:
    - path: "src/app.rs"
      issue: "m key dispatches EnterMetroPalette, PaletteMode::Metro arm still handled"
    - path: "src/action.rs"
      issue: "EnterMetroPalette variant still exists"
    - path: "src/ui/footer.rs"
      issue: "m>metro hint still rendered"
    - path: "src/ui/help_overlay.rs"
      issue: "m metro entry still shown"
  missing:
    - "Remove PaletteMode::Metro variant"
    - "Remove EnterMetroPalette action variant"
    - "Remove m key handling from handle_key"
    - "Remove metro palette arm from handle_key"
    - "Remove m hint from footer"
    - "Remove m entry from help overlay"

- truth: "Enter on worktree with app-managed metro should switch without conflict prompt"
  status: failed
  reason: "User reported: Enter shows external metro conflict prompt even when metro was started by the app"
  severity: major
  test: 9
  root_cause: "WorktreeSwitch flow checks port 8081 for external metro but does not distinguish app-managed metro from external metro"
  artifacts:
    - path: "src/app.rs"
      issue: "WorktreeSwitch conflict detection treats app-managed metro as external"
  missing:
    - "Skip external metro detection when MetroManager already has a running handle"
    - "Only trigger conflict prompt when metro is running externally (not managed by the app)"
