// src/infra/port.rs
//
// Port availability probe. Uses std::net::TcpListener::bind as the check — a successful
// bind means no process currently holds the port. This is the correct approach for
// verifying that metro's port 8081 is free after a kill (research pattern 3).

/// Returns true if no process is currently bound to `port` on 127.0.0.1.
///
/// Uses `TcpListener::bind` as the probe — bind succeeds only when the address is free.
/// Call this in a retry loop after killing metro; the port may briefly remain in
/// TIME_WAIT even after SIGKILL (research pitfall 3).
pub fn port_is_free(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Information about an external (non-dashboard) process occupying a port.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalMetroInfo {
    pub pid: u32,
    pub working_dir: String,
}

/// Detect if an external (non-dashboard) process is listening on the given port.
/// Uses `lsof -i :PORT -sTCP:LISTEN -t` to get PID, then `lsof -a -p PID -d cwd -Fn`
/// to get the working directory. Returns None if port is free or detection fails.
pub async fn detect_external_metro(port: u16) -> Option<ExternalMetroInfo> {
    let output = tokio::process::Command::new("lsof")
        .args(["-i", &format!(":{port}"), "-sTCP:LISTEN", "-t"])
        .output()
        .await
        .ok()?;

    let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let pid: u32 = pid_str.lines().next()?.parse().ok()?;

    let cwd_output = tokio::process::Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .await
        .ok()?;

    let cwd_text = String::from_utf8_lossy(&cwd_output.stdout);
    let working_dir = cwd_text
        .lines()
        .find(|l| l.starts_with('n'))
        .map(|l| l[1..].to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Some(ExternalMetroInfo { pid, working_dir })
}

/// Kill a process by PID using SIGKILL.
pub async fn kill_process(pid: u32) -> anyhow::Result<()> {
    tokio::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .await?;
    Ok(())
}
