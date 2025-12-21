Implementation Plan: Remote PC Controller

This document outlines the architecture and tasks required to build a Desktop Application that manages a background server for remote PC control.
🛠 Tech Stack

    Frontend: Svelte + Vite (SPA)

    Desktop Wrapper: Tauri (Rust-based)

    Server: Axum (Embedded within Rust)

    Config: YAML/JSON for custom commands

🏗 Architecture

    Main Process (Rust): Manages the application lifecycle, system tray, and the Axum server thread.

    Webview (Svelte): The "Management UI" where you configure port, auth, and commands.

    Background Server (Axum): Listens on the local network to receive requests from your phone.

📋 Task List
Phase 1: Project Initialization

    [ ] Initialize Tauri project with Svelte: npm create tauri-app@latest.

    [ ] Install Rust dependencies in src-tauri/Cargo.toml:

        axum, tokio, serde, serde_yaml, tower-http, windows (for Windows focus logic).

    [ ] Set up the Svelte SPA structure for the Management UI.

Phase 2: Configuration & State

    [ ] Create a Settings struct in Rust (Port, Username, Password).

    [ ] Implement a commands.yaml parser to load user-defined actions.

    [ ] Create a Global State in Tauri to track if the server is currently running.

Phase 3: The Axum Command Server

    [ ] Implement the start_server Tauri Command:

        Should accept port, username, and password.

        Spawns a tokio::spawn thread.

        Includes RequireAuthorizationLayer for Basic Auth.

    [ ] Implement the execute_command route:

        Maps incoming IDs to system shell commands.

        Windows: Use Command::new("cmd").args(["/C", ...]).

        Unix: Use Command::new("sh").args(["-c", ...]).

Phase 4: Svelte Management UI

    [ ] Build a "Settings" page to input Port and Auth.

    [ ] Build a "Command Editor" to add/edit the YAML actions.

    [ ] Add a "Start/Stop" toggle that invokes the Rust backend.

    [ ] SPA Asset Serving: Configure Vite to output multiple chunks and set up the Rust ServeDir to serve the dist folder to the phone.

Phase 5: OS Integration (Focusing Windows)

    [ ] Implement the Windows-specific focus logic using winapi or windows-rs.

    [ ] Add a focus type to the YAML schema so users can target specific app titles.

🚀 Code Snippets for Cursor
Server Management Bridge (src-tauri/src/main.rs)
Rust

#[tauri::command]
async fn toggle*server(
state: tauri::State<'*, AppState>,
settings: Settings
) -> Result<bool, String> {
let mut lock = state.server_handle.lock().unwrap();

    if let Some(handle) = lock.take() {
        handle.abort(); // Stop server
        Ok(false)
    } else {
        let handle = tokio::spawn(async move {
            run_axum_server(settings).await;
        });
        *lock = Some(handle);
        Ok(true)
    }

}

Vite Config for Modular SPA (vite.config.js)
JavaScript

export default defineConfig({
build: {
rollupOptions: {
output: {
manualChunks: (id) => {
if (id.includes('node_modules')) return 'vendor';
}
}
}
}
});

🔒 Security Considerations

    Bind Address: Always bind the management UI to 127.0.0.1, but the Axum server must bind to 0.0.0.0 for phone access.

    Firewall: User must manually allow the application through the OS firewall for the specific port chosen.

    Auth: Ensure Basic Auth is enforced on the 0.0.0.0 listener.
