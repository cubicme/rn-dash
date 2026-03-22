use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

/// Renders the help overlay. Called from view() when state.show_help == true.
/// Uses Clear widget before the table to erase background panels behind the overlay.
/// Size: 70% width, 85% height to accommodate all keybinding sections.
pub fn render_help(f: &mut Frame) {
    let area = centered_rect(f.area(), 70, 85);

    let section_style = Style::default().add_modifier(Modifier::BOLD);
    let dim_style = Style::default().fg(Color::DarkGray);

    let keybindings = vec![
        // Navigation section
        Row::new(vec!["Navigation", ""])
            .style(section_style),
        Row::new(vec!["Tab / Shift-Tab",  "Switch panel"]),
        Row::new(vec!["j / k",            "Navigate within panel"]),
        Row::new(vec!["? / F1",           "Open this help"]),
        Row::new(vec!["q / Esc",          "Quit / close overlay"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Worktree Table section
        Row::new(vec!["Worktree Table", ""])
            .style(section_style),
        Row::new(vec!["Enter",            "Switch metro to worktree"]),
        Row::new(vec!["a",               "Android submenu"]),
        Row::new(vec!["i",               "iOS submenu"]),
        Row::new(vec!["x",               "Clean submenu"]),
        Row::new(vec!["s",               "Sync submenu"]),
        Row::new(vec!["g",               "Git submenu"]),
        Row::new(vec!["m",               "Metro submenu"]),
        Row::new(vec!["C",               "Open Claude Code (tmux/zellij)"]),
        Row::new(vec!["T",               "Open shell tab at worktree"]),
        Row::new(vec!["L",               "Set custom branch label"]),
        Row::new(vec!["f",               "Toggle fullscreen"]),
        Row::new(vec!["!",               "Run shell command in worktree"]),
        Row::new(vec!["R",               "Refresh worktree list"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Android submenu section
        Row::new(vec!["Android  (a>)", ""])
            .style(section_style),
        Row::new(vec!["d",               "run-android (device select)"]),
        Row::new(vec!["e",               "Device list"]),
        Row::new(vec!["r",               "Release build (assembleRelease)"]),
        Row::new(vec!["Esc",             "Cancel"]),
        Row::new(vec!["", ""]).style(dim_style),

        // iOS submenu section
        Row::new(vec!["iOS  (i>)", ""])
            .style(section_style),
        Row::new(vec!["d",               "run-ios --device (auto-select)"]),
        Row::new(vec!["e",               "Simulator list (xcrun)"]),
        Row::new(vec!["p",               "pod-install"]),
        Row::new(vec!["Esc",             "Cancel"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Clean submenu section
        Row::new(vec!["Clean  (x>)", ""])
            .style(section_style),
        Row::new(vec!["n",               "Toggle node_modules"]),
        Row::new(vec!["p",               "Toggle pods"]),
        Row::new(vec!["a",               "Toggle android"]),
        Row::new(vec!["i",               "Toggle sync after"]),
        Row::new(vec!["x / Enter",       "Confirm and run"]),
        Row::new(vec!["Esc",             "Cancel"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Sync submenu section
        Row::new(vec!["Sync  (s>)", ""])
            .style(section_style),
        Row::new(vec!["i",               "yarn install"]),
        Row::new(vec!["u",               "yarn unit-tests"]),
        Row::new(vec!["t",               "yarn check-types --incremental"]),
        Row::new(vec!["j",               "yarn jest <filter>"]),
        Row::new(vec!["l",               "yarn lint --quiet --fix"]),
        Row::new(vec!["Esc",             "Cancel"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Git submenu section
        Row::new(vec!["Git  (g>)", ""])
            .style(section_style),
        Row::new(vec!["f",               "git fetch --all --tags"]),
        Row::new(vec!["p",               "git pull"]),
        Row::new(vec!["P",               "git push"]),
        Row::new(vec!["X",               "git fetch + reset --hard origin/<branch>"]),
        Row::new(vec!["b",               "git checkout <branch>"]),
        Row::new(vec!["c",               "git checkout -b <branch>"]),
        Row::new(vec!["r",               "git rebase <target>"]),
        Row::new(vec!["D",               "Remove worktree (purge)"]),
        Row::new(vec!["Esc",             "Cancel"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Metro palette section
        Row::new(vec!["Metro  (m>)", ""])
            .style(section_style),
        Row::new(vec!["s",               "Start metro"]),
        Row::new(vec!["x",               "Stop metro"]),
        Row::new(vec!["r",               "Restart metro"]),
        Row::new(vec!["j",               "Send debugger command"]),
        Row::new(vec!["R",               "Send reload command"]),
        Row::new(vec!["Esc",             "Cancel"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Output Pane section
        Row::new(vec!["Output Pane", ""])
            .style(section_style),
        Row::new(vec!["j / k",           "Scroll output"]),
        Row::new(vec!["X",               "Cancel running command"]),
        Row::new(vec!["C",               "Clear output"]),
        Row::new(vec!["f",               "Toggle fullscreen"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Icons legend section
        Row::new(vec!["Icons", ""])
            .style(section_style),
        Row::new(vec!["●  (green)",       "Metro is running"]),
        Row::new(vec!["\u{26A0}  (yellow)", "Stale dependencies (node_modules/pods)"]),
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
