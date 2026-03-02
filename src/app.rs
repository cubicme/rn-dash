#![allow(dead_code)]
use crate::action::Action;
use futures::StreamExt;
use ratatui::crossterm::event::{EventStream, KeyCode, KeyEventKind};

/// Maximum number of metro log lines retained in memory.
const MAX_LOG_LINES: usize = 1000;

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
///
/// No longer derives Default — MetroManager uses new() rather than Default::default().
#[derive(Debug)]
pub struct AppState {
    // Phase 1 fields
    pub focused_panel: FocusedPanel,
    pub show_help: bool,
    pub error_state: Option<ErrorState>,
    pub should_quit: bool,

    // Metro state — single-instance enforced by MetroManager's Option<MetroHandle>
    pub metro: crate::domain::metro::MetroManager,

    // Log panel
    pub metro_logs: std::collections::VecDeque<String>,
    pub log_scroll_offset: usize,
    pub log_panel_visible: bool,
    pub log_filter_active: bool,

    // Active worktree (stub until Phase 3 populates the real list)
    pub active_worktree_path: Option<std::path::PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            focused_panel: FocusedPanel::default(),
            show_help: false,
            error_state: None,
            should_quit: false,
            metro: crate::domain::metro::MetroManager::new(),
            metro_logs: std::collections::VecDeque::new(),
            log_scroll_offset: 0,
            log_panel_visible: false,
            log_filter_active: false,
            active_worktree_path: None,
        }
    }
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

    // Metro pane specific keybindings — checked before general navigation.
    // Error overlay intercept above already claims 'r' for RetryLastCommand, so
    // MetroRestart ('r') is only reachable when no error overlay is showing. Correct priority.
    if state.focused_panel == FocusedPanel::MetroPane {
        match key.code {
            Char('s') => return Some(Action::MetroStart),
            Char('x') => return Some(Action::MetroStop),
            Char('r') => return Some(Action::MetroRestart),
            Char('l') => return Some(Action::MetroToggleLog),
            // Shift-J / Shift-R avoid conflict with j=FocusDown and r=MetroRestart
            Char('J') => return Some(Action::MetroSendDebugger),
            Char('R') => return Some(Action::MetroSendReload),
            _ => {} // fall through to normal mode navigation
        }
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
        // Phase 1 actions
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

        // Metro control actions — async runtime behavior added in Plan 02
        Action::MetroStart => {
            // Plan 02 implements runtime behavior
        }
        Action::MetroStop => {
            // Plan 02 implements runtime behavior
        }
        Action::MetroRestart => {
            // Plan 02 implements runtime behavior
        }
        Action::MetroSendDebugger => {
            // Plan 02 implements runtime behavior
        }
        Action::MetroSendReload => {
            // Plan 02 implements runtime behavior
        }

        // Pure state mutations — implemented here (no async needed)
        Action::MetroToggleLog => {
            state.log_panel_visible = !state.log_panel_visible;
        }
        Action::MetroScrollUp => {
            state.log_scroll_offset = state.log_scroll_offset.saturating_sub(1);
        }
        Action::MetroScrollDown => {
            let max = state.metro_logs.len();
            if state.log_scroll_offset < max {
                state.log_scroll_offset += 1;
            }
        }
        Action::MetroLogLine(line) => {
            state.metro_logs.push_back(line);
            if state.metro_logs.len() > MAX_LOG_LINES {
                state.metro_logs.pop_front();
            }
        }
        Action::MetroExited => {
            state.metro.clear();
        }
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
