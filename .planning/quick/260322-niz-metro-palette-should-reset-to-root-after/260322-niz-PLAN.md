---
phase: quick
plan: 260322-niz
type: execute
wave: 1
depends_on: []
files_modified: [src/app.rs]
autonomous: true
requirements: [QUICK-FIX]

must_haves:
  truths:
    - "Metro palette dismisses after dispatching any metro command"
    - "Pressing m>s, m>x, m>r, m>j, m>R all return to normal key mode"
  artifacts:
    - path: "src/app.rs"
      provides: "palette_mode = None on metro action handlers"
      contains: "state.palette_mode = None"
  key_links:
    - from: "handle_key PaletteMode::Metro"
      to: "update() metro action handlers"
      via: "Action dispatch"
      pattern: "palette_mode = None"
---

<objective>
Fix metro palette staying open after dispatching a command. The metro palette (m>) dispatches MetroStart, MetroStop, MetroRestart, MetroSendDebugger, MetroSendReload but never clears palette_mode, leaving the user stuck in palette mode.

Purpose: UX bug — user presses m>s to start metro and the palette stays open, giving the impression nothing happened.
Output: palette_mode cleared on every metro action handler, matching how CommandRun clears it at line 754.
</objective>

<execution_context>
@~/.claude/get-shit-done/workflows/execute-plan.md
@~/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/app.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Clear palette_mode in metro action handlers</name>
  <files>src/app.rs</files>
  <action>
Add `state.palette_mode = None;` as the first line inside each of these five Action match arms in the update() function:

1. **Action::MetroStart** (line 525) — add `state.palette_mode = None;` before the `if state.metro.is_running()` check
2. **Action::MetroStop** (line 553) — add `state.palette_mode = None;` before `if let Some(mut handle)`
3. **Action::MetroRestart** (line 563) — add `state.palette_mode = None;` before `if state.metro.is_running()`
4. **Action::MetroSendDebugger** (line 572) — add `state.palette_mode = None;` before `if state.metro.is_running()`
5. **Action::MetroSendReload** (line 584) — add `state.palette_mode = None;` before `if state.metro.is_running()`

This matches the pattern used by Action::CommandRun at line 754: `state.palette_mode = None;`

Do NOT touch MetroStartConfirmed, MetroExited, or ExternalMetroDetected — those are internal actions not triggered from the palette.
  </action>
  <verify>
    <automated>cd /Users/cubicme/aljazeera/dashboard && cargo check --incremental 2>&1 | tail -5</automated>
  </verify>
  <done>All five metro palette commands (start/stop/restart/debugger/reload) clear palette_mode immediately on dispatch. Palette dismisses after any metro action.</done>
</task>

</tasks>

<verification>
cargo check --incremental passes with no errors.
</verification>

<success_criteria>
Metro palette (m>) dismisses immediately after any command key is pressed, returning user to normal key handling mode.
</success_criteria>

<output>
After completion, create `.planning/quick/260322-niz-metro-palette-should-reset-to-root-after/260322-niz-SUMMARY.md`
</output>
