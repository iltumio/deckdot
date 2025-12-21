use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_app: Option<String>,
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
        let commands_vec: Vec<CommandConfig> = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse commands.yaml: {}", e))?;

        let mut commands = HashMap::new();
        for cmd in commands_vec {
            commands.insert(cmd.id.clone(), cmd);
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

