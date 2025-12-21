use crate::commands::{CommandConfig, Commands};
use crate::config::Settings;
use crate::windows_focus;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::oneshot;

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

#[derive(Serialize)]
struct CommandInfo {
    id: String,
    name: String,
}

async fn commands_handler(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<CommandInfo>>, StatusCode> {
    let settings = state.settings.lock().await;
    
    if !verify_basic_auth(&headers, &settings) {
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

async fn mobile_ui_handler(
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let commands = state.commands.lock().await;
    let command_list: Vec<CommandConfig> = commands.all();
    drop(commands);

    let buttons_html: String = command_list
        .iter()
        .map(|cmd| {
            format!(
                r#"<button class="cmd-btn" onclick="executeCommand('{}')">
                    <span class="cmd-name">{}</span>
                    <span class="cmd-id">{}</span>
                </button>"#,
                cmd.id, cmd.name, cmd.id
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <title>Deck Remote</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
            min-height: 100vh;
            padding: 20px;
            color: white;
        }}
        .container {{
            max-width: 500px;
            margin: 0 auto;
        }}
        h1 {{
            text-align: center;
            font-size: 1.5rem;
            margin-bottom: 8px;
            background: linear-gradient(90deg, #3b82f6, #8b5cf6);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        .subtitle {{
            text-align: center;
            color: #64748b;
            font-size: 0.75rem;
            margin-bottom: 24px;
            text-transform: uppercase;
            letter-spacing: 0.1em;
        }}
        .auth-form {{
            background: rgba(255,255,255,0.05);
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: 16px;
            padding: 20px;
            margin-bottom: 20px;
        }}
        .auth-form input {{
            width: 100%;
            padding: 14px 16px;
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: 12px;
            background: rgba(255,255,255,0.05);
            color: white;
            font-size: 16px;
            margin-bottom: 12px;
        }}
        .auth-form input::placeholder {{ color: #64748b; }}
        .commands {{
            display: grid;
            gap: 12px;
        }}
        .cmd-btn {{
            width: 100%;
            padding: 20px;
            border: 1px solid rgba(59, 130, 246, 0.3);
            border-radius: 16px;
            background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(139, 92, 246, 0.1));
            color: white;
            font-size: 16px;
            cursor: pointer;
            transition: all 0.2s;
            text-align: left;
            display: flex;
            flex-direction: column;
            gap: 4px;
        }}
        .cmd-btn:active {{
            transform: scale(0.98);
            background: linear-gradient(135deg, rgba(59, 130, 246, 0.3), rgba(139, 92, 246, 0.3));
        }}
        .cmd-name {{
            font-weight: 600;
            font-size: 1.1rem;
        }}
        .cmd-id {{
            font-size: 0.7rem;
            color: #64748b;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        .status {{
            position: fixed;
            bottom: 20px;
            left: 20px;
            right: 20px;
            padding: 16px;
            border-radius: 12px;
            text-align: center;
            font-weight: 500;
            transform: translateY(100px);
            transition: transform 0.3s;
        }}
        .status.show {{ transform: translateY(0); }}
        .status.success {{ background: #22c55e; }}
        .status.error {{ background: #ef4444; }}
        .no-commands {{
            text-align: center;
            padding: 40px;
            color: #64748b;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>DECK.</h1>
        <p class="subtitle">Remote PC Controller</p>
        
        <div class="auth-form">
            <input type="text" id="username" placeholder="Username" autocomplete="username">
            <input type="password" id="password" placeholder="Password" autocomplete="current-password">
        </div>

        <div class="commands">
            {}
        </div>
        
        {}
    </div>

    <div class="status" id="status"></div>

    <script>
        function getAuthHeader() {{
            const username = document.getElementById('username').value;
            const password = document.getElementById('password').value;
            return 'Basic ' + btoa(username + ':' + password);
        }}

        function showStatus(message, type) {{
            const status = document.getElementById('status');
            status.textContent = message;
            status.className = 'status show ' + type;
            setTimeout(() => status.classList.remove('show'), 2000);
        }}

        async function executeCommand(id) {{
            try {{
                const response = await fetch('/execute', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                        'Authorization': getAuthHeader()
                    }},
                    body: JSON.stringify({{ id }})
                }});
                
                if (response.status === 401) {{
                    showStatus('Invalid credentials', 'error');
                    return;
                }}
                
                const data = await response.json();
                showStatus(data.success ? 'Command executed!' : data.message, data.success ? 'success' : 'error');
            }} catch (e) {{
                showStatus('Connection error', 'error');
            }}
        }}

        // Save credentials to localStorage
        document.getElementById('username').value = localStorage.getItem('deck_user') || '';
        document.getElementById('password').value = localStorage.getItem('deck_pass') || '';
        
        document.getElementById('username').onchange = (e) => localStorage.setItem('deck_user', e.target.value);
        document.getElementById('password').onchange = (e) => localStorage.setItem('deck_pass', e.target.value);
    </script>
</body>
</html>"#,
        buttons_html,
        if command_list.is_empty() {
            r#"<div class="no-commands">No commands configured.<br>Add commands in the desktop app.</div>"#
        } else { "" }
    );

    Html(html)
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
    _dist_path: Option<String>,
) -> Result<ServerHandle, String> {
    let port = settings.port;
    let state = ServerState {
        settings: Arc::new(tokio::sync::Mutex::new(settings)),
        commands: Arc::new(tokio::sync::Mutex::new(commands)),
    };

    let shared_state = Arc::new(state);
    
    // Always include the mobile UI route at root
    let router = Router::new()
        .route("/", get(mobile_ui_handler))
        .route("/health", get(health_handler))
        .route("/execute", post(execute_handler))
        .route("/api/commands", get(commands_handler))
        .with_state(shared_state);

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

