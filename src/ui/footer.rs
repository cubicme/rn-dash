use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::{app::{AppState, FocusedPanel}, ui::theme};

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
/// Help/error overlays show their own hints; normal mode shows panel-specific hints.
fn key_hints_for(state: &AppState) -> Vec<(&'static str, &'static str)> {
    if state.show_help {
        return vec![("q/Esc", "close help")];
    }
    if state.error_state.is_some() {
        return vec![("r", "retry"), ("q/Esc", "dismiss")];
    }
    // Common hints always shown in normal mode
    let common: Vec<(&str, &str)> = vec![("?/F1", "help"), ("q", "quit"), ("Tab", "next panel")];

    let panel_hints: Vec<(&str, &str)> = match state.focused_panel {
        FocusedPanel::WorktreeList => vec![("j/k", "navigate"), ("Enter", "select")],
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
        FocusedPanel::CommandOutput => vec![("j/k", "scroll")],
    };

    // Panel hints first, then common hints
    let mut all = panel_hints;
    all.extend(common);
    all
}
