//! Vibe Kanban CLI - Interactive terminal interface for Vibe Kanban.
//!
//! This crate provides a terminal-based user interface for interacting with
//! a Vibe Kanban server, allowing users to manage projects, tasks, workspaces,
//! and git operations without needing the web UI.

#![allow(clippy::module_inception)]

pub mod api;
pub mod app;
pub mod types;
pub mod ui;

pub use api::VibeKanbanClient;
pub use app::App;
