use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::{app::{AppState, FocusedPanel, PaletteMode}, domain::command::ModalState, ui::theme};

/// Renders the footer key hint bar. Always 1 line tall at the bottom of the layout.
/// Hints change based on which panel is focused — satisfies SHELL-02.
/// Icon legend (●=metro ⚠=stale) is rendered right-aligned.
pub fn render_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let hints = key_hints_for(state);

    // Build hint spans (left-aligned)
    let hint_spans: Vec<Span> = hints.iter().flat_map(|(key, desc)| {
        vec![
            Span::styled(*key, theme::style_key_hint()),
            Span::raw(format!(" {}  ", desc)),
        ]
    }).collect();

    // Icon legend (right-aligned)
    let legend = vec![
        Span::styled("\u{25B6}", Style::default().fg(Color::Green)),
        Span::raw("=metro  "),
        Span::styled("\u{26A0}", Style::default().fg(Color::Yellow)),
        Span::raw("=stale"),
    ];

    // Use horizontal layout: hints on left (flexible), legend on right (fixed)
    let [hint_area, legend_area] = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(20),
        ])
        .areas(area);

    let hint_line = Paragraph::new(Line::from(hint_spans));
    let legend_line = Paragraph::new(Line::from(legend))
        .alignment(ratatui::layout::Alignment::Right);

    f.render_widget(hint_line, hint_area);
    f.render_widget(legend_line, legend_area);
}

/// Returns context-sensitive key hints for the current app state.
/// Priority order: palette mode → modal state → overlay modes → panel-specific hints.
fn key_hints_for(state: &AppState) -> Vec<(&'static str, &'static str)> {
    if state.show_help {
        return vec![("q/Esc", "close help")];
    }
    if state.error_state.is_some() {
        return vec![("r", "retry"), ("q/Esc", "dismiss")];
    }

    // Palette mode hints — checked before panel hints
    if let Some(ref mode) = state.palette_mode {
        return match mode {
            PaletteMode::Android => vec![
                ("d", "run-android"),
                ("e", "device list"),
                ("r", "release build"),
                ("m", "set mode"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Ios => vec![
                ("d", "run-ios --device"),
                ("e", "simulator list"),
                ("p", "pod-install"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Yarn => vec![
                ("i", "install"),
                ("p", "pod-install"),
                ("u", "unit-tests"),
                ("t", "check-types"),
                ("j", "jest"),
                ("l", "lint"),
                ("a", "clean android"),
                ("c", "clean cocoapods"),
                ("n", "rm node_modules"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Git => vec![
                ("f", "fetch"),
                ("p", "pull"),
                ("P", "push"),
                ("X", "reset hard"),
                ("b", "checkout"),
                ("c", "checkout -b"),
                ("r", "rebase"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Worktree => vec![
                ("W", "add worktree"),
                ("D", "remove worktree"),
                ("B", "new branch worktree"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Metro => vec![
                ("s", "start"),
                ("x", "stop"),
                ("r", "restart"),
                ("j", "debugger"),
                ("R", "reload"),
                ("Esc", "cancel"),
            ],
        };
    }

    // Modal hints — checked after palette, before panel hints
    if let Some(ref modal) = state.modal {
        return match modal {
            ModalState::Confirm { .. } => vec![("Y", "confirm"), ("N/Esc", "cancel")],
            ModalState::TextInput { .. } => vec![("Enter", "submit"), ("Esc", "cancel")],
            ModalState::DevicePicker { .. } => {
                vec![("Enter", "select"), ("j/k", "navigate"), ("Esc", "cancel")]
            }
            ModalState::CleanToggle { .. } => vec![
                ("n", "node_modules"),
                ("p", "pods"),
                ("a", "android"),
                ("i", "sync after"),
                ("x/Enter", "confirm"),
                ("Esc", "cancel"),
            ],
            ModalState::SyncBeforeRun { .. } => vec![
                ("Y", "sync first"),
                ("N", "skip sync"),
                ("Esc", "cancel"),
            ],
            ModalState::ExternalMetroConflict { .. } => vec![
                ("Y", "kill it"),
                ("N/Esc", "cancel"),
            ],
            ModalState::BranchPicker { .. } => vec![
                ("Enter", "select"),
                ("Up/Down", "navigate"),
                ("type", "filter"),
                ("Esc", "cancel"),
            ],
        };
    }

    // Common hints always shown in normal mode
    let common: Vec<(&str, &str)> = vec![("?/F1", "help"), ("q", "quit"), ("Tab", "panel")];

    let panel_hints: Vec<(&str, &str)> = match state.focused_panel {
        FocusedPanel::WorktreeTable => vec![
            ("j/k", "navigate"),
            ("a", "android"),
            ("i", "ios"),
            ("y", "yarn"),
            ("w", "worktree"),
            ("g", "git"),
            ("m", "metro"),
            ("C", "claude"),
            ("T", "shell tab"),
            ("!", "shell"),
            ("Enter", "switch"),
        ],
        FocusedPanel::CommandOutput => {
            let mut hints: Vec<(&'static str, &'static str)> = vec![
                ("j/k", "scroll"),
                ("f", "fullscreen"),
            ];
            if state.running_command.is_some() {
                hints.push(("X", "cancel"));
            }
            hints
        },
    };

    // Panel hints first, then common hints
    let mut all = panel_hints;
    all.extend(common);
    all
}
