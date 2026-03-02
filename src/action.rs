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
}
