# Phase 3: Worktree Browser, Git, and RN Commands - Research

**Researched:** 2026-03-02
**Domain:** Rust async command execution, git worktree enumeration, ratatui List widget with StatefulWidget, label persistence, node_modules staleness detection, iOS/Android device enumeration
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WORK-01 | User sees a list of all worktrees with their current branch name | `git worktree list --porcelain` output parsed in infra layer; `Worktree` domain struct holds path + branch; rendered via ratatui `List` + `ListState` StatefulWidget |
| WORK-02 | User sees the JIRA ticket title next to the branch name (fetched via API from UMP-XXXX pattern) | Placeholder field in `Worktree` struct: `jira_title: Option<String>` — defaults to None (branch name shown); JIRA fetch deferred to Phase 4; Phase 3 shows branch name or placeholder label |
| WORK-03 | User can set a custom label on a branch that persists across worktrees (label follows the branch, not the worktree) | `HashMap<String, String>` (branch_name → label) persisted at `~/.config/ump-dash/labels.json`; `serde + serde_json` already transitive deps — just add them explicitly to Cargo.toml |
| WORK-05 | User sees dependency staleness hints when node_modules is outdated relative to package.json/yarn.lock | `std::fs::metadata(path).modified()` comparison: `max(package.json mtime, yarn.lock mtime) > node_modules mtime` → `stale: bool` field on `Worktree` |
| WORK-06 | Stale dependencies are lazily installed before launching the app if user hasn't manually installed | Before dispatching `run-android` or `run-ios` actions: check `stale` flag; if true, prepend a `yarn install` command task that must complete before the RN run command is spawned |
| GIT-01 | User can run git reset --hard origin/<current-branch> on a selected worktree | `CommandRunner` trait in infra; `run_git` concrete impl using `tokio::process::Command`; destructive → confirmation overlay required before execution |
| GIT-02 | User can run git pull on a selected worktree | Same `CommandRunner` trait; non-destructive; no confirmation required |
| GIT-03 | User can run git push on a selected worktree | Same `CommandRunner` trait; non-destructive |
| GIT-04 | User can run git rebase origin/<target-branch> on a selected worktree | Same `CommandRunner` trait; non-destructive; branch name input required |
| GIT-05 | User can run git checkout <branch> to switch branches in a worktree | Same `CommandRunner` trait; branch name input required |
| GIT-06 | User can run git checkout -b <branch> to create and switch to a new branch | Same `CommandRunner` trait; branch name input required |
| RN-01 | User can run npx react-native clean --include 'android' on a selected worktree | Same `CommandRunner` trait; destructive (clears build artifacts) → confirmation prompt |
| RN-02 | User can run npx react-native clean --include 'cocoapods' on a selected worktree | Same `CommandRunner` trait; destructive → confirmation prompt |
| RN-03 | User can run rm -rf node_modules on a selected worktree | Same `CommandRunner` trait; highly destructive → confirmation required |
| RN-04 | User can run yarn install on a selected worktree | Same `CommandRunner` trait; non-destructive |
| RN-05 | User can run yarn pod-install on a selected worktree | Same `CommandRunner` trait; non-destructive |
| RN-06 | User can run npx react-native run-android on a selected worktree with device selection (from adb devices list) | `adb devices` output parsed in infra; device selection popup before spawn; if only one device, skip popup |
| RN-07 | User can run yarn react-native run-ios on a selected worktree with device/simulator selection | `xcrun simctl list devices available --json` parsed in infra; selector popup; if only one booted sim, skip popup |
| RN-08 | User can run yarn unit-tests on a selected worktree | Same `CommandRunner` trait; non-destructive |
| RN-09 | User can run yarn jest with a test filter on a selected worktree | Same `CommandRunner` trait; text input required for filter string |
| RN-10 | User can run yarn lint --quiet --fix on a selected worktree | Same `CommandRunner` trait; non-destructive |
| RN-11 | User can run yarn check-types --incremental on a selected worktree | Same `CommandRunner` trait; `--incremental` flag is mandatory (CLAUDE.md constraint) |
| RN-12 | User sees streaming command output in a panel while commands execute | Same `stream_metro_logs` pattern from Phase 2; output lines sent as `Action::CommandOutputLine(String)` via mpsc; rendered in existing `CommandOutput` panel |
</phase_requirements>

---

## Summary

Phase 3 builds three interconnected systems on top of the Phase 1/2 scaffold: (1) a live worktree browser that reads all git worktrees from disk and displays them as a selectable list with branch names, metro status badges, JIRA title placeholders, and custom labels; (2) a generic command runner that streams any git or RN command's stdout/stderr into the existing `CommandOutput` panel — reusing the exact same mpsc + BufReader pattern established for metro logs in Phase 2; and (3) device selection popups for `run-android` (adb) and `run-ios` (xcrun simctl) with selection UI.

The core technical challenge is designing a single `CommandSpec` type that can represent all 18+ commands (6 git ops + 12 RN commands) uniformly, so the streaming task, UI rendering, and keybinding dispatch all work through one pipeline rather than 18 bespoke implementations. The pattern: `CommandSpec` (domain type — pure description of what to run) → `spawn_command_task` (infra — tokio::process, sends `CommandOutputLine` actions back) → `CommandOutput` panel (UI — renders VecDeque<String> using the scrollable Paragraph pattern already established).

Label persistence uses `serde_json` (already a transitive dep; just needs explicit Cargo.toml entry) to store a `HashMap<branch_name, label>` at `~/.config/ump-dash/labels.json`. Staleness detection uses `std::fs::metadata().modified()` to compare mtimes — no external crates needed.

**Primary recommendation:** Model all commands as variants of a `CommandSpec` enum in the domain layer. Keep command execution behind a `CommandRunner` trait in the infra layer. Reuse the Phase 2 log streaming pattern exactly (mpsc unbounded_channel, BufReader lines, Action variant) for all command output. Store ListState in AppState for worktree selection. Use the existing `Clear` + overlay pattern for confirmation dialogs and text input prompts.

---

## Standard Stack

### Core (new additions for Phase 3)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.x (already transitive) | Serialize/deserialize label config | Already in Cargo.lock via color-eyre deps |
| serde_json | 1.x (already transitive) | JSON format for labels.json | Already in Cargo.lock; simpler than TOML for a small HashMap |
| ratatui::widgets::List + ListState | 0.30 (already in Cargo.toml) | Selectable worktree list | Official ratatui StatefulWidget for lists with selection |
| ratatui::widgets::Clear | 0.30 (already in Cargo.toml) | Modal overlays (confirmation, text input, device picker) | Established pattern in codebase from Phase 1 overlays |

