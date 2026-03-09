use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Cell, Paragraph, Row, Scrollbar,
        ScrollbarOrientation,
        ScrollbarState, Table,
    },
    Frame,
};
use crate::{
    app::{AppState, FocusedPanel},
    domain::{metro::MetroStatus, worktree::WorktreeMetroStatus},
    ui::theme,
};

/// Truncates a string to max_width, appending "..." if truncated.
fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        return s.to_string();
    }
    if max_width <= 3 {
        return s[..max_width].to_string();
    }
    format!("{}...", &s[..max_width - 3])
}

/// Renders the worktree table (bottom section) with structured columns.
pub fn render_worktree_table(f: &mut Frame, area: Rect, state: &mut AppState) {
    let border_style = if state.focused_panel == FocusedPanel::WorktreeTable {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    let block = Block::bordered()
        .border_type(BorderType::Double)
        .title(" Worktrees ")
        .border_style(border_style);

    if state.worktrees.is_empty() {
        let placeholder = Paragraph::new("Loading worktrees...").block(block);
        f.render_widget(placeholder, area);
        return;
    }

    let rows: Vec<Row> = state
        .worktrees
        .iter()
        .map(|wt| {
            let label = wt.label.as_deref().unwrap_or("");
            let wt_name = wt
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            let branch = &wt.branch;

            // Extract ticket number from branch if possible
            let ticket_num = crate::infra::jira::extract_jira_key(branch).unwrap_or_default();
            let title = wt.jira_title.as_deref().unwrap_or("");

            // Status icons: metro=● stale=⚠
            let mut icons = String::new();
            if wt.metro_status == WorktreeMetroStatus::Running {
                icons.push_str("● ");
            }
            if wt.stale {
                icons.push('\u{26A0}');
            }

            let row_style = if wt.metro_status == WorktreeMetroStatus::Running {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(truncate(label, 12)),
                Cell::from(truncate(wt_name, 20)),
                Cell::from(truncate(branch, 30)),
                Cell::from(ticket_num),
                Cell::from(truncate(title, 30)),
                Cell::from(Span::styled(
                    icons,
                    Style::default().fg(Color::Yellow),
                )),
            ])
            .style(row_style)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from("LABEL"),
        Cell::from("WORKTREE"),
        Cell::from("BRANCH"),
        Cell::from("TICKET"),
        Cell::from("TITLE"),
        Cell::from(""), // icons column — no header
    ])
    .style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(14), // Label
            Constraint::Length(22), // Worktree name
            Constraint::Min(15),    // Branch (truncates)
            Constraint::Length(10), // Ticket #
            Constraint::Min(15),    // Title (truncates)
            Constraint::Length(4),  // Icons (fixed)
        ],
    )
    .header(header)
    .block(block)
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut state.worktree_table_state);
}

/// Renders the metro control pane (top-left) with status line and scrolling log output.
pub fn render_metro_pane(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::MetroPane {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    // Dynamic title based on metro status
    let title = match &state.metro.status {
        MetroStatus::Running {
            pid: _,
            worktree_id,
        } => {
            let wt_display = if worktree_id.len() > 30 {
                format!("{}...", &worktree_id[..27])
            } else {
                worktree_id.clone()
            };
            format!(" Metro -- running ({}) ", wt_display)
        }
        MetroStatus::Stopped => " Metro -- stopped ".to_string(),
        MetroStatus::Starting => " Metro -- starting... ".to_string(),
        MetroStatus::Stopping => " Metro -- stopping... ".to_string(),
    };

    let block = Block::bordered()
        .border_type(BorderType::Double)
        .title(title)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    // Metro log output fills entire inner area (status moved to title)
    let lines: Vec<Line> = state
        .metro_logs
        .iter()
        .map(|l| Line::from(l.as_str()))
        .collect();

    let visible_height = inner.height as usize;

    // Auto-scroll to bottom (always show latest output)
    let scroll = if state.log_scroll_offset == 0 && !lines.is_empty() {
        lines.len().saturating_sub(visible_height)
    } else {
        state.log_scroll_offset
    };

    let paragraph = Paragraph::new(Text::from(lines.clone())).scroll((scroll as u16, 0));

    f.render_widget(paragraph, inner);

    // Scrollbar when content exceeds visible area
    if lines.len() > visible_height {
        let mut scrollbar_state = ScrollbarState::new(lines.len()).position(scroll);

        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            inner,
            &mut scrollbar_state,
        );
    }
}

/// Renders the scrollable metro log panel. Visible when state.log_panel_visible is true.
pub fn render_log_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::MetroPane {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    let lines: Vec<Line> = state
        .metro_logs
        .iter()
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
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .title(" Metro Log ")
                .border_style(border_style),
        )
        .scroll((scroll as u16, 0));

    f.render_widget(paragraph, area);

    // Scrollbar — only rendered when content exceeds visible area
    if lines.len() > visible_height {
        let mut scrollbar_state = ScrollbarState::new(lines.len()).position(scroll);

        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// Renders the command output pane (top-right) with real streaming output.
pub fn render_command_output(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::CommandOutput {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    // Title shows running command name, [running] indicator, and queue count
    let title = match &state.running_command {
        Some(spec) => {
            let queue_count = state.command_queue.len();
            if queue_count > 0 {
                format!(" Output — {} [running] (+{} queued) ", spec.label(), queue_count)
            } else {
                format!(" Output — {} [running] ", spec.label())
            }
        }
        None => {
            let queue_count = state.command_queue.len();
            if queue_count > 0 {
                format!(" Output ({} queued) ", queue_count)
            } else {
                " Output ".to_string()
            }
        }
    };

    let lines: Vec<Line> = crate::app::active_output(state)
        .iter()
        .map(|l| Line::from(l.as_str()))
        .collect();

    let visible_height = area.height.saturating_sub(2) as usize; // subtract borders

    // Auto-scroll to bottom when scroll is 0 (default); manual scroll overrides
    let scroll_offset = crate::app::active_output_scroll(state);
    let scroll = if scroll_offset == 0 && !lines.is_empty() {
        lines.len().saturating_sub(visible_height)
    } else {
        scroll_offset
    };

    let block = Block::bordered()
        .border_type(BorderType::Double)
        .title(title)
        .border_style(border_style);

    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(block)
        .scroll((scroll as u16, 0));

    f.render_widget(paragraph, area);

    // Scrollbar — only rendered when content exceeds visible area
    if lines.len() > visible_height {
        let mut scrollbar_state = ScrollbarState::new(lines.len()).position(scroll);

        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}
