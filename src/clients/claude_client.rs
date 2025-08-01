use crate::ai_provider::AiProvider;
use crate::error::NotedError;
use crate::file_utils::FileData;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

// Request structs

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Source>,
}

#[derive(Serialize)]
struct Source {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

//  Response structs

#[derive(Deserialize, Debug)]
pub struct ClaudeResponse {
    pub content: Vec<ContentResponse>,
    #[serde(default)]
    pub error: Option<ClaudeError>,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeError {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct ContentResponse {
    pub text: String,
}

// Client
pub struct ClaudeClient {
    client: Client,
    api_key: String,
    model: String,
    prompt: Option<String>,
}

impl ClaudeClient {
    pub fn new(api_key: String, model: String, prompt: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            prompt,
        }
    }
}

#[async_trait]
impl AiProvider for ClaudeClient {
    async fn send_request(&self, files_data: Vec<FileData>) -> Result<String, NotedError> {
        let url = "https://api.anthropic.com/v1/messages".to_string();

        let prompt_text = if let Some(custom_prompt) = &self.prompt {
            custom_prompt.clone()
        } else {
            "Take the handwritten notes from this image and convert them into a clean, well-structured Markdown file. Pay attention to headings, lists, and any other formatting. Resemble the hierarchy. Use latex for mathematical equations. For latex use the $$ syntax instead of ```latex. Do not skip anything from the original text. The output should be suitable for use in Obsidian. Just give me the markdown, do not include other text in the response apart from the markdown file. No explanation on how the changes were made is needed".to_string()
        };

        let mut content_parts: Vec<Content> = Vec::new();

        content_parts.push(Content {
                        content_type: "text".to_string(),
            text: Some(prompt_text),
                        source: None,
        });

        for file_data in files_data {
            content_parts.push(Content {
                content_type: "image".to_string(),
                text: None,
                source: Some(Source {
                    source_type: "base64".to_string(),
                    media_type: file_data.mime_type,
                    data: file_data.encoded_data,
                }),
            });
        }

        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".to_string(),
                content: content_parts,
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        if status != StatusCode::OK {
            if status == StatusCode::UNAUTHORIZED {
                return Err(NotedError::InvalidApiKey);
            }
            let error_response: Result<ClaudeResponse, _> = serde_json::from_str(&response_body);
            if let Ok(err_resp) = error_response {
                if let Some(error) = err_resp.error {
                    return Err(NotedError::ApiError(error.message));
                }
            }
            return Err(NotedError::ApiError(format!(
                "Received status code: {}",
                status
            )));
        }

        let claude_response: ClaudeResponse = serde_json::from_str(&response_body)
            .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;

        if let Some(error) = claude_response.error {
            return Err(NotedError::ApiError(error.message));
        }

        let markdown_text = claude_response
            .content
            .first()
            .map(|c| c.text.as_str())
            .unwrap_or("");

        let cleaned_markdown = markdown_text
            .trim_start_matches("```markdown\n")
            .trim_end_matches("```");

        Ok(cleaned_markdown.to_string())
    }
}
