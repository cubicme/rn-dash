use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

/// Renders the help overlay. Called from view() when state.show_help == true.
/// Uses Clear widget before the table to erase background panels behind the overlay.
pub fn render_help(f: &mut Frame) {
    let area = centered_rect(f.area(), 60, 70);

    let keybindings = vec![
        Row::new(vec!["?  / F1",     "Open this help"]),
        Row::new(vec!["q  / Esc",    "Close help"]),
        Row::new(vec!["q",           "Quit (when no overlay)"]),
        Row::new(vec!["h j k l",     "Navigate within panel"]),
        Row::new(vec!["↑ ↓ ← →",    "Navigate within panel"]),
        Row::new(vec!["Tab",         "Focus next panel"]),
        Row::new(vec!["Shift-Tab",   "Focus previous panel"]),
        Row::new(vec!["",            ""]),
        Row::new(vec!["— Metro (Phase 2) —", ""]),
        Row::new(vec!["s",           "Start metro"]),
        Row::new(vec!["x",           "Stop metro"]),
        Row::new(vec!["r",           "Restart metro"]),
    ];

    let table = Table::new(
        keybindings,
        [Constraint::Length(18), Constraint::Fill(1)],
    )
    .block(
        Block::default()
            .title(" Keybindings — q/Esc to close ")
            .borders(Borders::ALL),
    );

    // Clear MUST be rendered before the table — otherwise background panels show through
    f.render_widget(Clear, area);
    f.render_widget(table, area);
}

/// Computes a centered Rect of percent_x% width and percent_y% height within the given area.
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .areas(area);
    area
}
