# Phase 4: Config and JIRA Integration - Research

**Researched:** 2026-03-02
**Domain:** JIRA REST API, Rust HTTP client (reqwest), file config with secure permissions, tmux detection
**Confidence:** HIGH (stack), MEDIUM (JIRA auth method — see Open Questions)

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INTG-01 | Dashboard reads JIRA API token from ~/.config/ump-dash/ config | Config file pattern using `labels.rs` as precedent; `PermissionsExt::from_mode(0o600)` for secure write |
| INTG-02 | Dashboard fetches JIRA ticket titles by extracting UMP-XXXX from branch names and querying the JIRA REST API | `reqwest 0.13` with `basic_auth()` or `bearer_auth()` + `serde` JSON deserialization; background async task via `tokio::spawn` |
| INTG-03 | Fetched JIRA titles are cached locally to avoid redundant API calls | `HashMap<String, String>` serialized to `~/.config/ump-dash/jira_cache.json` — same pattern as `labels.rs` |
| INTG-05 | Dashboard detects it is running inside tmux for tmux-dependent features | `std::env::var("TMUX").is_ok()` — one-liner, no crate needed |
</phase_requirements>

---

## Summary

Phase 4 adds three orthogonal capabilities: (1) a config file that stores the JIRA API token at `~/.config/ump-dash/config.toml` (or `.json`) with 0600 permissions on first write, (2) background fetching of JIRA ticket titles by extracting `UMP-XXXX` from branch names and calling the JIRA REST API, and (3) tmux presence detection gating tmux-dependent features.

The project already has `serde`, `serde_json`, and `tokio` in `Cargo.toml`, and `infra/labels.rs` establishes the config-dir pattern (`~/.config/ump-dash/`). The only new crate needed is `reqwest 0.13` with the `json` feature for HTTP calls. The JIRA title fetch must run in the background (same `tokio::spawn` pattern as `RefreshWorktrees`) and results must flow back into `AppState` via a new `Action::JiraTitlesFetched` variant.

The single unresolved question is whether the project's JIRA instance is Cloud or Data Center, because the two use different auth schemes (Cloud = Basic Auth with `email:API_token`, Data Center = Bearer PAT). The implementation must handle both, or the config file must record the auth scheme. The planner should treat this as a task-0 decision point.

**Primary recommendation:** Add `reqwest 0.13` with `features = ["json", "rustls-tls"]`, implement a `JiraClient` struct behind a trait in `src/infra/jira.rs`, read config from `~/.config/ump-dash/config.json` (same format as `labels.json`), cache results in `~/.config/ump-dash/jira_cache.json`, detect tmux via `std::env::var("TMUX").is_ok()`.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.13.1 | Async HTTP client for JIRA REST API | Industry standard Rust HTTP client; built on hyper + tokio; supports basic_auth(), bearer_auth(), JSON deserialization |
| serde | 1.x | Serialization for config and cache | Already in Cargo.toml; used by labels.rs |
| serde_json | 1.x | JSON format for config and cache | Already in Cargo.toml |
| tokio | 1.49 | Async runtime | Already in Cargo.toml |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::os::unix::fs::PermissionsExt | stdlib | Set 0600 file permissions on Unix | When writing config file for the first time |
| std::env | stdlib | tmux detection via TMUX env var | On startup, gates tmux features |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| reqwest (async) | ureq (sync) | ureq simpler but blocking; would need `tokio::task::spawn_blocking` wrapper; reqwest is already the natural choice given the tokio runtime in use |
| reqwest | hyper directly | hyper is lower-level; reqwest is the ergonomic wrapper; no benefit to raw hyper here |
| serde_json for config | toml crate | TOML is more human-friendly but adds a new dependency; JSON is already used for labels.json and is familiar to the project |

**Installation:**
```bash
# In Cargo.toml — no shell command needed for workspace; add dependency entry:
# reqwest = { version = "0.13", features = ["json", "rustls-tls"] }
```

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── infra/
│   ├── jira.rs          # JiraClient trait + HttpJiraClient impl
│   ├── config.rs        # load/save config.json with 0600 permissions
│   ├── jira_cache.rs    # load/save jira_cache.json (HashMap<ticket_key, title>)
│   ├── labels.rs        # existing — unchanged
│   └── mod.rs           # add: pub mod jira; pub mod config; pub mod jira_cache;
├── domain/
│   ├── worktree.rs      # existing — jira_title field already present
│   └── ...
├── action.rs            # add: JiraTitlesFetched(Vec<(String, String)>)
└── app.rs               # handle JiraTitlesFetched, startup tmux detection
```

### Pattern 1: Config File with Secure Permissions

**What:** Read/write `~/.config/ump-dash/config.json` containing the JIRA base URL, email, and API token. On first write, set file permissions to 0600.
**When to use:** On startup. If the file does not exist, JIRA integration is silently skipped.

```rust
// Source: std::os::unix::fs::PermissionsExt (stdlib)
use std::os::unix::fs::PermissionsExt;

