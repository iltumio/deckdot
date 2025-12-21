use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

impl CommandConfig {
    /// Migrate old-style command (with just `command` field) to new format
    pub fn migrate_from_legacy(mut self) -> Self {
        // If command_type is default (Shell) and command is Some, keep as-is
        // Legacy commands already have command field set
        if self.command_type == CommandType::Shell && self.command.is_none() {
            // This shouldn't happen in practice, but provide a sensible default
            self.command = Some(String::new());
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct Commands {
    commands: HashMap<String, CommandConfig>,
}

impl Commands {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read commands.yaml: {}", e))?;
        
        // First, try to parse with the new format
        let commands_vec: Vec<CommandConfig> = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse commands.yaml: {}", e))?;

        let mut commands = HashMap::new();
        for cmd in commands_vec {
            // Handle legacy format where `command` was a required field
            let migrated = cmd.migrate_from_legacy();
            commands.insert(migrated.id.clone(), migrated);
        }

        Ok(Self { commands })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let commands_vec: Vec<&CommandConfig> = self.commands.values().collect();
        let yaml = serde_yaml::to_string(&commands_vec)
            .map_err(|e| format!("Failed to serialize commands: {}", e))?;
        fs::write(path, yaml)
            .map_err(|e| format!("Failed to write commands.yaml: {}", e))?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&CommandConfig> {
        self.commands.get(id)
    }

    pub fn all(&self) -> Vec<CommandConfig> {
        self.commands.values().cloned().collect()
    }

    pub fn add(&mut self, cmd: CommandConfig) {
        self.commands.insert(cmd.id.clone(), cmd);
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.commands.remove(id).is_some()
    }

    pub fn update(&mut self, cmd: CommandConfig) {
        self.commands.insert(cmd.id.clone(), cmd);
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self::new()
    }
}
