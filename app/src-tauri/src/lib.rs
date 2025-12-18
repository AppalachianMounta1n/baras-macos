use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use tauri::State;

use baras_overlay::{MeterEntry, MeterOverlay, OverlayConfig};
use baras_overlay::renderer::colors;

/// Commands sent to the overlay thread
enum OverlayCommand {
    SetMoveMode(bool),
    UpdateEntries(Vec<MeterEntry>),
    Shutdown,
}

/// State managing the overlay thread
#[derive(Debug, Default)]
struct OverlayState {
    tx: Option<Sender<OverlayCommand>>,
    handle: Option<JoinHandle<()>>,
    is_running: bool,
    move_mode: bool,
}

/// Spawn the overlay on a separate thread
fn spawn_overlay() -> (Sender<OverlayCommand>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<OverlayCommand>();

    let handle = thread::spawn(move || {
        let config = OverlayConfig {
            x: 50,
            y: 50,
            width: 280,
            height: 200,
            namespace: "baras-dps".to_string(),
            click_through: true,
        };

        let mut overlay = match MeterOverlay::new(config, "DPS Meter") {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Failed to create overlay: {}", e);
                return;
            }
        };

        // Set up dummy data
        let dummy_entries = vec![
            MeterEntry {
                name: "Player One".to_string(),
                value: 15234.0,
                max_value: 15234.0,
                color: colors::dps_bar_fill(),
            },
            MeterEntry {
                name: "Player Two".to_string(),
                value: 12100.0,
                max_value: 15234.0,
                color: colors::dps_bar_fill(),
            },
            MeterEntry {
                name: "Player Three".to_string(),
                value: 9800.0,
                max_value: 15234.0,
                color: colors::dps_bar_fill(),
            },
            MeterEntry {
                name: "Player Four".to_string(),
                value: 7500.0,
                max_value: 15234.0,
                color: colors::dps_bar_fill(),
            },
        ];
        overlay.set_entries(dummy_entries);

        loop {
            // Check for commands (non-blocking)
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    OverlayCommand::SetMoveMode(enabled) => {
                        overlay.window_mut().set_click_through(!enabled);
                    }
                    OverlayCommand::UpdateEntries(entries) => {
                        overlay.set_entries(entries);
                    }
                    OverlayCommand::Shutdown => {
                        return;
                    }
                }
            }

            // Poll events and render
            if !overlay.poll_events() {
                break;
            }
            overlay.render();

            // Minimal sleep - just yield to OS scheduler
            // The event loop naturally blocks when waiting for events
            thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    (tx, handle)
}

#[tauri::command]
fn show_overlay(state: State<'_, Mutex<OverlayState>>) -> Result<bool, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    if state.is_running {
        return Ok(true); // Already running
    }

    let (tx, handle) = spawn_overlay();
    state.tx = Some(tx);
    state.handle = Some(handle);
    state.is_running = true;

    Ok(true)
}

#[tauri::command]
fn hide_overlay(state: State<'_, Mutex<OverlayState>>) -> Result<bool, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    if !state.is_running {
        return Ok(true); // Already stopped
    }

    if let Some(tx) = state.tx.take() {
        let _ = tx.send(OverlayCommand::Shutdown);
    }

    if let Some(handle) = state.handle.take() {
        let _ = handle.join();
    }

    state.is_running = false;
    state.move_mode = false;

    Ok(true)
}

#[tauri::command]
fn toggle_move_mode(state: State<'_, Mutex<OverlayState>>) -> Result<bool, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    if !state.is_running {
        return Err("Overlay not running".to_string());
    }

    state.move_mode = !state.move_mode;

    if let Some(tx) = &state.tx {
        tx.send(OverlayCommand::SetMoveMode(state.move_mode))
            .map_err(|e| e.to_string())?;
    }

    Ok(state.move_mode)
}

#[tauri::command]
fn get_overlay_status(state: State<'_, Mutex<OverlayState>>) -> Result<(bool, bool), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok((state.is_running, state.move_mode))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(OverlayState::default()))
        .invoke_handler(tauri::generate_handler![
            show_overlay,
            hide_overlay,
            toggle_move_mode,
            get_overlay_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
