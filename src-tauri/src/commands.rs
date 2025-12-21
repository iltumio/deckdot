use serde::{Deserialize, Serialize};

/// Available command types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    /// Execute a shell command directly
    Shell,
    /// Adjust system volume
    Volume,
    /// Open a directory in the file manager
    OpenDirectory,
    /// Focus an application window
    FocusApp,
    /// Send a keyboard shortcut
    Keybind,
}

impl Default for CommandType {
    fn default() -> Self {
        CommandType::Shell
    }
}

/// Volume direction for volume commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VolumeDirection {
    Up,
    Down,
    Mute,
}

/// Configuration for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub id: String,
    pub name: String,
    
    /// The type of command - defaults to "shell" for backward compatibility
    #[serde(default)]
    pub command_type: CommandType,
    
    /// Shell command to execute (for Shell type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    
    /// Volume direction (for Volume type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_direction: Option<VolumeDirection>,
    
    /// Volume step amount, default 5 (for Volume type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_step: Option<u8>,
    
    /// Directory path to open (for OpenDirectory type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_path: Option<String>,
    
    /// Application name or bundle ID to focus (for FocusApp type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    
    /// Key combination to send (for Keybind type), e.g., "cmd+shift+v"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keybind: Option<String>,
    
    /// Window title to focus after execution (Windows only, deprecated - use FocusApp type instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_app: Option<String>,
}
