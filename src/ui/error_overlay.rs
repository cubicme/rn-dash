use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::{app::ErrorState, ui::theme};

/// Renders the error state overlay. Called from view() when state.error_state.is_some().
/// Shows error message and available actions (retry and/or dismiss).
pub fn render_error(f: &mut Frame, error: &ErrorState) {
    let area = centered_rect(f.area(), 50, 30, 40, 5);

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

/// Computes a centered Rect within `area`, using percentage sizing with minimum dimensions.
/// Width is clamped to [min_w, area.width], height to [min_h, area.height].
/// Separate copy from help_overlay to avoid cross-widget coupling.
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16, min_w: u16, min_h: u16) -> Rect {
    let w = (area.width * percent_x / 100).clamp(min_w, area.width);
    let h = (area.height * percent_y / 100).clamp(min_h, area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