### No New Mandatory Crates

Everything needed is already present:
- `tokio` (features="full") — command spawning, mpsc channels, BufReader
- `anyhow` — error handling for command failures
- `serde` + `serde_json` — add explicitly to Cargo.toml (already in lockfile as transitive deps)

### Optional (add only if needed)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| dirs | 5.x | `dirs::config_dir()` for XDG config path | Only if hardcoding `~/.config` is insufficient; on macOS `~/.config` is reliable |

**Note on `dirs` crate:** The `dirs` crate is NOT currently in the lockfile. `~/.config/ump-dash/` is explicitly called out in PROJECT.md. On macOS, `std::env::var("HOME").map(|h| format!("{}/.config/ump-dash", h))` works without any extra crate. Avoid adding `dirs` unless needed.

### Installation

```toml
# Add to Cargo.toml [dependencies] — these are already in the lockfile, just not explicit deps
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Architecture Patterns

### Recommended Project Structure (Phase 3 additions)

```
src/
├── action.rs              # Add: WorktreeSelect, CommandRun(CommandSpec), CommandOutputLine(String),
│                          #      CommandExited, CommandOutputClear, SetLabel(String),
│                          #      DeviceSelected(DeviceId), ConfirmAction, DismissModal
│
├── app.rs                 # Add: worktrees: Vec<Worktree>, worktree_list_state: ListState,
│                          #      selected_worktree_idx: usize, command_output: VecDeque<String>,
│                          #      command_output_scroll: usize, running_command: Option<CommandSpec>,
│                          #      modal: Option<ModalState>
│
├── domain/
│   ├── mod.rs             # Add: pub mod command;
│   ├── worktree.rs        # Expand: add branch, path, metro_status, jira_title, label, stale fields
│   └── command.rs         # NEW: CommandSpec enum, DeviceId, ModalState enum
│
├── infra/
│   ├── mod.rs             # Add: pub mod worktrees; pub mod command_runner; pub mod devices;
│   ├── worktrees.rs       # NEW: WorktreeClient trait + GitWorktreeClient (git worktree list --porcelain)
│   ├── command_runner.rs  # NEW: CommandRunner trait + TokioCommandRunner (generic command spawner)
│   ├── devices.rs         # NEW: DeviceClient trait + impls for adb + xcrun simctl
│   └── labels.rs          # NEW: LabelStore trait + FileLabelStore (~/.config/ump-dash/labels.json)
│
└── ui/
    ├── panels.rs          # Replace worktree placeholder with real List; replace output placeholder
    │                      # with scrollable VecDeque rendering
    └── modals.rs          # NEW: render_confirm_modal(), render_text_input_modal(),
                           #      render_device_picker_modal()
```

### Pattern 1: Worktree Domain Struct (replacing stub)

**What:** Replace the stub `Worktree` struct with a fully-populated domain type. All fields populated from disk without any network call (WORK-01 success criterion).

**When to use:** This is the single source of truth for worktree data across all panels.

```rust
// src/domain/worktree.rs
// Source: project requirements WORK-01 through WORK-06

/// Unique identifier for a worktree — absolute filesystem path.
/// Using the path as the ID because it uniquely identifies a worktree at the OS level.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorktreeId(pub String);

/// Metro running state as observed per-worktree.
/// Derived from MetroManager.status in AppState — not stored independently here.
#[derive(Debug, Clone, PartialEq)]
pub enum WorktreeMetroStatus {
    Running,
    Stopped,
}

/// Fully-populated worktree domain type. All fields come from disk; no network.
#[derive(Debug, Clone)]
pub struct Worktree {
    pub id: WorktreeId,            // absolute path
    pub path: std::path::PathBuf,  // absolute path (same as id inner value)
    pub branch: String,            // e.g. "UMP-7424" or "rc-thunk"
    pub head_sha: String,          // 7-char abbreviated sha for display
    pub metro_status: WorktreeMetroStatus, // derived from AppState.metro on render
    pub jira_title: Option<String>,// None until Phase 4 fetches it; display branch if None
    pub label: Option<String>,     // user-set custom label (persisted per branch)
    pub stale: bool,               // true if node_modules older than package.json or yarn.lock
}

impl Worktree {
    /// Returns the display name: custom label > jira_title > branch name.
    pub fn display_name(&self) -> &str {
        if let Some(ref label) = self.label {
            label
        } else if let Some(ref title) = self.jira_title {
            title
        } else {
            &self.branch
        }
    }
}
```

### Pattern 2: git worktree list --porcelain Parser

**What:** Parse the machine-readable output of `git worktree list --porcelain` to enumerate all worktrees. Run once on startup, refresh on a periodic tick or explicit refresh action.

**When to use:** `WorktreeClient::list_worktrees()` called in infra layer. Domain layer receives `Vec<Worktree>`.

```rust
// src/infra/worktrees.rs
// Source: verified against actual UMP repo output (git 2.51.0)

// Actual output format for git worktree list --porcelain:
//
//   worktree /Users/cubicme/aljazeera/ump
//   HEAD 7ab5d79fb7e6e430e7af3323a3dd04f55adcb0c7
//   branch refs/heads/rc-thunk
//
//   worktree /Users/cubicme/aljazeera/ump-branches/develop
//   HEAD b8d9439db85257c4195df4821f73cf8a813f0659
//   branch refs/heads/UMP-6441-e2e-fix
//
// Note: each entry separated by blank line.
// Note: "branch" line missing for detached HEAD — "detached" keyword appears instead.

use crate::domain::worktree::{Worktree, WorktreeId, WorktreeMetroStatus};
use std::path::PathBuf;
use anyhow::Result;

#[async_trait::async_trait]
pub trait WorktreeClient: Send + Sync {
    /// Return all worktrees by parsing git worktree list --porcelain.
    /// Path is discovered via MetroHandle.worktree_id or by asking the git repo.
    async fn list_worktrees(&self, repo_root: PathBuf) -> Result<Vec<Worktree>>;
}

pub struct GitWorktreeClient;

#[async_trait::async_trait]
impl WorktreeClient for GitWorktreeClient {
    async fn list_worktrees(&self, repo_root: PathBuf) -> Result<Vec<Worktree>> {
        let output = tokio::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&repo_root)
            .output()
            .await?;

        let text = String::from_utf8_lossy(&output.stdout);
        parse_worktree_porcelain(&text)
    }
}

