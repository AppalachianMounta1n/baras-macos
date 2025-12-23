//! Tauri commands module
//!
//! All Tauri-invokable commands are centralized here for easy discovery.
//!
//! # Command Categories
//!
//! - `overlay` - Overlay show/hide, move mode, status, refresh
//! - `service` - Log files, tailing, config, session info, profiles

mod overlay;
mod service;

// Re-export all commands for the invoke_handler
pub use overlay::*;
pub use service::*;
