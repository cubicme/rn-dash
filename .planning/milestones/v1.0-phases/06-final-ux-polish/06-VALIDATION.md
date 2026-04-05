---
phase: 06
slug: final-ux-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-12
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo check + manual TUI verification |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo check --incremental` |
| **Full suite command** | `cargo check --incremental && cargo clippy` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --incremental`
- **After every plan wave:** Run `cargo check --incremental && cargo clippy`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | Metro log filter | compile | `cargo check --incremental` | N/A | ⬜ pending |
| 06-02-01 | 02 | 1 | Shell tab command | compile | `cargo check --incremental` | N/A | ⬜ pending |
| 06-03-01 | 03 | 1 | Metro running icon | compile | `cargo check --incremental` | N/A | ⬜ pending |
| 06-04-01 | 04 | 1 | Prefix ordering | compile | `cargo check --incremental` | N/A | ⬜ pending |
| 06-05-01 | 05 | 1 | Claude tab name | compile | `cargo check --incremental` | N/A | ⬜ pending |
| 06-06-01 | 06 | 1 | Double border title | compile | `cargo check --incremental` | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Metro log noise filtered | Metro log filter | TUI visual output | Start metro, verify watchman/noise logs suppressed |
| Shell tab opens in worktree | Shell tab command | tmux/zellij integration | Press T on worktree, verify tab opens at correct path |
| Green play icon visible | Metro running icon | TUI visual rendering | Start metro, verify green play icon in worktree row |
| Tab name uses correct prefix | Prefix ordering | tmux tab name | Open claude tab, verify format is `prefix-claude` |
| Optional name modal works | Claude tab name | Modal input UX | Press C, type name, verify tab suffix |
| Double border on title | Double border title | TUI visual rendering | Launch app, verify title has double border |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
