/// All actions the user can trigger. TEA pattern: handle_key() → Option<Action> → update().
/// Phase 1 actions only. Later phases append variants — never remove existing ones.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Navigation
    FocusNext,             // Tab
    FocusPrev,             // Shift-Tab
    FocusUp,               // k or Up arrow
    FocusDown,             // j or Down arrow
    FocusLeft,             // h or Left arrow
    FocusRight,            // l or Right arrow

    // Search
    Search, // / (Phase 1: stub — no-op. Phase 4+ will activate search mode.)

    // Overlays
    ShowHelp,    // ? or F1
    DismissHelp, // q or Esc (when help overlay visible)

    // Error handling
    DismissError,       // Esc or q (when error overlay visible)
    RetryLastCommand,   // r (when error overlay visible)

    // App lifecycle
    Quit, // q (when no overlay active)

    // Metro control (user-triggered)
    MetroStart,
    MetroStop,
    MetroRestart,
    MetroToggleLog,      // l key — toggles log panel visibility
    MetroScrollUp,       // k when log panel focused
    MetroScrollDown,     // j when log panel focused
    MetroSendDebugger,   // J (shift-j, when metro pane focused) — sends j\n to metro stdin
    MetroSendReload,     // R (shift-r, when metro pane focused) — sends r\n to metro stdin

    // Metro background events (not user-triggered — sent by background tasks)
    MetroLogLine(String),       // stdout/stderr line from metro streaming task
    MetroExited,                // metro process has stopped (port confirmed free)
    MetroSpawnFailed(String),   // spawn error message — surfaces to error_state

    // Phase 3: Worktree navigation
    WorktreeSelectNext,   // j/Down in WorktreeList panel
    WorktreeSelectPrev,   // k/Up in WorktreeList panel
    WorktreesLoaded(Vec<crate::domain::worktree::Worktree>), // background refresh done
    RefreshWorktrees,     // manual refresh keybinding

    // Phase 3: Command lifecycle
    CommandRun(crate::domain::command::CommandSpec), // dispatched when command is confirmed/ready
    CommandOutputLine(String),  // line from command stdout/stderr
    CommandExited,              // command process has finished
    CommandOutputClear,         // clear the output panel
    CommandCancel,              // abort running command

    // Phase 3: Modal flow
    ShowCommandPalette,     // 'g' for git palette, 'c' for RN command palette (from worktree list)
    ModalConfirm,           // user pressed Y in confirm dialog
    ModalCancel,            // user pressed N or Esc in any modal
    ModalInputChar(char),   // character typed in text input modal
    ModalInputBackspace,    // backspace in text input modal
    ModalInputSubmit,       // Enter in text input modal
    ModalDeviceNext,        // j/Down in device picker
    ModalDevicePrev,        // k/Up in device picker
    ModalDeviceConfirm,     // Enter on selected device

    // Phase 3: Label management
    SetLabel { branch: String, label: String },
    StartSetLabel,          // 'L' on selected worktree — opens text input for label

    // Phase 3: Device enumeration (internal — sent by background task, not user)
    DevicesEnumerated(Vec<crate::domain::command::DeviceInfo>),

    // Phase 3: Command palette activation
    EnterGitPalette,  // 'g' when WorktreeList focused — activates git palette mode
    EnterRnPalette,   // 'c' when WorktreeList focused — activates RN palette mode

    // Phase 4: JIRA title background fetch results
    JiraTitlesFetched(Vec<(String, String)>),  // (ticket_key, title)

    // Phase 5: Worktree switching and Claude Code
    WorktreeSwitchToSelected, // Enter on worktree — switch metro to selected worktree
    OpenClaudeCode,           // C on worktree — open claude in new tmux tab
}
