#![allow(dead_code)]
use crate::action::Action;
use crate::domain::command::{CommandSpec, ModalState};
use crate::domain::metro::MetroHandle;
use futures::StreamExt;
use ratatui::crossterm::event::{EventStream, KeyCode, KeyEventKind};
use std::path::PathBuf;

/// Maximum number of metro log lines retained in memory.
const MAX_LOG_LINES: usize = 1000;

/// Maximum number of command output lines retained in memory.
const MAX_COMMAND_LINES: usize = 1000;

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FocusedPanel {
    #[default]
    WorktreeTable,
    MetroPane,
    CommandOutput,
}

impl FocusedPanel {
    pub fn next(self) -> Self {
        match self {
            Self::WorktreeTable => Self::MetroPane,
            Self::MetroPane => Self::CommandOutput,
            Self::CommandOutput => Self::WorktreeTable,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Self::WorktreeTable => Self::CommandOutput,
            Self::MetroPane => Self::WorktreeTable,
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

/// Which submenu the command palette is in (Phase 05.1 expanded scheme).
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteMode {
    /// 'a' — Android submenu
    Android,
    /// 'i' — iOS submenu
    Ios,
    /// 'x' — Clean submenu (only sets palette mode before CleanToggle modal opens)
    Clean,
    /// 's' — Sync submenu
    Sync,
    /// 'g' — Git submenu
    Git,
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

    // Active worktree (updated from WorktreesLoaded + WorktreeSelectNext/Prev)
    pub active_worktree_path: Option<std::path::PathBuf>,

    // Set to true when MetroRestart or MetroStart-while-running triggers a stop-first-then-start.
    // When MetroExited fires and this is true, a new MetroStart is auto-dispatched.
    pub pending_restart: bool,

    // Phase 5: captured target worktree path during worktree switch (consumed by MetroExited)
    pub pending_switch_path: Option<std::path::PathBuf>,

    // --- Phase 3 fields ---

    // Worktree browser
    pub worktrees: Vec<crate::domain::worktree::Worktree>,
    pub worktree_table_state: ratatui::widgets::TableState,
    pub selected_worktree_id: Option<crate::domain::worktree::WorktreeId>,
    pub fullscreen_panel: Option<FocusedPanel>,

    // Command queue — FIFO, drained on CommandExited
    pub command_queue: std::collections::VecDeque<crate::domain::command::CommandSpec>,

    // Per-worktree output persistence
    pub command_output_by_worktree: std::collections::HashMap<crate::domain::worktree::WorktreeId, std::collections::VecDeque<String>>,
    pub command_output_scroll_by_worktree: std::collections::HashMap<crate::domain::worktree::WorktreeId, usize>,

    // Currently running command and its task handle
    pub running_command: Option<crate::domain::command::CommandSpec>,
    pub command_task: Option<tokio::task::JoinHandle<()>>,

    // Modal state — only one modal active at a time
    pub modal: Option<crate::domain::command::ModalState>,

    // Label store: branch_name -> label_text (persisted to labels.json)
    pub labels: std::collections::HashMap<String, String>,

    // Repo root — worktrees are listed relative to this path
    pub repo_root: std::path::PathBuf,

    // Command palette mode — Some when user pressed 'g' or 'c' in WorktreeList
    pub palette_mode: Option<PaletteMode>,

    // Pending device command — stored while async device enumeration is in flight
    pub pending_device_command: Option<crate::domain::command::CommandSpec>,

    // Pending label branch — set by StartSetLabel, consumed by ModalInputSubmit
    pub pending_label_branch: Option<String>,

    // --- Phase 4 fields ---
    pub jira_title_cache: std::collections::HashMap<String, String>,  // UMP-XXXX -> title
    pub jira_client: Option<std::sync::Arc<dyn crate::infra::jira::JiraClient>>,

    // --- Phase 5.2 fields ---
    /// When true, metro log pane auto-scrolls to bottom on new content.
    /// Set false on manual scroll up. Set true on ScrollToBottom (G).
    pub metro_log_auto_follow: bool,
    /// First 'g' press sets this true; second 'g' triggers ScrollToTop. Cleared on any other action.
    pub pending_g: bool,

    // --- Phase 5.1 fields ---
    /// Detected terminal multiplexer (tmux or zellij). None when not inside either.
    pub multiplexer: Option<Box<dyn crate::infra::multiplexer::Multiplexer>>,
    /// Claude Code launch flags loaded from config (e.g. "--dangerously-skip-permissions").
    pub claude_flags: String,
    /// Loaded dashboard config — kept for runtime access to claude_flags and other settings.
    pub config: Option<crate::infra::config::DashConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut worktree_table_state = ratatui::widgets::TableState::default();
        worktree_table_state.select(Some(0));
        Self {
            focused_panel: FocusedPanel::default(),
            show_help: false,
            error_state: None,
            should_quit: false,
            metro: crate::domain::metro::MetroManager::new(),
            metro_logs: std::collections::VecDeque::new(),
            log_scroll_offset: 0,
            log_panel_visible: false,
            log_filter_active: true,
            active_worktree_path: None,
            pending_restart: false,
            pending_switch_path: None,
            // Phase 3
            worktrees: Vec::new(),
            worktree_table_state,
            selected_worktree_id: None,
            fullscreen_panel: None,
            command_queue: std::collections::VecDeque::new(),
            command_output_by_worktree: std::collections::HashMap::new(),
            command_output_scroll_by_worktree: std::collections::HashMap::new(),
            running_command: None,
            command_task: None,
            modal: None,
            labels: std::collections::HashMap::new(),
            repo_root: PathBuf::from(
                std::env::var("HOME").unwrap_or_else(|_| ".".into()),
            )
            .join("aljazeera/ump"),
            palette_mode: None,
            pending_device_command: None,
            pending_label_branch: None,
            // Phase 5.2
            metro_log_auto_follow: true,
            pending_g: false,
            // Phase 4
            jira_title_cache: std::collections::HashMap::new(),
            jira_client: None,
            // Phase 5.1
            multiplexer: None,  // set properly in run()
            claude_flags: "--dangerously-skip-permissions".to_string(),
            config: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-worktree output accessor helpers (used by panels.rs)
// ---------------------------------------------------------------------------

/// Returns the WorktreeId for the currently selected worktree, or None if list is empty.
pub fn active_worktree_id(state: &AppState) -> Option<crate::domain::worktree::WorktreeId> {
    if state.worktrees.is_empty() {
        return None;
    }
    let idx = state.worktree_table_state.selected().unwrap_or(0)
        .min(state.worktrees.len() - 1);
    Some(state.worktrees[idx].id.clone())
}

/// Returns a reference to the active worktree's command output deque (empty if none selected).
pub fn active_output(state: &AppState) -> &std::collections::VecDeque<String> {
    static EMPTY: std::sync::LazyLock<std::collections::VecDeque<String>> =
        std::sync::LazyLock::new(std::collections::VecDeque::new);
    if let Some(id) = active_worktree_id(state) {
        state.command_output_by_worktree.get(&id).unwrap_or(&EMPTY)
    } else {
        &EMPTY
    }
}

/// Returns the scroll offset for the active worktree's command output (0 if none selected).
pub fn active_output_scroll(state: &AppState) -> usize {
    active_worktree_id(state)
        .and_then(|id| state.command_output_scroll_by_worktree.get(&id).copied())
        .unwrap_or(0)
}

/// Pure function: maps (state, key) → Action. No side effects.
/// Called from the event loop — keep it fast and allocation-free.
pub fn handle_key(state: &AppState, key: ratatui::crossterm::event::KeyEvent) -> Option<Action> {
    use KeyCode::*;

    // Guard: only process key-press events (prevents double-firing on Windows)
    if key.kind != KeyEventKind::Press {
        return None;
    }

    // --- MODAL INTERCEPTION — MUST be first (prevents key leak to navigation) ---
    if let Some(ref modal) = state.modal {
        return match modal {
            ModalState::Confirm { .. } => match key.code {
                Char('y') | Char('Y') => Some(Action::ModalConfirm),
                Char('n') | Char('N') | Esc => Some(Action::ModalCancel),
                _ => None,
            },
            ModalState::TextInput { .. } => match key.code {
                Esc => Some(Action::ModalCancel),
                Enter => Some(Action::ModalInputSubmit),
                Backspace => Some(Action::ModalInputBackspace),
                Char(c) => Some(Action::ModalInputChar(c)),
                _ => None,
            },
            ModalState::DevicePicker { .. } => match key.code {
                Esc => Some(Action::ModalCancel),
                Enter => Some(Action::ModalDeviceConfirm),
                Down => Some(Action::ModalDeviceNext),
                Up => Some(Action::ModalDevicePrev),
                Backspace => Some(Action::ModalInputBackspace),
                Char('j') => Some(Action::ModalDeviceNext),
                Char('k') => Some(Action::ModalDevicePrev),
                Char(c) if !c.is_ascii_control() => Some(Action::ModalInputChar(c)),
                _ => None,
            },
            ModalState::CleanToggle { .. } => match key.code {
                Char('n') => Some(Action::CleanToggleNodeModules),
                Char('p') => Some(Action::CleanTogglePods),
                Char('a') => Some(Action::CleanToggleAndroid),
                Char('i') => Some(Action::CleanToggleSyncAfter),
                Char('x') | Enter => Some(Action::CleanConfirm),
                Esc => Some(Action::ModalCancel),
                _ => None,
            },
            ModalState::SyncBeforeRun { .. } => match key.code {
                Char('y') | Char('Y') => Some(Action::SyncBeforeRunAccept),
                Char('n') | Char('N') | Esc => Some(Action::SyncBeforeRunDecline),
                _ => None,
            },
            ModalState::ExternalMetroConflict { pid, .. } => match key.code {
                Char('y') | Char('Y') | Enter => Some(Action::KillExternalMetro(*pid)),
                Char('n') | Char('N') | Esc => Some(Action::ModalCancel),
                _ => None,
            },
        };
    }

    // --- PALETTE MODE ROUTING — after modal, before overlays ---
    if let Some(ref mode) = state.palette_mode {
        return match mode {
            PaletteMode::Android => match key.code {
                Char('d') => Some(Action::CommandRun(CommandSpec::RnRunAndroid { device_id: String::new() })),
                Char('e') => Some(Action::CommandRun(CommandSpec::RnRunAndroid { device_id: String::new() })),
                Char('r') => Some(Action::CommandRun(CommandSpec::RnReleaseBuild)),
                Esc => Some(Action::ModalCancel),
                _ => Some(Action::ModalCancel),
            },
            PaletteMode::Ios => match key.code {
                Char('d') => Some(Action::CommandRun(CommandSpec::RnRunIos { device_id: String::new() })),
                Char('e') => Some(Action::CommandRun(CommandSpec::RnRunIos { device_id: String::new() })),
                Char('p') => Some(Action::CommandRun(CommandSpec::YarnPodInstall)),
                Esc => Some(Action::ModalCancel),
                _ => Some(Action::ModalCancel),
            },
            PaletteMode::Clean => match key.code {
                // Clean palette opens the CleanToggle modal immediately.
                // If we're here, user is in CleanToggle modal — keys handled by modal interception above.
                // This arm should not normally be reached; fallback to cancel.
                Esc => Some(Action::ModalCancel),
                _ => Some(Action::ModalCancel),
            },
            PaletteMode::Sync => match key.code {
                Char('i') => Some(Action::CommandRun(CommandSpec::YarnInstall)),
                Char('u') => Some(Action::CommandRun(CommandSpec::YarnUnitTests)),
                Char('t') => Some(Action::CommandRun(CommandSpec::YarnCheckTypes)),
                Char('j') => Some(Action::CommandRun(CommandSpec::YarnJest { filter: String::new() })),
                Char('l') => Some(Action::CommandRun(CommandSpec::YarnLint)),
                Esc => Some(Action::ModalCancel),
                _ => Some(Action::ModalCancel),
            },
            PaletteMode::Git => match key.code {
                Char('f') => Some(Action::CommandRun(CommandSpec::GitFetch)),
                Char('p') => Some(Action::CommandRun(CommandSpec::GitPull)),
                Char('P') => Some(Action::CommandRun(CommandSpec::GitPush)),
                Char('X') => Some(Action::CommandRun(CommandSpec::GitResetHardFetch)),
                Char('b') => Some(Action::CommandRun(CommandSpec::GitCheckout { branch: String::new() })),
                Char('c') => Some(Action::CommandRun(CommandSpec::GitCheckoutNew { branch: String::new() })),
                Char('r') => Some(Action::CommandRun(CommandSpec::GitRebase { target: String::new() })),
                Esc => Some(Action::ModalCancel),
                _ => Some(Action::ModalCancel),
            },
        };
    }

    // --- OVERLAY MODES ---
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

    // --- FULLSCREEN: Tab exits fullscreen ---
    if state.fullscreen_panel.is_some() {
        if key.code == Tab {
            return Some(Action::ToggleFullscreen);
        }
    }

    // --- METRO PANE SPECIFIC ---
    if state.focused_panel == FocusedPanel::MetroPane {
        match key.code {
            Char('s') => return Some(Action::MetroStart),
            Char('x') => return Some(Action::MetroStop),
            Char('r') => return Some(Action::MetroRestart),
            Char('l') => return Some(Action::MetroToggleLog),
            Char('j') => return Some(Action::MetroScrollDown),
            Char('k') => return Some(Action::MetroScrollUp),
            Char('J') => return Some(Action::MetroSendDebugger),
            Char('R') => return Some(Action::MetroSendReload),
            Char('f') => return Some(Action::ToggleFullscreen),
            Char('G') => return Some(Action::ScrollToBottom),
            Char('g') => {
                if state.pending_g {
                    return Some(Action::ScrollToTop);
                } else {
                    return Some(Action::SetPendingG);
                }
            }
            // Shift-X clears log panel (avoids collision with 'x' which stops metro)
            Char('X') => return Some(Action::LogPanelClear),
            _ => {} // fall through to normal navigation
        }
    }

    // --- WORKTREE TABLE SPECIFIC ---
    if state.focused_panel == FocusedPanel::WorktreeTable {
        match key.code {
            Char('j') | Down => return Some(Action::WorktreeSelectNext),
            Char('k') | Up => return Some(Action::WorktreeSelectPrev),
            Char('a') => return Some(Action::EnterAndroidPalette),
            Char('i') => return Some(Action::EnterIosPalette),
            Char('x') => return Some(Action::EnterCleanPalette),
            Char('s') => return Some(Action::EnterSyncPalette),
            Char('g') => return Some(Action::EnterGitPalette),
            Char('C') => return Some(Action::OpenClaudeCode),
            Char('L') => return Some(Action::StartSetLabel),
            Char('f') => return Some(Action::ToggleFullscreen),
            Char('!') => return Some(Action::StartShellCommand),
            Char('R') => return Some(Action::RefreshWorktrees),
            Enter => return Some(Action::WorktreeSwitchToSelected),
            _ => {}
        }
    }

    // --- COMMAND OUTPUT SPECIFIC ---
    if state.focused_panel == FocusedPanel::CommandOutput {
        match key.code {
            Char('j') | Down => return Some(Action::CommandOutputScrollDown),
            Char('k') | Up => return Some(Action::CommandOutputScrollUp),
            Char('G') => return Some(Action::ScrollToBottom),
            Char('g') => {
                if state.pending_g {
                    return Some(Action::ScrollToTop);
                } else {
                    return Some(Action::SetPendingG);
                }
            }
            Char('X') => return Some(Action::CommandCancel),
            Char('C') => return Some(Action::CommandOutputClear),
            Char('f') => return Some(Action::ToggleFullscreen),
            _ => {}
        }
    }

    // --- NORMAL MODE ---
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

/// Directly dispatches a command without going through the pre-processing pipeline.
/// Used by ModalConfirm to run confirmed destructive commands, and internally after
/// text-input and device-picker modals complete.
///
/// Appends separator to per-worktree output, sets running_command, spawns the process task.
fn dispatch_command(
    state: &mut AppState,
    spec: CommandSpec,
    metro_tx: &tokio::sync::mpsc::UnboundedSender<Action>,
) {
    let wt = if !state.worktrees.is_empty() {
        let idx = state.worktree_table_state.selected().unwrap_or(0);
        let idx = idx.min(state.worktrees.len() - 1);
        state.worktrees[idx].clone()
    } else {
        // No worktrees loaded yet — can't dispatch; log to a fallback message (no per-worktree key)
        tracing::warn!("dispatch_command: no worktree selected, dropping command {:?}", spec.label());
        return;
    };

    // Append a separator line to per-worktree output — output persists, not cleared on new command
    let wt_id = wt.id.clone();
    let output = state.command_output_by_worktree.entry(wt_id.clone()).or_default();
    output.push_back(format!("--- {} ---", spec.label()));
    // Cap per-worktree output at MAX_COMMAND_LINES
    while output.len() > MAX_COMMAND_LINES {
        output.pop_front();
    }
    // Reset scroll for this worktree to show the latest output
    state.command_output_scroll_by_worktree.insert(wt_id, 0);

    state.running_command = Some(spec.clone());

    // Abort any existing command task
    if let Some(task) = state.command_task.take() {
        task.abort();
    }

    let tx = metro_tx.clone();
    let path = wt.path.clone();
    let branch = wt.branch.clone();
    let spec_for_task = spec.clone();

    // spawn_command_task is async so we wrap it in a nested spawn
    let handle = tokio::spawn(async move {
        let task = crate::infra::command_runner::spawn_command_task(spec_for_task, path, branch, tx).await;
        let _ = task.await;
    });
    state.command_task = Some(handle);
}

/// TEA update function — the ONLY place AppState is mutated.
///
/// `metro_tx` and `handle_tx` are channels that connect update() to the async runtime:
/// - `metro_tx`: background tasks send Action events back to the loop
/// - `handle_tx`: spawn task sends the MetroHandle back so it can be registered in AppState
///
/// Async operations are always dispatched via tokio::spawn — update() never awaits.
pub fn update(
    state: &mut AppState,
    action: Action,
    metro_tx: &tokio::sync::mpsc::UnboundedSender<Action>,
    handle_tx: &tokio::sync::mpsc::UnboundedSender<MetroHandle>,
) {
    // Clear pending_g on any action except SetPendingG
    if !matches!(action, Action::SetPendingG) {
        state.pending_g = false;
    }

    match action {
        // Phase 1 actions
        Action::FocusNext => state.focused_panel = state.focused_panel.next(),
        Action::FocusPrev => state.focused_panel = state.focused_panel.prev(),
        Action::FocusUp => {
            if state.focused_panel == FocusedPanel::CommandOutput {
                if let Some(id) = active_worktree_id(state) {
                    let scroll = state.command_output_scroll_by_worktree.entry(id).or_insert(0);
                    *scroll = scroll.saturating_sub(1);
                }
            }
        }
        Action::FocusDown => {
            if state.focused_panel == FocusedPanel::CommandOutput {
                let max = active_output(state).len();
                if let Some(id) = active_worktree_id(state) {
                    let scroll = state.command_output_scroll_by_worktree.entry(id).or_insert(0);
                    if *scroll < max {
                        *scroll += 1;
                    }
                }
            }
        }
        Action::FocusLeft => {}
        Action::FocusRight => {}
        Action::Search => {
            // Phase 4+: stub
        }
        Action::ShowHelp => state.show_help = true,
        Action::DismissHelp => state.show_help = false,
        Action::DismissError => state.error_state = None,
        Action::RetryLastCommand => {
            state.error_state = None;
        }
        Action::Quit => state.should_quit = true,

        // --- Metro control actions ---

        Action::MetroStart => {
            if state.metro.is_running() {
                state.pending_restart = true;
                update(state, Action::MetroStop, metro_tx, handle_tx);
                return;
            }
            // Check for external metro conflict before spawning
            let tx = metro_tx.clone();
            tokio::spawn(async move {
                if let Some(info) = crate::infra::port::detect_external_metro(8081).await {
                    let _ = tx.send(Action::ExternalMetroDetected(info));
                } else {
                    let _ = tx.send(Action::MetroStartConfirmed);
                }
            });
        }

        Action::MetroStartConfirmed => {
            state.metro.set_starting();
            let tx = metro_tx.clone();
            let htx = handle_tx.clone();
            let worktree_path = state
                .active_worktree_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("."));
            tokio::spawn(spawn_metro_task(worktree_path, tx, htx));
        }

        Action::MetroStop => {
            if let Some(mut handle) = state.metro.take_handle() {
                state.metro.set_stopping();
                if let Some(kill_tx) = handle.kill_tx.take() {
                    let _ = kill_tx.send(());
                }
                handle.stdin_task.abort();
            }
        }

        Action::MetroRestart => {
            if state.metro.is_running() {
                state.pending_restart = true;
                update(state, Action::MetroStop, metro_tx, handle_tx);
            } else {
                update(state, Action::MetroStart, metro_tx, handle_tx);
            }
        }

        Action::MetroSendDebugger => {
            match state.metro.send_stdin(b"j\n".to_vec()) {
                Ok(()) => {
                    state.metro_logs.push_back("[debugger toggle sent (j)]".into());
                    if state.metro_logs.len() > MAX_LOG_LINES {
                        state.metro_logs.pop_front();
                    }
                }
                Err(e) => {
                    tracing::warn!("send debugger failed: {e}");
                    state.metro_logs.push_back(format!("[debugger send failed: {e}]"));
                    if state.metro_logs.len() > MAX_LOG_LINES {
                        state.metro_logs.pop_front();
                    }
                }
            }
        }

        Action::MetroSendReload => {
            if let Err(e) = state.metro.send_stdin(b"r\n".to_vec()) {
                tracing::warn!("send reload failed: {e}");
            }
        }

        Action::MetroToggleLog => {
            state.log_panel_visible = !state.log_panel_visible;
        }

        Action::MetroScrollUp => {
            state.metro_log_auto_follow = false;
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
            if state.pending_restart {
                state.pending_restart = false;
                // Consume pending_switch_path if set (worktree switch takes priority)
                if let Some(path) = state.pending_switch_path.take() {
                    state.active_worktree_path = Some(path);
                }
                update(state, Action::MetroStart, metro_tx, handle_tx);
            }
        }

        Action::MetroSpawnFailed(msg) => {
            state.metro.clear();
            state.pending_restart = false;
            state.pending_switch_path = None;
            state.error_state = Some(ErrorState {
                message: format!("Metro failed to start: {msg}"),
                can_retry: true,
            });
        }

        Action::ExternalMetroDetected(info) => {
            state.modal = Some(ModalState::ExternalMetroConflict {
                pid: info.pid,
                working_dir: info.working_dir,
            });
        }

        Action::KillExternalMetro(pid) => {
            state.modal = None;
            let tx = metro_tx.clone();
            tokio::spawn(async move {
                let _ = crate::infra::port::kill_process(pid).await;
                // Wait briefly for port to free, then auto-start metro
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let _ = tx.send(Action::MetroStartConfirmed);
            });
        }

        // --- Phase 3: Worktree navigation ---

        Action::WorktreeSelectNext => {
            let len = state.worktrees.len();
            if len > 0 {
                let i = state.worktree_table_state.selected().unwrap_or(0);
                let next = if i >= len - 1 { 0 } else { i + 1 };
                state.worktree_table_state.select(Some(next));
                // Update stable selection id
                state.selected_worktree_id = Some(state.worktrees[next].id.clone());
                // Update active worktree for metro
                state.active_worktree_path = Some(state.worktrees[next].path.clone());
            }
        }

        Action::WorktreeSelectPrev => {
            let len = state.worktrees.len();
            if len > 0 {
                let i = state.worktree_table_state.selected().unwrap_or(0);
                let prev = if i == 0 { len - 1 } else { i - 1 };
                state.worktree_table_state.select(Some(prev));
                // Update stable selection id
                state.selected_worktree_id = Some(state.worktrees[prev].id.clone());
                // Update active worktree for metro
                state.active_worktree_path = Some(state.worktrees[prev].path.clone());
            }
        }

        Action::WorktreesLoaded(mut worktrees) => {
            // Apply labels from state.labels: set wt.label for each worktree
            for wt in &mut worktrees {
                wt.label = state.labels.get(&wt.branch).cloned();
            }

            // Phase 4: re-apply cached JIRA titles
            for wt in &mut worktrees {
                if let Some(key) = crate::infra::jira::extract_jira_key(&wt.branch) {
                    if let Some(title) = state.jira_title_cache.get(&key) {
                        wt.jira_title = Some(title.clone());
                    }
                }
            }

            // Derive metro_status from current MetroManager state
            if let crate::domain::metro::MetroStatus::Running { ref worktree_id, .. } = state.metro.status {
                for wt in &mut worktrees {
                    let wt_name = wt.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    if wt_name == worktree_id {
                        wt.metro_status = crate::domain::worktree::WorktreeMetroStatus::Running;
                    }
                }
            }

            // Pin metro-active worktree to top of the list
            if let Some(metro_idx) = worktrees.iter().position(|wt| {
                wt.metro_status == crate::domain::worktree::WorktreeMetroStatus::Running
            }) {
                if metro_idx != 0 {
                    let metro_wt = worktrees.remove(metro_idx);
                    worktrees.insert(0, metro_wt);
                }
            }

            state.worktrees = worktrees;

            if !state.worktrees.is_empty() {
                // Re-derive selected index from selected_worktree_id (stable across sorts)
                let selected_idx = state
                    .selected_worktree_id
                    .as_ref()
                    .and_then(|id| state.worktrees.iter().position(|wt| &wt.id == id))
                    .unwrap_or(0);
                state.worktree_table_state.select(Some(selected_idx));
                state.active_worktree_path = Some(state.worktrees[selected_idx].path.clone());
            }

            // Phase 4: fetch titles for uncached branches
            if let Some(ref client) = state.jira_client {
                let keys_to_fetch: Vec<(String, String)> = state.worktrees.iter()
                    .filter_map(|wt| {
                        let key = crate::infra::jira::extract_jira_key(&wt.branch)?;
                        if state.jira_title_cache.contains_key(&key) { return None; }
                        Some((wt.branch.clone(), key))
                    })
                    .collect();

                if !keys_to_fetch.is_empty() {
                    let client = std::sync::Arc::clone(client);
                    let tx = metro_tx.clone();
                    tokio::spawn(async move {
                        let mut results = vec![];
                        for (_branch, key) in keys_to_fetch {
                            if let Some(title) = client.fetch_title(&key).await {
                                results.push((key, title));
                            }
                        }
                        if !results.is_empty() {
                            let _ = tx.send(Action::JiraTitlesFetched(results));
                        }
                    });
                }
            }
        }

        Action::RefreshWorktrees => {
            let repo_root = state.repo_root.clone();
            let tx = metro_tx.clone();
            tokio::spawn(async move {
                match crate::infra::worktrees::list_worktrees(&repo_root).await {
                    Ok(wts) => {
                        let _ = tx.send(Action::WorktreesLoaded(wts));
                    }
                    Err(e) => {
                        tracing::warn!("list_worktrees failed: {e}");
                    }
                }
            });
        }

        // --- Phase 3: Command dispatch ---

        Action::CommandRun(spec) => {
            // Clear palette mode whenever a command is dispatched
            state.palette_mode = None;

            // Get selected worktree (needed for all branches)
            let wt_branch = if !state.worktrees.is_empty() {
                let idx = state.worktree_table_state.selected().unwrap_or(0);
                let idx = idx.min(state.worktrees.len() - 1);
                Some((state.worktrees[idx].branch.clone(), state.worktrees[idx].stale))
            } else {
                None
            };

            // Sync-before-run: stale worktree + run command triggers prompt
            if let Some((_, stale)) = &wt_branch {
                if *stale {
                    if matches!(spec, CommandSpec::RnRunAndroid { .. } | CommandSpec::RnRunIos { .. } | CommandSpec::RnReleaseBuild) {
                        let needs_pods = matches!(spec, CommandSpec::RnRunIos { .. });
                        // Also check pods staleness for iOS
                        let needs_pods = if needs_pods {
                            let idx = state.worktree_table_state.selected().unwrap_or(0);
                            let wt_path = &state.worktrees[idx.min(state.worktrees.len() - 1)].path;
                            crate::infra::worktrees::check_stale_pods(wt_path)
                        } else {
                            false
                        };
                        state.modal = Some(ModalState::SyncBeforeRun {
                            run_command: Box::new(spec),
                            needs_pods,
                        });
                        state.palette_mode = None;
                        return;
                    }
                }
            }

            // Pre-processing pipeline
            if spec.is_destructive() {
                let branch_name = wt_branch
                    .map(|(b, _)| b)
                    .unwrap_or_else(|| "(unknown)".to_string());
                state.modal = Some(ModalState::Confirm {
                    prompt: format!("Run '{}' on {}?", spec.label(), branch_name),
                    pending_command: spec,
                });
                return;
            }

            if spec.needs_text_input() {
                let prompt = match &spec {
                    CommandSpec::GitRebase { .. } => "Rebase onto:".to_string(),
                    CommandSpec::GitCheckout { .. } => "Branch to checkout:".to_string(),
                    CommandSpec::GitCheckoutNew { .. } => "New branch name:".to_string(),
                    CommandSpec::YarnJest { .. } => "Jest filter:".to_string(),
                    _ => "Input:".to_string(),
                };
                state.modal = Some(ModalState::TextInput {
                    prompt,
                    buffer: String::new(),
                    pending_template: Box::new(spec),
                });
                return;
            }

            if spec.needs_device_selection() {
                state.pending_device_command = Some(spec.clone());
                let tx = metro_tx.clone();
                let is_android = matches!(spec, CommandSpec::RnRunAndroid { .. });
                tokio::spawn(async move {
                    let devices = if is_android {
                        crate::infra::devices::list_android_devices().await
                    } else {
                        crate::infra::devices::list_ios_physical_devices().await
                    };
                    match devices {
                        Ok(devs) => {
                            let _ = tx.send(Action::DevicesEnumerated(devs));
                        }
                        Err(e) => {
                            tracing::warn!("device enumeration failed: {e}");
                            let _ = tx.send(Action::DevicesEnumerated(vec![]));
                        }
                    }
                });
                return;
            }

            // Android release build: queue adb install to run after assembleRelease completes
            if matches!(spec, CommandSpec::RnReleaseBuild) {
                state.command_queue.push_back(CommandSpec::AdbInstallApk);
                dispatch_command(state, spec, metro_tx);
                return;
            }

            // GitResetHardFetch: two-step — dispatch fetch, queue reset --hard origin/<branch>
            if matches!(spec, CommandSpec::GitResetHardFetch) {
                state.command_queue.push_back(CommandSpec::GitResetHard);
                dispatch_command(state, CommandSpec::GitFetch, metro_tx);
                return;
            }

            // Normal dispatch
            dispatch_command(state, spec, metro_tx);
        }

        // --- Phase 3: Command output events ---

        Action::CommandOutputLine(line) => {
            if let Some(id) = active_worktree_id(state) {
                let output = state.command_output_by_worktree.entry(id).or_default();
                output.push_back(line);
                if output.len() > MAX_COMMAND_LINES {
                    output.pop_front();
                }
            }
        }

        Action::CommandExited => {
            let completed_cmd = state.running_command.take();
            state.command_task = None;

            // Drain command queue — pop_front and dispatch if non-empty
            if let Some(next_spec) = state.command_queue.pop_front() {
                dispatch_command(state, next_spec, metro_tx);
            }

            // Dispatch post-command refreshes based on the completed command
            if let Some(ref cmd) = completed_cmd {
                let refresh = crate::domain::refresh::refresh_needed(cmd);
                if refresh.worktrees {
                    // Full worktree reload (also re-checks staleness and triggers JIRA fetch
                    // via WorktreesLoaded handler when branch names change)
                    let repo_root = state.repo_root.clone();
                    let tx = metro_tx.clone();
                    tokio::spawn(async move {
                        match crate::infra::worktrees::list_worktrees(&repo_root).await {
                            Ok(wts) => { let _ = tx.send(Action::WorktreesLoaded(wts)); }
                            Err(e) => { tracing::warn!("post-command worktree refresh failed: {e}"); }
                        }
                    });
                } else if refresh.staleness {
                    // Staleness refresh: re-check ALL worktrees (cheap I/O, ensures
                    // correct state even if user changed selection during command)
                    for wt in state.worktrees.iter_mut() {
                        wt.stale = crate::infra::worktrees::check_stale(&wt.path);
                        wt.stale_pods = crate::infra::worktrees::check_stale_pods(&wt.path);
                    }
                }
            }
        }

        Action::CommandOutputClear => {
            if let Some(id) = active_worktree_id(state) {
                state.command_output_by_worktree.remove(&id);
                state.command_output_scroll_by_worktree.remove(&id);
            }
        }

        Action::CommandCancel => {
            if let Some(task) = state.command_task.take() {
                task.abort();
            }
            state.running_command = None;
            // Also clear pending queue items — cancel is all-or-nothing
            state.command_queue.clear();
            if let Some(id) = active_worktree_id(state) {
                let output = state.command_output_by_worktree.entry(id).or_default();
                output.push_back("[cancelled]".into());
                if output.len() > MAX_COMMAND_LINES {
                    output.pop_front();
                }
            }
        }

        // --- Phase 5.1: Command queue actions ---

        Action::CommandQueuePush(spec) => {
            state.command_queue.push_back(spec);
        }

        Action::CommandQueueClear => {
            state.command_queue.clear();
        }

        // --- Phase 3: Modal actions ---

        Action::ShowCommandPalette => {
            // Palette activation is handled via EnterGitPalette / EnterRnPalette.
            // This variant is kept for backward compatibility.
        }

        Action::ModalConfirm => {
            if let Some(ModalState::Confirm { pending_command, .. }) = state.modal.take() {
                // Dispatch directly — skip pre-processing (already confirmed)
                dispatch_command(state, pending_command, metro_tx);
            }
        }

        Action::ModalCancel => {
            state.modal = None;
            state.palette_mode = None;
        }

        Action::ModalInputChar(c) => {
            match state.modal.as_mut() {
                Some(ModalState::TextInput { buffer, .. }) => {
                    buffer.push(c);
                }
                Some(ModalState::DevicePicker { filter, selected, .. }) => {
                    filter.push(c);
                    *selected = 0; // reset selection when filter changes
                }
                _ => {}
            }
        }

        Action::ModalInputBackspace => {
            match state.modal.as_mut() {
                Some(ModalState::TextInput { buffer, .. }) => {
                    buffer.pop();
                }
                Some(ModalState::DevicePicker { filter, selected, .. }) => {
                    filter.pop();
                    *selected = 0; // reset selection when filter changes
                }
                _ => {}
            }
        }

        Action::ModalInputSubmit => {
            if let Some(modal) = state.modal.take() {
                match modal {
                    ModalState::TextInput {
                        buffer,
                        pending_template,
                        ..
                    } => {
                        // Check if this is a label submit
                        if let Some(branch) = state.pending_label_branch.take() {
                            update(
                                state,
                                Action::SetLabel {
                                    branch,
                                    label: buffer,
                                },
                                metro_tx,
                                handle_tx,
                            );
                        } else {
                            // Build the real CommandSpec by filling in the text
                            let real_spec = match *pending_template {
                                CommandSpec::GitRebase { .. } => {
                                    CommandSpec::GitRebase { target: buffer }
                                }
                                CommandSpec::GitCheckout { .. } => {
                                    CommandSpec::GitCheckout { branch: buffer }
                                }
                                CommandSpec::GitCheckoutNew { .. } => {
                                    CommandSpec::GitCheckoutNew { branch: buffer }
                                }
                                CommandSpec::YarnJest { .. } => {
                                    CommandSpec::YarnJest { filter: buffer }
                                }
                                CommandSpec::ShellCommand { .. } => {
                                    CommandSpec::ShellCommand { command: buffer }
                                }
                                other => other,
                            };
                            dispatch_command(state, real_spec, metro_tx);
                        }
                    }
                    other => {
                        // Restore modal if wrong type (shouldn't happen)
                        state.modal = Some(other);
                    }
                }
            }
        }

        Action::ModalDeviceNext => {
            if let Some(ModalState::DevicePicker {
                ref devices,
                ref mut selected,
                ref filter,
                ..
            }) = state.modal
            {
                let count = if filter.is_empty() {
                    devices.len()
                } else {
                    let lower = filter.to_lowercase();
                    devices.iter().filter(|d| d.name.to_lowercase().contains(&lower)).count()
                };
                if count > 0 {
                    *selected = if *selected >= count - 1 { 0 } else { *selected + 1 };
                }
            }
        }

        Action::ModalDevicePrev => {
            if let Some(ModalState::DevicePicker {
                ref devices,
                ref mut selected,
                ref filter,
                ..
            }) = state.modal
            {
                let count = if filter.is_empty() {
                    devices.len()
                } else {
                    let lower = filter.to_lowercase();
                    devices.iter().filter(|d| d.name.to_lowercase().contains(&lower)).count()
                };
                if count > 0 {
                    *selected = if *selected == 0 { count - 1 } else { *selected - 1 };
                }
            }
        }

        Action::ModalDeviceConfirm => {
            if let Some(ModalState::DevicePicker {
                devices,
                selected,
                pending_template,
                filter,
            }) = state.modal.take()
            {
                // Apply filter to get the actual visible list (mirrors render logic)
                let filtered: Vec<&crate::domain::command::DeviceInfo> = if filter.is_empty() {
                    devices.iter().collect()
                } else {
                    let lower = filter.to_lowercase();
                    devices.iter().filter(|d| d.name.to_lowercase().contains(&lower)).collect()
                };
                if let Some(device) = filtered.get(selected) {
                    let device_id = device.id.clone();
                    let is_ios = matches!(pending_template.as_ref(), CommandSpec::RnRunIos { .. });
                    let real_spec = match *pending_template {
                        CommandSpec::RnRunAndroid { .. } => CommandSpec::RnRunAndroid {
                            device_id: device_id.clone(),
                        },
                        CommandSpec::RnRunIos { .. } => CommandSpec::RnRunIos {
                            device_id: device_id.clone(),
                        },
                        other => other,
                    };
                    // Record iOS simulator usage for sort-by-recent
                    if is_ios {
                        let _ = metro_tx.send(Action::SimulatorUsed(device_id));
                    }
                    dispatch_command(state, real_spec, metro_tx);
                }
            }
        }

        // --- Phase 3: Device enumeration (async callback) ---

        Action::DevicesEnumerated(devices) => {
            if let Some(spec) = state.pending_device_command.take() {
                match devices.len() {
                    0 => {
                        if let Some(id) = active_worktree_id(state) {
                            let output = state.command_output_by_worktree.entry(id).or_default();
                            output.push_back("[error] no devices found".into());
                        }
                    }
                    1 => {
                        // Only one device — skip picker
                        let real_spec = match spec {
                            CommandSpec::RnRunAndroid { .. } => CommandSpec::RnRunAndroid {
                                device_id: devices[0].id.clone(),
                            },
                            CommandSpec::RnRunIos { .. } => CommandSpec::RnRunIos {
                                device_id: devices[0].id.clone(),
                            },
                            other => other,
                        };
                        dispatch_command(state, real_spec, metro_tx);
                    }
                    _ => {
                        // Multiple devices — show picker
                        // Sort iOS simulators by last-used from sim_history
                        let mut sorted_devices = devices;
                        if matches!(spec, CommandSpec::RnRunIos { .. }) {
                            let history = crate::infra::sim_history::load_sim_history();
                            sorted_devices.sort_by_key(|d| {
                                history.iter().position(|h| h == &d.id)
                                    .unwrap_or(usize::MAX)
                            });
                        }
                        state.modal = Some(ModalState::DevicePicker {
                            devices: sorted_devices,
                            selected: 0,
                            pending_template: Box::new(spec),
                            filter: String::new(),
                        });
                    }
                }
            }
        }

        // --- Phase 3: Label management ---

        Action::SetLabel { branch, label } => {
            state.labels.insert(branch.clone(), label.clone());
            if let Err(e) = crate::infra::labels::save_labels(&state.labels) {
                tracing::warn!("save_labels failed: {e}");
            }
            // Update the matching worktree's label field in memory
            for wt in &mut state.worktrees {
                if wt.branch == branch {
                    wt.label = Some(label.clone());
                }
            }
        }

        Action::StartSetLabel => {
            if !state.worktrees.is_empty() {
                let idx = state.worktree_table_state.selected().unwrap_or(0);
                let idx = idx.min(state.worktrees.len() - 1);
                let branch = state.worktrees[idx].branch.clone();
                let current_label = state.worktrees[idx].label.clone().unwrap_or_default();
                state.pending_label_branch = Some(branch.clone());
                state.modal = Some(ModalState::TextInput {
                    prompt: format!("Label for {}:", branch),
                    buffer: current_label,
                    pending_template: Box::new(CommandSpec::YarnLint), // sentinel — not used
                });
            }
        }

        // --- Phase 3: Palette mode activation ---

        Action::EnterGitPalette => {
            state.palette_mode = Some(PaletteMode::Git);
        }

        Action::EnterRnPalette => {
            // EnterRnPalette kept for backward compat — Phase 05.1 will remap 'c' key
            // to new submenu scheme. For now we just cancel palette mode.
            state.palette_mode = None;
        }

        // --- Phase 5: Worktree switching and Claude Code ---

        Action::WorktreeSwitchToSelected => {
            // Capture target path NOW — navigation may change active_worktree_path later
            let target_path = state.worktrees
                .get(state.worktree_table_state.selected().unwrap_or(0))
                .map(|wt| wt.path.clone());

            if state.metro.is_running() {
                // Kill current → wait for port free → start in new worktree
                state.pending_switch_path = target_path;
                state.pending_restart = true;
                update(state, Action::MetroStop, metro_tx, handle_tx);
            } else {
                // Not running — just start directly in selected worktree
                if let Some(path) = target_path {
                    state.active_worktree_path = Some(path);
                }
                update(state, Action::MetroStart, metro_tx, handle_tx);
            }
        }

        Action::OpenClaudeCode => {
            if state.multiplexer.is_none() {
                state.error_state = Some(ErrorState {
                    message: "Cannot open Claude Code: not inside a tmux or zellij session".into(),
                    can_retry: false,
                });
                return;
            }
            let wt = if !state.worktrees.is_empty() {
                let idx = state.worktree_table_state.selected().unwrap_or(0)
                    .min(state.worktrees.len() - 1);
                state.worktrees[idx].clone()
            } else {
                return;
            };
            let path = wt.path.clone();
            let name = format!("claude-{}", wt.preferred_prefix());
            let flags = state.claude_flags.clone();
            let command = if flags.is_empty() {
                "claude".to_string()
            } else {
                format!("claude {}", flags)
            };
            tokio::task::spawn_blocking(move || {
                // Re-detect inside spawn — env vars don't change mid-session, and this avoids
                // the Clone issue with Box<dyn Multiplexer>.
                if let Some(mux) = crate::infra::multiplexer::detect_multiplexer() {
                    if let Err(e) = mux.new_window(&path, &name, &command) {
                        tracing::warn!("multiplexer new_window failed: {e}");
                    }
                }
            });
        }

        // --- Phase 4: JIRA title fetch results ---

        Action::JiraTitlesFetched(titles) => {
            // Update in-memory cache
            for (key, title) in &titles {
                state.jira_title_cache.insert(key.clone(), title.clone());
            }
            // Persist cache to disk (fire-and-forget, log on error)
            if let Err(e) = crate::infra::jira_cache::save_jira_cache(&state.jira_title_cache) {
                tracing::warn!("save_jira_cache failed: {e}");
            }
            // Apply titles to currently loaded worktrees
            for wt in &mut state.worktrees {
                if let Some(key) = crate::infra::jira::extract_jira_key(&wt.branch) {
                    if let Some(title) = state.jira_title_cache.get(&key) {
                        wt.jira_title = Some(title.clone());
                    }
                }
            }
        }

        // --- Phase 5.1: New submenu and action stubs ---

        Action::EnterAndroidPalette => {
            state.palette_mode = Some(PaletteMode::Android);
        }
        Action::EnterIosPalette => {
            state.palette_mode = Some(PaletteMode::Ios);
        }
        Action::EnterCleanPalette => {
            state.palette_mode = Some(PaletteMode::Clean);
            state.modal = Some(crate::domain::command::ModalState::CleanToggle {
                options: crate::domain::command::CleanOptions::default(),
            });
        }
        Action::EnterSyncPalette => {
            state.palette_mode = Some(PaletteMode::Sync);
        }
        Action::CleanToggleNodeModules => {
            if let Some(ModalState::CleanToggle { ref mut options }) = state.modal {
                options.node_modules = !options.node_modules;
            }
        }
        Action::CleanTogglePods => {
            if let Some(ModalState::CleanToggle { ref mut options }) = state.modal {
                options.pods = !options.pods;
            }
        }
        Action::CleanToggleAndroid => {
            if let Some(ModalState::CleanToggle { ref mut options }) = state.modal {
                options.android = !options.android;
            }
        }
        Action::CleanToggleSyncAfter => {
            if let Some(ModalState::CleanToggle { ref mut options }) = state.modal {
                options.sync_after = !options.sync_after;
            }
        }
        Action::CleanConfirm => {
            if let Some(ModalState::CleanToggle { options }) = state.modal.take() {
                state.palette_mode = None;

                // Build command sequence from checked options
                let mut cmds: Vec<CommandSpec> = Vec::new();
                if options.node_modules {
                    cmds.push(CommandSpec::RmNodeModules);
                }
                if options.pods {
                    cmds.push(CommandSpec::RnCleanCocoapods);
                }
                if options.android {
                    cmds.push(CommandSpec::RnCleanAndroid);
                }
                if options.sync_after {
                    cmds.push(CommandSpec::YarnInstall);
                    cmds.push(CommandSpec::YarnPodInstall);
                }

                if cmds.is_empty() {
                    return;
                }

                // Dispatch first, queue rest
                let first = cmds.remove(0);
                for cmd in cmds {
                    state.command_queue.push_back(cmd);
                }
                dispatch_command(state, first, metro_tx);
            }
        }
        Action::ToggleFullscreen => {
            if state.fullscreen_panel.is_some() {
                state.fullscreen_panel = None;
                state.focused_panel = state.focused_panel.next();
            } else {
                // Only MetroPane and CommandOutput can be fullscreened
                match state.focused_panel {
                    FocusedPanel::MetroPane | FocusedPanel::CommandOutput => {
                        state.fullscreen_panel = Some(state.focused_panel);
                    }
                    _ => {} // no-op for WorktreeTable
                }
            }
        }
        Action::StartShellCommand => {
            state.modal = Some(ModalState::TextInput {
                prompt: "Shell command:".to_string(),
                buffer: String::new(),
                pending_template: Box::new(CommandSpec::ShellCommand { command: String::new() }),
            });
        }
        Action::SimulatorUsed(udid) => {
            // Fire-and-forget write to sim history
            tokio::task::spawn_blocking(move || {
                if let Err(e) = crate::infra::sim_history::record_sim_used(&udid) {
                    tracing::warn!("failed to save sim history: {e}");
                }
            });
        }
        Action::SyncBeforeRunAccept => {
            if let Some(ModalState::SyncBeforeRun { run_command, needs_pods }) = state.modal.take() {
                // Enqueue: yarn install, (pod-install if needs_pods), then the run command
                state.command_queue.push_back(*run_command);
                if needs_pods {
                    // Re-order: yarn install first, pod-install, then run
                    let run = state.command_queue.pop_back().unwrap();
                    state.command_queue.push_back(CommandSpec::YarnPodInstall);
                    state.command_queue.push_back(run);
                }
                dispatch_command(state, CommandSpec::YarnInstall, metro_tx);
            }
        }
        Action::SyncBeforeRunDecline => {
            if let Some(ModalState::SyncBeforeRun { run_command, .. }) = state.modal.take() {
                // Skip sync, dispatch run command directly
                dispatch_command(state, *run_command, metro_tx);
            }
        }
        Action::LogPanelClear => {
            state.metro_logs.clear();
            state.log_scroll_offset = 0;
            state.metro_log_auto_follow = true;
        }

        // --- Phase 5.2: Universal scroll ---

        Action::ScrollToTop => {
            match state.focused_panel {
                FocusedPanel::MetroPane => {
                    state.log_scroll_offset = 0;
                    state.metro_log_auto_follow = false;
                }
                FocusedPanel::CommandOutput => {
                    if let Some(id) = active_worktree_id(state) {
                        state.command_output_scroll_by_worktree.insert(id, 0);
                    }
                }
                _ => {}
            }
        }

        Action::ScrollToBottom => {
            match state.focused_panel {
                FocusedPanel::MetroPane => {
                    // Set offset to max so rendering shows the bottom
                    let max = state.metro_logs.len();
                    state.log_scroll_offset = max;
                    state.metro_log_auto_follow = true;
                }
                FocusedPanel::CommandOutput => {
                    if let Some(id) = active_worktree_id(state) {
                        let max = state.command_output_by_worktree
                            .get(&id)
                            .map(|o| o.len())
                            .unwrap_or(0);
                        state.command_output_scroll_by_worktree.insert(id, max);
                    }
                }
                _ => {}
            }
        }

        Action::SetPendingG => {
            state.pending_g = true;
        }

        Action::CommandOutputScrollUp => {
            if let Some(id) = active_worktree_id(state) {
                let scroll = state.command_output_scroll_by_worktree.entry(id).or_insert(0);
                *scroll = scroll.saturating_sub(1);
            }
        }

        Action::CommandOutputScrollDown => {
            let max = active_output(state).len();
            if let Some(id) = active_worktree_id(state) {
                let scroll = state.command_output_scroll_by_worktree.entry(id).or_insert(0);
                if *scroll < max {
                    *scroll += 1;
                }
            }
        }
    }
}

/// Main application loop. Runs on the tokio runtime.
/// Renders on every event and on a 250ms tick. Exits when state.should_quit is true.
pub async fn run(mut terminal: ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut state = AppState::default();
    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(250));

    // Channel for background tasks (log lines, MetroExited, WorktreesLoaded, etc.)
    let (metro_tx, mut metro_rx) = tokio::sync::mpsc::unbounded_channel::<Action>();

    // Channel for the spawn task to deliver the MetroHandle once spawning is complete.
    let (handle_tx, mut handle_rx) = tokio::sync::mpsc::unbounded_channel::<MetroHandle>();

    // Load labels from disk on startup
    state.labels = crate::infra::labels::load_labels().unwrap_or_default();

    // Phase 5.1: multiplexer detection (replaces tmux_available bool)
    state.multiplexer = crate::infra::multiplexer::detect_multiplexer();

    // Phase 4: Load config + JIRA client + cache
    if let Ok(Some(config)) = crate::infra::config::load_config() {
        // Extract claude_flags before moving config
        state.claude_flags = config.claude_flags.clone();

        match crate::infra::jira::HttpJiraClient::new(&config) {
            Ok(client) => {
                state.jira_client = Some(std::sync::Arc::new(client));
            }
            Err(e) => {
                tracing::warn!("JIRA client init failed: {e}");
            }
        }
        state.config = Some(config);
    }
    state.jira_title_cache = crate::infra::jira_cache::load_jira_cache().unwrap_or_default();

    // Spawn initial worktree load
    {
        let repo_root = state.repo_root.clone();
        let init_tx = metro_tx.clone();
        tokio::spawn(async move {
            match crate::infra::worktrees::list_worktrees(&repo_root).await {
                Ok(wts) => {
                    let _ = init_tx.send(Action::WorktreesLoaded(wts));
                }
                Err(e) => {
                    tracing::warn!("initial worktree load failed: {e}");
                }
            }
        });
    }

    // Check for external metro on startup
    {
        let startup_tx = metro_tx.clone();
        tokio::spawn(async move {
            if let Some(info) = crate::infra::port::detect_external_metro(8081).await {
                let _ = startup_tx.send(Action::ExternalMetroDetected(info));
            }
        });
    }

    loop {
        // Render once per iteration — after all pending actions have been drained
        terminal.draw(|f| crate::ui::view(f, &mut state))?;

        // Wait for at least one event (blocks until something happens)
        tokio::select! {
            _ = tick.tick() => {
                // Periodic tick: triggers redraw for time-based UI updates
            }
            maybe_event = events.next() => {
                let Some(Ok(event)) = maybe_event else { break };
                use ratatui::crossterm::event::Event as CE;
                match event {
                    CE::Key(key) => {
                        if let Some(action) = handle_key(&state, key) {
                            update(&mut state, action, &metro_tx, &handle_tx);
                        }
                    }
                    CE::Resize(_, _) => {}
                    _ => {}
                }
            }
            Some(action) = metro_rx.recv() => {
                update(&mut state, action, &metro_tx, &handle_tx);
            }
            Some(handle) = handle_rx.recv() => {
                state.metro.register(handle);
            }
        }

        // Drain all pending actions before redrawing — batches bursts of log lines
        // into a single frame instead of redrawing per-line
        loop {
            use tokio::sync::mpsc::error::TryRecvError;
            match metro_rx.try_recv() {
                Ok(action) => update(&mut state, action, &metro_tx, &handle_tx),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
            match handle_rx.try_recv() {
                Ok(handle) => state.metro.register(handle),
                Err(_) => {}
            }
        }

        if state.should_quit {
            break;
        }
    }

    // Cleanup: kill all child processes before exiting
    if let Some(mut handle) = state.metro.take_handle() {
        if let Some(kill_tx) = handle.kill_tx.take() {
            let _ = kill_tx.send(());
        }
        handle.stream_task.abort();
        handle.stdin_task.abort();
    }
    if let Some(task) = state.command_task.take() {
        task.abort();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Async helpers — all run inside tokio::spawn, never blocking the event loop
// ---------------------------------------------------------------------------

/// Spawns the metro process and delivers a `MetroHandle` via `handle_tx`.
async fn spawn_metro_task(
    worktree_path: PathBuf,
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    handle_tx: tokio::sync::mpsc::UnboundedSender<MetroHandle>,
) {
    use crate::infra::process::ProcessClient;
    use crate::infra::process::TokioProcessClient;

    let client = TokioProcessClient;
    match client.spawn_metro(worktree_path.clone()).await {
        Ok(mut child) => {
            let pid = child.id().unwrap_or(0);

            let stdout = child.stdout.take().expect("stdout piped");
            let stderr = child.stderr.take().expect("stderr piped");
            let stdin = child.stdin.take().expect("stdin piped");

            let (stdin_tx, stdin_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
            let stdin_task = tokio::spawn(stdin_writer(stdin, stdin_rx));

            let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();

            let stream_tx = action_tx.clone();
            let stream_task =
                tokio::spawn(metro_process_task(child, stdout, stderr, kill_rx, stream_tx));

            let worktree_id = worktree_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let handle = MetroHandle {
                pid,
                worktree_id,
                stdin_tx,
                stream_task,
                stdin_task,
                kill_tx: Some(kill_tx),
            };

            let _ = handle_tx.send(handle);
        }
        Err(e) => {
            tracing::error!("metro spawn failed: {e}");
            let _ = action_tx.send(Action::MetroSpawnFailed(format!("{e}")));
        }
    }
}

/// Owns the `Child` process. Handles kill signal and natural exit.
async fn metro_process_task(
    mut child: tokio::process::Child,
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    kill_rx: tokio::sync::oneshot::Receiver<()>,
    tx: tokio::sync::mpsc::UnboundedSender<Action>,
) {
    let log_tx = tx.clone();
    let log_task = tokio::spawn(stream_metro_logs(stdout, stderr, log_tx));

    tokio::select! {
        _ = kill_rx => {
            log_task.abort();
            if let Err(e) = child.kill().await {
                tracing::error!("metro kill failed: {e}");
            }
            for _ in 0..50 {
                if crate::infra::port::port_is_free(8081) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            let _ = tx.send(Action::MetroExited);
        }
        _ = child.wait() => {
            log_task.abort();
            let _ = tx.send(Action::MetroExited);
        }
    }
}

/// Reads stdout and stderr lines from the metro process and sends them as `MetroLogLine`.
async fn stream_metro_logs(
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    tx: tokio::sync::mpsc::UnboundedSender<Action>,
) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(Action::MetroLogLine(l)); }
                    _ => break,
                }
            }
            line = stderr_lines.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(Action::MetroLogLine(l)); }
                    _ => break,
                }
            }
        }
    }
}

/// Forwards byte buffers from the `rx` channel to the child's stdin handle.
async fn stdin_writer(
    mut stdin: tokio::process::ChildStdin,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) {
    use tokio::io::AsyncWriteExt;
    while let Some(bytes) = rx.recv().await {
        if let Err(e) = stdin.write_all(&bytes).await {
            tracing::warn!("metro stdin write failed: {e}");
            break;
        }
    }
}
