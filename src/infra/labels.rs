// src/infra/labels.rs
//
// Label persistence for branch-level display names.
// Labels are stored in ~/.config/ump-dash/labels.json as a flat
// JSON object: { "branch-name": "label-text", ... }.
//
// Labels follow the branch, not the worktree path — so renaming or deleting a
// worktree does not lose the label. See CLAUDE.md project note:
// "Branch labels are per-branch (persist across worktrees), not per-worktree"

#![allow(dead_code)]

use std::collections::HashMap;

/// Returns the `~/.config/ump-dash/` config directory path.
pub fn config_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    std::path::PathBuf::from(home).join(".config").join("ump-dash")
}

/// Returns the path to the labels JSON file.
pub fn labels_path() -> std::path::PathBuf {
    config_dir().join("labels.json")
}

/// Loads the labels map from disk. Returns an empty HashMap if the file does not exist.
///
/// The file stores branch_name → label mappings as a flat JSON object.
pub fn load_labels() -> anyhow::Result<HashMap<String, String>> {
    let path = labels_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let map: HashMap<String, String> = serde_json::from_str(&contents)?;
            Ok(map)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(e.into()),
    }
}

/// Saves the labels map to disk as pretty-printed JSON.
///
/// Creates the config directory if it does not already exist.
pub fn save_labels(labels: &HashMap<String, String>) -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(labels)?;
    std::fs::write(labels_path(), json)?;
    Ok(())
}
