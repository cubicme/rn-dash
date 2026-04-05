---
phase: 02-metro-process-control
verified: 2026-03-02T00:00:00Z
status: passed
score: 13/13 must-haves verified
re_verification: false
gaps: []
human_verification:
  - test: "Press 's' when MetroPane is focused while no metro is running"
    expected: "Status indicator transitions to STARTING... (yellow) then RUNNING (green, with pid and worktree name)"
    why_human: "Requires actual yarn/metro binary. Can't verify process spawn behavior without running the app."
  - test: "Press 'x' while metro is running"
    expected: "Status indicator transitions to STOPPING... (yellow) then STOPPED (gray) after port 8081 confirmed free"
    why_human: "Requires a running metro process and real port polling. Cannot simulate child.wait() in static analysis."
  - test: "Press 's' while metro is already running (single-instance enforcement)"
    expected: "Existing instance is killed, then new instance starts from the active worktree"
    why_human: "Requires two process lifetimes and the pending_restart flow to be observed end-to-end."
  - test: "Kill metro from outside the dashboard (e.g., kill <pid> in a shell)"
    expected: "Dashboard detects exit via child.wait() and status reverts to STOPPED without user action"
    why_human: "External process kill cannot be simulated statically."
  - test: "Press 'l' to toggle log panel, verify layout changes and log filter restarts metro"
    expected: "Three-panel layout appears (25%/40%/35%); if metro was running, it restarts with DEBUG=Metro:* set"
    why_human: "Layout changes and process filter restart require runtime observation."
  - test: "Press 'J' (shift-j) while metro is running"
    expected: "Metro sends j\\n to its stdin and enters JS debugger mode"
    why_human: "Stdin forwarding to a live metro process cannot be verified statically."
  - test: "Press 'R' (shift-r) while metro is running"
    expected: "Metro sends r\\n to its stdin and reloads the RN bundle"
    why_human: "Stdin forwarding to a live metro process cannot be verified statically."
---

# Phase 02: Metro Process Control Verification Report

**Phase Goal:** Users can start, stop, restart, and monitor metro with guaranteed single-instance enforcement — the zombie-process and port-binding bugs are addressed before any downstream features depend on this layer
**Verified:** 2026-03-02
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | MetroManager enforces single-instance invariant via Option<MetroHandle> | VERIFIED | `src/domain/metro.rs:64` — `handle: Option<MetroHandle>` (private field); `register()` panics if handle already set |
| 2 | MetroStatus enum exposes Running/Stopped/Starting/Stopping states for UI consumption | VERIFIED | `src/domain/metro.rs:17-26` — all four variants defined with derive(Debug,Clone,PartialEq) and Default=Stopped |
| 3 | ProcessClient trait hides tokio::process behind a boundary — domain has zero infra deps | VERIFIED | `src/infra/process.rs:16-27` — async_trait bound; domain/mod.rs has zero `use crate::infra` imports |
| 4 | Action enum has all 10 metro variants | VERIFIED | `src/action.rs:28-39` — MetroStart/Stop/Restart/ToggleLog/ScrollUp/ScrollDown/SendDebugger/SendReload/LogLine/Exited present |
| 5 | AppState has metro, metro_logs, log state fields and pending_restart | VERIFIED | `src/app.rs:56-69` — all fields present including `pending_restart: bool` |
| 6 | User can start metro with 's' when MetroPane focused | VERIFIED | `src/app.rs:121` — `Char('s') => return Some(Action::MetroStart)` in MetroPane branch; `update()` at line 181 spawns via `tokio::spawn(spawn_metro_task(...))` |
| 7 | Starting metro when another instance runs auto-kills existing one first | VERIFIED | `src/app.rs:182-187` — `if state.metro.is_running() { state.pending_restart = true; update(state, Action::MetroStop, ...)` |
| 8 | User can stop metro with 'x' and port 8081 verified free before UI shows Stopped | VERIFIED | `src/app.rs:123` keybinding; `metro_process_task` at line 407-417 kills child, polls `port_is_free(8081)` up to 50x, then sends `MetroExited` |
| 9 | User can restart metro with 'r' (kill + wait port free + start) | VERIFIED | `src/app.rs:212-219` — MetroRestart sets `pending_restart = true` → MetroStop → on MetroExited (line 261-266) → MetroStart |
| 10 | User can send debugger command with 'J' to running metro | VERIFIED | `src/app.rs:127` keybinding; `Action::MetroSendDebugger` at line 221 calls `state.metro.send_stdin(b"j\n".to_vec())` |
| 11 | User can send reload command with 'R' to running metro | VERIFIED | `src/app.rs:128` keybinding; `Action::MetroSendReload` at line 227 calls `state.metro.send_stdin(b"r\n".to_vec())` |
| 12 | Metro killed outside dashboard is detected and status updates to Stopped | VERIFIED | `src/app.rs:419-422` — `child.wait()` arm in `metro_process_task` sends `MetroExited` on natural child exit |
| 13 | User can see status indicator with correct colors and log panel with scroll | VERIFIED | `src/ui/panels.rs:34-47` — all 4 MetroStatus variants matched with Green/DarkGray/Yellow; `render_log_panel()` at line 77 with scrollbar; `src/ui/mod.rs:44` conditional layout |

