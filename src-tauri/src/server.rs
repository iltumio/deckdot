use crate::commands::{CommandConfig, Commands};
use crate::config::Settings;
use crate::system_commands;
use crate::windows_focus;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
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

async fn mobile_ui_handler(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    let commands = state.commands.lock().await;
    let command_list: Vec<CommandConfig> = commands.all();
    drop(commands);

    // Check if we have an auth code in the URL
    let auth_code_from_url = query.code.unwrap_or_default();
    let has_code = !auth_code_from_url.is_empty();

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

    let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <title>Deck Remote</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@500;700&family=Outfit:wght@400;600;800&display=swap" rel="stylesheet">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: 'Outfit', sans-serif;
            background: #0a0a0f;
            background-image: 
                radial-gradient(ellipse 80% 50% at 50% -20%, rgba(59, 130, 246, 0.15), transparent),
                radial-gradient(ellipse 60% 40% at 100% 100%, rgba(139, 92, 246, 0.1), transparent);
            min-height: 100vh;
            min-height: 100dvh;
            padding: 20px;
            color: white;
        }}
        .container {{
            max-width: 420px;
            margin: 0 auto;
        }}
        .header {{
            text-align: center;
            margin-bottom: 28px;
        }}
        h1 {{
            font-size: 2.2rem;
            font-weight: 800;
            letter-spacing: -0.02em;
            color: white;
        }}
        h1 span {{
            color: #3b82f6;
        }}
        .subtitle {{
            color: #4b5563;
            font-size: 0.7rem;
            text-transform: uppercase;
            letter-spacing: 0.2em;
            font-weight: 600;
        }}
        .auth-section {{
            background: rgba(255,255,255,0.02);
            border: 1px solid rgba(255,255,255,0.06);
            border-radius: 16px;
            padding: 16px;
            margin-bottom: 24px;
        }}
        .auth-header {{
            display: flex;
            align-items: center;
            gap: 10px;
            margin-bottom: 12px;
        }}
        .auth-icon {{
            width: 18px;
            height: 18px;
            color: #6366f1;
        }}
        .auth-label {{
            font-size: 0.65rem;
            text-transform: uppercase;
            letter-spacing: 0.15em;
            color: #6b7280;
            font-weight: 700;
        }}
        .auth-status {{
            margin-left: auto;
            font-size: 0.6rem;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            padding: 4px 8px;
            border-radius: 6px;
            font-weight: 700;
        }}
        .auth-status.valid {{
            background: rgba(34, 197, 94, 0.15);
            color: #22c55e;
        }}
        .auth-status.invalid {{
            background: rgba(239, 68, 68, 0.15);
            color: #ef4444;
        }}
        .auth-input-wrapper {{
            position: relative;
        }}
        .auth-form input {{
            width: 100%;
            padding: 14px 16px;
            border: 1px solid rgba(255,255,255,0.08);
            border-radius: 12px;
            background: rgba(0,0,0,0.4);
            color: white;
            font-size: 18px;
            font-family: 'JetBrains Mono', monospace;
            font-weight: 500;
            letter-spacing: 0.3em;
            text-align: center;
            text-transform: uppercase;
        }}
        .auth-form input::placeholder {{ 
            color: #374151; 
            letter-spacing: 0.15em;
            text-transform: none;
        }}
        .auth-form input:focus {{
            outline: none;
            border-color: #3b82f6;
            box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.15);
        }}
        .commands {{
            display: grid;
            gap: 10px;
        }}
        .cmd-btn {{
            width: 100%;
            padding: 18px 20px;
            border: 1px solid rgba(255,255,255,0.06);
            border-radius: 14px;
            background: rgba(255,255,255,0.02);
            color: white;
            font-size: 16px;
            cursor: pointer;
            transition: all 0.15s ease;
            text-align: left;
            display: flex;
            flex-direction: column;
            gap: 4px;
            font-family: 'Outfit', sans-serif;
        }}
        .cmd-btn:hover {{
            background: rgba(59, 130, 246, 0.08);
            border-color: rgba(59, 130, 246, 0.2);
        }}
        .cmd-btn:active {{
            transform: scale(0.98);
            background: rgba(59, 130, 246, 0.15);
        }}
        .cmd-name {{
            font-weight: 600;
            font-size: 1.05rem;
        }}
        .cmd-id {{
            font-size: 0.65rem;
            color: #4b5563;
            text-transform: uppercase;
            letter-spacing: 0.08em;
            font-family: 'JetBrains Mono', monospace;
        }}
        .status {{
            position: fixed;
            bottom: 20px;
            left: 20px;
            right: 20px;
            padding: 16px;
            border-radius: 14px;
            text-align: center;
            font-weight: 600;
            font-size: 0.9rem;
            transform: translateY(120px);
            transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
            backdrop-filter: blur(12px);
        }}
        .status.show {{ transform: translateY(0); }}
        .status.success {{ 
            background: rgba(34, 197, 94, 0.9);
            border: 1px solid rgba(34, 197, 94, 0.5);
        }}
        .status.error {{ 
            background: rgba(239, 68, 68, 0.9);
            border: 1px solid rgba(239, 68, 68, 0.5);
        }}
        .no-commands {{
            text-align: center;
            padding: 48px 24px;
            color: #4b5563;
            font-size: 0.9rem;
        }}
        .no-commands strong {{
            display: block;
            color: #6b7280;
            margin-bottom: 4px;
        }}
        .hidden {{ display: none; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>DECK<span>.</span></h1>
            <p class="subtitle">Remote PC Controller</p>
        </div>
        
        <div class="auth-section">
            <div class="auth-header">
                <svg class="auth-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
                    <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                </svg>
                <span class="auth-label">Access Code</span>
                <span id="auth-status" class="auth-status hidden">—</span>
            </div>
            <div class="auth-form">
                <div class="auth-input-wrapper">
                    <input 
                        type="text" 
                        id="authCode" 
                        placeholder="Enter code" 
                        autocomplete="off"
                        autocapitalize="characters"
                        maxlength="12"
                        value="{auth_code_value}"
                    >
                </div>
            </div>
        </div>

        <div class="commands">
            {buttons_html}
        </div>
        
        {no_commands_html}
    </div>

    <div class="status" id="status"></div>

    <script>
        const authCodeInput = document.getElementById('authCode');
        const authStatus = document.getElementById('auth-status');
        let hasValidated = false;
        
        // Auto-uppercase and store
        authCodeInput.addEventListener('input', (e) => {{
            e.target.value = e.target.value.toUpperCase();
            localStorage.setItem('deck_code', e.target.value);
        }});

        // Load from localStorage if not in URL
        if (!authCodeInput.value) {{
            authCodeInput.value = localStorage.getItem('deck_code') || '';
        }} else {{
            // Save URL code to localStorage
            localStorage.setItem('deck_code', authCodeInput.value);
        }}

        function getAuthCode() {{
            return authCodeInput.value.trim();
        }}

        function updateAuthStatus(valid) {{
            hasValidated = true;
            authStatus.classList.remove('hidden', 'valid', 'invalid');
            if (valid) {{
                authStatus.textContent = 'Valid';
                authStatus.classList.add('valid');
            }} else {{
                authStatus.textContent = 'Invalid';
                authStatus.classList.add('invalid');
            }}
        }}

        function showStatus(message, type) {{
            const status = document.getElementById('status');
            status.textContent = message;
            status.className = 'status show ' + type;
            setTimeout(() => status.classList.remove('show'), 2500);
        }}

        async function executeCommand(id) {{
            const code = getAuthCode();
            if (!code) {{
                showStatus('Enter access code first', 'error');
                authCodeInput.focus();
                return;
            }}
            
            try {{
                const response = await fetch('/execute?code=' + encodeURIComponent(code), {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json'
                    }},
                    body: JSON.stringify({{ id }})
                }});
                
                if (response.status === 401) {{
                    updateAuthStatus(false);
                    showStatus('Invalid access code', 'error');
                    return;
                }}
                
                updateAuthStatus(true);
                const data = await response.json();
                showStatus(data.success ? 'Executed!' : data.message, data.success ? 'success' : 'error');
            }} catch (e) {{
                showStatus('Connection error', 'error');
            }}
        }}
    </script>
</body>
</html>"##,
        auth_code_value = auth_code_from_url,
        buttons_html = buttons_html,
        no_commands_html = if command_list.is_empty() {
            r#"<div class="no-commands"><strong>No commands configured</strong>Add commands in the desktop app</div>"#
        } else { "" }
    );

    Html(html)
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

    // Execute the command using the new system_commands module
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
