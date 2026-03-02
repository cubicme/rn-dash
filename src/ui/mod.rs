//! UI layer — ratatui widgets, rendering, layout.
//! Imports: domain types and ratatui ONLY. Never imports infra directly.
pub mod theme;

/// Placeholder view function — renders nothing. Plan 03 replaces this with real panel layout.
pub fn view(_f: &mut ratatui::Frame, _state: &crate::app::AppState) {}
