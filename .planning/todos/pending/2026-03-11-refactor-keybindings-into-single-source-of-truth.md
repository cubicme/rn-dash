---
created: 2026-03-11T13:25:46.318Z
title: Refactor keybindings into single source of truth
area: ui
files:
  - src/app.rs:361-424
  - src/ui/footer.rs:134-168
---

## Problem

Keybindings are defined in two separate places that can drift apart:
- `handle_key()` in `src/app.rs` (lines 361-424) defines what each key does per pane/mode
- `key_hints_for()` in `src/ui/footer.rs` (lines 134-168) defines what hints to show in the footer bar

When a keybinding is added or changed in one place, the other can be missed. This already caused confusion when `J` (debugger) and `R` (reload) were available but not shown in the footer because metro wasn't detected as running.

## Solution

Define a `KeyBinding` struct (`key`, `action`, `hint_label`, `condition`) per pane/mode. Both `handle_key()` and `key_hints_for()` derive from the same `Vec<KeyBinding>` source. This ensures the footer always reflects the actual available keys.
