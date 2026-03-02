//! Tmux integration — fire-and-forget tmux commands.
//! Uses std::process::Command directly (not tmux_interface crate — overkill for 2 calls).

/// Opens Claude Code in a new tmux window at the given worktree path.
///
/// Uses `tmux new-window [shell-command]` form — passes `claude` as the initial
/// shell-command argument, NOT via send-keys. This eliminates the race condition
/// where keystrokes can arrive before the shell finishes initializing.
///
/// `-d` flag prevents focus switching away from the dashboard.
pub fn open_claude_in_worktree(path: &std::path::Path, window_name: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("tmux")
        .args([
            "new-window",
            "-d",
            "-c", path.to_str().unwrap_or("."),
            "-n", window_name,
            "claude",
        ])
        .status()?;
    if !status.success() {
        anyhow::bail!("tmux new-window failed: exit code {:?}", status.code());
    }
    Ok(())
}
