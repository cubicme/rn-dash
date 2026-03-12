//! Terminal multiplexer abstraction — trait with Tmux and Zellij adapters.
//! Uses std::process::Command (same pattern as existing tmux.rs).
//! No new crates required.

use std::path::Path;

/// Trait for terminal multiplexer operations.
/// Implementors must be Send + Sync + Debug for storage in AppState.
pub trait Multiplexer: Send + Sync + std::fmt::Debug {
    /// Creates a new window/tab at the given path with the given name, running the given command.
    /// The window should switch focus to the newly created tab.
    fn new_window(&self, path: &Path, name: &str, command: &str) -> anyhow::Result<()>;

    /// Returns true if this multiplexer is available in the current environment.
    fn is_available(&self) -> bool;
}

#[derive(Debug)]
pub struct TmuxAdapter;

impl Multiplexer for TmuxAdapter {
    fn new_window(&self, path: &Path, name: &str, command: &str) -> anyhow::Result<()> {
        let path_str = path.to_str().unwrap_or(".");
        let status = std::process::Command::new("tmux")
            .args(["new-window", "-c", path_str, "-n", name, command])
            .status()?;
        if !status.success() {
            anyhow::bail!("tmux new-window failed: exit code {:?}", status.code());
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        std::env::var("TMUX").is_ok()
    }
}

#[derive(Debug)]
pub struct ZellijAdapter;

impl Multiplexer for ZellijAdapter {
    fn new_window(&self, path: &Path, name: &str, command: &str) -> anyhow::Result<()> {
        // Zellij tab creation: create tab at CWD with name.
        // Zellij's new-tab does not support running an initial command directly
        // in the same way tmux does. We create the tab, then write the command.
        let path_str = path.to_str().unwrap_or(".");

        // Create tab at the given cwd
        let status = std::process::Command::new("zellij")
            .args(["action", "new-tab", "--name", name, "--cwd", path_str])
            .status()?;
        if !status.success() {
            anyhow::bail!("zellij new-tab failed: exit code {:?}", status.code());
        }

        // Write the command to the new tab's terminal
        // (zellij action write-chars sends keystrokes to the focused pane)
        let cmd_with_enter = format!("{}\n", command);
        let write_status = std::process::Command::new("zellij")
            .args(["action", "write-chars", &cmd_with_enter])
            .status()?;
        if !write_status.success() {
            tracing::warn!("zellij write-chars failed — tab created but command not started");
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        std::env::var("ZELLIJ").is_ok()
    }
}

/// Auto-detect the available multiplexer. Checks $TMUX first, then $ZELLIJ.
/// Returns None if no multiplexer is detected — features that need it are disabled.
pub fn detect_multiplexer() -> Option<Box<dyn Multiplexer>> {
    if std::env::var("TMUX").is_ok() {
        return Some(Box::new(TmuxAdapter));
    }
    if std::env::var("ZELLIJ").is_ok() {
        return Some(Box::new(ZellijAdapter));
    }
    None
}
