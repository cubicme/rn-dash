#![allow(dead_code)]
use crate::action::Action;
use futures::StreamExt;
use ratatui::crossterm::event::{EventStream, KeyCode, KeyEventKind};

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FocusedPanel {
    #[default]
    WorktreeList,
    MetroPane,
    CommandOutput, // stub for Phase 3
}

impl FocusedPanel {
    pub fn next(self) -> Self {
        match self {
            Self::WorktreeList => Self::MetroPane,
            Self::MetroPane => Self::CommandOutput,
            Self::CommandOutput => Self::WorktreeList,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Self::WorktreeList => Self::CommandOutput,
            Self::MetroPane => Self::WorktreeList,
            Self::CommandOutput => Self::MetroPane,
        }
    }
}

/// Error state shown in the error overlay. Phase 2+ will set this from real command failures.
#[derive(Debug, Clone)]
pub struct ErrorState {
    pub message: String,
    pub can_retry: bool,
}

/// Application state — the single source of truth. All mutations happen in update().
#[derive(Debug, Default)]
pub struct AppState {
    pub focused_panel: FocusedPanel,
    pub show_help: bool,
    pub error_state: Option<ErrorState>,
    pub should_quit: bool,
}

/// Pure function: maps (state, key) → Action. No side effects.
/// Called from the event loop — keep it fast and allocation-free.
pub fn handle_key(state: &AppState, key: ratatui::crossterm::event::KeyEvent) -> Option<Action> {
    use KeyCode::*;

    // Guard: only process key-press events (prevents double-firing on Windows)
    if key.kind != KeyEventKind::Press {
        return None;
    }

    // Overlay modes intercept keys first — overlay dismissal takes priority
    if state.show_help {
        return match key.code {
            Char('q') | Esc => Some(Action::DismissHelp),
            _ => None,
        };
    }

    if state.error_state.is_some() {
        return match key.code {
            Char('r') => Some(Action::RetryLastCommand),
            Char('q') | Esc => Some(Action::DismissError),
            _ => None,
        };
    }

    // Normal mode keybindings
    match key.code {
        Char('q') => Some(Action::Quit),
        Char('?') | F(1) => Some(Action::ShowHelp),
        Char('/') => Some(Action::Search),
        Char('j') | Down => Some(Action::FocusDown),
        Char('k') | Up => Some(Action::FocusUp),
        Char('h') | Left => Some(Action::FocusLeft),
        Char('l') | Right => Some(Action::FocusRight),
        Tab => Some(Action::FocusNext),
        BackTab => Some(Action::FocusPrev),
        _ => None,
    }
}

/// TEA update function — the ONLY place AppState is mutated.
/// Pure state transition: state + action → new state.
pub fn update(state: &mut AppState, action: Action) {
    match action {
        Action::FocusNext => state.focused_panel = state.focused_panel.next(),
        Action::FocusPrev => state.focused_panel = state.focused_panel.prev(),
        Action::FocusUp | Action::FocusDown | Action::FocusLeft | Action::FocusRight => {
            // Phase 1: no intra-panel navigation yet — actions dispatched but no-op within panels
        }
        Action::Search => {
            // Phase 1: stub — keybinding registered, search mode implemented in Phase 4+
        }
        Action::ShowHelp => state.show_help = true,
        Action::DismissHelp => state.show_help = false,
        Action::DismissError => state.error_state = None,
        Action::RetryLastCommand => {
            // Phase 2+ will populate retry logic; for now just clear the error
            state.error_state = None;
        }
        Action::Quit => state.should_quit = true,
    }
}

/// Main application loop. Runs on the tokio runtime.
/// Renders on every event and on a 250ms tick. Exits when state.should_quit is true.
pub async fn run(mut terminal: ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut state = AppState::default();
    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(250));

    loop {
        // Render first on each iteration — double-buffer diff handles no-change efficiently
        terminal.draw(|f| crate::ui::view(f, &state))?;

        tokio::select! {
            _ = tick.tick() => {
                // Periodic tick: triggers redraw for time-based UI updates (Phase 2+ will use this)
            }
            maybe_event = events.next() => {
                let Some(Ok(event)) = maybe_event else { break };
                use ratatui::crossterm::event::Event as CE;
                match event {
                    CE::Key(key) => {
                        if let Some(action) = handle_key(&state, key) {
                            update(&mut state, action);
                        }
                    }
                    CE::Resize(_, _) => {
                        // draw() is called at the top of the loop — resize redraws automatically
                    }
                    _ => {}
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    Ok(())
}
