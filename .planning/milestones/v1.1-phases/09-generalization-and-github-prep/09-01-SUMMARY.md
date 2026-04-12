---
phase: 09-generalization-and-github-prep
plan: "01"
subsystem: infra/config
tags: [config, generalization, rename, jira]
dependency_graph:
  requires: []
  provides: [DashConfig.repo_root, DashConfig.jira_project_prefix, DashConfig.app_title, config.example.json]
  affects: [src/app.rs, src/ui/panels.rs, src/infra/jira.rs, src/infra/worktrees.rs]
tech_stack:
  added: []
  patterns: [serde-default-fn, config-helper-method]
key_files:
  created:
    - config.example.json
  modified:
    - Cargo.toml
    - src/infra/config.rs
    - src/infra/jira.rs
    - src/infra/worktrees.rs
    - src/app.rs
    - src/ui/panels.rs
    - src/tui.rs
    - src/main.rs
    - src/infra/android_prefs.rs
    - src/infra/jira_cache.rs
    - src/infra/sim_history.rs
decisions:
  - "jira_key on Worktree set to None in list_worktrees(); populated in WorktreesLoaded handler with configured prefix — avoids passing prefix into async infra layer"
  - "config.example.json uses _comment_ pattern for inline documentation since JSON has no native comments"
metrics:
  duration_minutes: 5
  completed_date: "2026-04-05"
  tasks_completed: 2
  files_changed: 12
---

# Phase 09 Plan 01: Generalization and Config Extraction Summary

**One-liner:** Package renamed to rn-dash, all hardcoded AJ/UMP/path values extracted to DashConfig fields (repo_root, jira_project_prefix, app_title), config.example.json documents every setting.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Generalize config struct, config_dir, JIRA extraction, all hardcoded refs | d953feb | Cargo.toml, config.rs, jira.rs, worktrees.rs, app.rs, panels.rs, tui.rs, main.rs, +3 comment fixes |
| 2 | Create config.example.json with documented settings | 39012c9 | config.example.json |

## What Was Built

- **Package renamed** from `ump-dash` to `rn-dash` in Cargo.toml
- **`config_dir()`** now returns `~/.config/rn-dash/` (was `~/.config/ump-dash/`)
- **`DashConfig`** gains three new fields with serde defaults:
  - `repo_root: Option<String>` — path to RN monorepo root (supports `~/`)
  - `jira_project_prefix: String` — default `"UMP"` for backward compat
  - `app_title: String` — default `"RN Dash"`
- **`DashConfig::repo_root_path()`** helper resolves `~/` expansion to `PathBuf`
- **`extract_jira_key(branch, project_prefix)`** — prefix is now a parameter; all call sites updated including one discovered in `panels.rs` and one in `worktrees.rs`
- **`AppState`** gains `jira_project_prefix: String` field wired from config in `run()`
- **`repo_root`** in `AppState::default()` is now `PathBuf::new()` (set from config in `run()`)
- **`WorktreesLoaded` handler** populates `wt.jira_key` using configured prefix (was previously set during `list_worktrees` with hardcoded "UMP")
- **Title bar** reads `config.app_title` with `"RN Dash"` fallback
- **Log paths** updated to `rn-dash` in `tui.rs`
- **Tracing messages** updated in `main.rs`
- **`config.example.json`** created with all 8 DashConfig fields, `_comment_` documentation pattern, placeholder values only

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing extract_jira_key call sites in panels.rs and worktrees.rs**
- **Found during:** Task 1 — cargo build revealed two additional call sites not mentioned in the plan
- **Fix:** Updated `panels.rs` line 70 to pass `&state.jira_project_prefix`; set `jira_key: None` in `list_worktrees()` and moved jira_key population to `WorktreesLoaded` handler in `app.rs` where prefix is available
- **Files modified:** src/ui/panels.rs, src/infra/worktrees.rs, src/app.rs
- **Commits:** d953feb

## Known Stubs

None — all config fields are wired to real behavior.

## Threat Flags

None — `config.example.json` contains placeholder values only, consistent with T-09-02 mitigation.

## Self-Check: PASSED

- config.example.json exists: FOUND
- src/infra/config.rs has repo_root and jira_project_prefix: FOUND
- src/infra/jira.rs has project_prefix parameter: FOUND
- grep -r "ump-dash" src/: 0 matches
- grep -r "aljazeera" src/: 0 matches
- cargo build: success (0 errors)
- cargo test: 26 passed, 0 failed
- Commit d953feb exists: FOUND
- Commit 39012c9 exists: FOUND
