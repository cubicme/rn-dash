use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::{app::{AppState, FocusedPanel, PaletteMode}, domain::command::ModalState, ui::theme};

/// Renders the footer key hint bar. Always 1 line tall at the bottom of the layout.
/// Hints change based on which panel is focused — satisfies SHELL-02.
pub fn render_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let hints = key_hints_for(state);
    let spans: Vec<Span> = hints.iter().flat_map(|(key, desc)| {
        vec![
            Span::styled(*key, theme::style_key_hint()),
            Span::raw(format!(" {}  ", desc)),
        ]
    }).collect();
    let paragraph = Paragraph::new(Line::from(spans));
    f.render_widget(paragraph, area);
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
            PaletteMode::Git => vec![
                ("p", "pull"),
                ("P", "push"),
                ("X", "reset hard"),
                ("f", "fetch"),
                ("b", "checkout"),
                ("c", "checkout -b"),
                ("r", "rebase"),
                ("Esc", "cancel"),
            ],
            // Phase 05.1: Stubs — real hints wired in Plan 05 (keybinding remap)
            PaletteMode::Android => vec![
                ("d", "run-android"),
                ("e", "pick device"),
                ("r", "release build"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Ios => vec![
                ("d", "run-ios device"),
                ("e", "simulator"),
                ("p", "pod-install"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Clean => vec![
                ("n", "node_modules"),
                ("p", "pods"),
                ("a", "android"),
                ("i", "sync after"),
                ("x", "confirm"),
                ("Esc", "cancel"),
            ],
            PaletteMode::Sync => vec![
                ("i", "yarn install"),
                ("u", "unit-tests"),
                ("t", "check-types"),
                ("j", "jest"),
                ("l", "lint"),
                ("Esc", "cancel"),
            ],
        };
    }

    // Modal hints — checked after palette, before panel hints
    if state.modal.is_some() {
        return match &state.modal {
            Some(ModalState::Confirm { .. }) => vec![("Y", "confirm"), ("N/Esc", "cancel")],
            Some(ModalState::TextInput { .. }) => vec![("Enter", "submit"), ("Esc", "cancel")],
            Some(ModalState::DevicePicker { .. }) => {
                vec![("Enter", "select"), ("j/k", "navigate"), ("Esc", "cancel")]
            }
            // Phase 05.1: Stubs — real hints wired in Plan 06
            Some(ModalState::CleanToggle { .. }) => {
                vec![("n/p/a/i", "toggle"), ("x/Enter", "confirm"), ("Esc", "cancel")]
            }
            Some(ModalState::SyncBeforeRun { .. }) => {
                vec![("y", "sync + run"), ("n", "run without sync"), ("Esc", "cancel")]
            }
            None => unreachable!(),
        };
    }

    // Common hints always shown in normal mode
    let common: Vec<(&str, &str)> = vec![("?/F1", "help"), ("q", "quit"), ("Tab", "next panel")];

    let panel_hints: Vec<(&str, &str)> = match state.focused_panel {
        FocusedPanel::WorktreeList => vec![
            ("j/k", "navigate"),
            ("Enter", "switch"),
            ("g", "git"),
            ("c", "commands"),
            ("L", "set label"),
            ("C", "claude"),
        ],
        FocusedPanel::MetroPane => {
            let mut hints: Vec<(&'static str, &'static str)> = vec![
                ("s", "start"),
                ("x", "stop"),
                ("r", "restart"),
                ("l", "logs"),
            ];
            // Show stdin commands only when metro is running
            if state.metro.is_running() {
                hints.push(("J", "debugger"));
                hints.push(("R", "reload"));
            }
            hints
        },
        FocusedPanel::CommandOutput => {
            let mut hints: Vec<(&'static str, &'static str)> = vec![("j/k", "scroll")];
            if state.running_command.is_some() {
                hints.push(("Ctrl-c", "cancel"));
            }
            hints
        },
    };

    // Panel hints first, then common hints
    let mut all = panel_hints;
    all.extend(common);
    all
}
