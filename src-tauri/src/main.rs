// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod server;
mod windows_focus;

use commands::{CommandConfig, Commands};
use config::Settings;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

struct AppState {
    server_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    settings: Arc<Mutex<Settings>>,
    commands: Arc<Mutex<Commands>>,
    commands_path: PathBuf,
}

#[tauri::command]
async fn toggle_server(
    settings: Settings,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    settings.validate()?;

    let mut handle_lock = state.server_handle.lock().await;

    if let Some(handle) = handle_lock.take() {
        handle.abort();
        Ok(false)
    } else {
        let settings_clone = settings.clone();
        let commands_clone = {
            let commands = state.commands.lock().await;
            commands.clone()
        };

        // Update stored settings
        *state.settings.lock().await = settings_clone.clone();

        // Get dist path for static file serving
        // For development, try the project root dist
        let dist_path = std::env::current_dir()
            .ok()
            .map(|p: PathBuf| p.join("dist").to_string_lossy().to_string());

        let handle = tokio::spawn(async move {
            if let Err(e) = server::start_server(settings_clone, commands_clone, dist_path).await {
                eprintln!("Server error: {}", e);
            }
        });

        *handle_lock = Some(handle);
        Ok(true)
    }
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
async fn save_settings(
    settings: Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    settings.validate()?;
    *state.settings.lock().await = settings;
    Ok(())
}

#[tauri::command]
async fn get_commands(state: State<'_, AppState>) -> Result<Vec<CommandConfig>, String> {
    let commands = state.commands.lock().await;
    Ok(commands.all())
}

#[tauri::command]
async fn reload_commands(state: State<'_, AppState>) -> Result<(), String> {
    let mut commands = state.commands.lock().await;
    *commands = Commands::load_from_file(&state.commands_path)
        .map_err(|e| format!("Failed to reload commands: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn save_commands(
    commands_vec: Vec<CommandConfig>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut commands = Commands::new();
    for cmd in commands_vec {
        commands.add(cmd);
    }
    commands.save_to_file(&state.commands_path)
        .map_err(|e| format!("Failed to save commands: {}", e))?;
    
    *state.commands.lock().await = commands;
    Ok(())
}

#[tauri::command]
async fn get_server_status(state: State<'_, AppState>) -> Result<bool, String> {
    let handle = state.server_handle.lock().await;
    Ok(handle.is_some())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize app state
            let app_handle = app.handle();
            
            // Use app data directory for commands.yaml (writable location)
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|_| "Failed to get app data directory")?;
            
            // Create app data directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Failed to create app data directory: {}", e))?;
            
            let commands_path = app_data_dir.join("commands.yaml");
            
            // If commands.yaml doesn't exist, try to copy from resource dir or create default
            if !commands_path.exists() {
                if let Ok(resource_dir) = app_handle.path().resource_dir() {
                    let default_commands = resource_dir.join("commands.yaml");
                    if default_commands.exists() {
                        if let Err(e) = std::fs::copy(&default_commands, &commands_path) {
                            eprintln!("Failed to copy default commands.yaml: {}", e);
                        }
                    }
                }
            }
            
            // Load or create default commands
            let commands = if commands_path.exists() {
                Commands::load_from_file(&commands_path)
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to load commands.yaml: {}, using empty", e);
                        Commands::new()
                    })
            } else {
                Commands::new()
            };

            let app_state = AppState {
                server_handle: Arc::new(Mutex::new(None)),
                settings: Arc::new(Mutex::new(Settings::default())),
                commands: Arc::new(Mutex::new(commands)),
                commands_path,
            };

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_server,
            get_settings,
            save_settings,
            get_commands,
            reload_commands,
            save_commands,
            get_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

