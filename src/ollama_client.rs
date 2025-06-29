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
            "Take the handwritten notes from this image and convert them into a clean, well-structured Markdown file. Pay attention to headings, lists, and any other formatting. Resemble the hierarchy. Use latex for mathematical equations. For latex use the $$ syntax instead of ```latex. Do not skip anything from the original text. The output should be suitable for use in Obsidian. Just give me the markdown, do not include other text in the response apart from the markdown file. No explanation on how the changes where made is needed".to_string()
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