/// Pure function: parse the porcelain output into Worktree values.
/// Returns Err if the output is malformed.
fn parse_worktree_porcelain(text: &str) -> Result<Vec<Worktree>> {
    let mut result = Vec::new();
    for entry in text.split("\n\n").filter(|s| !s.trim().is_empty()) {
        let mut path: Option<PathBuf> = None;
        let mut head: Option<String> = None;
        let mut branch: Option<String> = None;

        for line in entry.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(PathBuf::from(p.trim()));
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                head = Some(h.trim().to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                branch = Some(b.trim().to_string());
            } else if line.trim() == "detached" {
                branch = Some("(detached)".to_string());
            }
        }

        if let (Some(path), Some(head)) = (path, head) {
            let branch_name = branch.unwrap_or_else(|| "(detached)".to_string());
            let head_sha = head.chars().take(7).collect();
            let stale = check_stale(&path);

            result.push(Worktree {
                id: WorktreeId(path.to_string_lossy().to_string()),
                path: path.clone(),
                branch: branch_name,
                head_sha,
                metro_status: WorktreeMetroStatus::Stopped, // derived later from MetroManager
                jira_title: None,  // Phase 4
                label: None,       // loaded from label store after construction
                stale,
            });
        }
    }
    Ok(result)
}

/// Returns true if node_modules is older than package.json or yarn.lock.
/// Uses std::fs::metadata().modified() — no external crates needed.
fn check_stale(worktree_path: &std::path::Path) -> bool {
    use std::fs;
    use std::time::SystemTime;

    let nm_mtime = fs::metadata(worktree_path.join("node_modules"))
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let pkg_mtime = fs::metadata(worktree_path.join("package.json"))
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let lock_mtime = fs::metadata(worktree_path.join("yarn.lock"))
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let newest_dep_file = pkg_mtime.max(lock_mtime);
    newest_dep_file > nm_mtime
}
```

### Pattern 3: CommandSpec — Uniform Command Representation

**What:** A domain enum representing every runnable command. Keeps command dispatch, streaming, and UI completely uniform — no special-casing per command.

**When to use:** All user-initiated commands flow through this type. `CommandRun(CommandSpec)` is the single action variant for launching any command.

```rust
// src/domain/command.rs
// Source: requirements GIT-01 through RN-12

#[derive(Debug, Clone, PartialEq)]
pub enum CommandSpec {
    // Git operations
    GitResetHard,                           // GIT-01: destructive
    GitPull,                                // GIT-02
    GitPush,                                // GIT-03
    GitRebase { target: String },           // GIT-04: target = "origin/<branch>"
    GitCheckout { branch: String },         // GIT-05
    GitCheckoutNew { branch: String },      // GIT-06

    // RN build/clean commands
    RnCleanAndroid,                         // RN-01: destructive
    RnCleanCocoapods,                       // RN-02: destructive
    RmNodeModules,                          // RN-03: highly destructive
    YarnInstall,                            // RN-04
    YarnPodInstall,                         // RN-05
    RnRunAndroid { device_id: String },     // RN-06: requires device selection
    RnRunIos { device_id: String },         // RN-07: requires device/simulator selection
    YarnUnitTests,                          // RN-08
    YarnJest { filter: String },            // RN-09: requires text input
    YarnLint,                               // RN-10
    YarnCheckTypes,                         // RN-11: always --incremental (CLAUDE.md)
}

impl CommandSpec {
    /// Returns the argv to pass to tokio::process::Command.
    /// The first element is the program; the rest are arguments.
    pub fn to_argv(&self) -> Vec<String> {
        match self {
            Self::GitResetHard => {
                // Caller injects the current branch name at spawn time
                vec!["git".into(), "reset".into(), "--hard".into()]
                // Note: full command needs current branch → infra layer adds "origin/<branch>"
            }
            Self::GitPull => vec!["git".into(), "pull".into()],
            Self::GitPush => vec!["git".into(), "push".into()],
            Self::GitRebase { target } =>
                vec!["git".into(), "rebase".into(), target.clone()],
            Self::GitCheckout { branch } =>
                vec!["git".into(), "checkout".into(), branch.clone()],
            Self::GitCheckoutNew { branch } =>
                vec!["git".into(), "checkout".into(), "-b".into(), branch.clone()],
            Self::RnCleanAndroid =>
                vec!["npx".into(), "react-native".into(), "clean".into(),
                     "--include".into(), "android".into()],
            Self::RnCleanCocoapods =>
                vec!["npx".into(), "react-native".into(), "clean".into(),
                     "--include".into(), "cocoapods".into()],
            Self::RmNodeModules =>
                vec!["rm".into(), "-rf".into(), "node_modules".into()],
            Self::YarnInstall => vec!["yarn".into(), "install".into()],
            Self::YarnPodInstall => vec!["yarn".into(), "pod-install".into()],
            Self::RnRunAndroid { device_id } =>
                vec!["npx".into(), "react-native".into(), "run-android".into(),
                     "--deviceId".into(), device_id.clone()],
            Self::RnRunIos { device_id } =>
                vec!["yarn".into(), "react-native".into(), "run-ios".into(),
                     "--udid".into(), device_id.clone()],
            Self::YarnUnitTests => vec!["yarn".into(), "unit-tests".into()],
            Self::YarnJest { filter } =>
                vec!["yarn".into(), "jest".into(), filter.clone()],
            Self::YarnLint =>
                vec!["yarn".into(), "lint".into(), "--quiet".into(), "--fix".into()],
            Self::YarnCheckTypes =>
                // CLAUDE.md: check-types ALWAYS uses --incremental flag
                vec!["yarn".into(), "check-types".into(), "--incremental".into()],
        }
    }

    /// True if this command requires a destructive confirmation dialog before running.
    pub fn is_destructive(&self) -> bool {
        matches!(self,
            Self::GitResetHard |
            Self::RnCleanAndroid |
            Self::RnCleanCocoapods |
            Self::RmNodeModules
        )
    }

    /// True if this command requires text input before spawning.
    pub fn needs_text_input(&self) -> bool {
        matches!(self,
            Self::GitRebase { .. } |
            Self::GitCheckout { .. } |
            Self::GitCheckoutNew { .. } |
            Self::YarnJest { .. }
        )
    }

    /// True if this command requires device selection before spawning.
    pub fn needs_device_selection(&self) -> bool {
        matches!(self, Self::RnRunAndroid { .. } | Self::RnRunIos { .. })
    }

