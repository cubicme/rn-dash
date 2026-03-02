//! UI layer — ratatui widgets, rendering, layout.
//! Imports: domain types and ratatui ONLY. Never imports infra directly.
//!
//! view() is the single render entry point called from app::run().
//! It accepts &mut AppState because render_stateful_widget requires &mut ListState.

pub mod footer;
pub mod help_overlay;
pub mod error_overlay;
pub mod modals;
pub mod panels;
pub mod theme;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use crate::app::AppState;

/// Root render function. Called on every draw cycle from app::run().
/// Layout: left column (worktree list) | right column (metro pane / log panel / command output)
/// Footer: always rendered at bottom.
/// Overlays: rendered last so they layer on top of all base content.
pub fn view(f: &mut Frame, state: &mut AppState) {
    let area = f.area();

    // Vertical split: main content area + footer (3 lines)
    let [main_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .areas(area);

    // Horizontal split: left 40% worktree list | right 60% metro + output
    let [left_area, right_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .areas(main_area);

    // Render worktree list (always in the left column)
    panels::render_worktree_list(f, left_area, state);

    // Right column split depends on log panel visibility:
    // - Log hidden: metro (40%) | output (60%)
    // - Log visible: metro (25%) | log (40%) | output (35%)
    if state.log_panel_visible {
        let [metro_area, log_area, output_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(40),
                Constraint::Percentage(35),
            ])
            .areas(right_area);

        panels::render_metro_pane(f, metro_area, state);
        panels::render_log_panel(f, log_area, state);
        panels::render_command_output(f, output_area, state);
    } else {
        let [metro_area, output_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .areas(right_area);

        panels::render_metro_pane(f, metro_area, state);
        panels::render_command_output(f, output_area, state);
    }

    // Render footer (always visible)
    footer::render_footer(f, footer_area, state);

    // Render overlays last — they draw on top of all panels
    if state.show_help {
        help_overlay::render_help(f);
    }
    if let Some(ref error) = state.error_state {
        error_overlay::render_error(f, error);
    }
    if let Some(ref modal) = state.modal {
        modals::render_modal(f, modal);
    }
}
