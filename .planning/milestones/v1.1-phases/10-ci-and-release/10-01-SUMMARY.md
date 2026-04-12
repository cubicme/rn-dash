---
phase: 10-ci-and-release
plan: "01"
subsystem: ci, release, github-actions
tags: [github-actions, ci, release, cargo, cross-platform]
dependency_graph:
  requires: []
  provides: [ci-workflow, release-workflow, prebuilt-binaries]
  affects: [.github/workflows/ci.yml, .github/workflows/release.yml]
tech_stack:
  added: [actions/checkout@v4, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2, actions/upload-artifact@v4, actions/download-artifact@v4, softprops/action-gh-release@v2]
  patterns: [matrix-build, tag-triggered-release, artifact-upload-download, cargo-build-release]
key_files:
  created:
    - .github/workflows/ci.yml
    - .github/workflows/release.yml
  modified: []
  deleted: []
decisions:
  - "CI matrix covers macos-latest + ubuntu-latest on stable Rust; clippy runs with -D warnings (zero-warnings policy)"
  - "Release matrix covers 3 targets: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu"
  - "Release assets are tar.gz tarballs named rn-dash-<target>.tar.gz, uploaded via softprops/action-gh-release@v2 with generate_release_notes: true"
  - "Permissions scoped to contents: write only (no write-all) per threat model T-10-02"
metrics:
  completed_date: "2026-04-05"
  tasks_completed: 2
  files_changed: 2
---

# Phase 10 Plan 01: CI + Release Workflow Summary

**One-liner:** GitHub Actions now runs `cargo build / clippy -D warnings / test` on every push across macOS+Linux, and pushing a `v*` tag triggers a matrix release build that publishes tar.gz binaries for Apple Silicon, Intel Mac, and Linux x86_64 to a GitHub Release.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create CI workflow (build + clippy + test, macOS + Linux) | 1fefecf | .github/workflows/ci.yml |
| 2 | Create release workflow (3-target matrix, GitHub Release on v* tags) | e623ab6 | .github/workflows/release.yml |

## What Was Built

**Task 1 — `.github/workflows/ci.yml`:**
- Triggers: `push` (all branches) + `pull_request` to `main`.
- Matrix: `{ os: [macos-latest, ubuntu-latest] }` on stable Rust.
- Steps: checkout → `dtolnay/rust-toolchain@stable` with clippy → `Swatinem/rust-cache@v2` → `cargo build` → `cargo clippy -- -D warnings` → `cargo test`.
- `CARGO_TERM_COLOR: always` set at workflow level for readable logs.

**Task 2 — `.github/workflows/release.yml`:**
- Trigger: `push` on tags matching `v*`.
- Permissions: `contents: write` (scoped, not write-all).
- `build` job matrix: `aarch64-apple-darwin` (macos-latest), `x86_64-apple-darwin` (macos-latest), `x86_64-unknown-linux-gnu` (ubuntu-latest).
- Per-target steps: checkout → toolchain with target → rust-cache keyed by target → `cargo build --release --target ${{ matrix.target }}` → `tar -czf rn-dash-<target>.tar.gz` of the release binary → upload artifact.
- `release` job: `needs: build`, downloads all artifacts with `merge-multiple: true`, publishes via `softprops/action-gh-release@v2` with `generate_release_notes: true` and `files: rn-dash-*.tar.gz`.

## Deviations from Plan

None of substance. The release pipeline was later extended (commit `02b7082` + follow-ups) with macOS codesigning and notarization — that work is additive to this plan's scope.

## Known Stubs

None.

## Threat Flags

- T-10-01 (tampering via tag push) — accepted; restricted by repo collaborator permissions.
- T-10-02 (elevation) — mitigated by scoping permissions to `contents: write` only.
- T-10-03 (action supply chain) — accepted; pinned major versions of widely-trusted actions.

## Self-Check: PASSED

- `.github/workflows/ci.yml` — FOUND (3 cargo commands present: build, clippy, test)
- `.github/workflows/release.yml` — FOUND (all 3 target triples present, softprops/action-gh-release referenced)
- Tag-triggered behavior validated in practice: v1.2.0 (`98500fb chore(release): v1.2.0`) produced a GitHub Release with binaries.
- commits 1fefecf, e623ab6 — FOUND