    /// Human-readable label for footer/prompt display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::GitResetHard => "git reset --hard",
            Self::GitPull => "git pull",
            Self::GitPush => "git push",
            Self::GitRebase { .. } => "git rebase",
            Self::GitCheckout { .. } => "git checkout",
            Self::GitCheckoutNew { .. } => "git checkout -b",
            Self::RnCleanAndroid => "clean android",
            Self::RnCleanCocoapods => "clean cocoapods",
            Self::RmNodeModules => "rm -rf node_modules",
            Self::YarnInstall => "yarn install",
            Self::YarnPodInstall => "yarn pod-install",
            Self::RnRunAndroid { .. } => "run-android",
            Self::RnRunIos { .. } => "run-ios",
            Self::YarnUnitTests => "yarn unit-tests",
            Self::YarnJest { .. } => "yarn jest",
            Self::YarnLint => "yarn lint",
            Self::YarnCheckTypes => "yarn check-types",
        }
    }
}
```

### Pattern 4: CommandRunner Trait + Generic Spawn Task

**What:** An infra trait that spawns any `CommandSpec` as a child process in a given worktree directory, streams stdout/stderr as `Action::CommandOutputLine` events.

**When to use:** Every command dispatch flows through this. Same mpsc pattern as Phase 2 metro log streaming.

```rust
// src/infra/command_runner.rs
// Source: ARCH-02; reuses tokio pattern from Phase 2 infra/process.rs

use crate::action::Action;
use crate::domain::command::CommandSpec;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

/// Spawns a command and streams its output back as Action variants.
/// Returns the task JoinHandle so callers can abort on cancel.
pub async fn spawn_command_task(
    spec: CommandSpec,
    worktree_path: PathBuf,
    current_branch: String,  // needed for GitResetHard to build "origin/<branch>"
    action_tx: UnboundedSender<Action>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let argv = build_argv(&spec, &current_branch);
        let program = &argv[0];
        let args = &argv[1..];

        let mut cmd = tokio::process::Command::new(program);
        cmd.args(args)
            .current_dir(&worktree_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        match cmd.spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take().expect("piped");
                let stderr = child.stderr.take().expect("piped");
                stream_command_output(stdout, stderr, action_tx.clone()).await;
                // Wait for process to finish
                let _ = child.wait().await;
            }
            Err(e) => {
                let _ = action_tx.send(Action::CommandOutputLine(
                    format!("[error] failed to spawn: {e}")
                ));
            }
        }
        let _ = action_tx.send(Action::CommandExited);
    })
}

/// Read stdout + stderr, send each line as CommandOutputLine.
async fn stream_command_output(
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    tx: UnboundedSender<Action>,
) {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut out_lines = BufReader::new(stdout).lines();
    let mut err_lines = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            line = out_lines.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(Action::CommandOutputLine(l)); }
                    _ => break,
                }
            }
            line = err_lines.next_line() => {
                match line {
                    Ok(Some(l)) => { let _ = tx.send(Action::CommandOutputLine(l)); }
                    _ => break,
                }
            }
        }
    }
}

/// Build the actual argv, resolving dynamic parts (branch name for reset --hard).
fn build_argv(spec: &CommandSpec, current_branch: &str) -> Vec<String> {
    match spec {
        CommandSpec::GitResetHard => vec![
            "git".into(),
            "reset".into(),
            "--hard".into(),
            format!("origin/{current_branch}"),
        ],
        _ => spec.to_argv(),
    }
}
```

### Pattern 5: Worktree List Widget (ratatui List + ListState)

**What:** Replace the stub `render_worktree_list` with a real `StatefulWidget` render using `List` and `ListState`. The `ListState` lives in `AppState` so selection persists across redraws.

**When to use:** Main left-panel render. Each `ListItem` is a multi-span `Line` showing branch, metro badge, staleness hint, and label/JIRA title.

```rust
// src/ui/panels.rs — replacement for render_worktree_list stub
// Source: ratatui 0.30 widgets_list.rs tests + ratatui docs

use ratatui::widgets::{List, ListItem, ListState, HighlightSpacing};
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Modifier, Style};