**Score:** 13/13 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/domain/metro.rs` | MetroManager, MetroHandle, MetroStatus | VERIFIED | 136 lines, all types exported; kill_tx field added in plan 02 |
| `src/infra/process.rs` | ProcessClient trait + TokioProcessClient | VERIFIED | process_group(0), kill_on_drop, piped IO, filter env var — fully implemented |
| `src/infra/port.rs` | port_is_free() function | VERIFIED | Single-purpose 14-line file; TcpListener::bind probe |
| `src/action.rs` | All metro Action variants | VERIFIED | 12 occurrences of "Metro" string; all 10 required variants present |
| `src/app.rs` | Metro keybinding dispatch, async spawn/kill/restart, channel integration | VERIFIED | 473 lines; handle_key metro branch; update() metro arms; spawn_metro_task, metro_process_task, stream_metro_logs, stdin_writer async helpers; dual-channel run() select loop |
| `src/ui/panels.rs` | Metro pane with status indicator + log panel | VERIFIED | render_metro_pane() with full MetroStatus match; render_log_panel() with auto-scroll and Scrollbar |
| `src/ui/mod.rs` | Conditional layout for log panel | VERIFIED | log_panel_visible branch at line 44; 2-panel vs 3-panel split |
| `src/ui/footer.rs` | Metro-specific key hints | VERIFIED | l/J/R hints in MetroPane arm; J/R conditional on metro.is_running() |
| `src/ui/help_overlay.rs` | Metro keybindings in help table | VERIFIED | Metro section header + 6 keybinding rows (s/x/r/l/J/R) |

---

## Key Link Verification

### Plan 01 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/domain/metro.rs` | `src/app.rs` | AppState.metro field | WIRED | `src/app.rs:56` — `pub metro: crate::domain::metro::MetroManager` |
| `src/action.rs` | `src/app.rs` | update() match arms | WIRED | `src/app.rs:181-268` — all 10 metro Action variants matched |

### Plan 02 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/app.rs (handle_key)` | `src/app.rs (update)` | Action dispatch | WIRED | `src/app.rs:299` — `update(&mut state, action, &metro_tx, &handle_tx)` |
| `src/app.rs (update)` | `src/domain/metro.rs (MetroManager)` | state.metro.register/clear/send_stdin | WIRED | Lines 185, 196, 200, 215, 222, 228, 237, 263 — all MetroManager methods called |
| `src/app.rs (update)` | `src/infra/process.rs (ProcessClient)` | tokio::spawn for spawn_metro | WIRED | `src/app.rs:196` — `tokio::spawn(spawn_metro_task(...))` → `src/app.rs:344` — `client.spawn_metro(...)` |
| `src/app.rs (run)` | `tokio::sync::mpsc` | metro_rx + handle_rx in select! loop | WIRED | Lines 279, 283, 308-315 — both channel arms in select! |

