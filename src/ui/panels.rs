use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use crate::{app::{AppState, FocusedPanel}, domain::metro::MetroStatus, ui::theme};

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

/// Renders the metro control pane (top-right) with real status indicator.
pub fn render_metro_pane(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::MetroPane {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    // Status indicator text and color
    let (status_text, status_style) = match &state.metro.status {
        MetroStatus::Running { pid, worktree_id } =>
            (format!(" RUNNING  pid={}  [{}]", pid, worktree_id),
             Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        MetroStatus::Stopped =>
            (" STOPPED ".to_string(),
             Style::default().fg(Color::DarkGray)),
        MetroStatus::Starting =>
            (" STARTING... ".to_string(),
             Style::default().fg(Color::Yellow)),
        MetroStatus::Stopping =>
            (" STOPPING... ".to_string(),
             Style::default().fg(Color::Yellow)),
    };

    let block = Block::default()
        .title(" Metro ")
        .borders(Borders::ALL)
        .border_style(border_style);

    // Inner area for status content
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render status line at top of inner area
    let status_line = Line::from(Span::styled(status_text, status_style));
    let status_para = Paragraph::new(status_line);
    if inner.height > 0 {
        let status_area = Rect { height: 1, ..inner };
        f.render_widget(status_para, status_area);
    }

    // If log filter is active, show a small hint below status
    if state.log_filter_active && inner.height > 1 {
        let filter_hint = Paragraph::new(
            Line::from(Span::styled(" [log filter active]", Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)))
        );
        let hint_area = Rect { y: inner.y + 1, height: 1, ..inner };
        f.render_widget(filter_hint, hint_area);
    }
}

/// Renders the scrollable metro log panel. Visible when state.log_panel_visible is true.
pub fn render_log_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::MetroPane {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    let lines: Vec<Line> = state.metro_logs.iter()
        .map(|l| Line::from(l.as_str()))
        .collect();

    let visible_height = area.height.saturating_sub(2) as usize; // subtract borders

    // Auto-scroll to bottom when scroll offset is 0 (default)
    let scroll = if state.log_scroll_offset == 0 && !lines.is_empty() {
        lines.len().saturating_sub(visible_height)
    } else {
        state.log_scroll_offset
    };

    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(Block::default()
            .title(" Metro Logs ")
            .borders(Borders::ALL)
            .border_style(border_style))
        .scroll((scroll as u16, 0));

    f.render_widget(paragraph, area);

    // Scrollbar — only rendered when content exceeds visible area
    if lines.len() > visible_height {
        let mut scrollbar_state = ScrollbarState::new(lines.len())
            .position(scroll);

        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
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
