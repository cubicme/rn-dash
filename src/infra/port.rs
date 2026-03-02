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
