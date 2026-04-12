---
quick_id: 260412-vl1
description: consolidate yarn clean commands into one with selection menu
date: 2026-04-12
commit: 85f0cc7
status: complete
---

# Summary

Regressed: the yarn palette had three separate clean entries (`a` android, `c` cocoapods, `n` node_modules). The `ModalState::CleanToggle` existed with full toggle UX but no key opened it.

## Changes

- `src/action.rs` — added `Action::OpenCleanMenu`.
- `src/app.rs`:
  - Yarn palette: replaced `a`/`c`/`n` individual clean entries with a single `c → OpenCleanMenu`.
  - `Action::OpenCleanMenu` clears the palette and opens `ModalState::CleanToggle` with default options.
  - `CleanConfirm`: reordered the dispatched command sequence to `pods → android → node_modules` (was `node_modules → pods → android`). Removing `node_modules` first breaks `react-native clean` scripts that read from it.
- `src/ui/footer.rs` — yarn palette hints collapsed to one `("c", "clean…")`.
- `src/ui/help_overlay.rs` — yarn section collapsed to one clean row.

## Verification

- `cargo check --all-targets` clean.
- `cargo test` — 26 passed.
- Manual path: `y → c` opens clean modal with `n/p/a/i` toggles and `x`/Enter confirm.

## Commit

- `85f0cc7` feat(quick-260412-vl1): consolidate yarn clean into single c-menu
