// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod database;
mod server;
mod system_commands;
mod windows_focus;

use commands::CommandConfig;
use config::Settings;
use database::{create_shared_database, SharedDatabase};
use server::ServerHandle;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

struct AppState {
    server_handle: Arc<Mutex<Option<ServerHandle>>>,
    database: SharedDatabase,
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
        // Save settings to database
        {
            let db = state.database.lock().await;
            db.save_settings(&settings).map_err(|e| e.to_string())?;
        }

        // Get mobile-dist path for static file serving
        let mobile_dist_path = std::env::current_dir()
            .ok()
            .map(|p: PathBuf| p.join("mobile-dist"))
            .filter(|p| p.exists())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .map(|p| p.join("..").join("mobile-dist"))
                    .filter(|p| p.exists())
                    .and_then(|p| p.canonicalize().ok())
            })
            .or_else(|| {
                dirs::data_dir().map(|p| p.join("deck").join("mobile-dist")).filter(|p| p.exists())
            })
            .map(|p| p.to_string_lossy().to_string());
        
        if let Some(ref path) = mobile_dist_path {
            println!("Mobile dist path: {}", path);
        } else {
            println!("Mobile dist not found - will use fallback HTML");
        }

        // Start the server with shared database reference
        let server_handle = server::start_server(
            settings.clone(),
            state.database.clone(),
            mobile_dist_path,
        ).await?;

        *handle_lock = Some(server_handle);
        println!("Server started on port {}", settings.port);
        Ok(true)
    }
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let db = state.database.lock().await;
    Ok(db.get_settings())
}

#[tauri::command]
async fn save_settings(
    settings: Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    settings.validate()?;
    
    let db = state.database.lock().await;
    db.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn regenerate_auth_code(state: State<'_, AppState>) -> Result<String, String> {
    let new_code = Settings::generate_random_code();
    
    let db = state.database.lock().await;
    let mut settings = db.get_settings();
    settings.auth_code = new_code.clone();
    db.save_settings(&settings).map_err(|e| e.to_string())?;
    
    Ok(new_code)
}

#[tauri::command]
async fn get_commands(state: State<'_, AppState>) -> Result<Vec<CommandConfig>, String> {
    let db = state.database.lock().await;
    db.get_all_commands().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_commands(
    commands_vec: Vec<CommandConfig>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.database.lock().await;
    db.save_all_commands(&commands_vec).map_err(|e| e.to_string())
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
        if interfaces.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = interfaces.local_addr() {
                ips.push(local_addr.ip().to_string());
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle();
            
            // Use app data directory for database
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|_| "Failed to get app data directory")?;
            
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Failed to create app data directory: {}", e))?;
            
            let db_path = app_data_dir.join("deck.db");
            let old_settings_path = app_data_dir.join("settings.json");
            let old_commands_path = app_data_dir.join("commands.yaml");
            
            // Create database
            let database = create_shared_database(&db_path)?;
            
            // Migrate from old files if they exist
            {
                let db = futures::executor::block_on(database.lock());
                if let Err(e) = db.migrate_from_files(&old_settings_path, &old_commands_path) {
                    eprintln!("Migration warning: {}", e);
                }
                
                // Initialize default settings if database is empty
                let settings = db.get_settings();
                if settings.auth_code.is_empty() {
                    let default_settings = Settings::default();
                    if let Err(e) = db.save_settings(&default_settings) {
                        eprintln!("Failed to save default settings: {}", e);
                    }
                }
            }
            
            // Also try to migrate from resource directory (for bundled commands.yaml)
            if let Ok(resource_dir) = app_handle.path().resource_dir() {
                let bundled_commands = resource_dir.join("commands.yaml");
                if bundled_commands.exists() {
                    let db = futures::executor::block_on(database.lock());
                    let existing_commands = db.get_all_commands().unwrap_or_default();
                    if existing_commands.is_empty() {
                        if let Err(e) = db.migrate_from_files(&PathBuf::new(), &bundled_commands) {
                            eprintln!("Failed to migrate bundled commands: {}", e);
                        }
                    }
                }
            }

            let app_state = AppState {
                server_handle: Arc::new(Mutex::new(None)),
                database,
            };

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_server,
            get_settings,
            save_settings,
            get_commands,
            save_commands,
            get_server_status,
            get_local_ips,
            get_running_applications,
            regenerate_auth_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
