use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub gemini: Option<GeminiConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GeminiConfig {
    pub api_key: String,
}

pub fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "company", "notedmd").map(|dirs| {
        let config_dir = dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).ok();
        }
        config_dir.join("config.toml")
    })
}

impl Config {
    pub fn load() -> Self {
        if let Some(config_path) = get_config_path() {
            if config_path.exists() {
                let content = fs::read_to_string(config_path).unwrap_or_default();
                return toml::from_str(&content).unwrap_or_default();
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_path) = get_config_path() {
            let toml_string = toml::to_string_pretty(self)?;
            fs::write(config_path, toml_string)?;
        }
        Ok(())
    }
}
