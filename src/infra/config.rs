// src/infra/config.rs
//
// Dashboard configuration persistence.
// Config is stored at ~/.config/ump-dash/config.json with 0600 permissions
// because it contains JIRA credentials (token/email).
//
// The config_dir() function is re-exported from labels.rs to ensure a single
// source of truth for the config directory path.

#![allow(dead_code)]

use crate::infra::labels::config_dir;
use serde::{Deserialize, Serialize};

fn default_auth_mode() -> String {
    "cloud".to_string()
}

fn default_claude_flags() -> String {
    "--dangerously-skip-permissions".to_string()
}

/// Application configuration stored in ~/.config/ump-dash/config.json.
///
/// Security note: this file is written with 0600 permissions on Unix because
/// `jira_token` is a credential. Never log or display the token value.
#[derive(Debug, Deserialize, Serialize)]
pub struct DashConfig {
    /// Base URL for the JIRA instance, e.g. "https://example.atlassian.net"
    pub jira_base_url: String,

    /// JIRA account email address. Required for Cloud (Basic Auth), not used for Data Center (Bearer).
    #[serde(default)]
    pub jira_email: Option<String>,

    /// JIRA API token (Cloud) or Personal Access Token (Data Center).
    pub jira_token: String,

    /// Authentication mode: "cloud" (Basic Auth email:token) or "datacenter" (Bearer PAT).
    /// Defaults to "cloud" if not specified in the config file.
    #[serde(default = "default_auth_mode")]
    pub auth_mode: String,

    /// Command-line flags to pass when launching Claude Code (e.g., "--dangerously-skip-permissions").
    /// Defaults to "--dangerously-skip-permissions" if not specified in the config file.
    #[serde(default = "default_claude_flags")]
    pub claude_flags: String,
}

/// Loads the dashboard configuration from disk.
///
/// Returns `Ok(None)` when the config file does not exist — callers should
/// treat this as "not configured" and either prompt the user or skip JIRA
/// integration silently. All other I/O errors are propagated.
pub fn load_config() -> anyhow::Result<Option<DashConfig>> {
    let path = config_dir().join("config.json");
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let config: DashConfig = serde_json::from_str(&contents)?;
            Ok(Some(config))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Saves the dashboard configuration to disk as pretty-printed JSON.
///
/// Creates the config directory if it does not already exist.
/// On Unix systems the file is immediately chmod'd to 0600 so that the JIRA
/// token is not world-readable.
pub fn save_config(config: &DashConfig) -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config.json");
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }

    #[cfg(not(unix))]
    {
        // Non-Unix platforms do not support POSIX permission bits.
        // The file is written but permissions cannot be restricted.
        let _ = &path;
    }

    Ok(())
}