### Plan 03 Key Links

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/ui/panels.rs` | `src/domain/metro.rs` | MetroStatus enum matching | WIRED | `src/ui/panels.rs:8` — `use crate::domain::metro::MetroStatus`; lines 35-46 match all variants |
| `src/ui/panels.rs` | `src/app.rs` | AppState.metro_logs, log_scroll_offset | WIRED | Lines 84 and 91 — `state.metro_logs.iter()`, `state.log_scroll_offset` |
| `src/ui/mod.rs` | `src/app.rs` | state.log_panel_visible for layout toggle | WIRED | `src/ui/mod.rs:44` — `if state.log_panel_visible` |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| METRO-01 | 02-01, 02-03 | User can see which worktree has metro running with status indicator | SATISFIED | `render_metro_pane()` — Running shows `pid=N [worktree]` in green; `MetroStatus::Running { pid, worktree_id }` |
| METRO-02 | 02-02 | User can start metro (yarn start --reset-cache) from the active worktree | SATISFIED | `TokioProcessClient::spawn_metro()` uses `yarn start --reset-cache`; `s` key dispatches MetroStart; uses `state.active_worktree_path` |
| METRO-03 | 02-02 | User can stop the running metro instance | SATISFIED | `x` key → MetroStop → kill_tx.send(()) → child.kill() → port-free polling → MetroExited → metro.clear() |
| METRO-04 | 02-02 | User can restart metro with one keystroke | SATISFIED | `r` key → MetroRestart → pending_restart=true → MetroStop → on MetroExited → MetroStart |
| METRO-05 | 02-01, 02-03 | User can view metro log output when filter applied (metro does not stream by default) | SATISFIED | `TokioProcessClient::spawn_metro(filter=true)` sets `DEBUG=Metro:*`; `render_log_panel()` in 3-panel layout when log_panel_visible |
| METRO-06 | 02-03 | User can scroll through metro log history | SATISFIED | `MetroScrollUp/Down` in action.rs; `log_scroll_offset` in AppState; Scrollbar rendered in `render_log_panel()` |
| METRO-07 | 02-02 | User can send debugger command (j) to running metro | SATISFIED | `J` key → MetroSendDebugger → `state.metro.send_stdin(b"j\n")` → stdin_writer task → child stdin |
| METRO-08 | 02-02 | User can send reload command (r) to running metro | SATISFIED | `R` key → MetroSendReload → `state.metro.send_stdin(b"r\n")` → stdin_writer task → child stdin |
| METRO-09 | 02-01, 02-02 | Only one metro instance can run at a time | SATISFIED | Structural: `Option<MetroHandle>` in MetroManager (type-level); runtime: MetroStart checks `is_running()` and kills first |

**All 9 requirements (METRO-01 through METRO-09) satisfied.**

No orphaned requirements found — all METRO-01..09 are mapped to this phase in REQUIREMENTS.md and covered by plans 02-01, 02-02, and 02-03.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/ui/panels.rs` | 21-22 | `"[ worktree list — Phase 3 ]"` placeholder | INFO | Expected — worktree list is Phase 3 scope; does not affect metro |
| `src/ui/panels.rs` | 130-131 | `"[ command output — Phase 3 ]"` placeholder | INFO | Expected — command output is Phase 3 scope; does not affect metro |
| `src/infra/process.rs` | 7 | `#![allow(dead_code)]` | INFO | TokioProcessClient is used indirectly via trait; lint suppression is expected at this stage |
| `src/app.rs` | 1 | `#![allow(dead_code)]` | INFO | Some fields are written but not yet read by future phases; expected pattern |
| `src/domain/metro.rs` | 49 | `stream_task` field never read warning (dead_code) | INFO | Field is stored for lifecycle management (abort on kill); the value is intentionally held, not read |

