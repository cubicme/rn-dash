---
phase: quick
plan: 260405-jqb
type: execute
wave: 1
depends_on: []
files_modified:
  - src/ui/modals.rs
  - src/ui/error_overlay.rs
  - src/ui/help_overlay.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "All modals render without panic or clipping in terminal heights as low as 10 rows"
    - "Modal content is fully visible with borders in small terminals"
    - "Help overlay fills available space in small terminals instead of being too tiny to read"
  artifacts:
    - path: "src/ui/modals.rs"
      provides: "Small-terminal-safe modal rendering"
    - path: "src/ui/error_overlay.rs"
      provides: "Small-terminal-safe error overlay"
    - path: "src/ui/help_overlay.rs"
      provides: "Small-terminal-safe help overlay"
  key_links:
    - from: "centered_rect"
      to: "each modal renderer"
      via: "minimum height enforcement"
---

<objective>
Fix prompt/modal/overlay rendering when the terminal window has a small height. Currently all overlays use percentage-based sizing via `centered_rect(area, pct_x, pct_y)` with no minimum height. On small terminals (e.g., 10-20 rows), this produces rects too small to hold the modal content, causing clipping or invisible content.

Purpose: Make the UI gracefully handle small terminal sizes for all overlay types.
Output: Updated modals.rs, error_overlay.rs, help_overlay.rs with minimum-height-aware centering.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/ui/modals.rs
@src/ui/error_overlay.rs
@src/ui/help_overlay.rs
@src/ui/mod.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add minimum-height-aware centered_rect to all overlay modules</name>
  <files>src/ui/modals.rs, src/ui/error_overlay.rs, src/ui/help_overlay.rs</files>
  <action>
Each of the three overlay files has its own `centered_rect(area, percent_x, percent_y) -> Rect` function. Replace each with a version that accepts minimum dimensions and clamps correctly.

**New signature** (in each file, keeping copies independent per existing pattern):
```rust
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16, min_w: u16, min_h: u16) -> Rect
```

**Implementation logic:**
1. Compute desired width: `(area.width * percent_x / 100)` clamped to `min_w..=area.width`
2. Compute desired height: `(area.height * percent_y / 100)` clamped to `min_h..=area.height`
3. Center the resulting rect within `area`:
   - `x = area.x + (area.width.saturating_sub(w)) / 2`
   - `y = area.y + (area.height.saturating_sub(h)) / 2`
4. Return `Rect::new(x, y, w, h)`

This replaces the Layout-based centering with direct arithmetic, which is simpler and supports min clamping.

**Update each call site with appropriate minimum heights** (content lines + 2 for borders):

In `modals.rs`:
- `render_confirm_modal`: `centered_rect(f.area(), 50, 25, 40, 5)` — 3 content lines + 2 borders
- `render_text_input_modal`: `centered_rect(f.area(), 60, 25, 40, 6)` — 4 content lines + 2 borders
- `render_device_picker_modal`: `centered_rect(f.area(), 60, 60, 40, 7)` — needs at least a few list items visible + borders
- `render_clean_modal`: `centered_rect(f.area(), 50, 60, 40, 10)` — 8 content lines + 2 borders
- `render_sync_prompt`: `centered_rect(f.area(), 60, 40, 40, 8)` — 6 content lines + 2 borders
- `render_external_metro_modal`: `centered_rect(f.area(), 60, 30, 40, 9)` — 7 content lines + 2 borders

In `error_overlay.rs`:
- `render_error`: `centered_rect(f.area(), 50, 30, 40, 5)` — 3 content lines + 2 borders

In `help_overlay.rs`:
- `render_help`: `centered_rect(f.area(), 70, 85, 40, 10)` — help table needs some minimum to be useful; on tiny terminals it will fill most of the screen

Do NOT extract centered_rect into a shared module — the existing pattern deliberately keeps copies independent per module to avoid cross-widget coupling (see comment in error_overlay.rs line 44).
  </action>
  <verify>
    <automated>cd /Users/cubicme/aljazeera/dashboard && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>All three overlay files use minimum-height-aware centered_rect. cargo check passes with no errors. Modals will now fill available space (up to area bounds) instead of shrinking to unusable sizes on small terminals.</done>
</task>

</tasks>

<verification>
- `cargo check` passes with no errors or warnings related to the changed files
- Manual test: resize terminal to ~10-15 rows height, open a modal (e.g., press `L` for label input, or trigger a confirm dialog) — modal should be visible and usable, filling most of the small terminal rather than being clipped
</verification>

<success_criteria>
- All modals render correctly at terminal heights of 10+ rows
- No panics or layout overflows on small terminals
- On normal-sized terminals, modal appearance is unchanged (percentages still drive sizing when above minimums)
</success_criteria>

<output>
After completion, create `.planning/quick/260405-jqb-fix-prompt-rendering-in-small-terminal-h/260405-jqb-SUMMARY.md`
</output>
