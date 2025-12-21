// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod server;
mod system_commands;
mod windows_focus;

use commands::{CommandConfig, Commands};
use config::Settings;
use server::ServerHandle;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

struct AppState {
    server_handle: Arc<Mutex<Option<ServerHandle>>>,
    settings: Arc<Mutex<Settings>>,
    commands: Arc<Mutex<Commands>>,
    commands_path: PathBuf,
    settings_path: PathBuf,
}

#[tauri::command]
async fn toggle_server(
    settings: Settings,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    settings.validate()?;

    let mut handle_lock = state.server_handle.lock().await;

    if let Some(handle) = handle_lock.take() {
        // Gracefully shutdown the server
        handle.shutdown();
        println!("Server stopped");
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

        // Start the server and get the handle
        let server_handle = server::start_server(settings_clone, commands_clone, dist_path).await?;

        *handle_lock = Some(server_handle);
        println!("Server started on port {}", settings.port);
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
    
    // Save to disk
    settings.save_to_file(&state.settings_path)?;
    
    // Update in memory
    *state.settings.lock().await = settings;
    Ok(())
}

#[tauri::command]
async fn regenerate_auth_code(state: State<'_, AppState>) -> Result<String, String> {
    let new_code = Settings::generate_random_code();
    
    let mut settings = state.settings.lock().await;
    settings.auth_code = new_code.clone();
    
    // Save to disk
    settings.save_to_file(&state.settings_path)?;
    
    Ok(new_code)
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

#[tauri::command]
fn get_local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    
    if let Ok(interfaces) = std::net::UdpSocket::bind("0.0.0.0:0") {
        // Connect to a public IP to determine local IP
        if interfaces.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = interfaces.local_addr() {
                ips.push(local_addr.ip().to_string());
            }
        }
    }
    
    // Fallback: try to get all network interfaces
    #[cfg(target_os = "windows")]
    {
        // On Windows, use ipconfig output parsing or network interfaces
        if ips.is_empty() {
            if let Ok(output) = std::process::Command::new("ipconfig")
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("IPv4") && line.contains(":") {
                        if let Some(ip_part) = line.split(':').last() {
                            let ip = ip_part.trim();
                            if !ip.starts_with("127.") && !ip.is_empty() {
                                ips.push(ip.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, try ifconfig or ip command
        if ips.is_empty() {
            if let Ok(output) = std::process::Command::new("ifconfig")
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("inet ") && !line.contains("127.0.0.1") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for (i, part) in parts.iter().enumerate() {
                            if *part == "inet" && i + 1 < parts.len() {
                                let ip = parts[i + 1];
                                if !ip.starts_with("127.") {
                                    ips.push(ip.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Remove duplicates
    ips.sort();
    ips.dedup();
    
    ips
}

/// Get a list of running applications that can be focused
#[tauri::command]
fn get_running_applications() -> Vec<String> {
    system_commands::get_running_applications()
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
            let settings_path = app_data_dir.join("settings.json");
            
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

            // Load or create default settings
            let settings = if settings_path.exists() {
                Settings::load_from_file(&settings_path)
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to load settings.json: {}, using default", e);
                        let default_settings = Settings::default();
                        // Save the default settings
                        if let Err(e) = default_settings.save_to_file(&settings_path) {
                            eprintln!("Failed to save default settings: {}", e);
                        }
                        default_settings
                    })
            } else {
                // Create new settings with random auth code
                let default_settings = Settings::default();
                if let Err(e) = default_settings.save_to_file(&settings_path) {
                    eprintln!("Failed to save default settings: {}", e);
                }
                default_settings
            };

            let app_state = AppState {
                server_handle: Arc::new(Mutex::new(None)),
                settings: Arc::new(Mutex::new(settings)),
                commands: Arc::new(Mutex::new(commands)),
                commands_path,
                settings_path,
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
            get_local_ips,
            get_running_applications,
            regenerate_auth_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
