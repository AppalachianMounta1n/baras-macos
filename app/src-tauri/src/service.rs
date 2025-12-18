//! Combat service - coordinates parsing, state management, and overlay updates
//!
//! This is the main integration layer between core business logic and the Tauri app.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use baras_core::context::{resolve, AppConfig, DirectoryIndex, ParsingSession};
use baras_core::{GameSignal, Reader, SignalHandler};

/// Updates sent to the overlay system
#[derive(Debug, Clone)]
pub enum OverlayUpdate {
    /// Combat has started
    CombatStarted,
    /// Combat has ended
    CombatEnded,
    /// Metrics have been updated
    MetricsUpdated(Vec<PlayerMetrics>),
}

/// Messages sent to the service from Tauri commands
pub enum ServiceCommand {
    /// Start tailing a specific log file
    StartTailing(PathBuf),
    /// Stop the current tail operation
    StopTailing,
    /// Refresh the directory index
    RefreshIndex,
    /// Shutdown the service
    Shutdown,
}

/// Handle to communicate with the combat service
#[derive(Clone)]
pub struct ServiceHandle {
    cmd_tx: mpsc::Sender<ServiceCommand>,
}

impl ServiceHandle {
    pub async fn start_tailing(&self, path: PathBuf) -> Result<(), String> {
        self.cmd_tx
            .send(ServiceCommand::StartTailing(path))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn stop_tailing(&self) -> Result<(), String> {
        self.cmd_tx
            .send(ServiceCommand::StopTailing)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn refresh_index(&self) -> Result<(), String> {
        self.cmd_tx
            .send(ServiceCommand::RefreshIndex)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Signal handler that forwards game signals to the overlay system
struct OverlaySignalHandler {
    tx: mpsc::Sender<OverlayUpdate>,
}

impl OverlaySignalHandler {
    fn new(tx: mpsc::Sender<OverlayUpdate>) -> Self {
        Self { tx }
    }
}

impl SignalHandler for OverlaySignalHandler {
    fn handle_signal(&mut self, signal: &GameSignal) {
        // Convert game signals to overlay updates
        let update = match signal {
            GameSignal::CombatStarted { .. } => Some(OverlayUpdate::CombatStarted),
            GameSignal::CombatEnded { .. } => Some(OverlayUpdate::CombatEnded),
            // TODO: Add more signal -> update mappings as needed
            _ => None,
        };

        if let Some(update) = update {
            // Non-blocking send - drop if channel is full
            let _ = self.tx.try_send(update);
        }
    }
}

/// Main combat service that owns all core state
pub struct CombatService {
    // Configuration
    config: AppConfig,

    // File index for browsing logs
    directory_index: DirectoryIndex,

    // Active parsing session (if tailing)
    session: Option<Arc<RwLock<ParsingSession>>>,

    // Channel to send updates to overlays
    overlay_tx: mpsc::Sender<OverlayUpdate>,

    // Command receiver
    cmd_rx: mpsc::Receiver<ServiceCommand>,

    // Handle for cancelling the tail task
    tail_handle: Option<tokio::task::JoinHandle<()>>,
}

impl CombatService {
    /// Create a new combat service and return a handle to communicate with it
    pub fn new(overlay_tx: mpsc::Sender<OverlayUpdate>) -> (Self, ServiceHandle) {
        let (cmd_tx, cmd_rx) = mpsc::channel(32);

        let config = AppConfig::load();
        let directory_index = DirectoryIndex::build_index(&PathBuf::from(&config.log_directory))
            .unwrap_or_default();

        let service = Self {
            config,
            directory_index,
            session: None,
            overlay_tx,
            cmd_rx,
            tail_handle: None,
        };

        let handle = ServiceHandle { cmd_tx };

        (service, handle)
    }

    /// Run the service event loop
    pub async fn run(mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                ServiceCommand::StartTailing(path) => {
                    self.start_tailing(path).await;
                }
                ServiceCommand::StopTailing => {
                    self.stop_tailing().await;
                }
                ServiceCommand::RefreshIndex => {
                    self.refresh_index();
                }
                ServiceCommand::Shutdown => {
                    self.stop_tailing().await;
                    break;
                }
            }
        }
    }

    /// Start tailing a log file
    async fn start_tailing(&mut self, path: PathBuf) {
        // Stop any existing tail first
        self.stop_tailing().await;

        // Create new parsing session
        let mut session = ParsingSession::new(path.clone());

        // Add signal handler for overlay updates
        let handler = OverlaySignalHandler::new(self.overlay_tx.clone());
        session.add_signal_handler(Box::new(handler));

        let session = Arc::new(RwLock::new(session));
        self.session = Some(session.clone());

        // Spawn the tail task
        let reader = Reader::from(path, session);
        let handle = tokio::spawn(async move {
            if let Err(e) = reader.tail_log_file().await {
                eprintln!("Tail error: {}", e);
            }
        });

        self.tail_handle = Some(handle);
    }

    /// Stop the current tail operation
    async fn stop_tailing(&mut self) {
        if let Some(handle) = self.tail_handle.take() {
            handle.abort();
            let _ = handle.await;
        }
        self.session = None;
    }

    /// Refresh the directory index
    fn refresh_index(&mut self) {
        if let Ok(index) = DirectoryIndex::build_index(&PathBuf::from(&self.config.log_directory)) {
            self.directory_index = index;
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Query methods for Tauri commands
    // ─────────────────────────────────────────────────────────────────────────────

    /// Get the current configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: AppConfig) {
        self.config = config;
        // TODO: Persist to disk with confy
    }

    /// Get log file entries for the UI
    pub fn log_files(&self) -> Vec<LogFileInfo> {
        self.directory_index
            .entries()
            .into_iter()
            .map(|e| LogFileInfo {
                path: e.path.clone(),
                display_name: e.display_name(),
                character_name: e.character_name.clone(),
                date: e.date.to_string(),
                is_empty: e.is_empty,
            })
            .collect()
    }

    /// Get current encounter metrics (for overlay updates)
    pub async fn current_metrics(&self) -> Option<Vec<PlayerMetrics>> {
        let session = self.session.as_ref()?;
        let session = session.read().await;
        let cache = session.session_cache.as_ref()?;
        let encounter = cache.current_encounter()?;

        let entity_metrics = encounter.calculate_entity_metrics()?;

        Some(
            entity_metrics
                .into_iter()
                .map(|m| PlayerMetrics {
                    entity_id: m.entity_id,
                    name: resolve(m.name).to_string(),
                    dps: m.dps as f64,
                    total_damage: m.total_damage as u64,
                    hps: m.hps as f64,
                    total_healing: m.total_healing as u64,
                })
                .collect(),
        )
    }

    /// Check if currently tailing a file
    pub fn is_tailing(&self) -> bool {
        self.session.is_some()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DTOs for Tauri IPC
// ─────────────────────────────────────────────────────────────────────────────

/// Log file info for the UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogFileInfo {
    pub path: PathBuf,
    pub display_name: String,
    pub character_name: Option<String>,
    pub date: String,
    pub is_empty: bool,
}

/// Player metrics for overlay display
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayerMetrics {
    pub entity_id: i64,
    pub name: String,
    pub dps: f64,
    pub total_damage: u64,
    pub hps: f64,
    pub total_healing: u64,
}
