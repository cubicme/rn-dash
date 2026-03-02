//! Domain layer — pure Rust. Zero dependencies on ratatui, crossterm, or infra.
//! Note: metro.rs references tokio types for the MetroHandle bridge type — see that
//! file's architectural note for the rationale. mod.rs itself imports nothing from infra.
pub mod command;
pub mod metro;
pub mod worktree;
