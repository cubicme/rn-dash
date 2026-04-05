# Phase 06: Final UX Polish - Research

**Researched:** 2026-03-12
**Domain:** Ratatui TUI polish — metro log filtering, multiplexer tab commands, visual indicators, prefix ordering, modal input, border styling
**Confidence:** HIGH

## Summary

Phase 06 is six discrete, low-risk polish items all operating on well-established code paths. The codebase is mature: TEA pattern, `Multiplexer` trait, `preferred_prefix()` naming, double-border `Block::bordered()` pattern, and `ModalState::TextInput` all exist. No new crates are needed.

The six items break into three categories:
1. **Filter/display** — metro log noise suppression (`stream_metro_logs`), metro running green-play icon (already partially implemented as `●` circle), double border on a new title bar widget.
2. **New action + modal** — open-shell tab (new `Action::OpenShellTab`, uses `Multiplexer::new_window` with `$SHELL`), optional Claude tab name (promote `OpenClaudeCode` to trigger `ModalState::TextInput` first).
3. **Naming fix** — prefix ordering: `{preferred_prefix}-{type}` instead of `{type}-{prefix}`.

All six can be planned as independent tasks with no inter-dependencies except that the prefix-ordering fix affects both `OpenClaudeCode` and the new `OpenShellTab`.

**Primary recommendation:** Implement each item as one plan in order: (1) metro log filter, (2) open-shell tab + key binding, (3) metro running green-play icon, (4) prefix ordering fix, (5) optional claude tab name modal, (6) double border on title bar.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Metro Log Filtering: Remove watchman warnings and other noisy/non-useful output from metro log before display
- Multiplexer Tab from Worktree: A command to open a tmux/zellij tab with a general-purpose shell (not Claude Code) at the selected worktree
- Metro Running Indicator: Green play icon at the START of the worktree row when metro is running (primary visual indicator)
- Prefix Ordering Fix: `preferred_prefix()` comes FIRST, then tab-type suffix — e.g. `e2e-claude` not `claude-e2e`
- Optional Name for Claude Tab: When opening Claude Code tab, prompt for optional suffix; default "claude" if Enter pressed without typing; use typed text as suffix
- Double Border on Title: Show double border on the title bar/header block (currently only on panes)

### Claude's Discretion
- Metro log filtering patterns (which specific log lines to suppress)
- Key binding choice for the new "open tab" command
- Input modal UX for the optional claude tab name

### Deferred Ideas (OUT OF SCOPE)
None — all items are in scope for this phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| UX-06-01 | Metro log filtering — suppress watchman warnings and other noise before storing to `metro_logs` | Filter in `stream_metro_logs()` before `tx.send(Action::MetroLogLine(l))` |
| UX-06-02 | Open general-purpose shell tab from worktree via multiplexer | New `Action::OpenShellTab`, re-uses `Multiplexer::new_window` with `$SHELL` command, name = `{prefix}-shell` |
| UX-06-03 | Green play icon at START of worktree row when metro is running | Already shows `●` circle; change to play icon `▶` (U+25B6) prepended before all other spans |
| UX-06-04 | Prefix ordering fix: `{preferred_prefix}-{type}` not `{type}-{prefix}` | One-line fix in `Action::OpenClaudeCode` handler: `format!("{}-claude", wt.preferred_prefix())` |
| UX-06-05 | Optional name for Claude tab — TextInput modal before opening | Promote `OpenClaudeCode` to open `ModalState::TextInput` first; `ModalInputSubmit` closes modal and dispatches real open |
| UX-06-06 | Double border on the main title/header bar | New `render_title_bar()` in `panels.rs`, `Block::bordered().border_type(BorderType::Double)`, added to layout in `ui/mod.rs` |
</phase_requirements>

## Standard Stack

### Core (all already in Cargo.toml — no new deps)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.29/0.30 | All rendering, Block, BorderType::Double | Already in use |
| tokio | full | async metro streaming, spawn_blocking | Already in use |
| std::env | stdlib | `$SHELL` for shell tab command | stdlib |

No new crates required for any of the six items.

## Architecture Patterns

### Metro Log Filtering — WHERE to filter

The filtering point is `stream_metro_logs()` in `src/app.rs` (lines ~1743–1766). Currently every line from stdout/stderr is sent as `Action::MetroLogLine(l)` unconditionally. The filter should be applied here before `tx.send()`, NOT in the `Action::MetroLogLine` handler in `update()`.

