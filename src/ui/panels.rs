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
        .title_style(Style::default().fg(Color::White))
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
            let branch = &wt.branch;

            // Extract ticket number from branch if possible
            let ticket_num = crate::infra::jira::extract_jira_key(branch).unwrap_or_default();
            let title = wt.jira_title.as_deref().unwrap_or("");

            // Merged ticket display: "UMP-1234 Title text" or just one or the other
            let ticket_display = match (ticket_num.is_empty(), title.is_empty()) {
                (false, false) => format!("{} {}", ticket_num, title),
                (false, true) => ticket_num,
                (true, false) => title.to_string(),
                (true, true) => String::new(),
            };

            // Status icons: always show Y (yarn) and /P (pods) with color indicating freshness
            let mut icon_spans: Vec<Span> = Vec::new();

            // Metro indicator (only when running)
            if wt.metro_status == WorktreeMetroStatus::Running {
                icon_spans.push(Span::styled("\u{25B6}", Style::default().fg(Color::Green)));
                icon_spans.push(Span::raw(" "));
            }

            // Yarn staleness: Y always shown, green=fresh, red=stale
            let yarn_color = if wt.stale { Color::Red } else { Color::Green };
            icon_spans.push(Span::styled("Y", Style::default().fg(yarn_color)));

            // Pods staleness: /P always shown, green=fresh, red=stale
            let pods_color = if wt.stale_pods { Color::Red } else { Color::Green };
            icon_spans.push(Span::styled("/P", Style::default().fg(pods_color)));

            let row_style = if wt.metro_status == WorktreeMetroStatus::Running {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Extract dir name from path
            let dir_name = wt.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            Row::new(vec![
                Cell::from(Line::from(icon_spans)),
                Cell::from(truncate(label, 12)),
                Cell::from(truncate(branch, 18)),
                Cell::from(ticket_display),
                Cell::from(dir_name),
            ])
            .style(row_style)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(""),
        Cell::from("LABEL"),
        Cell::from("BRANCH"),
        Cell::from("TICKET"),
        Cell::from("DIR"),
    ])
    .style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // Status icons (metro + Y + /P)
            Constraint::Length(14), // Label
            Constraint::Length(20), // Branch
            Constraint::Min(20),   // Ticket (merged number + title)
            Constraint::Length(16), // Dir
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
            // Look up the actual worktree to get the preferred display name
            let wt_display = state
                .worktrees
                .iter()
                .find(|wt| {
                    wt.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        == worktree_id
                })
                .map(|wt| wt.preferred_prefix())
                .unwrap_or_else(|| worktree_id.clone());

            let wt_display = if wt_display.len() > 30 {
                format!("{}...", &wt_display[..27])
            } else {
                wt_display
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
        .title_style(Style::default().fg(Color::White))
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

    // Auto-scroll to bottom when auto_follow is enabled
    let scroll = if state.metro_log_auto_follow && !lines.is_empty() {
        lines.len().saturating_sub(visible_height)
    } else {
        state.log_scroll_offset
    };

    let paragraph = Paragraph::new(Text::from(lines.clone())).scroll((scroll as u16, 0));

    f.render_widget(paragraph, inner);

    // Scrollbar when content exceeds visible area
    if lines.len() > visible_height {
        let max_scroll = lines.len() - visible_height;
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll.min(max_scroll));

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

    // Auto-scroll to bottom when auto_follow is enabled
    let scroll = if state.metro_log_auto_follow && !lines.is_empty() {
        lines.len().saturating_sub(visible_height)
    } else {
        state.log_scroll_offset
    };

    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .title(" Metro Log ")
                .title_style(Style::default().fg(Color::White))
                .border_style(border_style),
        )
        .scroll((scroll as u16, 0));

    f.render_widget(paragraph, area);

    // Scrollbar — only rendered when content exceeds visible area
    if lines.len() > visible_height {
        let max_scroll = lines.len() - visible_height;
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll.min(max_scroll));

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
        .title_style(Style::default().fg(Color::White))
        .border_style(border_style);

    let paragraph = Paragraph::new(Text::from(lines.clone()))
        .block(block)
        .scroll((scroll as u16, 0));

    f.render_widget(paragraph, area);

    // Scrollbar — only rendered when content exceeds visible area
    if lines.len() > visible_height {
        let max_scroll = lines.len() - visible_height;
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll.min(max_scroll));

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
