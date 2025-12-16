use crate::commands;
use baras_core::app_state::AppState;
use baras_core::directory_watcher::{self as core_watcher, DirectoryEvent, DirectoryWatcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Initialize the file index and start the watcher
pub async fn init_watcher(state: Arc<RwLock<AppState>>) -> Option<JoinHandle<()>> {
    let dir = {
        let s = state.read().await;
        PathBuf::from(&s.config.log_directory)
    };

    if !dir.exists() {
        println!("Warning: Log directory {} does not exist", dir.display());
        return None;
    }

    // Build initial index using core
    match core_watcher::build_index(&dir) {
        Ok((index, newest)) => {
            let file_count = index.len();

            {
                let mut s = state.write().await;
                s.file_index = Some(index);
            }

            println!("Indexed {} log files", file_count);

            // Auto-load newest file if available
            if let Some(newest_path) = newest {
                let path_str = newest_path.to_string_lossy().to_string();
                commands::parse_file(&path_str, Arc::clone(&state)).await;
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }

    // Create watcher
    let mut watcher = match DirectoryWatcher::new(&dir) {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to start directory watcher: {}", e);
            return None;
        }
    };

    println!("Watching directory: {}", dir.display());

    // CLI spawns and handles events
    let watcher_state = Arc::clone(&state);
    let handle = tokio::spawn(async move {
        while let Some(event) = watcher.next_event().await {
            handle_watcher_event(event, Arc::clone(&watcher_state)).await;
        }
    });

    Some(handle)
}

async fn handle_watcher_event(event: DirectoryEvent, state: Arc<RwLock<AppState>>) {
    match event {
        DirectoryEvent::NewFile(path) => {
            println!("New log file detected: {}", path.display());

            // Add to index
            {
                let mut s = state.write().await;
                if let Some(index) = &mut s.file_index {
                    index.add_file(&path);
                }
            }

            // Parse and tail the new file
            let path_str = path.to_string_lossy().to_string();
            commands::parse_file(&path_str, state).await;
        }

        DirectoryEvent::FileRemoved(path) => {
            let mut s = state.write().await;
            if let Some(index) = &mut s.file_index {
                index.remove_file(&path);
            }
        }

        DirectoryEvent::Message(msg) => {
            println!("{}", msg);
        }

        DirectoryEvent::Error(err) => {
            println!("Error: {}", err);
        }

        DirectoryEvent::DirectoryIndexed { .. } => {
            // Handled during init, not expected here
        }
    }
}
