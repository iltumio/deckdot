# Deckdot - Remote PC Controller

**Version 0.1.0**

A desktop application that allows you to control your PC remotely from your phone via a beautiful web interface. Built with Tauri and Svelte.

## What is DeckDot?

DeckDot is a remote control application that runs on your desktop computer and provides a web interface accessible from any device on your local network. You can execute commands, control volume, open applications, and more—all from your phone or tablet.

## Features

- **Remote Control**: Control your PC from any device on your local network
- **Multiple Command Types**:
  - Shell commands (execute any terminal command)
  - Volume control (adjust system volume up/down or mute)
  - Open directories (open folders in your file manager)
  - Focus applications (bring apps to the foreground)
  - Send keybinds (simulate keyboard shortcuts)
- **Simple Authentication**: Auto-generated access code for secure access
- **Beautiful Mobile UI**: Responsive grid interface optimized for phones and tablets
- **Customizable Grid**: Choose 2, 4, 6, or 8 columns for command display
- **Persistent Storage**: All commands and settings saved automatically
- **One-Click Sharing**: Share button generates a link with embedded access code

## How It Works

1. **Install & Launch**: Install Deck on your desktop computer and launch the application
2. **Configure**: Set up your server port and access code in the Settings tab
3. **Add Commands**: Create custom commands in the Commands tab—just enter a name and configure the action
4. **Start Server**: Click "Start Server" to begin accepting remote connections
5. **Access Remotely**: Open the shared link on your phone or tablet to access the control interface
6. **Execute Commands**: Tap any command button to execute it instantly

## Command Types

### Shell Commands

Execute any terminal command. Perfect for launching applications, running scripts, or system operations.

### Volume Control

Adjust your system volume by a specified percentage or toggle mute. Works on macOS, Windows, and Linux.

### Open Directory

Open any folder in your file manager. Use the browse button to select a directory visually.

### Focus Application

Bring any running application to the foreground. Great for switching between apps remotely.

### Send Keybind

Simulate keyboard shortcuts like `Cmd+Space` or `Ctrl+Shift+T`.
**Note**: On macOS, this requires accessibility permissions. The app will guide you through granting them.

## Getting Started

### Installation

1. Download the latest release for your platform
2. Install and launch the application
3. Configure your settings (port and access code)
4. Add your first command
5. Start the server and share the access link

### First Time Setup

1. **Settings Tab**:

   - Choose a port (default: 7776)
   - Your access code is auto-generated, but you can customize it
   - Click "Regenerate Code" to create a new random code

2. **Commands Tab**:

   - Click "Add Command" to create a new command
   - Enter a name (the ID is auto-generated)
   - Select the command type
   - Configure the specific settings for that type
   - Save your command

3. **Server Tab**:
   - Click "Start Server" to begin accepting connections
   - Use "Share Remote Access Link" to get a URL with your access code
   - Open the link on your phone or tablet

### Using the Mobile Interface

- **Access Code**: Enter the access code to authenticate
- **Command Grid**: Commands are displayed in a responsive grid
- **Column Selection**: Choose 2, 4, 6, or 8 columns (or use auto-responsive)
- **Execute**: Tap any command button to execute it
- **Refresh**: Use the refresh button to reload commands

## Security

- All commands require authentication via access code
- Server only accepts connections from devices with the correct access code
- Commands execute with your user permissions (no elevation)
- Access code can be regenerated at any time
- **Important**: Configure your firewall to allow connections on the chosen port

## Requirements

- **Desktop**: Windows, macOS, or Linux
- **Network**: Both devices must be on the same local network
- **macOS Keybinds**: Requires accessibility permission (the app will guide you)

## License

MIT License

## Support

For issues, questions, or contributions, please visit the project repository.
