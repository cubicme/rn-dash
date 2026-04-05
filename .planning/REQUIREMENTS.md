# Requirements: RN Dash

**Defined:** 2026-04-05
**Core Value:** One place to see and control everything about your React Native worktrees — which one is running, what branch each is on, and execute any command without context-switching.

## v1.1 Requirements

### Keybindings & Palettes

- [x] **KEY-01**: User sees (y)arn palette (renamed from sync) containing yarn install, pod-install, and all clean commands
- [x] **KEY-02**: User sees (w)orktree palette (extracted from git) containing worktree create, remove, and create-with-new-branch
- [ ] **KEY-03**: User can create a worktree with a new branch, selecting the base branch interactively
- [x] **KEY-04**: User sees metro R (reload) and J (debugger) keys only when metro is running
- [x] **KEY-05**: User can stop metro with ESC when metro is running; metro restart key removed (RET handles it)
- [x] **KEY-06**: User sees dynamically derived hint line based on currently available actions, not hardcoded strings
- [x] **KEY-07**: Footer legend updated — no stale ▶/⚠ indicators

### Cleanup

- [ ] **CLN-01**: Labels feature completely removed (domain types, infra persistence, UI rendering, keybindings)

### Generalization

- [x] **GEN-01**: All hardcoded AJ/UMP/system-specific values extracted to config (repo paths, JIRA project prefix, branch patterns)
- [x] **GEN-02**: Config example file documenting all available settings with comments

### GitHub Release

- [x] **GH-01**: MIT license file in repo root
- [x] **GH-02**: Cargo.toml metadata (description, license, repository, homepage, keywords)
- [x] **GH-03**: README with project description, screenshots, build instructions, usage guide, config reference
- [x] **GH-04**: .gitignore audited — no .planning/, credentials, or build artifacts leak
- [ ] **GH-05**: GitHub Actions CI workflow (build + clippy + test on macOS and Linux)
- [ ] **GH-06**: GitHub Actions release workflow — builds prebuilt binaries and creates GitHub Releases on tag push

## v2 Requirements

### Enhanced UX

- **UX-01**: Configurable keybinding overrides via config file
- **UX-02**: Theme/color customization
- **UX-03**: Multi-project support (switch between different RN repos)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Mobile app or web UI | Terminal dashboard only |
| Building/modifying the RN app itself | This tool manages it, doesn't build it |
| Real-time JIRA sync or ticket creation | Read-only ticket title fetching |
| Multi-user support | Single-user tool |
| Windows support | macOS primary, Linux for CI only |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLN-01 | Phase 07 | Pending |
| KEY-01 | Phase 08 | Complete |
| KEY-02 | Phase 08 | Complete |
| KEY-03 | Phase 08 | Pending |
| KEY-04 | Phase 08 | Complete |
| KEY-05 | Phase 08 | Complete |
| KEY-06 | Phase 08 | Complete |
| KEY-07 | Phase 08 | Complete |
| GEN-01 | Phase 09 | Complete |
| GEN-02 | Phase 09 | Complete |
| GH-01 | Phase 09 | Complete |
| GH-02 | Phase 09 | Complete |
| GH-03 | Phase 09 | Complete |
| GH-04 | Phase 09 | Complete |
| GH-05 | Phase 10 | Pending |
| GH-06 | Phase 10 | Pending |

**Coverage:**
- v1.1 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-05*
*Last updated: 2026-04-05 after v1.1 roadmap creation*
