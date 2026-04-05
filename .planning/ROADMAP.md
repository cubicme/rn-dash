# Roadmap: UMP Dashboard

## Milestones

- ✅ **v1.0 MVP** — Phases 01-06 (shipped 2026-04-05)
- 🚧 **v1.1 Public Release** — Phases 07-10 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 01-06) — SHIPPED 2026-04-05</summary>

- [x] Phase 01: Scaffold and TUI Shell (3/3 plans) — completed 2026-03-02
- [x] Phase 02: Metro Process Control (3/3 plans) — completed 2026-03-02
- [x] Phase 03: Worktree Browser, Git, and RN Commands (5/5 plans) — completed 2026-03-02
- [x] Phase 04: Config and JIRA Integration (3/3 plans) — completed 2026-03-02
- [x] Phase 05: Worktree Switching and Claude Code (2/2 plans) — completed 2026-03-03
- [x] Phase 05.1: Milestone Feedback — UX overhaul (8/8 plans)
- [x] Phase 05.2: Milestone Feedbacks — Bug fixes and polish (10/10 plans)
- [x] Phase 06: Final UX Polish (3/3 plans)

See: `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

### 🚧 v1.1 Public Release (In Progress)

**Milestone Goal:** Polish keybinding scheme, remove dead features, extract hardcoded values to config, and prepare for public GitHub release with README, license, CI, and prebuilt binaries.

- [ ] **Phase 07: Labels Removal** — Remove labels feature entirely from domain, infra, UI, and keybindings
- [ ] **Phase 08: Palette and Keybinding Rework** — Rename sync to yarn palette, extract worktree commands, add new worktree creation, context-sensitive metro keys, dynamic hints, updated footer
- [ ] **Phase 09: Generalization and GitHub Prep** — Extract hardcoded values to config, config example file, license, Cargo.toml metadata, README, .gitignore audit
- [ ] **Phase 10: CI and Release** — GitHub Actions CI workflow and prebuilt binary release workflow

## Phase Details

### Phase 07: Labels Removal
**Goal**: Labels feature is completely gone — no dead code, no hidden state, no orphaned UI
**Depends on**: Phase 06
**Requirements**: CLN-01
**Success Criteria** (what must be TRUE):
  1. User sees no label-related UI elements or keybindings in the dashboard
  2. No label files are written or read on startup
  3. Codebase compiles with zero warnings related to labels
  4. Worktree list renders correctly with branch and JIRA title only
**Plans**: 1 plan
Plans:
- [ ] 07-01-PLAN.md — Remove labels feature: relocate config_dir, delete labels.rs, clean domain/action/app/UI

### Phase 08: Palette and Keybinding Rework
**Goal**: Users interact with a clean, context-sensitive keybinding scheme — yarn palette, worktree palette, metro keys only when relevant, dynamic hints
**Depends on**: Phase 07
**Requirements**: KEY-01, KEY-02, KEY-03, KEY-04, KEY-05, KEY-06, KEY-07
**Success Criteria** (what must be TRUE):
  1. User opens (y)arn palette and sees yarn install, pod-install, and all clean commands
  2. User opens (w)orktree palette and sees create, remove, and create-with-new-branch commands
  3. User can create a worktree with a new branch by selecting a base branch from an interactive picker
  4. Metro R (reload) and J (debugger) keys appear in hints only when metro is running
  5. User can press ESC to stop metro when metro is running; no separate restart key exists
  6. Hint line reflects only currently available actions (no stale or inapplicable hints shown)
**Plans**: 3 plans
Plans:
- [x] 08-01-PLAN.md — Restructure palettes: rename Sync to Yarn, extract Worktree from Git, update hints and help
- [ ] 08-02-PLAN.md — New-branch worktree creation with interactive base branch picker
- [x] 08-03-PLAN.md — Context-sensitive metro keys, dynamic footer hints, remove stale legend
**UI hint**: yes

### Phase 09: Generalization and GitHub Prep
**Goal**: App works for any React Native monorepo (no hardcoded AJ/UMP values), and repo is ready for public GitHub release
**Depends on**: Phase 08
**Requirements**: GEN-01, GEN-02, GH-01, GH-02, GH-03, GH-04
**Success Criteria** (what must be TRUE):
  1. User can configure repo paths, JIRA project prefix, and branch patterns via config file with no source changes
  2. Config example file exists documenting all settings with comments
  3. MIT license file and updated Cargo.toml metadata (description, license, repository, keywords) are present
  4. README covers project description, build instructions, usage guide, and config reference
  5. .gitignore excludes .planning/, credentials, and build artifacts
**Plans**: TBD

### Phase 10: CI and Release
**Goal**: Every push is verified by CI and tagged releases produce downloadable prebuilt binaries
**Depends on**: Phase 09
**Requirements**: GH-05, GH-06
**Success Criteria** (what must be TRUE):
  1. GitHub Actions CI runs build + clippy + test on both macOS and Linux on every push
  2. Pushing a version tag produces a GitHub Release with prebuilt binaries attached
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 01. Scaffold and TUI Shell | v1.0 | 3/3 | Complete | 2026-03-02 |
| 02. Metro Process Control | v1.0 | 3/3 | Complete | 2026-03-02 |
| 03. Worktree Browser, Git, RN Commands | v1.0 | 5/5 | Complete | 2026-03-02 |
| 04. Config and JIRA Integration | v1.0 | 3/3 | Complete | 2026-03-02 |
| 05. Worktree Switching and Claude Code | v1.0 | 2/2 | Complete | 2026-03-03 |
| 05.1 Milestone Feedback | v1.0 | 8/8 | Complete | — |
| 05.2 Milestone Feedbacks | v1.0 | 10/10 | Complete | — |
| 06. Final UX Polish | v1.0 | 3/3 | Complete | — |
| 07. Labels Removal | v1.1 | 0/1 | Planned | - |
| 08. Palette and Keybinding Rework | v1.1 | 0/3 | Planned | - |
| 09. Generalization and GitHub Prep | v1.1 | 0/? | Not started | - |
| 10. CI and Release | v1.1 | 0/? | Not started | - |
