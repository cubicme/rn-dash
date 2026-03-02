use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

/// Renders the help overlay. Called from view() when state.show_help == true.
/// Uses Clear widget before the table to erase background panels behind the overlay.
pub fn render_help(f: &mut Frame) {
    let area = centered_rect(f.area(), 60, 80);

    let keybindings = vec![
        Row::new(vec!["?  / F1",     "Open this help"]),
        Row::new(vec!["q  / Esc",    "Close help"]),
        Row::new(vec!["q",           "Quit (when no overlay)"]),
        Row::new(vec!["h j k l",     "Navigate within panel"]),
        Row::new(vec!["↑ ↓ ← →",    "Navigate within panel"]),
        Row::new(vec!["Tab",         "Focus next panel"]),
        Row::new(vec!["Shift-Tab",   "Focus previous panel"]),
        Row::new(vec!["",            ""]),
        Row::new(vec!["Metro (when metro pane focused)", ""])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        Row::new(vec!["s",           "Start metro"]),
        Row::new(vec!["x",           "Stop metro"]),
        Row::new(vec!["r",           "Restart metro"]),
        Row::new(vec!["l",           "Toggle log panel"]),
        Row::new(vec!["J (shift-j)", "Send debugger command"]),
        Row::new(vec!["R (shift-r)", "Send reload command"]),
        Row::new(vec!["",            ""]),
        Row::new(vec!["Worktree List", ""])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        Row::new(vec!["j / k",        "Select next/prev worktree"]),
        Row::new(vec!["g",            "Git command palette"]),
        Row::new(vec!["c",            "RN command palette"]),
        Row::new(vec!["L (shift-l)",  "Set custom label"]),
        Row::new(vec!["R (shift-r)",  "Refresh worktree list"]),
        Row::new(vec!["",            ""]),
        Row::new(vec!["Git Palette (g then...)", ""])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        Row::new(vec!["p",            "git pull"]),
        Row::new(vec!["P (shift-p)",  "git push"]),
        Row::new(vec!["d",            "git reset --hard (confirm)"]),
        Row::new(vec!["b",            "git checkout <branch>"]),
        Row::new(vec!["B (shift-b)",  "git checkout -b <branch>"]),
        Row::new(vec!["r",            "git rebase origin/<branch>"]),
        Row::new(vec!["",            ""]),
        Row::new(vec!["RN Palette (c then...)", ""])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        Row::new(vec!["a",            "clean android"]),
        Row::new(vec!["c",            "clean cocoapods (confirm)"]),
        Row::new(vec!["n",            "rm node_modules (confirm)"]),
        Row::new(vec!["i",            "yarn install"]),
        Row::new(vec!["p",            "pod-install"]),
        Row::new(vec!["d",            "run-android (device select)"]),
        Row::new(vec!["s",            "run-ios (device select)"]),
        Row::new(vec!["t",            "unit-tests"]),
        Row::new(vec!["j",            "jest <filter>"]),
        Row::new(vec!["l",            "lint --quiet --fix"]),
        Row::new(vec!["y",            "check-types --incremental"]),
    ];

    let table = Table::new(
        keybindings,
        [Constraint::Length(28), Constraint::Fill(1)],
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
