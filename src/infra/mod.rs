//! Infrastructure layer — process spawning, git, JIRA, tmux, file I/O.
//! All concrete implementations are behind trait boundaries (ARCH-02).

pub mod port;
pub mod process;
pub mod worktrees;
pub mod command_runner;
pub mod labels;
pub mod devices;
pub mod config;
pub mod jira;
pub mod jira_cache;
pub mod tmux;