No blockers found. All anti-patterns are expected scaffolding for future phases, not implementation gaps.

---

## Architecture Constraints Verified

| Constraint | Rule | Status | Evidence |
|------------|------|--------|----------|
| ARCH-01: Domain imports no infra | `src/domain/` has zero `use crate::infra` | VERIFIED | grep returned no matches in domain/metro.rs or domain/mod.rs |
| ARCH-02: Infra behind trait boundaries | ProcessClient trait wraps TokioProcessClient | VERIFIED | `src/infra/process.rs:16-27` — async_trait trait definition + impl |
| ARCH-03: UI imports domain only, never infra | UI files have no `use crate::infra` | VERIFIED | grep returned no matches in any src/ui/*.rs file |

---

## Build Status

```
cargo build — 0 errors, 2 pre-existing dead_code warnings (stream_task field + #[allow] suppressions)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
```

The two warnings are architectural: `stream_task` is stored in MetroHandle for lifecycle management (the JoinHandle is held to keep the task alive; abort via log_task is called on kill, not via MetroHandle.stream_task directly). This is a known pattern and does not indicate a bug.

---

## Human Verification Required

The following behaviors require a running app with an actual yarn/metro binary to verify:

### 1. Metro Start Flow

**Test:** Focus MetroPane (Tab), press 's'
**Expected:** Status indicator transitions STOPPED → STARTING... (yellow) → RUNNING pid=N [worktree] (bold green)
**Why human:** Requires yarn binary and real process spawn to observe status transitions

### 2. Metro Stop with Port-Free Verification

**Test:** While metro is running, press 'x'
**Expected:** Status → STOPPING... (yellow), then STOPPED (gray) only after port 8081 is confirmed free
**Why human:** Requires live process kill and port polling observation

### 3. Single-Instance Enforcement

**Test:** Start metro, then press 's' again while it is running
**Expected:** Existing instance is killed first, then new instance starts (no two metros running simultaneously)
**Why human:** Requires two process lifetimes and the pending_restart chain to be observed

### 4. External Kill Detection

**Test:** Start metro from dashboard, then in a separate shell run `kill <pid>`
**Expected:** Dashboard status automatically reverts to STOPPED without any user interaction
**Why human:** External process kill cannot be simulated statically; requires child.wait() to fire

### 5. Log Panel Toggle and Filter

**Test:** Start metro, press 'l' to toggle logs
**Expected:** Three-panel layout appears; metro restarts with DEBUG=Metro:* active; log lines stream into log panel; scrollbar appears when content overflows
**Why human:** Log streaming, filter restart, and scrollbar rendering require runtime observation

### 6. Debugger Command (J)

**Test:** While metro is running, press shift-j
**Expected:** Metro enters JS debugger mode (observable in metro output or browser devtools)
**Why human:** Requires a running metro process and observable effect in metro itself

### 7. Reload Command (R)

**Test:** While metro is running, press shift-r
**Expected:** Metro performs a bundle reload (observable in metro log output)
**Why human:** Requires a running metro process and observable reload behavior

---

## Gaps Summary

No gaps found. All must-haves from all three plans are verified in the codebase:

- Plan 01 (type contracts): All types, traits, and state extensions present and substantive
- Plan 02 (async runtime): Full spawn/kill/restart/stdin/death-detection pipeline wired end-to-end
- Plan 03 (UI): Status indicator, log panel, footer hints, and help overlay all implemented with real data

The phase goal is achieved: users can start, stop, restart, and monitor metro via a guaranteed single-instance enforcer. The zombie-process bug is addressed via `process_group(0)` and port-free polling. The port-binding bug is addressed by verifying port 8081 free before signaling Stopped. All downstream phases can build on these contracts.

---

_Verified: 2026-03-02_
_Verifier: Claude (gsd-verifier)_
