use crate::error::NotedError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub active_provider: Option<String>,
    pub gemini: Option<GeminiConfig>,
    pub ollama: Option<OllamaConfig>,
    pub claude: Option<ClaudeConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GeminiConfig {
    pub api_key: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
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
    pub fn load() -> Result<Self, NotedError> {
        if let Some(config_path) = get_config_path() {
            if config_path.exists() {
                let content = fs::read_to_string(config_path)?;
                return Ok(toml::from_str(&content)?);
            }
        }
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<(), NotedError> {
        if let Some(config_path) = get_config_path() {
            let toml_string = toml::to_string_pretty(self)?;
            fs::write(config_path, toml_string)?;
        }
        Ok(())
    }
}
