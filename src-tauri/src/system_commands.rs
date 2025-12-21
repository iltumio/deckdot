//! System command handlers for built-in command types
//! 
//! This module provides platform-specific implementations for:
//! - Volume control
//! - Opening directories
//! - Focusing applications
//! - Sending keyboard shortcuts

use crate::commands::{CommandConfig, CommandType, VolumeDirection};
use std::process::Command;

/// Result of executing a system command
#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

impl CommandResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

/// Execute a command based on its type
pub fn execute_command(config: &CommandConfig) -> CommandResult {
    match config.command_type {
        CommandType::Shell => execute_shell(&config.command),
        CommandType::Volume => execute_volume(&config.volume_direction, config.volume_step),
        CommandType::OpenDirectory => execute_open_directory(&config.directory_path),
        CommandType::FocusApp => execute_focus_app(&config.app_name),
        CommandType::Keybind => execute_keybind(&config.keybind),
    }
}

/// Execute a shell command
fn execute_shell(command: &Option<String>) -> CommandResult {
    let command_str = match command {
        Some(cmd) if !cmd.is_empty() => cmd,
        _ => return CommandResult::err("No command specified"),
    };

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command_str])
            .output()
    } else {
        Command::new("sh")
            .args(["-c", command_str])
            .output()
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                CommandResult::ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => CommandResult::err(format!("Failed to execute command: {}", e)),
    }
}

/// Control system volume
fn execute_volume(direction: &Option<VolumeDirection>, step: Option<u8>) -> CommandResult {
    let direction = match direction {
        Some(dir) => dir,
        None => return CommandResult::err("Volume direction not specified"),
    };
    
    let step = step.unwrap_or(5);

    #[cfg(target_os = "macos")]
    {
        let script = match direction {
            VolumeDirection::Up => {
                format!(
                    r#"set currentVolume to output volume of (get volume settings)
                    set volume output volume (currentVolume + {})"#,
                    step
                )
            }
            VolumeDirection::Down => {
                format!(
                    r#"set currentVolume to output volume of (get volume settings)
                    set volume output volume (currentVolume - {})"#,
                    step
                )
            }
            VolumeDirection::Mute => {
                r#"set isMuted to output muted of (get volume settings)
                set volume output muted (not isMuted)"#.to_string()
            }
        };

        let output = Command::new("osascript")
            .args(["-e", &script])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Volume {:?}", direction))
            }
            Ok(output) => {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
            Err(e) => CommandResult::err(format!("Failed to control volume: {}", e)),
        }
    }

    #[cfg(target_os = "windows")]
    {
        let script = match direction {
            VolumeDirection::Up => {
                format!(
                    r#"$wshell = New-Object -ComObject wscript.shell; for($i=0; $i -lt {}; $i++) {{ $wshell.SendKeys([char]175) }}"#,
                    step / 2 // Each key press is ~2% volume
                )
            }
            VolumeDirection::Down => {
                format!(
                    r#"$wshell = New-Object -ComObject wscript.shell; for($i=0; $i -lt {}; $i++) {{ $wshell.SendKeys([char]174) }}"#,
                    step / 2
                )
            }
            VolumeDirection::Mute => {
                r#"$wshell = New-Object -ComObject wscript.shell; $wshell.SendKeys([char]173)"#.to_string()
            }
        };

        let output = Command::new("powershell")
            .args(["-Command", &script])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Volume {:?}", direction))
            }
            Ok(output) => {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
            Err(e) => CommandResult::err(format!("Failed to control volume: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        let args = match direction {
            VolumeDirection::Up => format!("{}%+", step),
            VolumeDirection::Down => format!("{}%-", step),
            VolumeDirection::Mute => "toggle".to_string(),
        };

        // Try pactl first (PulseAudio/PipeWire), then amixer
        let output = Command::new("pactl")
            .args(["set-sink-volume", "@DEFAULT_SINK@", &args])
            .output()
            .or_else(|_| {
                Command::new("amixer")
                    .args(["set", "Master", &format!("{}%{}", step, 
                        if matches!(direction, VolumeDirection::Up) { "+" } else { "-" })])
                    .output()
            });

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Volume {:?}", direction))
            }
            Ok(output) => {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
            Err(e) => CommandResult::err(format!("Failed to control volume: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        CommandResult::err("Volume control not supported on this platform")
    }
}

/// Open a directory in the file manager
fn execute_open_directory(path: &Option<String>) -> CommandResult {
    let path = match path {
        Some(p) if !p.is_empty() => p,
        _ => return CommandResult::err("No directory path specified"),
    };

    // Expand ~ to home directory
    let expanded_path = if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            path.replacen('~', &home.to_string_lossy(), 1)
        } else {
            path.clone()
        }
    } else {
        path.clone()
    };

    #[cfg(target_os = "macos")]
    let output = Command::new("open").arg(&expanded_path).output();

    #[cfg(target_os = "windows")]
    let output = Command::new("explorer").arg(&expanded_path).output();

    #[cfg(target_os = "linux")]
    let output = Command::new("xdg-open").arg(&expanded_path).output();

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    let output: Result<std::process::Output, std::io::Error> = 
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported platform"));

    match output {
        Ok(output) if output.status.success() => {
            CommandResult::ok(format!("Opened: {}", expanded_path))
        }
        Ok(output) => {
            CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
        }
        Err(e) => CommandResult::err(format!("Failed to open directory: {}", e)),
    }
}

