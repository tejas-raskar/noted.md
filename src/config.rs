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
    pub openai: Option<OpenAIConfig>,
    pub notion: Option<NotionConfig>,
    pub examples: Option<ExamplesConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExamplesConfig {
    pub database_path: String,
    pub examples_dir: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NotionConfig {
    pub api_key: String,
    pub database_id: String,
    #[serde(default)]
    pub title_property_name: String,
    #[serde(default)]
    pub properties: Vec<NotionPropertyConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NotionPropertyConfig {
    pub name: String,
    pub property_type: String,
    pub default_value: serde_json::Value,
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OpenAIConfig {
    pub url: String,
    pub model: String,
    pub api_key: Option<String>,
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

    pub fn get_examples_config(&self) -> ExamplesConfig {
        if let Some(examples_config) = &self.examples {
            examples_config.clone()
        } else {
            self.default_examples_config()
        }
    }

    fn default_examples_config(&self) -> ExamplesConfig {
        if let Some(project_dirs) = ProjectDirs::from("com", "company", "notedmd") {
            let data_dir = project_dirs.data_dir();
            ExamplesConfig {
                database_path: data_dir.join("examples.json").to_string_lossy().to_string(),
                examples_dir: data_dir.join("examples").to_string_lossy().to_string(),
            }
        } else {
            ExamplesConfig {
                database_path: "examples.json".to_string(),
                examples_dir: "examples".to_string(),
            }
        }
    }
}

impl Clone for ExamplesConfig {
    fn clone(&self) -> Self {
        Self {
            database_path: self.database_path.clone(),
            examples_dir: self.examples_dir.clone(),
        }
    }
}
