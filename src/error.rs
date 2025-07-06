use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotedError {
    #[error(" Configuration file not found. Please run 'notedmd config --edit' to set it up.")]
    ConfigNotFound,

    #[error(" Failed to save configuration: {0}")]
    ConfigSaveError(#[from] toml::ser::Error),

    #[error(" Failed to read configuration: {0}")]
    ConfigReadError(#[from] toml::de::Error),

    #[error(" I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error(" Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error(" API key is invalid or missing. Please check your configuration.")]
    InvalidApiKey,

    #[error(" The AI provider returned an error: {0}")]
    ApiError(String),

    #[error(" Failed to decode API response: {0}")]
    ResponseDecodeError(String),

    #[error(" Could not determine the file name for the path: {0}")]
    FileNameError(String),

    #[error(" File type not supported: {0}")]
    UnsupportedFileType(String),

    #[error(" Ollama is not configured properly. Please run 'notedmd config --edit' to set it up.")]
    OllamaNotConfigured,

    #[error(" Gemini is not configured properly. Please run 'notedmd config --edit' to set it up.")]
    GeminiNotConfigured,

    #[error(" Claude is not configured properly. Please run 'notedmd config --edit' to set it up.")]
    ClaudeNotConfigured,

    #[error(" No active provider. Please run 'notedmd config --edit' to set a provider.")]
    NoActiveProvider,

    #[error(" Dialoguer error: {0}")]
    DialoguerError(#[from] dialoguer::Error),
}