/// Focus an application window
fn execute_focus_app(app_name: &Option<String>) -> CommandResult {
    let app_name = match app_name {
        Some(name) if !name.is_empty() => name,
        _ => return CommandResult::err("No application name specified"),
    };

    #[cfg(target_os = "macos")]
    {
        // Try to activate the app by name
        let script = format!(
            r#"tell application "{}" to activate"#,
            app_name.replace('"', r#"\""#)
        );

        let output = Command::new("osascript")
            .args(["-e", &script])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Focused: {}", app_name))
            }
            Ok(_) => {
                // Try with "System Events" for more generic matching
                let script2 = format!(
                    r#"tell application "System Events"
                        set appList to every process whose name contains "{}"
                        if (count of appList) > 0 then
                            set frontmost of (item 1 of appList) to true
                        end if
                    end tell"#,
                    app_name.replace('"', r#"\""#)
                );
                
                let output2 = Command::new("osascript")
                    .args(["-e", &script2])
                    .output();

                match output2 {
                    Ok(output) if output.status.success() => {
                        CommandResult::ok(format!("Focused: {}", app_name))
                    }
                    _ => CommandResult::err(format!("Could not find or focus: {}", app_name)),
                }
            }
            Err(e) => CommandResult::err(format!("Failed to focus app: {}", e)),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Use the windows_focus module
        crate::windows_focus::focus_window_by_title(app_name)
            .map(|_| CommandResult::ok(format!("Focused: {}", app_name)))
            .unwrap_or_else(|e| CommandResult::err(format!("Failed to focus: {}", e)))
    }

    #[cfg(target_os = "linux")]
    {
        // Try wmctrl first, then xdotool
        let output = Command::new("wmctrl")
            .args(["-a", app_name])
            .output()
            .or_else(|_| {
                Command::new("xdotool")
                    .args(["search", "--name", app_name, "windowactivate"])
                    .output()
            });

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Focused: {}", app_name))
            }
            Ok(output) => {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
            Err(e) => CommandResult::err(format!("Failed to focus app: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        CommandResult::err("Focus app not supported on this platform")
    }
}

/// Send a keyboard shortcut
fn execute_keybind(keybind: &Option<String>) -> CommandResult {
    let keybind = match keybind {
        Some(k) if !k.is_empty() => k,
        _ => return CommandResult::err("No keybind specified"),
    };

    #[cfg(target_os = "macos")]
    {
        // Parse the keybind string and convert to AppleScript
        let script = convert_keybind_to_applescript(keybind);
        
        let output = Command::new("osascript")
            .args(["-e", &script])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Sent: {}", keybind))
            }
            Ok(output) => {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
            Err(e) => CommandResult::err(format!("Failed to send keybind: {}", e)),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Convert to PowerShell SendKeys format
        let sendkeys = convert_keybind_to_sendkeys(keybind);
        let script = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait("{}")"#,
            sendkeys
        );

        let output = Command::new("powershell")
            .args(["-Command", &script])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Sent: {}", keybind))
            }
            Ok(output) => {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
            Err(e) => CommandResult::err(format!("Failed to send keybind: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Use xdotool
        let xdotool_keys = convert_keybind_to_xdotool(keybind);
        
        let output = Command::new("xdotool")
            .args(["key", &xdotool_keys])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                CommandResult::ok(format!("Sent: {}", keybind))
            }
            Ok(output) => {
                CommandResult::err(String::from_utf8_lossy(&output.stderr).to_string())
            }
            Err(e) => CommandResult::err(format!("Failed to send keybind: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        CommandResult::err("Keybind not supported on this platform")
    }
}

/// Convert a keybind string like "cmd+shift+v" to AppleScript keystroke
#[cfg(target_os = "macos")]
fn convert_keybind_to_applescript(keybind: &str) -> String {
    let parts: Vec<String> = keybind.split('+').map(|s| s.trim().to_lowercase()).collect();
    
    let mut modifiers = Vec::new();
    let mut key = String::new();

    for part in &parts {
        match part.as_str() {
            "cmd" | "command" | "super" => modifiers.push("command down"),
            "ctrl" | "control" => modifiers.push("control down"),
            "alt" | "option" => modifiers.push("option down"),
            "shift" => modifiers.push("shift down"),
            k => key = k.to_string(),
        }
    }

    let modifier_str = if modifiers.is_empty() {
        String::new()
    } else {
        format!(" using {{{}}}", modifiers.join(", "))
    };

    // Handle special keys
    let keystroke = match key.as_str() {
        "enter" | "return" => r#"key code 36"#.to_string(),
        "tab" => r#"key code 48"#.to_string(),
        "escape" | "esc" => r#"key code 53"#.to_string(),
        "space" => r#"keystroke " ""#.to_string(),
        "up" => r#"key code 126"#.to_string(),
        "down" => r#"key code 125"#.to_string(),
        "left" => r#"key code 123"#.to_string(),
        "right" => r#"key code 124"#.to_string(),
        "delete" | "backspace" => r#"key code 51"#.to_string(),
        "f1" => r#"key code 122"#.to_string(),
        "f2" => r#"key code 120"#.to_string(),
        "f3" => r#"key code 99"#.to_string(),
        "f4" => r#"key code 118"#.to_string(),
        "f5" => r#"key code 96"#.to_string(),
        "f6" => r#"key code 97"#.to_string(),
        "f7" => r#"key code 98"#.to_string(),
        "f8" => r#"key code 100"#.to_string(),
        "f9" => r#"key code 101"#.to_string(),
        "f10" => r#"key code 109"#.to_string(),
        "f11" => r#"key code 103"#.to_string(),
        "f12" => r#"key code 111"#.to_string(),
        k => format!(r#"keystroke "{}""#, k),
    };

    format!(
        r#"tell application "System Events" to {}{}"#,
        keystroke, modifier_str
    )
}

/// Convert a keybind string to Windows SendKeys format
#[cfg(target_os = "windows")]
fn convert_keybind_to_sendkeys(keybind: &str) -> String {
    let parts: Vec<&str> = keybind.split('+').map(|s| s.trim()).collect();
    let mut result = String::new();

    for part in &parts {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => result.push('^'),
            "alt" => result.push('%'),
            "shift" => result.push('+'),
            "cmd" | "command" | "super" | "win" => result.push_str("^"), // Map Cmd to Ctrl on Windows
            _ => {}
        }
    }

    // Find the actual key (last non-modifier)
    if let Some(key) = parts.last() {
        let lower = key.to_lowercase();
        let key_str = match lower.as_str() {
            "ctrl" | "control" | "alt" | "shift" | "cmd" | "command" | "super" | "win" => "",
            "enter" | "return" => "{ENTER}",
            "tab" => "{TAB}",
            "escape" | "esc" => "{ESC}",
            "space" => " ",
            "up" => "{UP}",
            "down" => "{DOWN}",
            "left" => "{LEFT}",
            "right" => "{RIGHT}",
            "delete" => "{DELETE}",
            "backspace" => "{BACKSPACE}",
            "home" => "{HOME}",
            "end" => "{END}",
            "pageup" => "{PGUP}",
            "pagedown" => "{PGDN}",
            "f1" => "{F1}",
            "f2" => "{F2}",
            "f3" => "{F3}",
            "f4" => "{F4}",
            "f5" => "{F5}",
            "f6" => "{F6}",
            "f7" => "{F7}",
            "f8" => "{F8}",
            "f9" => "{F9}",
            "f10" => "{F10}",
            "f11" => "{F11}",
            "f12" => "{F12}",
            k if !["ctrl", "control", "alt", "shift", "cmd", "command", "super", "win"].contains(&k) => {
                return result + k;
            }
            _ => "",
        };
        result.push_str(key_str);
    }

    result
}

/// Convert a keybind string to xdotool format
#[cfg(target_os = "linux")]
fn convert_keybind_to_xdotool(keybind: &str) -> String {
    let parts: Vec<&str> = keybind.split('+').map(|s| s.trim()).collect();
    let mut result = Vec::new();

    for part in parts {
        let lower = part.to_lowercase();
        let xdotool_key = match lower.as_str() {
            "ctrl" | "control" => "ctrl",
            "alt" => "alt",
            "shift" => "shift",
            "cmd" | "command" | "super" | "win" => "super",
            "enter" | "return" => "Return",
            "tab" => "Tab",
            "escape" | "esc" => "Escape",
            "space" => "space",
            "up" => "Up",
            "down" => "Down",
            "left" => "Left",
            "right" => "Right",
            "delete" => "Delete",
            "backspace" => "BackSpace",
            k => k,
        };
        result.push(xdotool_key.to_string());
    }

    result.join("+")
}

/// Get list of running applications
pub fn get_running_applications() -> Vec<String> {
    let mut apps = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let script = r#"tell application "System Events"
            set appNames to name of every process whose background only is false
            return appNames
        end tell"#;

        if let Ok(output) = Command::new("osascript")
            .args(["-e", script])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                apps = output_str
                    .trim()
                    .split(", ")
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let script = r#"Get-Process | Where-Object {$_.MainWindowTitle -ne ''} | Select-Object -ExpandProperty MainWindowTitle"#;
        
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", script])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                apps = output_str
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try wmctrl first
        if let Ok(output) = Command::new("wmctrl").args(["-l"]).output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                apps = output_str
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.splitn(4, ' ').collect();
                        parts.get(3).map(|s| s.to_string())
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    apps.sort();
    apps.dedup();
    apps
}

