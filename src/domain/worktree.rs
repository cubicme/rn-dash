//! Stub Worktree types. Phase 3 will populate these with real fields.
#![allow(dead_code)]

/// Unique identifier for a worktree. Newtype around String to prevent accidental mixing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorktreeId(pub String);

/// Stub Worktree struct. Will gain fields (branch, metro_status, label, path) in Phase 3.
#[derive(Debug, Clone)]
pub struct Worktree {
    pub id: WorktreeId,
}