pub fn save_config(config: &DashConfig) -> anyhow::Result<()> {
    let dir = config_dir();  // reuse infra::labels::config_dir()
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config.json");
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, json)?;
    // Set 0600 permissions: owner read+write, no group/other access
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}
```

### Pattern 2: JiraClient Trait Behind Boundary (ARCH-02)

**What:** `JiraClient` trait with a single `fetch_title` method. `HttpJiraClient` is the production impl. Tests can inject a `FakeJiraClient`.
**When to use:** Anywhere JIRA titles are fetched.

```rust
// Source: ARCH-02 pattern (same structure as ProcessClient in infra/process.rs)
use async_trait::async_trait;

#[async_trait]
pub trait JiraClient: Send + Sync {
    /// Returns the ticket title for the given key (e.g. "UMP-1234"), or None on any error.
    async fn fetch_title(&self, ticket_key: &str) -> Option<String>;
}

pub struct HttpJiraClient {
    client: reqwest::Client,  // pre-configured with Authorization header + base_url
    base_url: String,
}

#[async_trait]
impl JiraClient for HttpJiraClient {
    async fn fetch_title(&self, ticket_key: &str) -> Option<String> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=summary",
            self.base_url, ticket_key
        );
        let resp: serde_json::Value = self.client.get(&url).send().await.ok()?
            .json().await.ok()?;
        resp["fields"]["summary"].as_str().map(|s| s.to_string())
    }
}
```

### Pattern 3: Background JIRA Fetch with TEA Action

**What:** After worktrees are loaded, extract ticket keys from branch names, spawn a background task that fetches titles, and deliver results via `Action::JiraTitlesFetched`.
**When to use:** In `update()` when handling `Action::WorktreesLoaded`.

```rust
// Source: Pattern matching infra/worktrees.rs list_worktrees / RefreshWorktrees flow

// In action.rs — new variant:
// JiraTitlesFetched(Vec<(String, String)>),  // (branch_name, title)

// In app.rs update(), branch WorktreesLoaded:
fn extract_ticket_key(branch: &str) -> Option<String> {
    // Branch names like "feature/UMP-1234-some-title" or "UMP-5678"
    let re_match = branch.split('/').flat_map(|s| s.split('-'))
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "UMP")
        .map(|w| format!("UMP-{}", w[1]))?;
    // Validate: UMP- followed by digits
    if re_match.split('-').nth(1).map(|s| s.chars().all(|c| c.is_ascii_digit())).unwrap_or(false) {
        Some(re_match)
    } else {
        None
    }
}

// Spawn title fetch for each uncached branch
let keys_to_fetch: Vec<(String, String)> = worktrees.iter()
    .filter_map(|wt| {
        let key = extract_ticket_key(&wt.branch)?;
        if cache.contains_key(&key) { return None; }
        Some((wt.branch.clone(), key))
    })
    .collect();

if !keys_to_fetch.is_empty() {
    let client = jira_client.clone();  // Arc<dyn JiraClient>
    let tx = metro_tx.clone();
    tokio::spawn(async move {
        let mut results = vec![];
        for (branch, key) in keys_to_fetch {
            if let Some(title) = client.fetch_title(&key).await {
                results.push((branch, title));
            }
        }
        let _ = tx.send(Action::JiraTitlesFetched(results));
    });
}
```

### Pattern 4: UMP-XXXX Extraction from Branch Names

**What:** Pure function extracting the JIRA ticket key from a branch name without regex dependency.
**When to use:** Per worktree, when loading or refreshing worktrees.

```rust
// Source: Standard Rust string parsing — no regex crate needed
pub fn extract_jira_key(branch: &str) -> Option<String> {
    // Handles: "feature/UMP-1234-some-desc", "UMP-5678", "UMP-1234"
    for segment in branch.split('/') {
        let parts: Vec<&str> = segment.splitn(3, '-').collect();
        if parts.len() >= 2 && parts[0] == "UMP" {
            let digits = parts[1];
            if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                return Some(format!("UMP-{}", digits));
            }
        }
    }
    None
}
```

### Pattern 5: tmux Detection

**What:** One-liner on startup to determine if running inside tmux. Store result in `AppState`.
**When to use:** Once at startup in `run()`.

```rust
// Source: std::env (stdlib)
// tmux always sets TMUX env var to the socket path when a session is active.
pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

