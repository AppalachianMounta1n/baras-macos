use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{RwLock};

use baras_core::context::{AppConfig, DirectoryIndex, ParsingSession};



// ─────────────────────────────────────────────────────────────────────────────
// Shared State
// ─────────────────────────────────────────────────────────────────────────────

/// State shared between the service and Tauri commands
pub struct SharedState {
    pub config: RwLock<AppConfig>,
    pub directory_index: RwLock<DirectoryIndex>,
    pub session: RwLock<Option<Arc<RwLock<ParsingSession>>>>,
    /// Whether we're currently in active combat (for metrics updates)
    pub in_combat: AtomicBool,
}

impl SharedState {
    pub fn new(config: AppConfig, directory_index: DirectoryIndex) -> Self {
        Self {
            config: RwLock::new(config),
            directory_index: RwLock::new(directory_index),
            session: RwLock::new(None),
            in_combat: AtomicBool::new(false),
        }
    }
}
