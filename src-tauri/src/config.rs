use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl Settings {
    pub fn default() -> Self {
        Self {
            port: 8080,
            username: "admin".to_string(),
            password: "password".to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        Ok(())
    }
}

