use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use crate::{
    app::{AppState, FocusedPanel},
    domain::{metro::MetroStatus, worktree::WorktreeMetroStatus},
    ui::theme,
};

/// Renders the worktree list panel (left column) with real selectable list.
pub fn render_worktree_list(f: &mut Frame, area: Rect, state: &mut AppState) {
    let border_style = if state.focused_panel == FocusedPanel::WorktreeList {
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

    let items: Vec<ListItem> = state.worktrees.iter().map(|wt| {
        // Metro badge: [M] green-bold if Running, [ ] dark gray if Stopped
        let (badge_text, badge_style) = match wt.metro_status {
            WorktreeMetroStatus::Running => (
                "[M] ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            WorktreeMetroStatus::Stopped => (
                "[ ] ",
                Style::default().fg(Color::DarkGray),
            ),
        };

        // Branch name first (always visible), then secondary text
        let mut spans = vec![
            Span::styled(badge_text, badge_style),
            Span::raw(&wt.branch),
        ];

        // Secondary text: label takes priority over jira_title
        if let Some(label) = &wt.label {
            spans.push(Span::styled(
                format!(" - {label}"),
                Style::default().fg(Color::Gray),
            ));
        } else if let Some(title) = &wt.jira_title {
            spans.push(Span::styled(
                format!(" - {title}"),
                Style::default().fg(Color::Gray),
            ));
        }

        // Staleness icon (Unicode warning sign, compact and recognizable)
        if wt.stale {
            spans.push(Span::styled(
                " \u{26A0}",
                Style::default().fg(Color::Yellow),
            ));
        }

        ListItem::new(Line::from(spans))
    }).collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(list, area, &mut state.worktree_list_state);
}

/// Renders the metro control pane (top-right) with status line and scrolling log output.
pub fn render_metro_pane(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::MetroPane {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    // Status indicator text and color
    let (status_text, status_style) = match &state.metro.status {
        MetroStatus::Running { pid: _, worktree_id } =>
            (format!(" RUNNING  [{}]", worktree_id),
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

        let lines: Vec<Line> = state.metro_logs.iter()
            .map(|l| Line::from(l.as_str()))
            .collect();

        let visible_height = log_area.height as usize;

        // Auto-scroll to bottom (always show latest output)
        let scroll = if state.log_scroll_offset == 0 && !lines.is_empty() {
            lines.len().saturating_sub(visible_height)
        } else {
            state.log_scroll_offset
        };

        let paragraph = Paragraph::new(Text::from(lines.clone()))
            .scroll((scroll as u16, 0));

        f.render_widget(paragraph, log_area);

        // Scrollbar when content exceeds visible area
        if lines.len() > visible_height {
            let mut scrollbar_state = ScrollbarState::new(lines.len())
                .position(scroll);

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

/// Renders the command output pane (bottom-right) with real streaming output.
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

    let lines: Vec<Line> = state.command_output.iter()
        .map(|l| Line::from(l.as_str()))
        .collect();

    let visible_height = area.height.saturating_sub(2) as usize; // subtract borders

    // Auto-scroll to bottom when scroll is 0 (default); manual scroll overrides
    let scroll = if state.command_output_scroll == 0 && !lines.is_empty() {
        lines.len().saturating_sub(visible_height)
    } else {
        state.command_output_scroll
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
        let mut scrollbar_state = ScrollbarState::new(lines.len())
            .position(scroll);

        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}