// In AppState (new field):
pub tmux_available: bool,

// In run() before the loop:
state.tmux_available = is_inside_tmux();
```

### Pattern 6: JiraClient Construction with Cloud Auth

**What:** Build `HttpJiraClient` with Basic Auth header baked into a reusable `reqwest::Client`.
**When to use:** Once at startup, if config file exists.

```rust
// Source: docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
use reqwest::header;

pub fn build_jira_client(config: &DashConfig) -> anyhow::Result<HttpJiraClient> {
    // Cloud: Basic Auth — base64("email:api_token")
    let credentials = base64_encode(format!("{}:{}", config.jira_email, config.jira_token));
    let auth_value = format!("Basic {}", credentials);

    let mut auth_header = header::HeaderValue::from_str(&auth_value)?;
    auth_header.set_sensitive(true);  // prevents logging credentials

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, auth_header);
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(HttpJiraClient { client, base_url: config.jira_base_url.clone() })
}

// For Data Center (PAT bearer token — use if config specifies dc mode):
// let auth_value = format!("Bearer {}", config.jira_token);
```

### Anti-Patterns to Avoid

- **Blocking the event loop with JIRA HTTP calls:** Never call reqwest without `tokio::spawn`. JIRA calls can take 500ms–5s. One blocking call freezes the entire TUI.
- **Storing raw token in AppState:** Token belongs in infra config only. AppState should hold `Option<Arc<dyn JiraClient>>`, not the token string.
- **Panicking when JIRA is unreachable:** Use `Option<String>` return, `.ok()` on errors. Show branch name instead of title when fetch fails — never show error to user (success criterion 3).
- **Using regex crate for UMP-XXXX extraction:** Simple string split is sufficient and keeps the dependency count low.
- **Fetching all tickets on every refresh:** Check the cache first. Only fetch tickets not already cached.
- **Writing config with default permissions:** Always set 0600 immediately after write — default umask may allow group read on some systems.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP Basic Auth encoding | Custom base64 implementation | `reqwest::RequestBuilder::basic_auth()` | reqwest handles base64 + header format automatically |
| TLS certificate validation | Custom cert pinning | `reqwest` with default rustls | reqwest uses system/bundled certs; hand-rolling TLS is a security risk |
| JSON deserialization of JIRA response | Manual string parsing | `serde_json::Value` indexing | JIRA response schema can change; Value indexing is safe and stable |
| Async HTTP pool management | Custom connection pool | `reqwest::Client` (stateful, reusable) | Client is designed to be reused; creates connection pool automatically |

**Key insight:** reqwest + serde_json eliminate 95% of the HTTP+JSON plumbing. The only custom code needed is config reading, key extraction, and caching.

---

## Common Pitfalls

### Pitfall 1: Wrong Auth Scheme (Cloud vs Data Center)

**What goes wrong:** Calling Cloud endpoint with Bearer PAT returns HTTP 401/403. Calling Data Center with Basic Auth may also fail.
**Why it happens:** Atlassian Cloud uses Basic Auth with `email:api_token` (base64 encoded). Atlassian Data Center uses Bearer PAT (no email prefix, no base64). The two are incompatible.
**How to avoid:** Config file must store an `auth_mode` field: `"cloud"` or `"datacenter"`. Build the client accordingly. If `auth_mode` is absent, default to Cloud (most common).
**Warning signs:** HTTP 401 response even with a valid-looking token.

### Pitfall 2: reqwest 0.13 Default TLS Feature

**What goes wrong:** Linking errors or TLS failures on macOS if native-tls is used instead of rustls.
**Why it happens:** reqwest 0.12+ switched defaults; 0.13 uses rustls by default. macOS systems with specific OpenSSL configurations can have issues with native-tls.
**How to avoid:** Use `features = ["json", "rustls-tls"]` explicitly. Do not add `native-tls` unless testing confirms it is needed.
**Warning signs:** Link errors mentioning OpenSSL, TLS handshake failures.

### Pitfall 3: JIRA Response JSON Shape

**What goes wrong:** Attempting to deserialize into a typed struct and crashing when JIRA returns unexpected shape.
**Why it happens:** JIRA Cloud returns `{"fields": {"summary": "..."}}` for GET issue, but the exact nesting path changes between v2 and v3.
**How to avoid:** Use `serde_json::Value` and index with `resp["fields"]["summary"].as_str()`. This returns `None` gracefully if the path is wrong. The v3 endpoint is `GET /rest/api/3/issue/{issueIdOrKey}?fields=summary`.
**Warning signs:** Panics or unexpected None values when titles appear in the JIRA web UI.

### Pitfall 4: Race Between WorktreesLoaded and JiraTitlesFetched

**What goes wrong:** Titles arrive and are applied, then `WorktreesLoaded` fires again (e.g., on refresh) and clears `jira_title` fields back to None.
**Why it happens:** `WorktreesLoaded` sets `jira_title: None` on new Worktree structs from the parser. A subsequent re-load wipes the applied titles.
**How to avoid:** In `update()` for `WorktreesLoaded`, after setting worktrees, apply cached titles from the in-memory `jira_title_cache: HashMap<String, String>` in AppState (keyed by ticket key, not branch). Always re-apply cache on every load.
**Warning signs:** Titles flash on screen then disappear after a manual refresh.

### Pitfall 5: Config File Not Created on First Run

**What goes wrong:** User places config file manually in `~/.config/ump-dash/` but the directory doesn't exist.
**Why it happens:** The requirement says "user can place a JIRA API token" — the dashboard should not create the config, just read it. But the directory must exist for labels to work.
**How to avoid:** `labels.rs` already calls `create_dir_all` when saving labels. Reading config only needs `match std::fs::read_to_string(path) { Err(NotFound) => Ok(None), ... }` — same pattern as `load_labels()`.
**Warning signs:** Missing file causes JIRA fetch attempt with empty credentials.

### Pitfall 6: Blocking tokio::spawn Tasks During Shutdown

**What goes wrong:** JIRA fetch tasks still running when user presses 'q'. Tasks hold reqwest connections open.
**Why it happens:** There's no cancellation token for JIRA fetches.
**How to avoid:** Store the `JoinHandle<()>` for the JIRA fetch task in AppState (like `command_task`). Abort it in the `Quit` action handler or simply `drop` the handle (tokio detaches — this is acceptable since the process exits immediately).
**Warning signs:** Slow process exit when network is unavailable.

---

## Code Examples

Verified patterns from official sources:

### Config Struct for `~/.config/ump-dash/config.json`

```rust
// No external source needed — uses serde derive already in Cargo.toml
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DashConfig {
    /// JIRA instance base URL, e.g. "https://your-org.atlassian.net"
    pub jira_base_url: String,
    /// Email address associated with the API token (Cloud only)
    #[serde(default)]
    pub jira_email: Option<String>,
    /// API token (Cloud) or Personal Access Token (Data Center)
    pub jira_token: String,
    /// "cloud" or "datacenter" — defaults to "cloud" if absent
    #[serde(default = "default_auth_mode")]
    pub auth_mode: String,
}

