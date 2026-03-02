use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::{app::{AppState, FocusedPanel}, ui::theme};

/// Renders the worktree list panel (left column). Phase 3 will populate with real worktrees.
pub fn render_worktree_list(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::WorktreeList {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };
    let block = Block::default()
        .title(" Worktrees ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let placeholder = Paragraph::new("[ worktree list — Phase 3 ]").block(block);
    f.render_widget(placeholder, area);
}

/// Renders the metro control pane (top-right). Phase 2 will populate with real metro state.
pub fn render_metro_pane(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::MetroPane {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };
    let block = Block::default()
        .title(" Metro ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let placeholder = Paragraph::new("[ metro control — Phase 2 ]").block(block);
    f.render_widget(placeholder, area);
}

/// Renders the command output pane (bottom-right). Phase 3 will stream real command output.
pub fn render_command_output(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::CommandOutput {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };
    let block = Block::default()
        .title(" Output ")
        .borders(Borders::ALL)
        .border_style(border_style);
    let placeholder = Paragraph::new("[ command output — Phase 3 ]").block(block);
    f.render_widget(placeholder, area);
}
