# Phase 09: Generalization and GitHub Prep - Context

**Gathered:** 2026-04-05
**Status:** Ready for planning
**Mode:** Auto-generated (infrastructure phase)

<domain>
## Phase Boundary

App works for any React Native monorepo (no hardcoded AJ/UMP values), and repo is ready for public GitHub release. License, Cargo.toml metadata, README, config example, .gitignore audit.

</domain>

<decisions>
## Implementation Decisions

### Config Extraction
- All hardcoded AJ/UMP/system-specific values must be extracted to config
- This includes: repo paths, JIRA project prefix (currently hardcoded as "UMP"), branch naming patterns
- Config location should be configurable (default ~/.config/rn-dash/)
- The app name should be "rn-dash" (not ump-dash)

### Config Example
- Create a config.example.toml (or whatever format is used) documenting all available settings with comments
- Should be complete enough for a new user to set up the tool

### License
- MIT license — user confirmed

### Cargo.toml Metadata
- description, license = "MIT", repository URL, homepage, keywords (ratatui, react-native, terminal, dashboard, worktree)

### README
- Project description with what it does
- Screenshots (placeholder — user will add later)
- Build instructions (cargo build --release)
- Usage guide (how to run, basic keybindings)
- Config reference (all settings)

### .gitignore
- Must exclude .planning/, credentials, build artifacts
- Audit current .gitignore for completeness

### Claude's Discretion
- README structure and formatting
- Config file format (TOML vs JSON — follow existing pattern)
- Specific Cargo.toml keywords

</decisions>

<code_context>
## Existing Code Insights

Codebase context will be gathered during plan-phase research.

</code_context>

<specifics>
## Specific Ideas

- User said: "make sure nothing is hardcoded that is unique to AJ, ump or my system and everything is available in config"
- Current config is at ~/.config/ump-dash/ — needs to become configurable, default to ~/.config/rn-dash/

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>
