---
phase: 04-config-and-jira-integration
plan: 01
subsystem: infra
tags: [jira, reqwest, config, serde_json, async-trait, tmux]

# Dependency graph
requires:
  - phase: 03-worktree-browser-git-and-rn-commands
    provides: labels.rs config_dir() pattern and infra module structure
provides:
  - DashConfig struct with load_config()/save_config() (0600 perms on Unix)
  - JiraClient trait + HttpJiraClient (basic_auth cloud / bearer_auth datacenter)
  - load_jira_cache()/save_jira_cache() HashMap persistence at jira_cache.json
  - extract_jira_key() pure branch-name parser
  - is_inside_tmux() one-liner tmux detection
affects: [04-02-jira-integration-in-tea-loop]

# Tech tracking
tech-stack:
  added: [reqwest 0.12 (json + rustls-tls)]
  patterns: [config_dir() shared across all infra modules, NotFound → None/empty pattern]

key-files:
  created:
    - src/infra/config.rs
    - src/infra/jira.rs
    - src/infra/jira_cache.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/infra/mod.rs

key-decisions:
  - "reqwest 0.12 used (0.13 does not exist yet; plan specified 0.13 but 0.12 is current stable)"
  - "extract_jira_key() uses match/continue instead of ? operator — ? inside for loop exits the whole function, not just the iteration"
  - "HttpJiraClient::new() builds bare reqwest::Client with no default auth — auth applied per-request for clarity and auditability"
  - "save_config() 0600 permissions guarded by #[cfg(unix)] with #[cfg(not(unix))] no-op fallback"

patterns-established:
  - "JiraClient trait: async fetch_title returns Option<String>, never panics, absorbs all errors"
  - "NotFound pattern: match read_to_string; Err(e) if NotFound => Ok(None/empty); else Err(e.into())"
  - "Auth dispatch: auth_mode == datacenter => bearer_auth; else => basic_auth with email:token"

requirements-completed: [INTG-01, INTG-02, INTG-03, INTG-05]

# Metrics
duration: 2min
completed: 2026-03-02
---

# Phase 04 Plan 01: Config and JIRA Infrastructure Summary

**reqwest-based JIRA HTTP client with Basic/Bearer auth dispatch, DashConfig 0600-protected credentials file, and title cache persistence following the labels.rs NotFound pattern**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-02T12:57:16Z
- **Completed:** 2026-03-02T12:59:16Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Four new infra modules (config.rs, jira.rs, jira_cache.rs via mod.rs) all compile clean
- JIRA HTTP client supports Cloud (basic_auth email:token) and Data Center (bearer_auth PAT) — auth mode selected at runtime from config
- DashConfig written with 0600 permissions on Unix; missing file returns Ok(None) gracefully
- extract_jira_key() correctly parses branch segments; all 5 unit test cases pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add reqwest dependency and create config + cache + jira + tmux modules** - `5244eda` (feat)

**Plan metadata:** committed with SUMMARY.md docs commit

## Files Created/Modified

- `Cargo.toml` - Added reqwest 0.12 with json + rustls-tls features
- `Cargo.lock` - Updated with reqwest dependency tree
- `src/infra/mod.rs` - Registered config, jira, jira_cache modules
- `src/infra/config.rs` - DashConfig struct, load_config(), save_config() with 0600 perms
- `src/infra/jira_cache.rs` - load_jira_cache(), save_jira_cache() for title persistence
- `src/infra/jira.rs` - JiraClient trait, HttpJiraClient, extract_jira_key(), is_inside_tmux()

## Decisions Made

- **reqwest 0.12 not 0.13**: Plan specified 0.13 but cargo reports latest stable is 0.12. Used 0.12 (deviation Rule 1 — plan was specifying a non-existent version).
- **match/continue in extract_jira_key**: Initial implementation used `?` operator inside `for` loop which exits the *function*, not the iteration. Fixed to `match/continue` pattern.
- **Bare reqwest::Client**: No default auth headers set on the client; auth applied per-request for auditability and correctness when auth_mode changes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed ? operator misuse inside for loop in extract_jira_key()**
- **Found during:** Task 1 (verification — cargo test)
- **Issue:** Using `parts.next()?` inside `for segment in branch.split('/')` causes the `?` to return `None` from the outer function rather than continuing to the next loop iteration. This caused "feature/UMP-1234-login" to return `None` instead of `Some("UMP-1234")`.
- **Fix:** Replaced `?` with `match ... { Some(v) => v, None => continue }` pattern
- **Files modified:** src/infra/jira.rs
- **Verification:** All 5 unit tests now pass
- **Committed in:** 5244eda (Task 1 commit)

**2. [Rule 1 - Bug] Used reqwest 0.12 instead of non-existent 0.13**
- **Found during:** Task 1 (Cargo.toml update — `cargo add reqwest --dry-run` revealed 0.12 is current)
- **Issue:** Plan specified reqwest 0.13 but this version does not exist; cargo resolves to 0.12
- **Fix:** Used `reqwest = { version = "0.12", ... }` which resolves correctly
- **Files modified:** Cargo.toml, Cargo.lock
- **Verification:** cargo check passes
- **Committed in:** 5244eda (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 — bugs in plan spec)
**Impact on plan:** Both fixes were necessary for correctness. No scope creep.

## Issues Encountered

None beyond the two auto-fixed deviations above.

## User Setup Required

None — no external service configuration required at this stage. JIRA credentials will be configured via `~/.config/ump-dash/config.json` (instructions in Phase 4 Plan 02).

## Next Phase Readiness

- All infra modules compile and are registered in `src/infra/mod.rs`
- Plan 04-02 can directly `use crate::infra::config::load_config` and `use crate::infra::jira::HttpJiraClient` to wire JIRA into the TEA loop
- Blocker from STATE.md partially resolved: auth_mode field in DashConfig supports both "cloud" and "datacenter"; user must still set their config file with correct auth_mode

---
*Phase: 04-config-and-jira-integration*
*Completed: 2026-03-02*
