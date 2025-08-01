use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{ai_provider::AiProvider, error::NotedError, file_utils::FileData};

// Request struct
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    images: Vec<String>,
    stream: bool,
}

// Response struct
#[derive(Deserialize, Debug)]
pub struct OllamaResponse {
    pub response: String,
    #[serde(default)]
    pub error: Option<String>,
}

// Client struct
pub struct OllamaClient {
    client: Client,
    url: String,
    model: String,
    prompt: Option<String>,
}

impl OllamaClient {
    pub fn new(url: String, model: String, prompt: Option<String>) -> Self {
        Self {
            client: Client::new(),
            url,
            model,
            prompt,
        }
    }
}

#[async_trait]
impl AiProvider for OllamaClient {
    async fn send_request(&self, files_data: Vec<FileData>) -> Result<String, NotedError> {
        let url = format!("{}/api/generate", self.url);
        let prompt = if let Some(custom_prompt) = &self.prompt {
            custom_prompt.clone()
        } else {
            "The user has provided an image of handwritten notes. Your task is to accurately transcribe these notes into a well-structured Markdown file. Preserve the original hierarchy, including headings and lists. Use LaTeX for any mathematical equations that appear in the notes. The output should only be the markdown content.".to_string()
        };

        let images: Vec<String> = files_data.into_iter().map(|fd| fd.encoded_data).collect();

        let request_body = OllamaRequest {
            model: self.model.clone(),
            prompt,
            images,
            stream: false,
        };

        let response = self.client.post(&url).json(&request_body).send().await?;

        let status = response.status();
        let response_body = response.text().await?;

        if status != StatusCode::OK {
            let error_response: Result<OllamaResponse, _> = serde_json::from_str(&response_body);
            if let Ok(err_resp) = error_response {
                if let Some(error) = err_resp.error {
                    return Err(NotedError::ApiError(error));
                }
            }
            return Err(NotedError::ApiError(format!(
                "Received status code: {}",
                status
            )));
        }

        let ollama_response: OllamaResponse = serde_json::from_str(&response_body)
            .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;

        if let Some(error) = ollama_response.error {
            return Err(NotedError::ApiError(error));
        }

        let cleaned_markdown = ollama_response
            .response
            .trim_start_matches("```markdown\n")
            .trim_end_matches("```");

        Ok(cleaned_markdown.to_string())
    }
}