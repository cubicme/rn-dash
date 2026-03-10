// src/infra/worktrees.rs
//
// Worktree enumeration: parse `git worktree list --porcelain` output into
// domain Worktree values. All I/O is behind async functions; the parser itself
// is pure (no I/O) so it can be unit-tested without a real git repo.

#![allow(dead_code)]

use crate::domain::worktree::{Worktree, WorktreeId, WorktreeMetroStatus};
use std::path::Path;

/// Pure parser. Converts `git worktree list --porcelain` text output into a
/// Vec<Worktree>. Each stanza in the output is separated by a blank line.
///
/// Example stanza:
/// ```text
/// worktree /Users/me/projects/ump
/// HEAD abc1234def5678901234567890abcdef12345678
/// branch refs/heads/feature/UMP-1234-login
/// ```
///
/// Detached HEAD stanzas have "detached" on the third line instead of a branch.
pub fn parse_worktree_porcelain(text: &str) -> anyhow::Result<Vec<Worktree>> {
    let mut worktrees = Vec::new();

    for stanza in text.split("\n\n") {
        let stanza = stanza.trim();
        if stanza.is_empty() {
            continue;
        }

        let mut path_str: Option<&str> = None;
        let mut head_sha: Option<&str> = None;
        let mut branch: Option<String> = None;
        let mut is_bare = false;

        for line in stanza.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path_str = Some(p);
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                // Take only first 7 chars for the short SHA
                head_sha = Some(&h[..h.len().min(7)]);
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                branch = Some(b.to_string());
            } else if line == "detached" {
                branch = Some("(detached)".to_string());
            } else if line == "bare" {
                is_bare = true;
            }
        }

        // Skip bare repos — they have no working tree content to display
        if is_bare {
            continue;
        }

        // Skip stanzas without a path (malformed output)
        let path_str = match path_str {
            Some(p) => p,
            None => continue,
        };

        let path = std::path::PathBuf::from(path_str);
        let head_sha = head_sha.unwrap_or("unknown").to_string();
        let branch = branch.unwrap_or_else(|| "(unknown)".to_string());

        // WorktreeId is derived from the path — stable across renames of the branch
        let id = WorktreeId(path_str.to_string());

        let stale = check_stale(&path);

        worktrees.push(Worktree {
            id,
            path,
            branch,
            head_sha,
            metro_status: WorktreeMetroStatus::Stopped, // derived later from AppState
            jira_title: None,
            label: None,
            stale,
        });
    }

    Ok(worktrees)
}

/// Returns true when dependencies are stale (need `yarn install`).
///
/// Multi-sentinel approach to support different yarn versions:
/// 1. `node_modules/.yarn-integrity` — yarn v1 (classic) sentinel
/// 2. `node_modules/.yarn-state` — yarn berry (v2/v3/v4) with nodeLinker: node-modules
/// 3. If `node_modules` exists but no sentinel found — assume NOT stale (benefit of the doubt)
/// 4. If `node_modules` does not exist — stale (never installed)
///
/// When a sentinel IS found, staleness = sentinel mtime < max(package.json, yarn.lock) mtime.
pub fn check_stale(worktree_path: &Path) -> bool {
    let node_modules = worktree_path.join("node_modules");

    // If node_modules doesn't exist at all, definitely stale
    if !node_modules.exists() {
        tracing::debug!(
            path = %worktree_path.display(),
            "check_stale: true — node_modules absent"
        );
        return true;
    }

    // Try sentinel files in order of preference
    let sentinels = [
        node_modules.join(".yarn-integrity"), // yarn v1 classic
        node_modules.join(".yarn-state"),     // yarn berry (nodeLinker: node-modules)
    ];

    let sentinel_mtime = sentinels.iter().find_map(|sentinel| {
        std::fs::metadata(sentinel)
            .and_then(|m| m.modified())
            .ok()
    });

    let sentinel_mtime = match sentinel_mtime {
        Some(t) => t,
        None => {
            // node_modules exists but no known sentinel — benefit of the doubt
            tracing::debug!(
                path = %worktree_path.display(),
                "check_stale: false — node_modules exists, no sentinel found (unknown yarn version)"
            );
            return false;
        }
    };

    // Gather mtimes for the lock files that indicate dependencies changed
    let mut max_lock_mtime: Option<std::time::SystemTime> = None;

    for lock_file in &["package.json", "yarn.lock"] {
        let lock_path = worktree_path.join(lock_file);
        if let Ok(mtime) = std::fs::metadata(&lock_path).and_then(|m| m.modified()) {
            max_lock_mtime = Some(match max_lock_mtime {
                Some(current) => current.max(mtime),
                None => mtime,
            });
        }
    }

    let stale = match max_lock_mtime {
        Some(lock_mtime) => sentinel_mtime < lock_mtime,
        None => false, // no lock files → can't determine staleness
    };

    if stale {
        tracing::debug!(
            path = %worktree_path.display(),
            "check_stale: true — sentinel older than lock files"
        );
    }

    stale
}

/// Returns true when `ios/Pods` directory is missing or older than `ios/Podfile.lock`.
/// Used by sync-before-run to determine if pod-install is needed before iOS runs.
pub fn check_stale_pods(worktree_path: &Path) -> bool {
    let pods_dir = worktree_path.join("ios").join("Pods");
    let podfile_lock = worktree_path.join("ios").join("Podfile.lock");

    let pods_mtime = match std::fs::metadata(&pods_dir).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true, // Pods/ absent → stale
    };

    match std::fs::metadata(&podfile_lock).and_then(|m| m.modified()) {
        Ok(lock_mtime) => pods_mtime < lock_mtime,
        Err(_) => false, // no Podfile.lock → can't determine staleness
    }
}

/// Runs `git worktree list --porcelain` in `repo_root` and parses the output.
pub async fn list_worktrees(repo_root: &Path) -> anyhow::Result<Vec<Worktree>> {
    let output = tokio::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git worktree list failed: {}", stderr);
    }

    let text = String::from_utf8(output.stdout)?;
    parse_worktree_porcelain(&text)
}
