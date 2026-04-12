---
phase: 08-palette-and-keybinding-rework
plan: "05"
subsystem: infra/config
tags: [config, toml, json, readme, cleanup]
dependency_graph:
  requires: []
  provides: [TOML-based config loading and saving]
  affects: [src/infra/config.rs, README.md]
tech_stack:
  added: [toml = "0.8"]
  patterns: [toml::from_str, toml::to_string_pretty]
key_files:
  created: [config.example.toml]
  modified: [src/infra/config.rs, Cargo.toml, README.md]
  deleted: [config.example.json]
decisions:
  - "toml crate used instead of serde_json for config.rs; serde_json retained for jira_cache, android_prefs, sim_history, devices"
metrics:
  duration_minutes: 8
  completed_date: "2026-04-05"
  tasks_completed: 2
  files_changed: 5
---

# Phase 08 Plan 05: Config JSON-to-TOML Switch and README Cleanup Summary

**One-liner:** TOML config format replaces JSON for rn-dash configuration using `toml::from_str`/`toml::to_string_pretty`, with annotated `config.example.toml` and cleaned README.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Switch DashConfig parser from JSON to TOML | cc72eba | src/infra/config.rs, Cargo.toml, config.example.toml, (deleted config.example.json) |
| 2 | Update README — TOML references, remove screenshot section | effdec6 | README.md |

## What Was Built

- `src/infra/config.rs`: `load_config()` now reads `config.toml` via `toml::from_str`; `save_config()` writes `config.toml` via `toml::to_string_pretty`. All `serde_json` references removed from this file.
- `Cargo.toml`: Added `toml = "0.8"` dependency. `serde_json` retained (still used by other infra files).
- `config.example.toml`: New annotated example with native TOML comments. Optional fields commented out to distinguish required vs optional.
- `config.example.json`: Deleted via `git rm`.
- `README.md`: Screenshot section removed, config file path/copy command/template reference updated to `.toml`.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — threat model reviewed. Config file permissions (0600) and trust boundary unchanged; only serialization format changed. `toml` crate is well-audited Rust ecosystem standard.

## Deferred Issues

**Pre-existing clippy warning in config.rs (out of scope):**
- `manual_strip` warning in `repo_root_path()` at line 79-81 — pre-dates this plan, not introduced by these changes
- File: `src/infra/config.rs`, function `repo_root_path()`
- Fix: replace `&s[2..]` with `s.strip_prefix("~/").unwrap_or(s)`

## Self-Check: PASSED

- `src/infra/config.rs` — FOUND
- `config.example.toml` — FOUND
- `config.example.json` — correctly deleted (git rm)
- `README.md` — FOUND, 0 "Screenshot" hits, 0 "config.json" hits, 2 "config.toml" hits
- commit cc72eba — FOUND
- commit effdec6 — FOUND
- `cargo check` — PASSED
