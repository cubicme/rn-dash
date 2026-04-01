use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::{app::ErrorState, ui::theme};

/// Renders the error state overlay. Called from view() when state.error_state.is_some().
/// Shows error message and available actions (retry and/or dismiss).
pub fn render_error(f: &mut Frame, error: &ErrorState) {
    let area = centered_rect(f.area(), 50, 30);

    let mut lines = vec![
        Line::from(error.message.as_str()),
        Line::from(""),
    ];

    let action_line = if error.can_retry {
        Line::from(vec![
            Span::styled("r", theme::style_key_hint()),
            Span::raw(" retry    "),
            Span::styled("q/Esc", theme::style_key_hint()),
            Span::raw(" dismiss"),
        ])
    } else {
        Line::from(vec![
            Span::styled("q/Esc", theme::style_key_hint()),
            Span::raw(" dismiss"),
        ])
    };
    lines.push(action_line);

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(theme::style_error_border());

    // Clear MUST come before the paragraph — erases background panels
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: true }), area);
}

/// Computes a centered Rect. Separate copy from help_overlay to avoid cross-widget coupling.
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .areas(area);
    area
}