**Rationale:** Filtering at source keeps `metro_logs` VecDeque clean. The handler is the right layer for routing; the streaming task is the right layer for suppression. This is the Ousterhout "deep module" principle — the complexity is hidden in the streaming function, not scattered.

```rust
// Source: src/app.rs stream_metro_logs (current code to modify)
Ok(Some(l)) => {
    if !should_suppress_metro_line(&l) {
        let _ = tx.send(Action::MetroLogLine(l));
    }
}
```

```rust
/// Returns true for lines that should be dropped from the metro log.
/// Claude's discretion on exact patterns — canonical noisy patterns:
fn should_suppress_metro_line(line: &str) -> bool {
    // Watchman warnings are the primary noise source
    line.contains("watchman") && line.contains("warning") ||
    line.contains("watchman: ") ||
    // Metro startup boilerplate (repetitive on every restart)
    line.starts_with("                    ") ||   // deeply indented blank decoration lines
    // Add patterns discovered during UAT
    false
}
```

**Common watchman noise patterns** (from Metro community knowledge, MEDIUM confidence — verify against actual log):
- `warn  watchman warning: …` — watchman version mismatch warnings
- Lines containing `watchman` that are not actual file-change events
- Empty lines or lines of only whitespace/box-drawing characters (metro startup banner decoration)

The function lives in `src/app.rs` (near `stream_metro_logs`) or can be extracted to a `src/infra/metro_filter.rs` module if more than 5 patterns are needed. Given the small scope, keeping it in `app.rs` as a private function is preferred (Ousterhout: avoid unnecessary file proliferation).

### Open Shell Tab — New Action + Binding

The `OpenClaudeCode` action is the exact template. The new `OpenShellTab` action follows the identical pattern:

```rust
// action.rs addition
Action::OpenShellTab,  // 'T' on worktree — open shell in new tmux/zellij tab
```

```rust
// app.rs update() handler
Action::OpenShellTab => {
    // Same guard: multiplexer must exist
    let wt = ...; // selected worktree (same as OpenClaudeCode)
    let path = wt.path.clone();
    let name = format!("{}-shell", wt.preferred_prefix());  // prefix-ordering already fixed
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    tokio::task::spawn_blocking(move || {
        if let Some(mux) = crate::infra::multiplexer::detect_multiplexer() {
            if let Err(e) = mux.new_window(&path, &name, &shell) {
                tracing::warn!("multiplexer new_window (shell) failed: {e}");
            }
        }
    });
}
```

