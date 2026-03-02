//! Infrastructure layer — process spawning, git, JIRA, tmux, file I/O.
//! All concrete implementations are behind trait boundaries (ARCH-02).

pub mod port;
pub mod process;
pub mod worktrees;
pub mod labels;
