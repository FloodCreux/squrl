//! squrl -- a terminal-based HTTP and WebSocket client.
//!
//! This crate provides both an interactive TUI (via `ratatui`) and a full-featured CLI
//! for managing request collections, sending HTTP/WebSocket requests, and inspecting responses.

pub mod app;
pub(crate) mod cli;
pub(crate) mod errors;
pub mod models;
pub(crate) mod tui;
