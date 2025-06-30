use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{ai_provider::AiProvider, file_utils::FileData};

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
    async fn send_request(
        &self,
        file_data: FileData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/api/generate", self.url);
        let prompt = if let Some(custom_prompt) = &self.prompt {
            custom_prompt.clone()
        } else {
            "The user has provided an image of handwritten notes. Your task is to accurately transcribe these notes into a well-structured Markdown file. Preserve the original hierarchy, including headings and lists. Use LaTeX for any mathematical equations that appear in the notes. The output should only be the markdown content.".to_string()
        };

        let request_body = OllamaRequest {
            model: self.model.clone(),
            prompt,
            images: vec![file_data.encoded_data],
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        let response_text = response.response;
        let cleaned_markdown = response_text
            .trim_start_matches("```markdown\n")
            .trim_end_matches("```");

        Ok(cleaned_markdown.to_string())
    }
}