fn default_auth_mode() -> String { "cloud".to_string() }

pub fn load_config() -> anyhow::Result<Option<DashConfig>> {
    let path = config_dir().join("config.json");
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(serde_json::from_str(&contents)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
```

### reqwest Client Construction (Cloud Basic Auth)

```rust
// Source: docs.rs/reqwest/0.13/reqwest/struct.ClientBuilder.html
use reqwest::header;
use std::fmt::Write;

pub fn build_cloud_client(email: &str, token: &str) -> anyhow::Result<reqwest::Client> {
    // reqwest::RequestBuilder::basic_auth handles base64 encoding automatically
    // But per-request basic_auth is more ergonomic than building into default headers
    // Use default_headers approach for a reusable pre-authorized client:
    let credentials = base64_encode(&format!("{}:{}", email, token));
    let mut auth = header::HeaderValue::from_str(&format!("Basic {}", credentials))?;
    auth.set_sensitive(true);  // suppresses value from debug output

    let mut map = header::HeaderMap::new();
    map.insert(header::AUTHORIZATION, auth);

    Ok(reqwest::Client::builder().default_headers(map).build()?)
}

fn base64_encode(input: &str) -> String {
    use std::io::Write as _;
    // base64 encoding without adding an external crate:
    // reqwest's basic_auth() method does this internally via the http-auth crate it bundles.
    // Alternative: use the basic_auth() method per-request instead (no manual base64 needed):
    //   client.get(url).basic_auth(email, Some(token)).send().await
    // This is cleaner — avoids manual base64.
    // Note: std does not include base64. Use .basic_auth() or add `base64` crate.
    todo!("use .basic_auth() per-request or add base64 = \"0.22\" to Cargo.toml")
}
```

**Recommended simpler approach (avoids manual base64):**

```rust
// Source: docs.rs/reqwest/0.13/reqwest/struct.RequestBuilder.html#method.basic_auth
// basic_auth() handles base64 encoding internally — no extra crate needed.
let resp = client
    .get(&url)
    .basic_auth(&config.jira_email.as_deref().unwrap_or(""), Some(&config.jira_token))
    .send()
    .await?;
```

**Or pre-bake auth into the Client using bearer_auth for Data Center:**

```rust
// Data Center: Bearer PAT
let resp = client
    .get(&url)
    .bearer_auth(&config.jira_token)
    .send()
    .await?;
```

### JIRA Title Fetch

```rust
// Source: JIRA REST API v3 official docs — GET /rest/api/3/issue/{issueIdOrKey}
pub async fn fetch_jira_title(
    client: &reqwest::Client,
    base_url: &str,
    ticket_key: &str,
    config: &DashConfig,
) -> Option<String> {
    let url = format!("{}/rest/api/3/issue/{}?fields=summary", base_url, ticket_key);

    let builder = client.get(&url);
    let builder = if config.auth_mode == "datacenter" {
        builder.bearer_auth(&config.jira_token)
    } else {
        // Cloud: Basic Auth with email:token
        builder.basic_auth(
            config.jira_email.as_deref().unwrap_or(""),
            Some(&config.jira_token),
        )
    };

    let resp: serde_json::Value = builder.send().await.ok()?.json().await.ok()?;
    resp["fields"]["summary"].as_str().map(|s| s.to_string())
}
```

### JIRA Cache (same pattern as labels.rs)

```rust
// ~/.config/ump-dash/jira_cache.json
// Format: { "UMP-1234": "Fix the login screen crash", ... }

pub fn cache_path() -> std::path::PathBuf {
    config_dir().join("jira_cache.json")  // config_dir() from labels.rs
}

pub fn load_jira_cache() -> anyhow::Result<std::collections::HashMap<String, String>> {
    let path = cache_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(serde_json::from_str(&contents)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Default::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_jira_cache(cache: &std::collections::HashMap<String, String>) -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(cache)?;
    std::fs::write(cache_path(), json)?;
    // Cache is not sensitive — no 0600 needed (titles are not credentials)
    Ok(())
}
```

### tmux Detection

```rust
// Source: std::env stdlib — no crate needed
// TMUX env var is set by tmux to the socket path (e.g. "/tmp/tmux-1000/default,1234,0")
// when a pane is inside a tmux session.
pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}
```

### UMP Ticket Key Extraction

```rust
// Source: pure Rust string parsing — verified against branch naming pattern
// Handles: "feature/UMP-1234-desc", "fix/UMP-5678", "UMP-9012"
pub fn extract_jira_key(branch: &str) -> Option<String> {
    for segment in branch.split('/') {
        let mut parts = segment.splitn(3, '-');
        if let (Some(prefix), Some(digits)) = (parts.next(), parts.next()) {
            if prefix == "UMP" && !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                return Some(format!("UMP-{}", digits));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_jira_key() {
        assert_eq!(extract_jira_key("feature/UMP-1234-login"), Some("UMP-1234".into()));
        assert_eq!(extract_jira_key("UMP-5678"), Some("UMP-5678".into()));
        assert_eq!(extract_jira_key("fix/UMP-9-short"), Some("UMP-9".into()));
        assert_eq!(extract_jira_key("main"), None);
        assert_eq!(extract_jira_key("feature/no-ticket"), None);
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| reqwest 0.11 with openssl | reqwest 0.13 with rustls | 2024-2025 | No OpenSSL system dependency; simpler macOS builds |
| reqwest 0.11 blocking client | reqwest 0.13 async Client | stable since 0.11 | Matches tokio runtime already in project |
| JIRA Basic Auth with password | JIRA Basic Auth with API token | Atlassian policy change ~2022 | Password-based auth deprecated; API tokens are required |
| Manual base64 for Basic Auth | reqwest::RequestBuilder::basic_auth() | reqwest ~0.10 | No external base64 crate needed |

**Deprecated/outdated:**
- JIRA v2 API (`/rest/api/2/`): Still works but v3 is current; use v3 for new code
- JIRA password-based Basic Auth: Atlassian removed password auth for API calls; tokens required

---

## Open Questions

1. **Cloud vs Data Center JIRA**
   - What we know: Cloud = Basic Auth with email:token. Data Center = Bearer PAT (no email).
   - What's unclear: Which environment does this project's JIRA instance use? STATE.md confirms this is unresolved: "JIRA auth method (Cloud vs Data Center) unconfirmed — validate before Phase 4 implementation."
   - Recommendation: Add `auth_mode` field to `config.json` (defaults to `"cloud"`). Support both in `fetch_jira_title()` — branch on the field. User documents which to set. This eliminates the guessing risk with minimal code cost.

2. **JIRA base URL — how does the user configure it?**
   - What we know: Cloud URLs follow `https://org-name.atlassian.net`. Data Center URLs vary per installation.
   - What's unclear: Should the base URL be hardcoded for the UMP project, or read from config?
   - Recommendation: Read from `config.json` as `jira_base_url`. Do not hardcode. This keeps the tool portable.

3. **Concurrent vs sequential title fetches**
   - What we know: `tokio::join_all` or `futures::join_all` can fetch all tickets concurrently.
   - What's unclear: Is the JIRA instance rate-limited? Fetching 10+ tickets simultaneously may hit rate limits.
   - Recommendation: Fetch sequentially in the background task on first implementation (simpler, safer). Can be parallelized in a follow-up if it is too slow.

---

## AppState Changes Required

The following fields must be added to `AppState` (app.rs):

```rust
// Phase 4 fields
pub tmux_available: bool,
pub jira_title_cache: std::collections::HashMap<String, String>,  // UMP-XXXX -> title
pub jira_client: Option<Arc<dyn JiraClient + Send + Sync>>,
pub jira_config: Option<DashConfig>,
pub jira_fetch_task: Option<tokio::task::JoinHandle<()>>,
```

And `Default` impl must initialize:
- `tmux_available: is_inside_tmux()`
- `jira_title_cache: load_jira_cache().unwrap_or_default()`
- `jira_client: None` (populated in `run()` after config load)

---

## Sources

### Primary (HIGH confidence)
- [reqwest 0.13 RequestBuilder docs](https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html) — basic_auth(), bearer_auth(), header(), send() API
- [reqwest 0.13 ClientBuilder docs](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html) — default_headers(), build()
- [Atlassian Basic Auth docs](https://developer.atlassian.com/cloud/jira/platform/basic-auth-for-rest-apis/) — Cloud = email:api_token Basic Auth
- [Atlassian PAT docs](https://confluence.atlassian.com/enterprise/using-personal-access-tokens-1026032365.html) — Data Center = Bearer PAT
- [std::os::unix::fs::PermissionsExt](https://doc.rust-lang.org/std/os/unix/fs/trait.PermissionsExt.html) — from_mode(0o600)
- [std::env](https://doc.rust-lang.org/std/env/index.html) — var("TMUX")
- Existing project code: `src/infra/labels.rs` — config dir pattern, JSON read/write pattern

### Secondary (MEDIUM confidence)
- JIRA REST API v3 GET issue endpoint: `GET /rest/api/3/issue/{key}?fields=summary` — response has `fields.summary` — confirmed by multiple developer community sources
- reqwest version 0.13.1 released December 30, 2025 — from GitHub README

### Tertiary (LOW confidence)
- JIRA rate limiting behavior — not documented in official sources; sequential fetch recommended as conservative default

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — reqwest 0.13 verified via GitHub, serde already in project, stdlib functions confirmed in official docs
- Architecture: HIGH — follows established patterns from process.rs (trait boundary) and labels.rs (config dir)
- JIRA API: MEDIUM — endpoint and response shape confirmed via multiple sources, but auth mode is project-specific
- Pitfalls: HIGH — Cloud/DC auth split is a documented hard difference; race condition is an architectural inference from the existing WorktreesLoaded pattern
- tmux detection: HIGH — TMUX env var is tmux's documented behavior, confirmed by multiple sources

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (reqwest 0.13 is stable; JIRA REST API v3 is stable)
