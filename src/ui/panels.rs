use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
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

    let block = Block::default()
        .title(" Worktrees ")
        .borders(Borders::ALL)
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

    // Status indicator text and color
    let (status_text, status_style) = match &state.metro.status {
        MetroStatus::Running {
            pid: _,
            worktree_id,
        } => (
            format!(" RUNNING  [{}]", worktree_id),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        MetroStatus::Stopped => (
            " STOPPED ".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
        MetroStatus::Starting => (
            " STARTING... ".to_string(),
            Style::default().fg(Color::Yellow),
        ),
        MetroStatus::Stopping => (
            " STOPPING... ".to_string(),
            Style::default().fg(Color::Yellow),
        ),
    };

    let block = Block::default()
        .title(" Metro ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    // Status line at top
    let status_line = Line::from(Span::styled(status_text, status_style));
    let status_area = Rect { height: 1, ..inner };
    f.render_widget(Paragraph::new(status_line), status_area);

    // Metro log output below status line (remaining height)
    if inner.height > 1 {
        let log_area = Rect {
            y: inner.y + 1,
            height: inner.height - 1,
            ..inner
        };

        let lines: Vec<Line> = state
            .metro_logs
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect();

        let visible_height = log_area.height as usize;

        // Auto-scroll to bottom (always show latest output)
        let scroll = if state.log_scroll_offset == 0 && !lines.is_empty() {
            lines.len().saturating_sub(visible_height)
        } else {
            state.log_scroll_offset
        };

        let paragraph = Paragraph::new(Text::from(lines.clone())).scroll((scroll as u16, 0));

        f.render_widget(paragraph, log_area);

        // Scrollbar when content exceeds visible area
        if lines.len() > visible_height {
            let mut scrollbar_state = ScrollbarState::new(lines.len()).position(scroll);

            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                log_area,
                &mut scrollbar_state,
            );
        }
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
            Block::default()
                .title(" Metro Logs ")
                .borders(Borders::ALL)
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

    // Title shows running command name + [running] indicator when active
    let title = match &state.running_command {
        Some(spec) => format!(" Output — {} [running] ", spec.label()),
        None => " Output ".to_string(),
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

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
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
