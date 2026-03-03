// src/infra/process.rs
//
// ProcessClient trait and TokioProcessClient implementation.
// ARCH-02: All infra behind trait boundaries — swap TokioProcessClient for a
// FakeProcessClient in tests without touching any domain or app code.

#![allow(dead_code)]

use std::path::PathBuf;
use tokio::process::Child;

/// Trait boundary for metro process spawning.
///
/// The domain and app layers depend only on this trait. TokioProcessClient is the
/// production implementation; tests may supply a fake.
#[async_trait::async_trait]
pub trait ProcessClient: Send + Sync {
    /// Spawn a metro dev server in the given worktree directory.
    ///
    /// Returns the `Child` handle with stdout, stderr, and stdin all piped.
    /// The caller is responsible for taking those handles before any kill call
    /// (see research pitfall 5).
    ///
    /// Always sets `DEBUG=Metro:*` so metro output streams to stdout.
    async fn spawn_metro(&self, worktree_path: PathBuf) -> anyhow::Result<Child>;
}

/// Production implementation that calls `tokio::process::Command` directly.
pub struct TokioProcessClient;

#[async_trait::async_trait]
impl ProcessClient for TokioProcessClient {
    async fn spawn_metro(&self, worktree_path: PathBuf) -> anyhow::Result<Child> {
        let mut cmd = tokio::process::Command::new("yarn");
        cmd.args(["start", "--reset-cache"])
            .current_dir(worktree_path)
            // CRITICAL: process_group(0) puts yarn + all Node children in their own
            // process group. kill() on the Child will send SIGKILL to the whole group,
            // ensuring the Node subprocess that holds port 8081 is also killed.
            // Without this, only yarn dies and the port stays bound (research pitfall 2).
            .process_group(0)
            // Drop safety net: if the Child is dropped without an explicit kill() call
            // (e.g., panic), tokio will issue SIGKILL automatically.
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped());

        // Always set DEBUG=Metro:* so metro output streams to stdout.
        cmd.env("DEBUG", "Metro:*");

        Ok(cmd.spawn()?)
    }
}
