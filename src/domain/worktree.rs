//! Worktree domain types.
//!
//! A `Worktree` represents one git worktree in the UMP repository. The struct
//! holds all display-relevant state so the UI layer can render the worktree
//! list without making any I/O calls itself.

/// Unique identifier for a worktree. Newtype around String to prevent accidental mixing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorktreeId(pub String);

/// Metro bundler status for a given worktree.
#[derive(Debug, Clone, PartialEq)]
pub enum WorktreeMetroStatus {
    Running,
    Stopped,
}

/// Fully-populated Worktree struct. All fields needed for list rendering and
/// command dispatch are present here. Populated by the infrastructure layer
/// via `git worktree list --porcelain` + optional JIRA/label lookups.
#[derive(Debug, Clone, PartialEq)]
pub struct Worktree {
    /// Stable identifier derived from the worktree path.
    pub id: WorktreeId,
    /// Absolute path to the worktree root.
    pub path: std::path::PathBuf,
    /// Current branch name (e.g. "feature/UMP-1234-login").
    pub branch: String,
    /// Short SHA of HEAD commit (first 7 chars).
    pub head_sha: String,
    /// Whether the Metro bundler is currently running in this worktree.
    pub metro_status: WorktreeMetroStatus,
    /// JIRA ticket title fetched in Phase 4 (None until then).
    pub jira_title: Option<String>,
    /// User-assigned custom label that follows the branch, not the worktree path.
    /// Persisted to `~/.config/ump-dash/labels.json`.
    pub label: Option<String>,
    /// True when `node_modules` mtime is older than `package.json` or `yarn.lock`.
    pub stale: bool,
    /// True when `ios/Pods` is missing or older than `ios/Podfile.lock`.
    pub stale_pods: bool,
}

impl Worktree {
    /// Returns the best available display name for this worktree, in priority order:
    /// 1. Custom label (user-assigned, follows branch)
    /// 2. JIRA ticket title (Phase 4+)
    /// 3. Branch name (always available)
    ///
    /// Used for single-string contexts such as modal titles and status messages.
    /// The worktree list widget accesses fields directly for layout control.
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        if let Some(label) = &self.label {
            return label.as_str();
        }
        if let Some(title) = &self.jira_title {
            return title.as_str();
        }
        self.branch.as_str()
    }
}
