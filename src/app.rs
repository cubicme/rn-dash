#![allow(dead_code)]
use crate::action::Action;
use crate::domain::metro::MetroHandle;
use futures::StreamExt;
use ratatui::crossterm::event::{EventStream, KeyCode, KeyEventKind};
use std::path::PathBuf;

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

    // Set to true when MetroRestart or MetroStart-while-running triggers a stop-first-then-start.
    // When MetroExited fires and this is true, a new MetroStart is auto-dispatched.
    pub pending_restart: bool,
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
            pending_restart: false,
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
///
/// `metro_tx` and `handle_tx` are channels that connect update() to the async runtime:
/// - `metro_tx`: background tasks send Action events (MetroLogLine, MetroExited) back to the loop
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

        // --- Metro control actions ---

        Action::MetroStart => {
            if state.metro.is_running() {
                // Another instance is running — stop it first, then auto-start when it exits.
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
                // Signal the process task to kill the child via the oneshot channel.
                if let Some(kill_tx) = handle.kill_tx.take() {
                    let _ = kill_tx.send(());
                }
                // Stop stdin writes — no more bytes needed after stop.
                handle.stdin_task.abort();
                // metro_process_task owns the child and will send MetroExited when done.
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
            // Restart with updated filter if metro is currently running.
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
                update(state, Action::MetroStart, metro_tx, handle_tx);
            }
        }

        // Phase 3 actions — stubs that will be implemented in Plan 03-02 (app logic).
        Action::WorktreeSelectNext
        | Action::WorktreeSelectPrev
        | Action::RefreshWorktrees
        | Action::WorktreesLoaded(_)
        | Action::CommandRun(_)
        | Action::CommandOutputLine(_)
        | Action::CommandExited
        | Action::CommandOutputClear
        | Action::CommandCancel
        | Action::ShowCommandPalette
        | Action::ModalConfirm
        | Action::ModalCancel
        | Action::ModalInputChar(_)
        | Action::ModalInputBackspace
        | Action::ModalInputSubmit
        | Action::ModalDeviceNext
        | Action::ModalDevicePrev
        | Action::ModalDeviceConfirm
        | Action::SetLabel { .. }
        | Action::StartSetLabel => {
            // Phase 3 stubs — implemented in Plan 03-02 (app state logic)
        }
    }
}

/// Main application loop. Runs on the tokio runtime.
/// Renders on every event and on a 250ms tick. Exits when state.should_quit is true.
pub async fn run(mut terminal: ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut state = AppState::default();
    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(250));

    // Channel for background tasks (log lines, MetroExited) to send Actions back to the loop.
    let (metro_tx, mut metro_rx) = tokio::sync::mpsc::unbounded_channel::<Action>();

    // Channel for the spawn task to deliver the MetroHandle once spawning is complete.
    // MetroHandle is not Clone, so it cannot go through the Action enum — separate channel.
    let (handle_tx, mut handle_rx) = tokio::sync::mpsc::unbounded_channel::<MetroHandle>();

    loop {
        // Render first on each iteration — double-buffer diff handles no-change efficiently
        terminal.draw(|f| crate::ui::view(f, &state))?;

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
                // Background tasks deliver log lines and MetroExited through this arm.
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
///
/// If spawning fails, sends `MetroExited` via `action_tx` so the UI transitions
/// back to Stopped (rather than staying in Starting forever).
async fn spawn_metro_task(
    worktree_path: PathBuf,
    filter: bool,
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
    handle_tx: tokio::sync::mpsc::UnboundedSender<MetroHandle>,
) {
    use crate::infra::process::TokioProcessClient;
    use crate::infra::process::ProcessClient;

    let client = TokioProcessClient;
    match client.spawn_metro(worktree_path.clone(), filter).await {
        Ok(mut child) => {
            let pid = child.id().unwrap_or(0);

            // Take IO handles immediately — before any kill() call (research pitfall 5).
            let stdout = child.stdout.take().expect("stdout piped");
            let stderr = child.stderr.take().expect("stderr piped");
            let stdin = child.stdin.take().expect("stdin piped");

            // Stdin channel: update() sends bytes via MetroHandle.stdin_tx.
            let (stdin_tx, stdin_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
            let stdin_task = tokio::spawn(stdin_writer(stdin, stdin_rx));

            // Kill channel: MetroStop sends () to signal process task to kill child.
            let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();

            // Process task owns Child, handles kill and external-death detection.
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

            // Deliver the handle — run() will call state.metro.register(handle).
            let _ = handle_tx.send(handle);
        }
        Err(e) => {
            tracing::error!("metro spawn failed: {e}");
            // Transition UI back to Stopped so user can retry.
            let _ = action_tx.send(Action::MetroExited);
        }
    }
}

/// Owns the `Child` process. Handles two cases:
/// 1. Kill signal via `kill_rx` — kills child, polls port free, sends MetroExited.
/// 2. Child exits on its own (crash, external kill) — sends MetroExited directly.
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
            // User-initiated stop — kill child and wait for port to free.
            log_task.abort();
            if let Err(e) = child.kill().await {
                tracing::error!("metro kill failed: {e}");
            }
            // Poll for port free: metro's Node subprocess may hold 8081 briefly after SIGKILL.
            for _ in 0..50 {
                if crate::infra::port::port_is_free(8081) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            let _ = tx.send(Action::MetroExited);
        }
        _ = child.wait() => {
            // Metro died on its own — external kill, crash, or normal exit.
            log_task.abort();
            let _ = tx.send(Action::MetroExited);
        }
    }
}

/// Reads stdout and stderr lines from the metro process and sends them as `MetroLogLine` actions.
///
/// Exits when either stream closes (process exited) or the task is aborted by metro_process_task.
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
///
/// Exits when the channel is closed (stdin_tx dropped by MetroStop) or a write fails.
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
