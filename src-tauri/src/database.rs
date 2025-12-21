use crate::commands::{CommandConfig, CommandType, VolumeDirection};
use crate::config::Settings;
use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> SqliteResult<()> {
        // Create settings table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Create commands table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                command_type TEXT NOT NULL DEFAULT 'shell',
                command TEXT,
                volume_direction TEXT,
                volume_step INTEGER,
                directory_path TEXT,
                app_name TEXT,
                keybind TEXT,
                focus_app TEXT
            )",
            [],
        )?;

        Ok(())
    }

    // Settings operations
    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                [key],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            [key, value],
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> Settings {
        let port = self
            .get_setting("port")
            .and_then(|s| s.parse().ok())
            .unwrap_or(7776);
        
        let auth_code = self
            .get_setting("auth_code")
            .unwrap_or_else(Settings::generate_random_code);

        Settings { port, auth_code }
    }

    pub fn save_settings(&self, settings: &Settings) -> SqliteResult<()> {
        self.set_setting("port", &settings.port.to_string())?;
        self.set_setting("auth_code", &settings.auth_code)?;
        Ok(())
    }

    // Commands operations
    pub fn get_all_commands(&self) -> SqliteResult<Vec<CommandConfig>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, command_type, command, volume_direction, volume_step, 
                    directory_path, app_name, keybind, focus_app 
             FROM commands"
        )?;

        let commands = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let command_type_str: String = row.get(2)?;
            let command: Option<String> = row.get(3)?;
            let volume_direction_str: Option<String> = row.get(4)?;
            let volume_step: Option<u8> = row.get(5)?;
            let directory_path: Option<String> = row.get(6)?;
            let app_name: Option<String> = row.get(7)?;
            let keybind: Option<String> = row.get(8)?;
            let focus_app: Option<String> = row.get(9)?;

            let command_type = match command_type_str.as_str() {
                "shell" => CommandType::Shell,
                "volume" => CommandType::Volume,
                "open_directory" => CommandType::OpenDirectory,
                "focus_app" => CommandType::FocusApp,
                "keybind" => CommandType::Keybind,
                _ => CommandType::Shell,
            };

            let volume_direction = volume_direction_str.map(|s| match s.as_str() {
                "up" => VolumeDirection::Up,
                "down" => VolumeDirection::Down,
                "mute" => VolumeDirection::Mute,
                _ => VolumeDirection::Up,
            });

            Ok(CommandConfig {
                id,
                name,
                command_type,
                command,
                volume_direction,
                volume_step,
                directory_path,
                app_name,
                keybind,
                focus_app,
            })
        })?;

        commands.collect()
    }

    pub fn get_command(&self, id: &str) -> Option<CommandConfig> {
        self.conn
            .query_row(
                "SELECT id, name, command_type, command, volume_direction, volume_step, 
                        directory_path, app_name, keybind, focus_app 
                 FROM commands WHERE id = ?",
                [id],
                |row| {
                    let id: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let command_type_str: String = row.get(2)?;
                    let command: Option<String> = row.get(3)?;
                    let volume_direction_str: Option<String> = row.get(4)?;
                    let volume_step: Option<u8> = row.get(5)?;
                    let directory_path: Option<String> = row.get(6)?;
                    let app_name: Option<String> = row.get(7)?;
                    let keybind: Option<String> = row.get(8)?;
                    let focus_app: Option<String> = row.get(9)?;

                    let command_type = match command_type_str.as_str() {
                        "shell" => CommandType::Shell,
                        "volume" => CommandType::Volume,
                        "open_directory" => CommandType::OpenDirectory,
                        "focus_app" => CommandType::FocusApp,
                        "keybind" => CommandType::Keybind,
                        _ => CommandType::Shell,
                    };

                    let volume_direction = volume_direction_str.map(|s| match s.as_str() {
                        "up" => VolumeDirection::Up,
                        "down" => VolumeDirection::Down,
                        "mute" => VolumeDirection::Mute,
                        _ => VolumeDirection::Up,
                    });

                    Ok(CommandConfig {
                        id,
                        name,
                        command_type,
                        command,
                        volume_direction,
                        volume_step,
                        directory_path,
                        app_name,
                        keybind,
                        focus_app,
                    })
                },
            )
            .ok()
    }

    pub fn save_command(&self, cmd: &CommandConfig) -> SqliteResult<()> {
        let command_type = match cmd.command_type {
            CommandType::Shell => "shell",
            CommandType::Volume => "volume",
            CommandType::OpenDirectory => "open_directory",
            CommandType::FocusApp => "focus_app",
            CommandType::Keybind => "keybind",
        };

        let volume_direction = cmd.volume_direction.as_ref().map(|d| match d {
            VolumeDirection::Up => "up",
            VolumeDirection::Down => "down",
            VolumeDirection::Mute => "mute",
        });

        self.conn.execute(
            "INSERT OR REPLACE INTO commands 
             (id, name, command_type, command, volume_direction, volume_step, 
              directory_path, app_name, keybind, focus_app)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                cmd.id,
                cmd.name,
                command_type,
                cmd.command,
                volume_direction,
                cmd.volume_step,
                cmd.directory_path,
                cmd.app_name,
                cmd.keybind,
                cmd.focus_app,
            ],
        )?;
        Ok(())
    }

    pub fn delete_command(&self, id: &str) -> SqliteResult<bool> {
        let rows = self.conn.execute("DELETE FROM commands WHERE id = ?", [id])?;
        Ok(rows > 0)
    }

    pub fn save_all_commands(&self, commands: &[CommandConfig]) -> SqliteResult<()> {
        // Clear all commands and re-insert
        self.conn.execute("DELETE FROM commands", [])?;
        for cmd in commands {
            self.save_command(cmd)?;
        }
        Ok(())
    }

    /// Migrate data from old YAML/JSON files to SQLite
    pub fn migrate_from_files(&self, settings_path: &Path, commands_path: &Path) -> SqliteResult<()> {
        // Check if we already have data
        let has_data: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM settings",
            [],
            |row| row.get(0),
        )?;

        if has_data > 0 {
            return Ok(()); // Already migrated
        }

        // Migrate settings from JSON
        if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(settings_path) {
                if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
                    self.save_settings(&settings)?;
                    println!("Migrated settings from JSON to SQLite");
                }
            }
        }

        // Migrate commands from YAML
        if commands_path.exists() {
            if let Ok(content) = std::fs::read_to_string(commands_path) {
                if let Ok(commands) = serde_yaml::from_str::<Vec<CommandConfig>>(&content) {
                    for cmd in commands {
                        self.save_command(&cmd)?;
                    }
                    println!("Migrated {} commands from YAML to SQLite", self.get_all_commands()?.len());
                }
            }
        }

        Ok(())
    }
}

/// Thread-safe database wrapper for async contexts
pub type SharedDatabase = Arc<Mutex<Database>>;

pub fn create_shared_database<P: AsRef<Path>>(path: P) -> Result<SharedDatabase, String> {
    let db = Database::new(path).map_err(|e| format!("Failed to create database: {}", e))?;
    Ok(Arc::new(Mutex::new(db)))
}

