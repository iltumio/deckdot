use crate::commands::Commands;
use crate::config::Settings;
use crate::windows_focus;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Serialize, Deserialize)]
struct ExecuteRequest {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct ExecuteResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

fn verify_basic_auth(headers: &HeaderMap, settings: &Settings) -> bool {
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Basic ") {
                // Decode base64
                let encoded = &auth_str[6..];
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                    if let Ok(credentials) = String::from_utf8(decoded) {
                        if let Some((username, password)) = credentials.split_once(':') {
                            return username == settings.username && password == settings.password;
                        }
                    }
                }
            }
        }
    }
    false
}

async fn execute_handler(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, StatusCode> {
    let settings = state.settings.lock().await;
    
    if !verify_basic_auth(&headers, &settings) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    drop(settings);

    let commands = state.commands.lock().await;
    let cmd_config = commands.get(&req.id)
        .ok_or_else(|| StatusCode::NOT_FOUND)?;

    let command_str = cmd_config.command.clone();
    let focus_app = cmd_config.focus_app.clone();
    drop(commands);

    // Execute the command
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command_str])
            .output()
    } else {
        Command::new("sh")
            .args(["-c", &command_str])
            .output()
    };

    match output {
        Ok(output) => {
            let success = output.status.success();
            let message = if success {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            };

            // Handle focus app if specified
            if let Some(ref app_title) = focus_app {
                if let Err(e) = windows_focus::focus_window_by_title(app_title) {
                    eprintln!("Failed to focus window '{}': {}", app_title, e);
                }
            }

            Ok(Json(ExecuteResponse {
                success,
                message,
            }))
        }
        Err(e) => {
            Ok(Json(ExecuteResponse {
                success: false,
                message: format!("Failed to execute command: {}", e),
            }))
        }
    }
}

pub struct ServerState {
    pub settings: Arc<tokio::sync::Mutex<Settings>>,
    pub commands: Arc<tokio::sync::Mutex<Commands>>,
}

pub async fn start_server(
    settings: Settings,
    commands: Commands,
    dist_path: Option<String>,
) -> Result<(), String> {
    let port = settings.port;
    let state = ServerState {
        settings: Arc::new(tokio::sync::Mutex::new(settings)),
        commands: Arc::new(tokio::sync::Mutex::new(commands)),
    };

    let mut router = Router::new()
        .route("/health", get(health_handler))
        .route("/execute", post(execute_handler))
        .with_state(Arc::new(state));

    // Serve static files if dist_path is provided
    if let Some(path) = dist_path {
        let index_path = std::path::Path::new(&path).join("index.html");
        let serve_dir = ServeDir::new(&path)
            .append_index_html_on_directories(true)
            .precompressed_gzip()
            .precompressed_br()
            .fallback(ServeFile::new(index_path));
        
        // Fallback to static file serving for all non-API routes (SPA routing)
        router = router.fallback_service(serve_dir);
    }

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    println!("Server listening on http://{}", addr);

    axum::serve(listener, router)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

