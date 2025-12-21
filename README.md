# Deck - Remote PC Controller

A desktop application built with Tauri, Svelte, and Axum that allows you to control your PC remotely from your phone via HTTP commands.

## Features

- **Desktop Management UI**: Configure server settings, manage commands, and control the server
- **HTTP Command Server**: Lightweight Axum server that listens on your local network
- **Simple Access Code**: Single auto-generated access code for authentication (no username needed)
- **One-Click Sharing**: Share button copies a link with embedded auth code for easy phone access
- **Custom Commands**: Define custom shell commands via YAML configuration
- **Windows Focus Support**: Automatically focus specific applications after command execution (Windows only)
- **Phone-Accessible UI**: Beautiful Svelte mobile web app served from the Rust backend

## Tech Stack

- **Frontend**: Svelte 5 + Vite + Tailwind CSS v4
- **UI Components**: shadcn-svelte + Bits UI + Lucide Svelte
- **Desktop Wrapper**: Tauri 2.0
- **Backend Server**: Axum (embedded in Rust)
- **Mobile Web UI**: Svelte (same components as desktop)
- **Configuration**: YAML files for commands

## Project Structure

```
deck/
├── src-tauri/          # Rust backend (Tauri + Axum)
│   ├── src/
│   │   ├── main.rs     # Tauri app entry, command bridge
│   │   ├── config.rs   # Settings management
│   │   ├── commands.rs # Command configuration loading
│   │   ├── server.rs   # Axum HTTP server + static file serving
│   │   └── windows_focus.rs  # Windows window focus logic
│   └── Cargo.toml
├── src/                # Svelte desktop frontend
│   ├── App.svelte
│   ├── Settings.svelte
│   ├── Commands.svelte
│   ├── ServerControl.svelte
│   └── lib/            # Shared UI components (shadcn-svelte)
├── mobile/             # Svelte mobile web app (reuses src/lib)
│   ├── App.svelte      # Mobile-optimized command interface
│   ├── main.js
│   └── app.css
├── mobile-dist/        # Built mobile web app (served by Rust)
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

1. **Server Settings**: Configure the port in the Settings tab
2. **Access Code**: A random access code is auto-generated on first launch. You can:
   - View/copy the current code
   - Regenerate a new random code
   - Set a custom code
3. **Commands**: Add/edit commands in the Commands tab. Each command has:
   - **ID**: Unique identifier
   - **Name**: Display name
   - **Command**: Shell command to execute
   - **Focus App** (optional): Window title to focus after execution (Windows only)

### Starting the Server

1. Configure your settings in the Settings tab
2. Click "Start Server" in the Server Control tab
3. The server will listen on `0.0.0.0:{port}` for network access
4. Click **"Share Remote Access Link"** to copy a URL with embedded auth code
5. Open the link on your phone — it will auto-fill the access code

### Command Execution

Send POST requests to `/execute` endpoint with your access code:

```bash
# Using query parameter (recommended)
curl -X POST "http://[pc-ip]:7776/execute?code=YOUR_CODE" \
  -H "Content-Type: application/json" \
  -d '{"id": "open_notepad"}'

# Or using Bearer token header
curl -X POST http://[pc-ip]:7776/execute \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_CODE" \
  -d '{"id": "open_notepad"}'
```

### Health Check

Check if the server is running:

```bash
curl http://[pc-ip]:7776/health
```

## Security Considerations

- The management UI (Tauri webview) only binds to `127.0.0.1`
- The command server binds to `0.0.0.0` for network access
- Access code authentication is enforced on all command execution routes
- **Regenerate code** if you suspect it's been compromised
- **Important**: You must manually configure your firewall to allow connections on the chosen port
- Commands execute with the user's permissions (no elevation)

## Development

### Desktop Frontend Development

```bash
npm run dev
```

### Mobile Web App Development

```bash
npm run dev:mobile
```

This starts a dev server at `http://localhost:1421` for the mobile web app.

### Backend Development

The Rust backend is compiled automatically when running `npm run tauri dev`.

### Building for Production

```bash
# Build both desktop and mobile apps
npm run build:all

# Then build the Tauri application
npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

### Build Scripts

- `npm run dev` - Start desktop Vite dev server
- `npm run dev:mobile` - Start mobile web app dev server
- `npm run build` - Build desktop frontend
- `npm run build:mobile` - Build mobile web app to `mobile-dist/`
- `npm run build:all` - Build both desktop and mobile

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