**Key binding choice (Claude's discretion):** `T` (shift-t) on `WorktreeTable`. Rationale: `C` is Claude, `T` is Tab/Terminal. Uppercase avoids collision with lowercase vim navigation. Existing uppercase bindings: `L` (label), `C` (Claude), `R` (refresh), `J` (metro debugger), `R` (metro reload) — `T` is free.

Footer hint addition in `WorktreeTable` panel hints: `("T", "shell tab")`.

### Metro Running Indicator — Green Play Icon

Currently `panels.rs` renders `●` (U+25CF, BULLET) as the metro running indicator. The CONTEXT.md says "green play icon." The difference is cosmetic: replace `●` with `▶` (U+25B6, BLACK RIGHT-POINTING TRIANGLE) or keep `●` if user means the green dot is the play indicator. Given CONTEXT.md says "green play icon at start of worktree row," interpret as changing the unicode char to a right-pointing play triangle.

**Current code** (`src/ui/panels.rs` lines ~72–75):
```rust
if wt.metro_status == WorktreeMetroStatus::Running {
    icon_spans.push(Span::styled("●", Style::default().fg(Color::Green)));
    icon_spans.push(Span::raw(" "));
}
```

**Change to:**
```rust
if wt.metro_status == WorktreeMetroStatus::Running {
    icon_spans.push(Span::styled("▶", Style::default().fg(Color::Green)));
    icon_spans.push(Span::raw(" "));
}
```

The icon already appears first (before Y/P spans). The column width `Constraint::Length(8)` for the icons column already accommodates this. No layout change needed.

Also update the footer legend (`src/ui/footer.rs`) which currently shows `●=metro` — change to `▶=metro`.

### Prefix Ordering Fix — One-Line Change

**Current** (`src/app.rs` line ~1299):
```rust
let name = format!("claude-{}", wt.preferred_prefix());
```

**Fixed:**
```rust
let name = format!("{}-claude", wt.preferred_prefix());
```

This fix also applies to `OpenShellTab` (new code should use the correct order from the start).

**Verify:** `preferred_prefix()` is defined in `src/domain/worktree.rs` and returns label > jira_key > branch > dir-name. The format `{prefix}-claude` (e.g. `e2e-claude`, `UMP-1234-claude`) is the correct ordering per CONTEXT.md.

### Optional Claude Tab Name — Modal-First Flow

Currently `OpenClaudeCode` builds the name immediately and calls `new_window`. The new flow:

1. `Char('C')` → `Action::OpenClaudeCode`
2. `OpenClaudeCode` in `update()` → open `ModalState::TextInput { prompt: "Tab name (Enter for 'claude'):".into(), buffer: String::new(), pending_spec: None }` — **no** `new_window` call yet
3. `ModalInputSubmit` in `update()` — detect this is a "pending claude open" (needs new `pending_claude_worktree: Option<WorktreeId>` in `AppState`, or re-use the pending_label_branch pattern)
4. On submit: take buffer, if empty → suffix = "claude", else suffix = buffer; build name = `format!("{}-{}", prefix, suffix)`; spawn `new_window`

**State field approach:** Add `pending_claude_open: Option<crate::domain::worktree::WorktreeId>` to `AppState`. When `ModalInputSubmit` fires and `pending_claude_open` is `Some`, treat as claude tab name submit.

**Pattern reference:** This is the same pattern as `pending_label_branch: Option<String>` (added in Phase 03-03 decision). Existing `ModalInputSubmit` handler already has a branch to handle pending_label_branch — add a new `else if` arm.

```rust
// In ModalInputSubmit handler (app.rs update())
} else if let Some(wt_id) = state.pending_claude_open.take() {
    let suffix = if buffer.trim().is_empty() { "claude".to_string() } else { buffer };
    let wt = state.worktrees.iter().find(|wt| wt.id == wt_id).cloned();
    if let Some(wt) = wt {
        let name = format!("{}-{}", wt.preferred_prefix(), suffix);
        let path = wt.path.clone();
        let flags = state.claude_flags.clone();
        let command = if flags.is_empty() { "claude".to_string() } else { format!("claude {}", flags) };
        tokio::task::spawn_blocking(move || {
            if let Some(mux) = detect_multiplexer() {
                let _ = mux.new_window(&path, &name, &command);
            }
        });
    }
}
```

### Double Border on Title Bar — New Widget

There is currently NO title bar in the layout. The `ui/mod.rs` layout is:
- `top_area` (metro + output) | `table_area` (worktrees) | `footer_area` (1 line)

The "double border on title" means adding a new title bar row at the top of the layout with `BorderType::Double`. This is a new `render_title_bar()` function.

**Layout change in `ui/mod.rs`:** Add `title_area` (1–3 lines) before `top_area`.

```rust
// Normal layout with title bar
let title_height: u16 = 3; // borders + 1 line content
let [title_area, top_area, table_area, footer_area] = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(title_height),
        Constraint::Min(8),
        Constraint::Length(table_height),
        Constraint::Length(1),
    ])
    .areas(area);

panels::render_title_bar(f, title_area, state);
```

**Title bar widget:**
```rust
pub fn render_title_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let title = " UMP Dashboard ";
    let block = Block::bordered()
        .border_type(BorderType::Double)
        .title(title)
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(block, area);
}
```

**Note:** The fullscreen layout path in `ui/mod.rs` should NOT render the title bar (the fullscreen path renders only one panel + footer — adding title bar there wastes vertical space). Only the normal layout path gets the title bar.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Metro log regex filtering | Custom regex engine | Simple `str::contains` / `str::starts_with` checks | Metro noise patterns are stable, simple string matching is sufficient and zero-cost |
| Shell detection | Enumerate known shells | `std::env::var("SHELL")` | Already available in all Unix environments |
| Modal input for tab name | New ModalState variant | Re-use `ModalState::TextInput` + `pending_claude_open` flag | Already tested, same UX as label input |

## Common Pitfalls

### Pitfall 1: Filtering Too Aggressively
**What goes wrong:** Patterns that are too broad suppress useful metro output (e.g., suppressing all lines containing "warn" removes legitimate build warnings).
**Why it happens:** Over-eager filtering during initial implementation.
**How to avoid:** Filter patterns should be specific: match full prefix (e.g., `"warn  watchman"`) not just substrings. Start conservative (only watchman), expand based on UAT.
**Warning signs:** Metro pane is empty even when metro is actively building.

### Pitfall 2: Prefix Ordering Fix Missed in New Code
**What goes wrong:** `OpenShellTab` (new code) accidentally uses the old `format!("{type}-{prefix}")` order.
**How to avoid:** Write `OpenShellTab` handler AFTER the prefix fix is applied to `OpenClaudeCode`, or fix both in the same plan.

### Pitfall 3: `pending_claude_open` Leaks on Modal Cancel
**What goes wrong:** User opens the "Tab name" modal, presses Esc — `pending_claude_open` stays `Some` in `AppState`, causes the next `ModalInputSubmit` (from an unrelated modal) to open a Claude tab unexpectedly.
**How to avoid:** `ModalCancel` arm in `update()` MUST clear `pending_claude_open`:
```rust
Action::ModalCancel => {
    state.modal = None;
    state.pending_claude_open = None; // ADD THIS
    state.palette_mode = None;
    // ...
}
```

### Pitfall 4: Title Bar Shrinks Metro Pane Too Much
**What goes wrong:** Adding 3 lines for the title bar reduces vertical space for the metro/output panes when the terminal is short.
**How to avoid:** Title bar height should be exactly 3 (2 border lines + 1 content line). Consider making it `Constraint::Length(3)` and verifying the minimum terminal height is acceptable. If title bar is too tall, reduce to a borderless 1-line `Paragraph` with double-underline style instead.

### Pitfall 5: Fullscreen Layout Also Gets Title Bar (Wasted Space)
**What goes wrong:** Title bar added to fullscreen layout path in `ui/mod.rs`.
**How to avoid:** The early-return fullscreen branch in `view()` must NOT call `render_title_bar()`.

### Pitfall 6: `ModalState::TextInput` Prompt Truncation
**What goes wrong:** Long prompt text like "Tab name (Enter for 'claude'):" is truncated in narrow terminals, making the modal confusing.
**How to avoid:** Keep prompt short: `"Claude tab suffix:"` with footer hint `"(Enter = 'claude')"`.

## Code Examples

Verified patterns from existing codebase:

### ModalState::TextInput opening (from Phase 03-03 decision, pending_label_branch pattern)
```rust
// From app.rs StartSetLabel handler
Action::StartSetLabel => {
    state.pending_label_branch = Some(wt.branch.clone());
    state.modal = Some(ModalState::TextInput {
        prompt: "Label:".to_string(),
        buffer: String::new(),
    });
}
```

### ModalInputSubmit pending-branch dispatch (from app.rs)
```rust
// From app.rs ModalInputSubmit handler (pending_label_branch branch)
if let Some(branch) = state.pending_label_branch.take() {
    update(state, Action::SetLabel { branch, label: buffer }, metro_tx, handle_tx);
    return;
}
```

### Multiplexer new_window spawn_blocking pattern (from app.rs OpenClaudeCode)
```rust
tokio::task::spawn_blocking(move || {
    if let Some(mux) = crate::infra::multiplexer::detect_multiplexer() {
        if let Err(e) = mux.new_window(&path, &name, &command) {
            tracing::warn!("multiplexer new_window failed: {e}");
        }
    }
});
```

### Block::bordered() with BorderType::Double (from panels.rs — canonical pattern)
```rust
let block = Block::bordered()
    .border_type(BorderType::Double)
    .title(" Title ")
    .title_style(Style::default().fg(Color::White))
    .border_style(border_style);
```

### Layout with 4 rows (current 3-row pattern + new title slot)
```rust
let [title_area, top_area, table_area, footer_area] = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),             // title bar (new)
        Constraint::Min(8),                // metro + output
        Constraint::Length(table_height),  // worktree table
        Constraint::Length(1),             // footer
    ])
    .areas(area);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `format!("claude-{}", prefix)` | `format!("{}-claude", prefix)` | Phase 06 | Consistent `prefix-type` naming |
| `●` bullet as metro indicator | `▶` play icon as metro indicator | Phase 06 | Clearer "running" semantics |
| No title bar | `Block::bordered().border_type(Double)` title bar | Phase 06 | Visual framing for the app |
| Claude tab: immediate open | Claude tab: TextInput modal first | Phase 06 | User can name the tab |

## Open Questions

1. **Exact watchman log patterns**
   - What we know: Watchman warnings are the primary noise (from CONTEXT.md). Watchman outputs lines like `warn  watchman warning: …` and version-mismatch messages.
   - What's unclear: The exact prefix/format of watchman lines in THIS project's metro output (depends on metro version).
   - Recommendation: Start with `line.contains("watchman")` as a broad initial filter. Refine during UAT by reading `~/.config/ump-dash/logs/ump-dash.log` where raw lines are logged before filtering.

2. **Title bar content beyond the border**
   - What we know: CONTEXT.md says "double border on the title bar/header itself."
   - What's unclear: Should the title bar show any content (e.g., app name, version, current time)?
   - Recommendation: Minimal content: `" UMP Dashboard "` as the block title. No clock or version — keep it clean.

3. **Key binding for `OpenShellTab`**
   - What we know: `T` (shift-t) is free on the WorktreeTable panel.
   - What's unclear: User preference.
   - Recommendation: Use `T` (shift-t) — mnemonic "Terminal". Update footer hint and help overlay.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | Cargo.toml (standard) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo build` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UX-06-01 | `should_suppress_metro_line()` suppresses watchman lines | unit | `cargo test suppress` | ❌ Wave 0 |
| UX-06-02 | OpenShellTab uses `$SHELL` and `{prefix}-shell` name | manual-only | visual verification | N/A |
| UX-06-03 | `▶` icon shown only when metro running | manual-only | visual verification | N/A |
| UX-06-04 | Prefix ordering: `{prefix}-claude` not `claude-{prefix}` | unit | `cargo test prefix_order` | ❌ Wave 0 |
| UX-06-05 | Modal cancels clear `pending_claude_open` | unit | `cargo test pending_claude_cancel` | ❌ Wave 0 |
| UX-06-06 | Title bar rendered with BorderType::Double | manual-only | visual verification | N/A |

### Sampling Rate
- **Per task commit:** `cargo build` (compilation is sufficient for structural correctness)
- **Per wave merge:** `cargo test`
- **Phase gate:** `cargo test && cargo build --release` green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/metro_filter.rs` — unit tests for `should_suppress_metro_line()` covering watchman, empty lines, non-suppressed lines
- [ ] `tests/prefix_ordering.rs` — unit test for `preferred_prefix()` + format string yielding `{prefix}-claude`
- [ ] `tests/pending_claude.rs` — unit test that `ModalCancel` clears `pending_claude_open`

## Sources

### Primary (HIGH confidence)
- `src/app.rs` — `stream_metro_logs`, `Action::OpenClaudeCode` handler, `pending_label_branch` pattern, `ModalInputSubmit` handler
- `src/ui/panels.rs` — `render_worktree_table` icon spans, `Block::bordered().border_type(BorderType::Double)` pattern
- `src/ui/mod.rs` — layout structure (3-row: top/table/footer), fullscreen branch
- `src/infra/multiplexer.rs` — `Multiplexer::new_window` trait, `detect_multiplexer()`, `TmuxAdapter`, `ZellijAdapter`
- `src/domain/worktree.rs` — `preferred_prefix()` implementation
- `src/action.rs` — full `Action` enum, existing key binding pattern
- `src/ui/footer.rs` — `key_hints_for()`, footer legend

### Secondary (MEDIUM confidence)
- `.planning/STATE.md` decisions log — confirms prefix bug exists as `format!("claude-{}", wt.preferred_prefix())` in line 1299 of app.rs, confirmed by code read

### Tertiary (LOW confidence)
- Watchman noise patterns: based on metro community knowledge; actual patterns must be confirmed against this project's log output

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps, all patterns already in codebase
- Architecture: HIGH — six items mapped to exact code locations
- Pitfalls: HIGH — identified from code inspection (pending field leak, fullscreen layout, filter over-eagerness)

**Research date:** 2026-03-12
**Valid until:** 2026-04-12 (stable domain)
