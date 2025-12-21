use crate::commands::Commands;
use crate::config::Settings;
use crate::system_commands;
use crate::windows_focus;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;

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

#[derive(Deserialize, Default)]
struct AuthQuery {
    code: Option<String>,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[derive(Serialize)]
struct CommandInfo {
    id: String,
    name: String,
}

async fn commands_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<AuthQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<CommandInfo>>, StatusCode> {
    let settings = state.settings.lock().await;
    
    if !verify_auth(&query, &headers, &settings) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    drop(settings);

    let commands = state.commands.lock().await;
    let command_list: Vec<CommandInfo> = commands.all()
        .into_iter()
        .map(|c| CommandInfo { id: c.id, name: c.name })
        .collect();
    
    Ok(Json(command_list))
}

/// Fallback handler that serves index.html for SPA routing
async fn fallback_handler(
    State(state): State<Arc<ServerState>>,
    uri: Uri,
) -> Response {
    // If it's an API route, return 404
    if uri.path().starts_with("/api/") || uri.path().starts_with("/execute") || uri.path().starts_with("/health") {
        return (StatusCode::NOT_FOUND, "Not Found").into_response();
    }
    
    // Try to read index.html from the mobile dist path
    if let Some(ref dist_path) = state.mobile_dist_path {
        let index_path = PathBuf::from(dist_path).join("index.html");
        if let Ok(contents) = tokio::fs::read_to_string(&index_path).await {
            return Html(contents).into_response();
        }
    }
    
    // Fallback to inline HTML if dist not available
    Html(get_fallback_html()).into_response()
}

/// Get inline fallback HTML for when the mobile dist is not available
fn get_fallback_html() -> String {
    r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <title>Deck Remote</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0a0a0f;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            padding: 20px;
        }
        .container {
            text-align: center;
            max-width: 400px;
        }
        h1 {
            font-size: 2rem;
            margin-bottom: 8px;
        }
        h1 span { color: #3b82f6; }
        p {
            color: #64748b;
            font-size: 0.9rem;
            line-height: 1.6;
        }
        .hint {
            margin-top: 24px;
            padding: 16px;
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        code {
            background: rgba(59, 130, 246, 0.2);
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.85rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>DECK<span>.</span></h1>
        <p>Mobile UI not built yet.</p>
        <div class="hint">
            <p>Run <code>npm run build:mobile</code> to build the mobile web app.</p>
        </div>
    </div>
</body>
</html>"##.to_string()
}

/// Verify authentication - checks query parameter first, then Authorization header
fn verify_auth(query: &AuthQuery, headers: &HeaderMap, settings: &Settings) -> bool {
    // First check query parameter
    if let Some(ref code) = query.code {
        if code == &settings.auth_code {
            return true;
        }
    }
    
    // Then check Authorization header (Bearer token)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if token == settings.auth_code {
                    return true;
                }
            }
        }
    }
    
    false
}

async fn execute_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, StatusCode> {
    let settings = state.settings.lock().await;
    
    if !verify_auth(&query, &headers, &settings) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    drop(settings);

    let commands = state.commands.lock().await;
    let cmd_config = commands.get(&req.id)
        .ok_or_else(|| StatusCode::NOT_FOUND)?
        .clone();
    drop(commands);

    // Execute the command using the system_commands module
    let result = system_commands::execute_command(&cmd_config);

    // Handle legacy focus_app field for backward compatibility
    if result.success {
        if let Some(ref app_title) = cmd_config.focus_app {
            if let Err(e) = windows_focus::focus_window_by_title(app_title) {
                eprintln!("Failed to focus window '{}': {}", app_title, e);
            }
        }
    }

    Ok(Json(ExecuteResponse {
        success: result.success,
        message: result.message,
    }))
}

pub struct ServerState {
    pub settings: Arc<tokio::sync::Mutex<Settings>>,
    pub commands: Arc<tokio::sync::Mutex<Commands>>,
    pub mobile_dist_path: Option<String>,
}

/// Handle for controlling the running server
pub struct ServerHandle {
    shutdown_tx: oneshot::Sender<()>,
}

impl ServerHandle {
    /// Gracefully shutdown the server
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

pub async fn start_server(
    settings: Settings,
    commands: Commands,
    mobile_dist_path: Option<String>,
) -> Result<ServerHandle, String> {
    let port = settings.port;
    
    let state = ServerState {
        settings: Arc::new(tokio::sync::Mutex::new(settings)),
        commands: Arc::new(tokio::sync::Mutex::new(commands)),
        mobile_dist_path: mobile_dist_path.clone(),
    };

    let shared_state = Arc::new(state);
    
    // Build router with API routes
    let mut router = Router::new()
        .route("/health", get(health_handler))
        .route("/execute", post(execute_handler))
        .route("/api/commands", get(commands_handler));
    
    // If mobile dist path exists, serve static files from it
    if let Some(ref dist_path) = mobile_dist_path {
        let path = PathBuf::from(dist_path);
        if path.exists() {
            println!("Serving mobile UI from: {}", dist_path);
            router = router
                .nest_service("/assets", ServeDir::new(path.join("assets")))
                .fallback(get(fallback_handler));
        } else {
            println!("Mobile dist not found at: {}, using fallback HTML", dist_path);
            router = router.fallback(get(fallback_handler));
        }
    } else {
        router = router.fallback(get(fallback_handler));
    }
    
    let router = router.with_state(shared_state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    println!("Server listening on http://{}", addr);

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn the server task
    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
                println!("Server shutting down gracefully...");
            })
            .await
            .ok();
    });

    Ok(ServerHandle { shutdown_tx })
}
