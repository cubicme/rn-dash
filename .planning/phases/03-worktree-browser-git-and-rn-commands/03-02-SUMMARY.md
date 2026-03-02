---
phase: 03-worktree-browser-git-and-rn-commands
plan: 02
subsystem: infra
tags: [rust, ratatui, infra, git, worktrees, command-runner, labels, devices, serde_json, tokio]

# Dependency graph
requires:
  - phase: 03-01
    provides: CommandSpec.to_argv(), DeviceInfo, Worktree struct, Action::CommandOutputLine/CommandExited
  - phase: 02-metro-process-control
    provides: stream_metro_logs pattern (copied in command_runner.rs)
provides:
  - parse_worktree_porcelain: pure fn parsing git worktree list --porcelain into Vec<Worktree>
  - check_stale: node_modules mtime vs package.json/yarn.lock mtime comparison
  - list_worktrees: async fn running git and piping to parser
  - spawn_command_task: spawns any CommandSpec, streams output via mpsc, sends CommandExited
  - build_argv: injects origin/{branch} for GitResetHard
  - load_labels / save_labels: HashMap<String,String> round-trip through ~/.config/ump-dash/labels.json
  - parse_adb_devices: pure fn, tab-split, state=="device" filter
  - parse_xcrun_simctl: pure fn, JSON parse, isAvailable filter, "{name} ({state})" format
  - list_android_devices / list_ios_devices: async runners
affects:
  - 03-03 (app logic: will call list_worktrees, spawn_command_task, load_labels/save_labels)
  - 03-04 (UI: WorktreeList widget reads list, CommandOutputPanel reads output lines)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Porcelain parser: split on \\n\\n stanzas, strip_prefix per line, bare/detached special-cases"
    - "stream_command_output: tokio::select! loop matching stream_metro_logs pattern from app.rs"
    - "build_argv injects origin/{branch} for GitResetHard — only special-case override"
    - "load_labels returns empty HashMap on ENOENT — callers never need to handle missing file"

key-files:
  created:
    - src/infra/worktrees.rs
    - src/infra/labels.rs
    - src/infra/command_runner.rs
    - src/infra/devices.rs
  modified:
    - src/infra/mod.rs

key-decisions:
  - "WorktreeId derived from worktree path string — stable identifier across branch renames"
  - "check_stale returns true when node_modules absent, false when no lock files exist"
  - "stream_command_output tracks stdout_done/stderr_done flags — select! guards prevent polling closed streams"
  - "parse_adb_devices uses id=serial for both id and name — adb list output has no model names"
  - "parse_xcrun_simctl formats display name as '{name} ({state})' to distinguish Booted vs Shutdown sims"

# Metrics
duration: 2min
completed: 2026-03-02
---

# Phase 03 Plan 02: Infrastructure Modules — Worktrees, Command Runner, Labels, Devices Summary

**Four infra modules behind clean function boundaries: git porcelain parser, generic async command runner with mpsc streaming, JSON label persistence, and adb/xcrun device list parsers**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-02T08:27:41Z
- **Completed:** 2026-03-02T08:29:37Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Created `src/infra/worktrees.rs` with `parse_worktree_porcelain` (pure, handles detached HEAD and bare), `check_stale` (mtime comparison), and `list_worktrees` (async runner)
- Created `src/infra/labels.rs` with `config_dir`, `labels_path`, `load_labels` (empty HashMap on file not found), and `save_labels` (creates dir, pretty JSON)
- Created `src/infra/command_runner.rs` with `spawn_command_task` (spawns any CommandSpec, tokio::select! streaming, CommandExited on finish), `build_argv` (GitResetHard injects `origin/{branch}`), and `stream_command_output` (dual-stream select loop)
- Created `src/infra/devices.rs` with `parse_adb_devices` (tab-split, state="device" filter), `parse_xcrun_simctl` (JSON, isAvailable filter, formatted display names), `list_android_devices`, and `list_ios_devices`
- Updated `src/infra/mod.rs` to export all 6 modules: port, process, worktrees, command_runner, labels, devices

## Task Commits

Each task was committed atomically:

1. **Task 1: Create worktrees.rs and labels.rs** - `09b3ce7` (feat)
2. **Task 2: Create command_runner.rs and devices.rs, update infra/mod.rs** - `bc2547d` (feat)

**Plan metadata:** (docs commit to follow)

## Files Created/Modified

- `src/infra/worktrees.rs` - parse_worktree_porcelain, check_stale, list_worktrees
- `src/infra/labels.rs` - config_dir, labels_path, load_labels, save_labels
- `src/infra/command_runner.rs` - spawn_command_task, build_argv, stream_command_output
- `src/infra/devices.rs` - parse_adb_devices, parse_xcrun_simctl, list_android_devices, list_ios_devices
- `src/infra/mod.rs` - now exports 6 modules

## Decisions Made

- `WorktreeId` is the full path string — gives stable identity independent of branch name
- `check_stale` returns `true` when node_modules is absent (safe default: always stale means always install)
- `stream_command_output` uses explicit `stdout_done`/`stderr_done` guards on `select!` arms — prevents polling closed streams which would spin-loop
- `parse_adb_devices` sets both `id` and `name` to the adb serial — adb list doesn't expose model names; Phase 5 could enrich with `adb shell getprop ro.product.model`
- `parse_xcrun_simctl` formats display name as `"{name} ({state})"` to distinguish booted from shutdown simulators in the device picker UI

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

- All infra I/O boundaries are implemented — Plan 03-03 (app logic) can call `list_worktrees`, `spawn_command_task`, `load_labels`/`save_labels`, and the device list functions
- `spawn_command_task` returns a `JoinHandle` so `CommandCancel` can call `.abort()` on it
- `build_argv` handles the only special case (GitResetHard) — all other CommandSpec variants go through `spec.to_argv()` unchanged
- Pure parsers (`parse_worktree_porcelain`, `parse_adb_devices`, `parse_xcrun_simctl`) are ready for unit tests in Phase 3 verification

---
*Phase: 03-worktree-browser-git-and-rn-commands*
*Completed: 2026-03-02*
