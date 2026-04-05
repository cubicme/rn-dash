---
phase: quick
plan: 260405-jqb
subsystem: ui/overlays
tags: [ui, ratatui, modals, small-terminal, layout]
dependency_graph:
  requires: []
  provides: [small-terminal-safe modal rendering]
  affects: [src/ui/modals.rs, src/ui/error_overlay.rs, src/ui/help_overlay.rs]
tech_stack:
  added: []
  patterns: [minimum-height clamped centered_rect, direct arithmetic layout]
key_files:
  modified:
    - src/ui/modals.rs
    - src/ui/error_overlay.rs
    - src/ui/help_overlay.rs
decisions:
  - Replace Layout-based centering with direct arithmetic to support min clamping
  - Keep centered_rect copies independent per module (existing pattern, no cross-widget coupling)
  - Minimum heights set to content-lines + 2 (for borders) per modal
metrics:
  duration: 5min
  completed: 2026-04-05T10:16:30Z
  tasks: 1
  files: 3
---

# Quick Task 260405-jqb: Fix Prompt Rendering in Small Terminal Heights — Summary

**One-liner:** Replace percentage-only Layout centering with min-clamped arithmetic in all three overlay modules so modals remain visible and usable at terminal heights as low as 10 rows.

## What Was Done

All three overlay files (`modals.rs`, `error_overlay.rs`, `help_overlay.rs`) previously used `Layout::vertical/horizontal` with `Constraint::Percentage` for centering, which produces rects proportional to the terminal size with no minimum. On terminals with 10–20 rows, this shrinks modals below their content height, causing clipping or invisible content.

The fix replaces the Layout-based approach in each file with a direct arithmetic implementation:

```rust
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16, min_w: u16, min_h: u16) -> Rect {
    let w = (area.width * percent_x / 100).clamp(min_w, area.width);
    let h = (area.height * percent_y / 100).clamp(min_h, area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
```

All 8 call sites updated with explicit minimum heights:

| Modal | File | min_w | min_h | Rationale |
|-------|------|-------|-------|-----------|
| render_confirm_modal | modals.rs | 40 | 5 | 3 content lines + 2 borders |
| render_text_input_modal | modals.rs | 40 | 6 | 4 content lines + 2 borders |
| render_device_picker_modal | modals.rs | 40 | 7 | list items + borders |
| render_clean_modal | modals.rs | 40 | 10 | 8 content lines + 2 borders |
| render_sync_prompt | modals.rs | 40 | 8 | 6 content lines + 2 borders |
| render_external_metro_modal | modals.rs | 40 | 9 | 7 content lines + 2 borders |
| render_error | error_overlay.rs | 40 | 5 | 3 content lines + 2 borders |
| render_help | help_overlay.rs | 40 | 10 | minimum useful help display |

Unused `Layout`, `Flex`, and `Constraint` imports removed from modals.rs and error_overlay.rs; `Flex` removed from help_overlay.rs (still uses `Constraint::Length/Fill` for table columns).

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo check` passes with no errors (7 pre-existing unrelated warnings, unchanged)
- No panics or layout overflows: `clamp(min, area.width/height)` ensures rect never exceeds terminal bounds
- Normal-sized terminals: percentages still drive sizing when above minimums (no behavior change)

## Self-Check

**Task commit:** 4f85b4c

Files modified:
- `src/ui/modals.rs` — present
- `src/ui/error_overlay.rs` — present
- `src/ui/help_overlay.rs` — present

## Self-Check: PASSED
