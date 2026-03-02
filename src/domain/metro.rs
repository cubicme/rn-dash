// src/domain/metro.rs
//
// Metro domain types — single-instance invariant and status tracking.
//
// Architectural note: MetroHandle lives in domain/ but references tokio types
// (UnboundedSender, JoinHandle). This is a deliberate trade-off: MetroHandle is an
// infrastructure-bridging type whose sole purpose is to be held inside MetroManager's
// Option<MetroHandle>. The domain invariant (only one metro instance at a time) is
// enforced by MetroManager's Option wrapper at the TYPE level — you cannot hold two
// MetroHandle values without explicitly constructing two MetroManagers, which nothing
// in the codebase does. The tokio types used here (channels, JoinHandle) are inert
// data — they carry no behavior until the infra layer acts on them. Domain/mod.rs does
// NOT import infra, so ARCH-01 is maintained.

/// Current observable state of the metro process as seen by the domain layer.
#[derive(Debug, Clone, PartialEq)]
pub enum MetroStatus {
    /// No metro instance is running.
    Stopped,
    /// Metro is running with the given OS pid and the worktree it was started from.
    Running { pid: u32, worktree_id: String },
    /// Spawn is in flight — transient state between MetroStart and first log line.
    Starting,
    /// Kill + port-free wait is in flight — transient state between MetroStop and port free.
    Stopping,
}

impl Default for MetroStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Live handle to a running metro process.
///
/// Owned exclusively by `MetroManager::handle` — never shared or cloned.
/// Fields are pub so the infra layer (app.rs spawn logic in Plan 02) can construct
/// and pass this struct across the domain boundary.
pub struct MetroHandle {
    /// OS process ID — used for status display and external-kill detection.
    pub pid: u32,
    /// Worktree this instance was started from — displayed in the metro pane.
    pub worktree_id: String,
    /// Sender half of the stdin channel. Infra stdin-writer task holds the receiver.
    /// Drop this sender to signal the stdin task to stop.
    pub stdin_tx: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    /// Background task that reads metro stdout/stderr and sends MetroLogLine actions.
    pub stream_task: tokio::task::JoinHandle<()>,
    /// Background task that writes bytes from stdin_tx channel to the child's stdin.
    pub stdin_task: tokio::task::JoinHandle<()>,
}

/// Enforces the single-instance invariant: at most one metro process may run at a time.
///
/// All metro state transitions go through MetroManager methods. The update() function
/// in app.rs calls these methods — it never manipulates handles directly.
pub struct MetroManager {
    /// Private — callers cannot bypass the single-instance check.
    handle: Option<MetroHandle>,
    /// Public read-only status for UI rendering.
    pub status: MetroStatus,
}

impl MetroManager {
    /// Create a new manager in the Stopped state.
    pub fn new() -> Self {
        Self {
            handle: None,
            status: MetroStatus::Stopped,
        }
    }

    /// True if a metro handle is currently registered (process is running or finishing).
    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    /// Register a freshly spawned process handle.
    ///
    /// # Panics
    /// Panics if called while a handle already exists. Callers MUST call `take_handle()`
    /// and kill the process before registering a new one.
    pub fn register(&mut self, handle: MetroHandle) {
        assert!(
            self.handle.is_none(),
            "BUG: MetroManager::register() called with an existing handle — kill first"
        );
        let pid = handle.pid;
        let worktree_id = handle.worktree_id.clone();
        self.handle = Some(handle);
        self.status = MetroStatus::Running { pid, worktree_id };
    }

    /// Clear the handle after the process has been killed and reaped.
    /// Transitions status to Stopped.
    pub fn clear(&mut self) {
        self.handle = None;
        self.status = MetroStatus::Stopped;
    }

    /// Send a raw byte sequence to metro's stdin via the background stdin-writer task.
    ///
    /// No-op if metro is not running.
    pub fn send_stdin(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        if let Some(ref h) = self.handle {
            h.stdin_tx
                .send(bytes)
                .map_err(|e| anyhow::anyhow!("metro stdin send failed: {e}"))?;
        }
        Ok(())
    }

    /// Transition to Starting state (spawn is in flight).
    pub fn set_starting(&mut self) {
        self.status = MetroStatus::Starting;
    }

    /// Transition to Stopping state (kill + port-free wait is in flight).
    pub fn set_stopping(&mut self) {
        self.status = MetroStatus::Stopping;
    }

    /// Take ownership of the handle for kill operations.
    ///
    /// Returns None if metro is not running. After this call is_running() returns false,
    /// so register() can be called again once the kill completes.
    pub fn take_handle(&mut self) -> Option<MetroHandle> {
        self.handle.take()
    }
}