pub fn render_worktree_list(f: &mut Frame, area: Rect, state: &AppState) {
    let border_style = if state.focused_panel == FocusedPanel::WorktreeList {
        theme::style_focused_border()
    } else {
        theme::style_inactive_border()
    };

    let items: Vec<ListItem> = state.worktrees.iter().map(|wt| {
        // Metro badge
        let metro_badge = match wt.metro_status {
            WorktreeMetroStatus::Running =>
                Span::styled(" [M] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            WorktreeMetroStatus::Stopped =>
                Span::styled(" [ ] ", Style::default().fg(Color::DarkGray)),
        };

        // Staleness hint
        let stale_hint = if wt.stale {
            Span::styled(" [stale] ", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        };

        // Display name: label > jira_title > branch
        let name = Span::raw(wt.display_name().to_string());

        let line = Line::from(vec![metro_badge, name, stale_hint]);
        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .title(" Worktrees ")
            .borders(Borders::ALL)
            .border_style(border_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    // CRITICAL: render_stateful_widget, not render_widget — needs mutable ListState
    f.render_stateful_widget(list, area, &mut state.worktree_list_state.clone());
}
```

**Note on mutable state:** `render_stateful_widget` requires `&mut ListState`. Since `view()` takes `&AppState` (immutable), use a `.clone()` of the ListState for rendering, or change `view()` to take `&mut AppState`. The cleaner TEA-pattern approach: clone the ListState for the render call — it's a small struct (usize + Option<usize>).

Alternatively, store `worktree_list_state: std::cell::Cell<ListState>` to enable interior mutability without changing `view()` signature. **Recommended:** pass `&mut AppState` to `view()` — the Phase 1 scaffold has `view(f, &state)` with `&AppState`, but ratatui's `render_stateful_widget` requires `&mut`. Change `view()` signature to accept `&mut AppState` in Phase 3.

### Pattern 6: Label Persistence

**What:** Store a `HashMap<String, String>` (branch_name → label) at `~/.config/ump-dash/labels.json`. Load on startup, write on each `SetLabel` action.

**When to use:** `AppState.labels` is the in-memory store. Every `Worktree.label` field is populated by looking up `branch` in `labels`.

```rust
// src/infra/labels.rs
// Source: serde_json docs, std::fs

use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("ump-dash")
}

pub fn labels_path() -> PathBuf {
    config_dir().join("labels.json")
}

/// Load labels from disk. Returns empty HashMap if file doesn't exist.
pub fn load_labels() -> Result<HashMap<String, String>> {
    let path = labels_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Persist labels to disk. Creates ~/.config/ump-dash/ if it doesn't exist.
pub fn save_labels(labels: &HashMap<String, String>) -> Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let content = serde_json::to_string_pretty(labels)?;
    std::fs::write(labels_path(), content)?;
    Ok(())
}
```

### Pattern 7: Confirmation Modal Overlay

**What:** Uses the established `Clear` + overlay pattern from Phase 1 error/help overlays. A destructive confirmation shows a centered popup with the command description and Y/N keybindings.

**When to use:** Before executing any `CommandSpec` where `is_destructive() == true`.

```rust
// src/ui/modals.rs
// Source: existing error_overlay.rs and help_overlay.rs pattern (Phase 1)

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_confirm_modal(f: &mut Frame, prompt: &str) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);  // Must be first — erases background
    let text = vec![
        Line::from(format!("  {prompt}")),
        Line::from(""),
        Line::from("  [Y] confirm   [N / Esc] cancel"),
    ];
    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

/// Returns a centered Rect that is `percent_x` wide and `percent_y` tall.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

### Pattern 8: Text Input Modal (branch name, jest filter)

**What:** A simple single-line text input using a `Paragraph` rendering the current input buffer. No tui-textarea dependency (confirmed incompatible with ratatui 0.30 from Phase 1 research).

**When to use:** `CommandSpec::GitCheckout/New`, `GitRebase`, `YarnJest` — commands requiring string input.

**Implementation:** Store `input_buffer: String` in `ModalState::TextInput { prompt, buffer }`. Handle `KeyCode::Char(c)` to push chars, `KeyCode::Backspace` to pop, `KeyCode::Enter` to submit, `KeyCode::Esc` to cancel.

```rust
// In domain/command.rs — ModalState

#[derive(Debug, Clone)]
pub enum ModalState {
    /// Confirmation dialog for destructive operations.
    Confirm { prompt: String, pending_command: CommandSpec },
    /// Single-line text input for branch name or jest filter.
    TextInput { prompt: String, buffer: String, pending_template: Box<CommandSpec> },
    /// Device picker for run-android or run-ios.
    DevicePicker { devices: Vec<DeviceInfo>, pending_template: Box<CommandSpec> },
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,    // adb serial or xcrun udid
    pub name: String,  // human-readable label
}
```

### Pattern 9: Device Enumeration

**What:** Parse `adb devices` and `xcrun simctl list devices available --json` to build device lists for run-android/run-ios selection.

**When to use:** When user triggers `RnRunAndroid` or `RnRunIos` — before showing the device picker.

```rust
// src/infra/devices.rs
// Source: verified against real adb and xcrun output on this machine

/// Parse adb devices output.
/// Format: "List of devices attached\n<serial>\t<state>\n..."
pub fn parse_adb_devices(output: &str) -> Vec<DeviceInfo> {
    output.lines()
        .skip(1) // skip "List of devices attached" header
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let serial = parts.next()?.trim().to_string();
            let state = parts.next()?.trim();
            if state == "device" {  // "offline" means not ready
                Some(DeviceInfo { id: serial.clone(), name: serial })
            } else {
                None
            }
        })
        .collect()
}

/// Parse xcrun simctl list devices available --json output.
/// Returns available simulators with name + udid.
pub fn parse_xcrun_simctl(json_output: &str) -> Vec<DeviceInfo> {
    // serde_json::from_str to get the "devices" map
    // key = "com.apple.CoreSimulator.SimRuntime.iOS-18-3"
    // value = array of { udid, name, state, isAvailable }
    // Filter: isAvailable == true
    // Optional: filter by state == "Booted" to show only running simulators first
    //
    // Actual format verified on this machine (iOS-18-2/18-3 runtimes present)
    let parsed: serde_json::Value = serde_json::from_str(json_output)
        .unwrap_or(serde_json::Value::Null);

    let devices_map = parsed.get("devices")
        .and_then(|d| d.as_object())
        .cloned()
        .unwrap_or_default();

    let mut result = Vec::new();
    for (_runtime, sims) in &devices_map {
        if let Some(arr) = sims.as_array() {
            for sim in arr {
                let available = sim.get("isAvailable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !available { continue; }

                let udid = sim.get("udid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = sim.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let state = sim.get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Shutdown")
                    .to_string();

                result.push(DeviceInfo {
                    id: udid,
                    name: format!("{name} ({state})"),
                });
            }
        }
    }
    result
}
```

### Pattern 10: Lazy yarn install Before RN Run (WORK-06)

**What:** Before dispatching `RnRunAndroid` or `RnRunIos`, check `worktree.stale`. If stale, first spawn `YarnInstall` as a blocking task. When `CommandExited` arrives and `pending_command` is `Some(RnRunAndroid/Ios)`, dispatch the deferred run command.

**When to use:** After device selection resolves the full `CommandSpec`, check stale before spawning.

```rust
// In app.rs update() — WORK-06 lazy install pattern

Action::CommandRun(spec) => {
    let selected_wt = &state.worktrees[state.selected_worktree_idx];

    // Lazy install: if stale and about to run the app, install first
    if selected_wt.stale && matches!(spec, CommandSpec::RnRunAndroid { .. } | CommandSpec::RnRunIos { .. }) {
        state.pending_command_after_install = Some(spec);
        // Dispatch YarnInstall first
        // When CommandExited fires, check pending_command_after_install
        let install_spec = CommandSpec::YarnInstall;
        // ... spawn install_spec
        return;
    }
    // Normal dispatch
    // ... spawn spec
}

Action::CommandExited => {
    state.running_command = None;
    // Check if there's a deferred command waiting for install to complete
    if let Some(deferred) = state.pending_command_after_install.take() {
        update(state, Action::CommandRun(deferred), ...);
    }
}
```

### AppState Extensions for Phase 3

```rust
// Additional fields in AppState (app.rs)
pub struct AppState {
    // ... existing Phase 1/2 fields ...

    // Worktree browser
    pub worktrees: Vec<crate::domain::worktree::Worktree>,
    pub worktree_list_state: ratatui::widgets::ListState,  // tracks selected index
    pub selected_worktree_idx: usize,

    // Command output panel
    pub command_output: std::collections::VecDeque<String>,
    pub command_output_scroll: usize,
    pub running_command: Option<crate::domain::command::CommandSpec>,
    pub command_task: Option<tokio::task::JoinHandle<()>>,  // abort on Cancel

    // Lazy install
    pub pending_command_after_install: Option<crate::domain::command::CommandSpec>,

    // Modal state
    pub modal: Option<crate::domain::command::ModalState>,

    // Label store (loaded from disk on startup)
    pub labels: std::collections::HashMap<String, String>,

    // Repo root — where to run git worktree list from
    pub repo_root: std::path::PathBuf,
}
```

### Action Enum Additions for Phase 3

```rust
// Additional variants in action.rs
pub enum Action {
    // ... existing Phase 1/2 variants ...

    // Worktree navigation
    WorktreeSelectNext,          // j/Down in WorktreeList panel
    WorktreeSelectPrev,          // k/Up in WorktreeList panel
    WorktreesLoaded(Vec<crate::domain::worktree::Worktree>), // background refresh done

    // Command lifecycle
    CommandRun(crate::domain::command::CommandSpec),  // dispatched when command is confirmed/ready
    CommandOutputLine(String),    // line from command stdout/stderr
    CommandExited,                // command process has finished
    CommandOutputClear,           // clear the output panel
    CommandCancel,                // abort running command (Ctrl-C in output panel)

    // Modal flow
    ModalConfirm,                 // user pressed Y in confirm dialog
    ModalCancel,                  // user pressed N or Esc in any modal
    ModalInputChar(char),         // character typed in text input modal
    ModalInputBackspace,          // backspace in text input modal
    ModalInputSubmit,             // Enter in text input modal
    ModalDeviceSelect(usize),     // index selected in device picker
    ModalDeviceConfirm,           // Enter on selected device

    // Label management
    SetLabel { branch: String, label: String },

    // Worktree refresh
    RefreshWorktrees,             // manual refresh (keybinding)
}
```

### Anti-Patterns to Avoid

- **One action variant per command (18 variants):** Creates a massive match arm explosion in `update()`. Use `CommandRun(CommandSpec)` as the single dispatch point.
- **Spawning commands in `handle_key()` directly:** `handle_key()` is pure (`Option<Action>`) — it returns an Action, never spawns. Spawning always happens in `update()` via `tokio::spawn`.
- **Storing `Child` in AppState:** `tokio::process::Child` is not `Clone`. Store the `JoinHandle<()>` returned by `spawn_command_task` for abort support.
- **Using `Command::current_dir()` relative path:** Always use the worktree's absolute `path: PathBuf` field. Relative paths break when running inside tmux.
- **Blocking `view()` with list mutation:** `render_stateful_widget` requires `&mut ListState`. Don't create a fresh `ListState` on every render (drops selection). Store it in `AppState` and pass a `&mut` clone or change `view()` to take `&mut AppState`.
- **Running `git worktree list` synchronously:** It's a subprocess — must be `tokio::spawn` to keep the event loop responsive.
- **Not aborting the command task on `CommandCancel`:** The `JoinHandle` must be `.abort()`ed when the user cancels; otherwise the process continues writing to a dead channel.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Worktree enumeration | Scan filesystem for .git files | `git worktree list --porcelain` | Git guarantees correctness; filesystem scan misses detached worktrees |
| Command streaming | Custom pipe reader | Same BufReader+mpsc pattern from Phase 2 | Already verified, battle-tested in the codebase |
| Overlay/modal rendering | Custom frame buffer manipulation | `Clear` widget + centered_rect | Already used by error_overlay and help_overlay in Phase 1 |
| Text input | tui-textarea crate | `String` buffer in `ModalState` + char-by-char handling | tui-textarea 0.7 incompatible with ratatui 0.30 (Phase 1 research decision) |
| iOS simulator list | Parse `xcrun simctl list devices` text | `xcrun simctl list devices available --json` | JSON output is stable; text format is locale-dependent |
| Android device list | Parse `adb devices -l` long format | `adb devices` (tab-separated short format) | Simple tab-split; serial + state is all that's needed for `--deviceId` |
| Config directory | Hardcode absolute path | `std::env::var("HOME")` + `/.config/ump-dash/` | Follows PROJECT.md convention; `dirs` crate unnecessary |
| Staleness check | Hash package.json and yarn.lock | `metadata().modified()` mtime comparison | No file reads needed; mtime reflects any write, including package manager updates |

**Key insight:** Phase 2 established the canonical streaming pattern (BufReader + mpsc + Action enum). Phase 3 simply reuses it for all commands — no new primitives needed.

---

## Common Pitfalls

### Pitfall 1: `render_stateful_widget` Requires `&mut State`

**What goes wrong:** `view(f, &state)` is called with `&AppState` (immutable), but `render_stateful_widget(list, area, &mut list_state)` needs `&mut ListState`. The compiler rejects this.

**Why it happens:** ratatui's `StatefulWidget` trait mutates the state during render (to update the internal scroll offset). This is a design choice in ratatui — state is mutable during rendering.

**How to avoid:** Change `pub fn view(f: &mut Frame, state: &AppState)` to `pub fn view(f: &mut Frame, state: &mut AppState)`. This propagates to the `app::run()` draw call: `terminal.draw(|f| crate::ui::view(f, &mut state))`. The borrow checker is satisfied because `state` and the terminal closure are distinct borrows.

**Warning signs:** Compiler error "cannot borrow `state` as mutable because it is also borrowed as immutable" at the draw call.

### Pitfall 2: git worktree list Shows the Dashboard Repo, Not the UMP Repo

**What goes wrong:** `git worktree list` run from the dashboard's working directory lists the dashboard's own worktrees, not UMP's.

**Why it happens:** The dashboard is in `/Users/cubicme/aljazeera/dashboard` — a different repo. UMP lives at `/Users/cubicme/aljazeera/ump`.

**How to avoid:** The `repo_root` field in `AppState` must be discovered correctly. Options:
1. Read from config file `~/.config/ump-dash/config.json` (cleanest — user sets it once)
2. Default to `$HOME/aljazeera/ump` (project-specific, hardcoded default)
3. Discover by scanning for repos with worktrees (too complex for Phase 3)

**Recommended for Phase 3:** Use a hardcoded default of `~/aljazeera/ump` with a config file override. Phase 4 config work will formalize this.

**Warning signs:** Worktree list shows only "main" (the dashboard itself) instead of 3-4 UMP worktrees.

### Pitfall 3: `git reset --hard` Without Target Remote Branch

**What goes wrong:** `git reset --hard origin/main` when branch is `UMP-7424` resets to the wrong target. The command must use `origin/<current-branch>`.

**Why it happens:** The current branch name must be dynamically injected into the argv. If `CommandSpec::GitResetHard` doesn't carry the branch name, the infra layer can't build the correct command.

**How to avoid:** `spawn_command_task` receives `current_branch: String` and constructs `origin/<current_branch>` at spawn time. The `Worktree.branch` field provides this. The `build_argv()` function in the infra layer handles this injection.

**Warning signs:** `git reset --hard` fails with "fatal: ambiguous argument 'origin/': unknown revision".

### Pitfall 4: Device Selection Popup for run-ios With No Booted Simulator

**What goes wrong:** `xcrun simctl list devices available` returns many simulators (all Shutdown). Showing a picker with 15+ entries is overwhelming. `run-ios --udid <shutdown-sim>` will try to boot it — this works but takes 30+ seconds.

**Why it happens:** No simulator is currently booted on this machine (verified: all state "Shutdown").

**How to avoid:**
1. Show booted simulators at the top of the picker, distinguished by state label "Booted"
2. If exactly one simulator is booted, skip the picker and use it automatically
3. If none are booted, show the picker but mark all as "Shutdown" — the user knowingly picks one

**Warning signs:** User picks a simulator that takes forever to boot with no feedback.

### Pitfall 5: Command Output Panel Not Cleared Between Commands

**What goes wrong:** Running `git pull` shows old output from the previous `yarn install`. User sees stale/confusing output mix.

**Why it happens:** `command_output: VecDeque<String>` is not cleared when a new command starts.

**How to avoid:** In `update()` when `CommandRun(spec)` is dispatched, always clear `state.command_output` and reset `state.command_output_scroll = 0` before spawning. Add a header line: `format!("--- {} ---", spec.label())`.

**Warning signs:** User sees output from two different commands concatenated.

### Pitfall 6: Text Input Modal — Char Events While Running

**What goes wrong:** When the modal is open, `j`/`k` keypresses go to the modal's text input AND trigger `FocusDown`/`FocusUp`. The user types "j" in a branch name and the worktree selection also moves.

**Why it happens:** `handle_key()` must check `state.modal.is_some()` before any other key routing — same priority pattern as `state.show_help` and `state.error_state.is_some()`.

**How to avoid:** Add `if state.modal.is_some()` at the top of `handle_key()`, intercept ALL keys for modal handling, return early. Only `Esc` cancels, `Enter` submits, printable chars go to the buffer, `Backspace` removes.

**Warning signs:** Branch name typed in modal accidentally scrolls the worktree list.

### Pitfall 7: Worktrees list_state.selected() Out of Sync After Refresh

**What goes wrong:** Worktrees are refreshed in the background. The list shrinks (a worktree was deleted). `worktree_list_state.selected()` still points to index 5 but the new list has only 3 items. Ratatui renders no highlight (index out of bounds is silently ignored), and user's selection is lost.

**Why it happens:** `WorktreesLoaded` action replaces `state.worktrees` but doesn't update `worktree_list_state`.

**How to avoid:** In `update()` for `WorktreesLoaded(worktrees)`:
1. Clamp selected index: `new_idx = old_idx.min(worktrees.len().saturating_sub(1))`
2. Call `state.worktree_list_state.select(Some(new_idx))`
3. Set `state.worktrees = worktrees`

### Pitfall 8: Labels HashMap Written to Disk on Every Keystroke

**What goes wrong:** If `save_labels()` is called on every `SetLabel` action AND the action fires on each character typed (in a label-editing modal), it writes to disk on every keystroke. On a slow disk this causes visible lag.

**Why it happens:** Eager persistence without debounce.

**How to avoid:** Only call `save_labels()` on `ModalInputSubmit` — when the user presses Enter to confirm the label, not on every character. The `labels` HashMap is updated in memory on every char (for live preview in the modal), but disk write happens only at submission.

---

## Code Examples

### Reading git worktree list --porcelain

```rust
// Source: verified against git 2.51.0 output on the UMP repository
// Actual output format confirmed:
//   "worktree /path\nHEAD <sha>\nbranch refs/heads/<name>\n\n"
//   (blank line between entries)

let output = tokio::process::Command::new("git")
    .args(["worktree", "list", "--porcelain"])
    .current_dir(&repo_root)
    .output()
    .await?;

let text = String::from_utf8_lossy(&output.stdout);
for entry in text.split("\n\n").filter(|s| !s.trim().is_empty()) {
    for line in entry.lines() {
        if let Some(path) = line.strip_prefix("worktree ") { /* ... */ }
        if let Some(sha) = line.strip_prefix("HEAD ") { /* ... */ }
        if let Some(branch) = line.strip_prefix("branch refs/heads/") { /* ... */ }
    }
}
```

### Node_modules Staleness Check

```rust
// Source: std::fs::Metadata — no external crates needed
// Verified: package.json mtime 1772429367 > node_modules mtime 1772391359 on real UMP worktree

fn check_stale(path: &std::path::Path) -> bool {
    use std::fs;
    use std::time::SystemTime;

    let nm = fs::metadata(path.join("node_modules"))
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let pkg = fs::metadata(path.join("package.json"))
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let lock = fs::metadata(path.join("yarn.lock"))
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    pkg.max(lock) > nm
}
```

### ratatui List with StatefulWidget

```rust
// Source: ratatui 0.30 widgets_list.rs tests (cargo registry)
// widgets: List, ListItem, ListState, HighlightSpacing

use ratatui::widgets::{List, ListItem, ListState, HighlightSpacing};

let items = vec![
    ListItem::new("rc-thunk"),
    ListItem::new("UMP-6441-e2e-fix"),
    ListItem::new("UMP-7424"),
];
let list = List::new(items)
    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
    .highlight_symbol("> ")
    .highlight_spacing(HighlightSpacing::Always);

// state is &mut ListState stored in AppState
f.render_stateful_widget(list, area, &mut state.worktree_list_state);
```

### xcrun simctl JSON Parsing

```rust
// Source: verified against actual xcrun simctl list devices available --json output
// Runtime keys: "com.apple.CoreSimulator.SimRuntime.iOS-18-2", etc.
// Device fields: udid, name, state, isAvailable

let output = tokio::process::Command::new("xcrun")
    .args(["simctl", "list", "devices", "available", "--json"])
    .output()
    .await?;

let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
if let Some(devices) = json["devices"].as_object() {
    for (_, sims) in devices {
        if let Some(arr) = sims.as_array() {
            for sim in arr {
                let udid = sim["udid"].as_str().unwrap_or("");
                let name = sim["name"].as_str().unwrap_or("?");
                let state = sim["state"].as_str().unwrap_or("Shutdown");
                // ... build DeviceInfo
            }
        }
    }
}
```

### adb devices Parsing

```rust
// Source: adb CLI output format — verified on this machine
// Format: "List of devices attached\n<serial>\t<state>\n"
// state == "device" means online; "offline" means not ready

let output = tokio::process::Command::new("adb")
    .arg("devices")
    .output()
    .await?;

let text = String::from_utf8_lossy(&output.stdout);
let devices: Vec<DeviceInfo> = text.lines()
    .skip(1)
    .filter(|l| !l.trim().is_empty())
    .filter_map(|line| {
        let (serial, state) = line.split_once('\t')?;
        if state.trim() == "device" {
            Some(DeviceInfo { id: serial.trim().to_string(), name: serial.trim().to_string() })
        } else {
            None
        }
    })
    .collect();
```

### Confirmation Overlay (reusing Clear pattern)

```rust
// Source: existing src/ui/error_overlay.rs pattern (Phase 1)
// Clear MUST be rendered first — erases background panels

f.render_widget(Clear, modal_area);
f.render_widget(
    Paragraph::new("  Are you sure? [Y]es / [N]o")
        .block(Block::default().title(" Confirm ").borders(Borders::ALL)
               .border_style(Style::default().fg(Color::Red))),
    modal_area,
);
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tui-textarea for text input | Paragraph + char buffer in ModalState | Phase 1 research (tui-textarea 0.7 incompatible with ratatui 0.30) | Must hand-roll minimal text input — but it's only 20 lines |
| Scan .git directories for worktrees | `git worktree list --porcelain` | git 2.5+ | Machine-readable, handles detached HEAD, linked worktrees, etc. |
| xcrun simctl text output parsing | `xcrun simctl list --json` | Xcode CLI ~9+ | JSON is stable; text format is fragile |
| `render_widget` for lists | `render_stateful_widget` + `ListState` | ratatui 0.x → stable | StatefulWidget is the canonical pattern for selection |
| VecDeque in AppState for logs only | VecDeque for both metro logs AND command output | Phase 3 | Same scrollable pattern reused — two separate VecDeques |

**Deprecated/outdated:**
- `tui-textarea 0.7`: Incompatible with ratatui 0.30. Will be resolved in tui-textarea 0.8 — but don't wait.
- Direct `render_widget` for `List`: Cannot show selection highlighting without `ListState`.
- `xcrun simctl list devices` (text output): Hard to parse reliably; always use `--json`.

---

## Open Questions

1. **Repo root discovery — hardcode or config?**
   - What we know: UMP lives at `~/aljazeera/ump`. Dashboard at `~/aljazeera/dashboard`. These are different repos.
   - What's unclear: Whether Phase 4 will introduce a config file that Phase 3 should also read, or whether Phase 3 should use a hardcoded default.
   - Recommendation: Hardcode default to `~/aljazeera/ump` using `std::env::var("HOME")` expansion. Store in `AppState.repo_root`. Phase 4 config will override this via `~/.config/ump-dash/config.json`.

2. **Worktree list refresh cadence**
   - What we know: Worktrees are added/removed infrequently (via `git worktree add`). The 250ms tick already drives redraws.
   - What's unclear: Should refresh be: (a) once on startup, (b) on explicit `r` keybinding, or (c) periodic (every 5s)?
   - Recommendation: Once on startup + explicit `RefreshWorktrees` action on a keybinding (e.g., `Shift-R` when WorktreeList panel focused). Avoid periodic refresh to keep the event loop simple in Phase 3.

3. **`run-ios` with `yarn` vs `npx`**
   - What we know: Requirements say "yarn react-native run-ios" (RN-07 spec). Other run commands use `npx react-native`.
   - What's unclear: Whether the project's package.json defines a `react-native` script, making `yarn react-native run-ios` work.
   - Recommendation: Use `yarn react-native run-ios` as specified in RN-07. If it fails at runtime, the streaming output panel will show the error and the user can adjust.

4. **CommandSpec::GitResetHard — target branch injection**
   - What we know: The command is `git reset --hard origin/<current-branch>`. The current branch is in `Worktree.branch`.
   - What's unclear: Should the branch name live in the `CommandSpec` variant or be injected at spawn time?
   - Recommendation: Keep `CommandSpec::GitResetHard` parameter-free (simpler domain model). Inject the branch from `state.worktrees[state.selected_worktree_idx].branch` at the `tokio::spawn` call site in `update()`. This keeps `CommandSpec` as a pure description of intent, not execution detail.

---

## Sources

### Primary (HIGH confidence)
- `src/domain/metro.rs`, `src/infra/process.rs`, `src/app.rs` — Phase 2 patterns that Phase 3 reuses directly
- `src/ui/error_overlay.rs`, `src/ui/help_overlay.rs` — confirmed `Clear` overlay pattern
- `/Users/cubicme/.cargo/registry/src/.../ratatui-0.30.0/tests/widgets_list.rs` — List + ListState + HighlightSpacing API verified from source
- `git worktree list --porcelain` output — verified against actual UMP repo (`~/aljazeera/ump`) with 4 worktrees
- `xcrun simctl list devices available --json` — verified JSON format against this machine's iOS 18.2/18.3 simulators
- `adb devices` — verified tab-separated format (empty device list on this machine, format confirmed via docs)
- `std::fs::metadata().modified()` — staleness check verified: UMP node_modules is 10.6 hours older than package.json + yarn.lock
- Cargo.lock — confirmed `serde`, `serde_json` already transitive deps; `dirs` crate NOT present

### Secondary (MEDIUM confidence)
- ratatui 0.30 docs — `HighlightSpacing` API existence confirmed from test file; `StatefulWidget` pattern confirmed
- git 2.51.0 man page — `--porcelain` flag stable since git 2.7.0; detached HEAD shows "detached" line

### Tertiary (LOW confidence)
- `yarn react-native run-ios --udid <udid>` flag syntax — assumed from standard RN CLI documentation; verify at runtime
- `npx react-native run-android --deviceId <serial>` flag — assumed from RN CLI docs; verify at runtime

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates already in lockfile; ratatui List verified from source
- Architecture: HIGH — follows established Phase 1/2 patterns directly
- Pitfalls: HIGH — most verified against actual code and real system state
- Code examples: HIGH — git worktree and xcrun output verified against real data; List API from ratatui test source

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (stable APIs; git porcelain format is stable by design; xcrun JSON format stable across Xcode versions)
