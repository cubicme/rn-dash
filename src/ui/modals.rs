//! Modal overlay renderers. Each modal type uses Clear + centered_rect pattern.
//! render_modal() is the single dispatch point called from view().

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::domain::command::ModalState;

/// Dispatch to the correct modal renderer based on the current ModalState.
pub fn render_modal(f: &mut Frame, modal: &ModalState) {
    match modal {
        ModalState::Confirm { prompt, .. } => render_confirm_modal(f, prompt),
        ModalState::TextInput { prompt, buffer, .. } => render_text_input_modal(f, prompt, buffer),
        ModalState::DevicePicker { devices, selected, .. } => {
            render_device_picker_modal(f, devices, *selected)
        }
        // Phase 05.1: Stubs — real rendering wired in Plan 06 (clean/sync modals)
        ModalState::CleanToggle { .. } => {
            render_placeholder_modal(f, " Clean Options ", "Plan 06: clean toggle UI")
        }
        ModalState::SyncBeforeRun { .. } => {
            render_placeholder_modal(f, " Sync Before Run ", "Worktree is stale. Sync before running?")
        }
    }
}

/// Renders a yes/no confirmation modal with a red border.
fn render_confirm_modal(f: &mut Frame, prompt: &str) {
    let area = centered_rect(f.area(), 50, 25);

    let lines = vec![
        Line::from(Span::raw(prompt)),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" confirm    "),
            Span::styled("[N/Esc]", Style::default().fg(Color::Red)),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// Renders a text input modal with a cyan border.
fn render_text_input_modal(f: &mut Frame, prompt: &str, buffer: &str) {
    let area = centered_rect(f.area(), 60, 25);

    let lines = vec![
        Line::from(Span::raw(prompt)),
        Line::from(Span::styled(
            format!("{}_", buffer),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" submit    "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" cancel"),
        ]),
    ];

    let block = Block::default()
        .title(" Input ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// Renders a device picker modal with a green border and list selection.
fn render_device_picker_modal(
    f: &mut Frame,
    devices: &[crate::domain::command::DeviceInfo],
    selected: usize,
) {
    let area = centered_rect(f.area(), 60, 50);

    let block = Block::default()
        .title(" Select Device ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let items: Vec<ListItem> = devices
        .iter()
        .map(|d| ListItem::new(Line::from(d.name.as_str())))
        .collect();

    if items.is_empty() {
        let para = Paragraph::new(vec![
            Line::from("No devices found."),
            Line::from(""),
            Line::from(vec![
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" cancel"),
            ]),
        ])
        .block(block);

        f.render_widget(Clear, area);
        f.render_widget(para, area);
        return;
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

    let mut ls = ListState::default();
    ls.select(Some(selected));

    f.render_widget(Clear, area);
    f.render_stateful_widget(list, area, &mut ls);
}

/// Placeholder modal used for Phase 05.1 stubs while real renderers are implemented in later plans.
fn render_placeholder_modal(f: &mut Frame, title: &str, message: &str) {
    let area = centered_rect(f.area(), 50, 25);
    let lines = vec![
        Line::from(Span::raw(message)),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" cancel"),
        ]),
    ];
    let block = Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// Computes a centered Rect of percent_x% width and percent_y% height within the given area.
/// Follows the same pattern as help_overlay.rs and error_overlay.rs.
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .areas(area);
    area
}
