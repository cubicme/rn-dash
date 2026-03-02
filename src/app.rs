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
    WorktreeList,
    MetroPane,
    CommandOutput,
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

/// Whether the command palette is in git or RN mode.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteMode {
    /// 'g' was pressed — next key selects a git command.
    Git,
    /// 'c' was pressed — next key selects an RN command.
    Rn,
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
    pub worktree_list_state: ratatui::widgets::ListState,

    // Command output panel
    pub command_output: std::collections::VecDeque<String>,
    pub command_output_scroll: usize,
    pub running_command: Option<crate::domain::command::CommandSpec>,
    pub command_task: Option<tokio::task::JoinHandle<()>>,

    // Lazy install (WORK-06): run yarn install before run-android/run-ios if worktree is stale
    pub pending_command_after_install: Option<crate::domain::command::CommandSpec>,

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
    pub tmux_available: bool,
    pub jira_title_cache: std::collections::HashMap<String, String>,  // UMP-XXXX -> title
    pub jira_client: Option<std::sync::Arc<dyn crate::infra::jira::JiraClient>>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut worktree_list_state = ratatui::widgets::ListState::default();
        worktree_list_state.select(Some(0));
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
            pending_restart: false,
            pending_switch_path: None,
            // Phase 3
            worktrees: Vec::new(),
            worktree_list_state,
            command_output: std::collections::VecDeque::new(),
            command_output_scroll: 0,
            running_command: None,
            command_task: None,
            pending_command_after_install: None,
            modal: None,
            labels: std::collections::HashMap::new(),
            repo_root: PathBuf::from(
                std::env::var("HOME").unwrap_or_else(|_| ".".into()),
            )
            .join("aljazeera/ump"),
            palette_mode: None,
            pending_device_command: None,
            pending_label_branch: None,
            // Phase 4
            tmux_available: false,  // set properly in run()
            jira_title_cache: std::collections::HashMap::new(),
            jira_client: None,
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
                Char('j') | Down => Some(Action::ModalDeviceNext),
                Char('k') | Up => Some(Action::ModalDevicePrev),
                _ => None,
            },
        };
    }

    // --- PALETTE MODE ROUTING — after modal, before overlays ---
    if let Some(ref mode) = state.palette_mode {
        return match mode {
            PaletteMode::Git => match key.code {
                Char('p') => Some(Action::CommandRun(CommandSpec::GitPull)),
                Char('P') => Some(Action::CommandRun(CommandSpec::GitPush)),
                Char('d') => Some(Action::CommandRun(CommandSpec::GitResetHard)),
                Char('b') => Some(Action::CommandRun(CommandSpec::GitCheckout {
                    branch: String::new(),
                })),
                Char('B') => Some(Action::CommandRun(CommandSpec::GitCheckoutNew {
                    branch: String::new(),
                })),
                Char('r') => Some(Action::CommandRun(CommandSpec::GitRebase {
                    target: String::new(),
                })),
                Esc => Some(Action::ModalCancel), // exits palette mode
                _ => Some(Action::ModalCancel),   // unknown key exits palette
            },
            PaletteMode::Rn => match key.code {
                Char('a') => Some(Action::CommandRun(CommandSpec::RnCleanAndroid)),
                Char('c') => Some(Action::CommandRun(CommandSpec::RnCleanCocoapods)),
                Char('n') => Some(Action::CommandRun(CommandSpec::RmNodeModules)),
                Char('i') => Some(Action::CommandRun(CommandSpec::YarnInstall)),
                Char('p') => Some(Action::CommandRun(CommandSpec::YarnPodInstall)),
                Char('d') => Some(Action::CommandRun(CommandSpec::RnRunAndroid {
                    device_id: String::new(),
                })),
                Char('s') => Some(Action::CommandRun(CommandSpec::RnRunIos {
                    device_id: String::new(),
                })),
                Char('t') => Some(Action::CommandRun(CommandSpec::YarnUnitTests)),
                Char('j') => Some(Action::CommandRun(CommandSpec::YarnJest {
                    filter: String::new(),
                })),
                Char('l') => Some(Action::CommandRun(CommandSpec::YarnLint)),
                Char('y') => Some(Action::CommandRun(CommandSpec::YarnCheckTypes)),
                Esc => Some(Action::ModalCancel), // exits palette mode
                _ => Some(Action::ModalCancel),   // unknown key exits palette
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

    // --- METRO PANE SPECIFIC ---
    if state.focused_panel == FocusedPanel::MetroPane {
        match key.code {
            Char('s') => return Some(Action::MetroStart),
            Char('x') => return Some(Action::MetroStop),
            Char('r') => return Some(Action::MetroRestart),
            Char('l') => return Some(Action::MetroToggleLog),
            Char('J') => return Some(Action::MetroSendDebugger),
            Char('R') => return Some(Action::MetroSendReload),
            _ => {} // fall through to normal navigation
        }
    }

    // --- WORKTREE LIST SPECIFIC ---
    if state.focused_panel == FocusedPanel::WorktreeList {
        match key.code {
            Char('j') | Down => return Some(Action::WorktreeSelectNext),
            Char('k') | Up => return Some(Action::WorktreeSelectPrev),
            Char('L') => return Some(Action::StartSetLabel),
            Char('g') => return Some(Action::EnterGitPalette),
            Char('c') => return Some(Action::EnterRnPalette),
            Char('R') => return Some(Action::RefreshWorktrees),
            Enter => return Some(Action::WorktreeSwitchToSelected),
            Char('C') => return Some(Action::OpenClaudeCode),
            _ => {}
        }
    }

    // --- COMMAND OUTPUT SPECIFIC ---
    if state.focused_panel == FocusedPanel::CommandOutput {
        match key.code {
            Char('j') | Down => return Some(Action::FocusDown),
            Char('k') | Up => return Some(Action::FocusUp),
            Char('X') => return Some(Action::CommandCancel),
            Char('C') => return Some(Action::CommandOutputClear),
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
/// Clears command_output, sets running_command, spawns the process task.
fn dispatch_command(
    state: &mut AppState,
    spec: CommandSpec,
    metro_tx: &tokio::sync::mpsc::UnboundedSender<Action>,
) {
    let wt = if !state.worktrees.is_empty() {
        let idx = state.worktree_list_state.selected().unwrap_or(0);
        let idx = idx.min(state.worktrees.len() - 1);
        state.worktrees[idx].clone()
    } else {
        // No worktrees loaded yet — can't dispatch
        state.command_output.push_back("[error] no worktree selected".into());
        return;
    };

    // Clear output and prepare header
    state.command_output.clear();
    state.command_output_scroll = 0;
    state
        .command_output
        .push_back(format!("--- {} ---", spec.label()));
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
    match action {
        // Phase 1 actions
        Action::FocusNext => state.focused_panel = state.focused_panel.next(),
        Action::FocusPrev => state.focused_panel = state.focused_panel.prev(),
        Action::FocusUp => {
            if state.focused_panel == FocusedPanel::CommandOutput {
                state.command_output_scroll = state.command_output_scroll.saturating_sub(1);
            }
        }
        Action::FocusDown => {
            if state.focused_panel == FocusedPanel::CommandOutput {
                let max = state.command_output.len();
                if state.command_output_scroll < max {
                    state.command_output_scroll += 1;
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
            state.metro.set_starting();
            let tx = metro_tx.clone();
            let htx = handle_tx.clone();
            let worktree_path = state
                .active_worktree_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("."));
            let filter = state.log_filter_active;
            tokio::spawn(spawn_metro_task(worktree_path, filter, tx, htx));
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
            if let Err(e) = state.metro.send_stdin(b"j\n".to_vec()) {
                tracing::warn!("send debugger failed: {e}");
            }
        }

        Action::MetroSendReload => {
            if let Err(e) = state.metro.send_stdin(b"r\n".to_vec()) {
                tracing::warn!("send reload failed: {e}");
            }
        }

        Action::MetroToggleLog => {
            state.log_panel_visible = !state.log_panel_visible;
            state.log_filter_active = state.log_panel_visible;
            if state.metro.is_running() {
                state.pending_restart = true;
                update(state, Action::MetroStop, metro_tx, handle_tx);
            }
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
            if state.pending_restart {
                state.pending_restart = false;
                // Consume pending_switch_path if set (worktree switch takes priority)
                if let Some(path) = state.pending_switch_path.take() {
                    state.active_worktree_path = Some(path);
                }
                update(state, Action::MetroStart, metro_tx, handle_tx);
            }
        }

        // --- Phase 3: Worktree navigation ---

        Action::WorktreeSelectNext => {
            let len = state.worktrees.len();
            if len > 0 {
                let i = state.worktree_list_state.selected().unwrap_or(0);
                let next = if i >= len - 1 { 0 } else { i + 1 };
                state.worktree_list_state.select(Some(next));
                // Update active worktree for metro
                state.active_worktree_path = Some(state.worktrees[next].path.clone());
            }
        }

        Action::WorktreeSelectPrev => {
            let len = state.worktrees.len();
            if len > 0 {
                let i = state.worktree_list_state.selected().unwrap_or(0);
                let prev = if i == 0 { len - 1 } else { i - 1 };
                state.worktree_list_state.select(Some(prev));
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

            // Clamp selected index within new list
            let clamped = if worktrees.is_empty() {
                0
            } else {
                let current = state.worktree_list_state.selected().unwrap_or(0);
                current.min(worktrees.len().saturating_sub(1))
            };

            if !worktrees.is_empty() {
                state.worktree_list_state.select(Some(clamped));
                state.active_worktree_path = Some(worktrees[clamped].path.clone());
            }

            state.worktrees = worktrees;

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
                let idx = state.worktree_list_state.selected().unwrap_or(0);
                let idx = idx.min(state.worktrees.len() - 1);
                Some((state.worktrees[idx].branch.clone(), state.worktrees[idx].stale))
            } else {
                None
            };

            // WORK-06: lazy install — stale worktree + run command triggers yarn install first
            if let Some((_, stale)) = &wt_branch {
                if *stale {
                    if matches!(spec, CommandSpec::RnRunAndroid { .. } | CommandSpec::RnRunIos { .. }) {
                        state.pending_command_after_install = Some(spec);
                        dispatch_command(state, CommandSpec::YarnInstall, metro_tx);
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
                        crate::infra::devices::list_ios_devices().await
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

            // Normal dispatch
            dispatch_command(state, spec, metro_tx);
        }

        // --- Phase 3: Command output events ---

        Action::CommandOutputLine(line) => {
            state.command_output.push_back(line);
            if state.command_output.len() > MAX_COMMAND_LINES {
                state.command_output.pop_front();
            }
        }

        Action::CommandExited => {
            state.running_command = None;
            state.command_task = None;

            // WORK-06: if yarn install just finished, run the deferred command
            if let Some(deferred) = state.pending_command_after_install.take() {
                dispatch_command(state, deferred, metro_tx);
                return;
            }
        }

        Action::CommandOutputClear => {
            state.command_output.clear();
            state.command_output_scroll = 0;
        }

        Action::CommandCancel => {
            if let Some(task) = state.command_task.take() {
                task.abort();
            }
            state.running_command = None;
            state.command_output.push_back("[cancelled]".into());
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
            if let Some(ModalState::TextInput { buffer, .. }) = state.modal.as_mut() {
                buffer.push(c);
            }
        }

        Action::ModalInputBackspace => {
            if let Some(ModalState::TextInput { buffer, .. }) = state.modal.as_mut() {
                buffer.pop();
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
                ..
            }) = state.modal
            {
                if !devices.is_empty() {
                    *selected = if *selected >= devices.len() - 1 {
                        0
                    } else {
                        *selected + 1
                    };
                }
            }
        }

        Action::ModalDevicePrev => {
            if let Some(ModalState::DevicePicker {
                ref devices,
                ref mut selected,
                ..
            }) = state.modal
            {
                if !devices.is_empty() {
                    *selected = if *selected == 0 {
                        devices.len() - 1
                    } else {
                        *selected - 1
                    };
                }
            }
        }

        Action::ModalDeviceConfirm => {
            if let Some(ModalState::DevicePicker {
                devices,
                selected,
                pending_template,
            }) = state.modal.take()
            {
                if let Some(device) = devices.get(selected) {
                    let real_spec = match *pending_template {
                        CommandSpec::RnRunAndroid { .. } => CommandSpec::RnRunAndroid {
                            device_id: device.id.clone(),
                        },
                        CommandSpec::RnRunIos { .. } => CommandSpec::RnRunIos {
                            device_id: device.id.clone(),
                        },
                        other => other,
                    };
                    dispatch_command(state, real_spec, metro_tx);
                }
            }
        }

        // --- Phase 3: Device enumeration (async callback) ---

        Action::DevicesEnumerated(devices) => {
            if let Some(spec) = state.pending_device_command.take() {
                match devices.len() {
                    0 => {
                        state
                            .command_output
                            .push_back("[error] no devices found".into());
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
                        state.modal = Some(ModalState::DevicePicker {
                            devices,
                            selected: 0,
                            pending_template: Box::new(spec),
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
                let idx = state.worktree_list_state.selected().unwrap_or(0);
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
            state.palette_mode = Some(PaletteMode::Rn);
        }

        // --- Phase 5: Worktree switching and Claude Code ---

        Action::WorktreeSwitchToSelected => {
            // Capture target path NOW — navigation may change active_worktree_path later
            let target_path = state.worktrees
                .get(state.worktree_list_state.selected().unwrap_or(0))
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
            if !state.tmux_available {
                state.error_state = Some(ErrorState {
                    message: "Cannot open Claude Code: not inside a tmux session".into(),
                    can_retry: false,
                });
                return;
            }
            if let Some(wt) = state.worktrees.get(
                state.worktree_list_state.selected().unwrap_or(0)
            ) {
                let path = wt.path.clone();
                let branch = wt.branch.clone();
                tokio::spawn(async move {
                    let window_name = format!("claude:{}", branch.split('/').last().unwrap_or(&branch));
                    if let Err(e) = crate::infra::tmux::open_claude_in_worktree(&path, &window_name) {
                        tracing::warn!("open claude code failed: {e}");
                    }
                });
            }
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

    // Phase 4: tmux detection
    state.tmux_available = crate::infra::jira::is_inside_tmux();

    // Phase 4: Load config + JIRA client + cache
    if let Ok(Some(config)) = crate::infra::config::load_config() {
        match crate::infra::jira::HttpJiraClient::new(&config) {
            Ok(client) => {
                state.jira_client = Some(std::sync::Arc::new(client));
            }
            Err(e) => {
                tracing::warn!("JIRA client init failed: {e}");
            }
        }
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

    loop {
        // Render first on each iteration — double-buffer diff handles no-change efficiently
        terminal.draw(|f| crate::ui::view(f, &mut state))?;

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
                    CE::Resize(_, _) => {
                        // draw() is called at the top of the loop — resize redraws automatically
                    }
                    _ => {}
                }
            }
            Some(action) = metro_rx.recv() => {
                // Background tasks deliver log lines, MetroExited, WorktreesLoaded, etc.
                update(&mut state, action, &metro_tx, &handle_tx);
            }
            Some(handle) = handle_rx.recv() => {
                // Spawn task has successfully created the metro process — register the handle.
                state.metro.register(handle);
            }
        }

        if state.should_quit {
            break;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Async helpers — all run inside tokio::spawn, never blocking the event loop
// ---------------------------------------------------------------------------

/// Spawns the metro process and delivers a `MetroHandle` via `handle_tx`.
async fn spawn_metro_task(
    worktree_path: PathBuf,
    filter: bool,
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    handle_tx: tokio::sync::mpsc::UnboundedSender<MetroHandle>,
) {
    use crate::infra::process::ProcessClient;
    use crate::infra::process::TokioProcessClient;

    let client = TokioProcessClient;
    match client.spawn_metro(worktree_path.clone(), filter).await {
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
            let _ = action_tx.send(Action::MetroExited);
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
