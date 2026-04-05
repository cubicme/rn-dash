use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Row, Table},
    Frame,
};

/// Renders the help overlay. Called from view() when state.show_help == true.
/// Uses Clear widget before the table to erase background panels behind the overlay.
/// Size: 70% width, 85% height to accommodate all keybinding sections.
pub fn render_help(f: &mut Frame) {
    let area = centered_rect(f.area(), 70, 85, 40, 10);

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
        Row::new(vec!["y",               "Yarn submenu"]),
        Row::new(vec!["w",               "Worktree submenu"]),
        Row::new(vec!["g",               "Git submenu"]),
        Row::new(vec!["C",               "Open Claude Code (tmux/zellij)"]),
        Row::new(vec!["T",               "Open shell tab at worktree"]),
        Row::new(vec!["f",               "Toggle fullscreen"]),
        Row::new(vec!["!",               "Run shell command in worktree"]),
        Row::new(vec!["R",               "Reload metro (when running) / Refresh list"]),
        Row::new(vec!["J",               "Metro debugger (when running)"]),
        Row::new(vec!["Esc",             "Stop metro (when running)"]),
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

        // Yarn submenu section
        Row::new(vec!["Yarn  (y>)", ""])
            .style(section_style),
        Row::new(vec!["i",               "yarn install"]),
        Row::new(vec!["p",               "yarn pod-install"]),
        Row::new(vec!["u",               "yarn unit-tests"]),
        Row::new(vec!["t",               "yarn check-types --incremental"]),
        Row::new(vec!["j",               "yarn jest <filter>"]),
        Row::new(vec!["l",               "yarn lint --quiet --fix"]),
        Row::new(vec!["a",               "Clean Android (react-native clean)"]),
        Row::new(vec!["c",               "Clean CocoaPods (react-native clean)"]),
        Row::new(vec!["n",               "Remove node_modules"]),
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
        Row::new(vec!["Esc",             "Cancel"]),
        Row::new(vec!["", ""]).style(dim_style),

        // Worktree submenu section
        Row::new(vec!["Worktree  (w>)", ""])
            .style(section_style),
        Row::new(vec!["w",               "Add new worktree"]),
        Row::new(vec!["d",               "Remove worktree (purge)"]),
        Row::new(vec!["b",               "New branch + worktree"]),
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

/// Computes a centered Rect within `area`, using percentage sizing with minimum dimensions.
/// Width is clamped to [min_w, area.width], height to [min_h, area.height].
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16, min_w: u16, min_h: u16) -> Rect {
    let w = (area.width * percent_x / 100).clamp(min_w, area.width);
    let h = (area.height * percent_y / 100).clamp(min_h, area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}
