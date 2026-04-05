// src/infra/jira.rs
//
// JIRA HTTP client trait and implementation.
// Supports two authentication modes:
//   - "cloud"      → Basic Auth (email:api_token)   — Atlassian Cloud instances
//   - "datacenter" → Bearer Auth (PAT)               — JIRA Data Center / Server
//
// The client never panics and never surfaces errors to the TUI layer.
// Any failure (network, auth, parse) results in None from fetch_title().

#![allow(dead_code)]

use crate::infra::config::DashConfig;
use async_trait::async_trait;

/// Abstraction over JIRA title fetching.
///
/// Implementing this as a trait lets unit tests inject a fake client without
/// making real HTTP calls. The bound `Send + Sync` is required so that
/// implementations can be stored in `Arc<dyn JiraClient>` in the app state.
/// The `Debug` bound is required because `AppState` derives `Debug`.
#[async_trait]
pub trait JiraClient: Send + Sync + std::fmt::Debug {
    /// Fetches the summary (title) for the given JIRA ticket key (e.g. "UMP-1234").
    ///
    /// Returns `None` on any failure — network error, auth error, missing key, or
    /// unexpected JSON shape. The TUI should treat `None` as "title not available"
    /// and fall back to displaying the raw ticket key.
    async fn fetch_title(&self, ticket_key: &str) -> Option<String>;
}

/// Concrete JIRA client that makes real HTTP requests using reqwest.
#[derive(Debug)]
pub struct HttpJiraClient {
    client: reqwest::Client,
    base_url: String,
    auth_mode: String,
    email: Option<String>,
    token: String,
}

impl HttpJiraClient {
    /// Constructs an `HttpJiraClient` from the loaded `DashConfig`.
    ///
    /// Builds a bare `reqwest::Client` with no default auth headers — auth is
    /// applied per-request in `fetch_title` for clarity and correctness.
    pub fn new(config: &DashConfig) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .build()?;

        Ok(Self {
            client,
            base_url: config.jira_base_url.trim_end_matches('/').to_string(),
            auth_mode: config.auth_mode.clone(),
            email: config.jira_email.clone(),
            token: config.jira_token.clone(),
        })
    }
}

#[async_trait]
impl JiraClient for HttpJiraClient {
    async fn fetch_title(&self, ticket_key: &str) -> Option<String> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=summary",
            self.base_url, ticket_key
        );

        let request = self.client.get(&url);

        // Apply authentication based on the configured auth mode.
        let request = if self.auth_mode == "datacenter" {
            // Data Center / Server: Personal Access Token sent as Bearer.
            request.bearer_auth(&self.token)
        } else {
            // Cloud: Basic Auth using email and API token.
            request.basic_auth(
                self.email.as_deref().unwrap_or(""),
                Some(&self.token),
            )
        };

        let response = request.send().await.ok()?;
        let json: serde_json::Value = response.json().await.ok()?;
        let title = json["fields"]["summary"].as_str()?.to_string();
        Some(title)
    }
}

/// Extracts a JIRA ticket key from a git branch name using the given project prefix.
///
/// Supports branch formats like:
///   - "feature/UMP-1234-some-description"  → Some("UMP-1234")  (with prefix "UMP")
///   - "UMP-5678"                           → Some("UMP-5678")  (with prefix "UMP")
///   - "feature/PROJ-42-thing"              → Some("PROJ-42")   (with prefix "PROJ")
///   - "main"                               → None
///   - "feature/no-ticket"                  → None
///
/// The function splits the branch by `/`, then examines each segment.
/// A segment matches if splitting it by `-` (up to 3 parts) yields
/// `project_prefix` as the first part and an all-ASCII-digits string as the second.
/// No regex crate is required.
pub fn extract_jira_key(branch: &str, project_prefix: &str) -> Option<String> {
    for segment in branch.split('/') {
        let mut parts = segment.splitn(3, '-');
        let first = match parts.next() {
            Some(v) => v,
            None => continue,
        };
        let second = match parts.next() {
            Some(v) => v,
            None => continue,
        };

        if first == project_prefix && !second.is_empty() && second.chars().all(|c| c.is_ascii_digit()) {
            return Some(format!("{}-{}", project_prefix, second));
        }
    }
    None
}

/// Returns `true` when the process is running inside a tmux session.
///
/// Tmux sets the `TMUX` environment variable to the path of the server socket,
/// so its presence is a reliable indicator of a tmux session.
pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_key_from_feature_branch() {
        assert_eq!(
            extract_jira_key("feature/UMP-1234-login", "UMP"),
            Some("UMP-1234".to_string())
        );
    }

    #[test]
    fn extracts_key_from_bare_ticket_segment() {
        assert_eq!(
            extract_jira_key("UMP-5678", "UMP"),
            Some("UMP-5678".to_string())
        );
    }

    #[test]
    fn returns_none_for_main_branch() {
        assert_eq!(extract_jira_key("main", "UMP"), None);
    }

    #[test]
    fn returns_none_for_branch_without_ticket() {
        assert_eq!(extract_jira_key("feature/no-ticket", "UMP"), None);
    }

    #[test]
    fn extracts_key_with_single_digit_number() {
        assert_eq!(
            extract_jira_key("fix/UMP-9-short", "UMP"),
            Some("UMP-9".to_string())
        );
    }

    #[test]
    fn extracts_key_with_custom_prefix() {
        assert_eq!(
            extract_jira_key("feature/PROJ-42-thing", "PROJ"),
            Some("PROJ-42".to_string())
        );
    }
}
