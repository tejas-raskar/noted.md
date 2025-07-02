use crate::ai_provider::AiProvider;
use crate::file_utils::FileData;
use async_trait::async_trait;
use reqwest::Client;
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
    async fn send_request(
        &self,
        file_data: FileData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = "https://api.anthropic.com/v1/messages".to_string();

        let prompt = if let Some(custom_prompt) = &self.prompt {
            custom_prompt.clone()
        } else {
            "Take the handwritten notes from this image and convert them into a clean, well-structured Markdown file. Pay attention to headings, lists, and any other formatting. Resemble the hierarchy. Use latex for mathematical equations. For latex use the $$ syntax instead of ```latex. Do not skip anything from the original text. The output should be suitable for use in Obsidian. Just give me the markdown, do not include other text in the response apart from the markdown file. No explanation on how the changes where made is needed".to_string()
        };

        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![
                    Content {
                        content_type: "image".to_string(),
                        text: None,
                        source: Some(Source {
                            source_type: "base64".to_string(),
                            media_type: file_data.mime_type,
                            data: file_data.encoded_data,
                        }),
                    },
                    Content {
                        content_type: "text".to_string(),
                        text: Some(prompt),
                        source: None,
                    },
                ],
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await?
            .json::<ClaudeResponse>()
            .await?;

        let response_text = response.content.first();
        let markdown_text = if let Some(part) = response_text {
            &part.text
        } else {
            println!("{}", "Could not find text in Claude response.");
            std::process::exit(1);
        };

        let cleaned_markdown = markdown_text
            .trim_start_matches("```markdown")
            .trim_end_matches("```");

        Ok(cleaned_markdown.to_string())
    }
}
