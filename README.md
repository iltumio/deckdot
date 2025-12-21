# Deck - Remote PC Controller

A desktop application built with Tauri, Svelte, and Axum that allows you to control your PC remotely from your phone via HTTP commands.

## Features

- **Desktop Management UI**: Configure server settings, manage commands, and control the server
- **HTTP Command Server**: Lightweight Axum server that listens on your local network
- **Basic Authentication**: Secure your server with username/password authentication
- **Custom Commands**: Define custom shell commands via YAML configuration
- **Windows Focus Support**: Automatically focus specific applications after command execution (Windows only)
- **Phone-Accessible UI**: Access the management interface from your phone on the same network

## Tech Stack

- **Frontend**: Svelte 5 + Vite + Tailwind CSS v4
- **UI Components**: shadcn-svelte + Bits UI + Lucide Svelte
- **Desktop Wrapper**: Tauri 2.0
- **Backend Server**: Axum (embedded in Rust)
- **Configuration**: YAML files for commands

## Project Structure

```
deck/
├── src-tauri/          # Rust backend (Tauri + Axum)
│   ├── src/
│   │   ├── main.rs     # Tauri app entry, command bridge
│   │   ├── config.rs   # Settings management
│   │   ├── commands.rs # Command configuration loading
│   │   ├── server.rs   # Axum HTTP server
│   │   └── windows_focus.rs  # Windows window focus logic
│   └── Cargo.toml
├── src/                # Svelte frontend
│   ├── App.svelte
│   ├── Settings.svelte
│   ├── Commands.svelte
│   └── ServerControl.svelte
├── commands.yaml       # Default command configuration
└── package.json
```

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js and npm
- System dependencies for Tauri (see [Tauri docs](https://tauri.app/v1/guides/getting-started/prerequisites))

### Installation

1. Install dependencies:

```bash
npm install
```

2. Build the application:

```bash
npm run tauri build
```

Or run in development mode:

```bash
npm run tauri dev
```

## Usage

### Configuration

1. **Server Settings**: Configure the port, username, and password in the Settings tab
2. **Commands**: Add/edit commands in the Commands tab. Each command has:
   - **ID**: Unique identifier
   - **Name**: Display name
   - **Command**: Shell command to execute
   - **Focus App** (optional): Window title to focus after execution (Windows only)

### Starting the Server

1. Configure your settings in the Settings tab
2. Click "Start Server" in the Server Control tab
3. The server will listen on `0.0.0.0:{port}` for network access
4. Access the management UI from your phone at `http://[your-pc-ip]:{port}`

### Command Execution

Send POST requests to `/execute` endpoint:

```bash
curl -X POST http://[pc-ip]:8080/execute \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic $(echo -n 'username:password' | base64)" \
  -d '{"id": "open_notepad"}'
```

### Health Check

Check if the server is running:

```bash
curl http://[pc-ip]:8080/health
```

## Security Considerations

- The management UI (Tauri webview) only binds to `127.0.0.1`
- The command server binds to `0.0.0.0` for network access
- Basic Auth is enforced on all server routes
- **Important**: You must manually configure your firewall to allow connections on the chosen port
- Commands execute with the user's permissions (no elevation)

## Development

### Frontend Development

```bash
npm run dev
```

### Backend Development

The Rust backend is compiled automatically when running `npm run tauri dev`.

### Building for Production

```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

## Command Configuration

Commands are stored in `commands.yaml` in the app data directory. Example:

```yaml
- id: open_notepad
  name: Open Notepad
  command: notepad.exe
  focus_app: Notepad

- id: lock_screen
  name: Lock Screen
  command: rundll32.exe user32.dll,LockWorkStation
```

## License

[Your License Here]
