//! UI layer — ratatui widgets, rendering, layout.
//! Imports: domain types and ratatui ONLY. Never imports infra directly.
//!
//! view() is the single render entry point called from app::run().
//! It accepts &mut AppState because render_stateful_widget requires &mut TableState.

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
use crate::app::{AppState, FocusedPanel};

/// Root render function. Called on every draw cycle from app::run().
/// Layout: top (metro + output) | bottom (worktree table) | footer
/// Fullscreen mode: single panel + footer, activated by ToggleFullscreen.
/// Overlays: rendered last so they layer on top of all base content.
pub fn view(f: &mut Frame, state: &mut AppState) {
    let area = f.area();

    // Fullscreen mode — render only the fullscreened panel + footer
    if let Some(panel) = state.fullscreen_panel {
        let [main_area, footer_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .areas(area);

        match panel {
            FocusedPanel::MetroPane => {
                panels::render_metro_pane(f, main_area, state);
            }
            FocusedPanel::CommandOutput => {
                panels::render_command_output(f, main_area, state);
            }
            _ => {} // WorktreeTable cannot be fullscreened
        }

        footer::render_footer(f, footer_area, state);

        // Overlays on top
        if state.show_help {
            help_overlay::render_help(f);
        }
        if let Some(ref error) = state.error_state {
            error_overlay::render_error(f, error);
        }
        if let Some(ref modal) = state.modal {
            modals::render_modal(f, modal);
        }
        return;
    }

    // Normal layout: top (metro + output) | bottom (worktree table) | footer
    let table_height = (state.worktrees.len() as u16 + 3).max(5); // rows + borders + header
    let [top_area, table_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),              // metro + output (flexible)
            Constraint::Length(table_height), // worktree table (fixed)
            Constraint::Length(1),            // footer
        ])
        .areas(area);

    // Top section: metro (left) | command output (right)
    // When log panel visible: metro (left-top) | log (left-bottom) | output (right)
    if state.log_panel_visible {
        let [left_top, right_top] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(top_area);

        let [metro_area, log_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .areas(left_top);

        panels::render_metro_pane(f, metro_area, state);
        panels::render_log_panel(f, log_area, state);
        panels::render_command_output(f, right_top, state);
    } else {
        let [metro_area, output_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .areas(top_area);

        panels::render_metro_pane(f, metro_area, state);
        panels::render_command_output(f, output_area, state);
    }

    // Bottom section: worktree table
    panels::render_worktree_table(f, table_area, state);

    // Footer
    footer::render_footer(f, footer_area, state);

    // Overlays last
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
