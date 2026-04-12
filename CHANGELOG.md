# Changelog

All notable changes to this project will be documented in this file.

## [1.2.0] - 2026-04-12

### New
- **Stale-dependency guard on Enter** — pressing Enter on a worktree now checks for stale yarn/pods and prompts to sync before Metro starts.
- **Stale-dependency guard on iOS/Android run** — sync prompt now triggers on pods-only staleness (previously only yarn staleness was checked), and only runs the syncs actually needed (`yarn install` and/or `pod-install`) instead of both unconditionally.
- **`auto_sync` config flag** — set `auto_sync = true` in `~/.config/rn-dash/config.toml` to bypass the sync confirmation modals; syncs proceed automatically. Default off.

### Fixed
- Metro no longer boots against stale dependencies — `yarn install` now runs before Metro when deps are stale, not after.
- Metro process group is killed cleanly on stop (previously only yarn's PID was killed, leaving orphan node processes).
- Metro start no longer hangs on macOS when `lsof` stalls scanning mounts — external-Metro detection now fast-paths via `TcpListener::bind` and wraps the slow path in a 2s timeout.
- Metro's stale-dep check now only considers yarn, not pods (Metro itself doesn't need pods — only iOS builds do, and those have their own check).
- iOS device picker (`i>e`) correctly lists simulators instead of physical devices.
- Race condition when performing worktree operations during an in-flight command.

### Changed
- **Yarn palette `clean` is now a single entry** — `y>c` opens a toggle modal where you pick any combination of pods / android / node_modules (and optionally sync after). Replaces the three separate entries (`y>a`, `y>c`, `y>n`). Clean order is now pods → android → node_modules, so `react-native clean` runs before node_modules is removed.

### Docs
- README: corrected clone URL to match the actual repo.
- README: added macOS Gatekeeper workaround for running unsigned development builds.

