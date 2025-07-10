use crate::{ai_provider::AiProvider, error::NotedError, file_utils::FileData};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

// Request structs

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
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
    image_url: Option<Image>,
}

#[derive(Serialize)]
struct Image {
    url: String,
}

// Response structs
#[derive(Deserialize, Debug)]
pub struct OpenAIResponse {
    pub choices: Vec<Choice>,

    #[serde(default)]
    pub error: Option<OpenAIError>,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIError {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: ResponseMessage,
}

#[derive(Deserialize, Debug)]
pub struct ResponseMessage {
    pub content: String,
}

//Client
pub struct OpenAIClient {
    client: Client,
    url: String,
    model: String,
    api_key: Option<String>,
    prompt: Option<String>,
}

impl OpenAIClient {
    pub fn new(
        url: String,
        model: String,
        api_key: Option<String>,
        prompt: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            url,
            model,
            api_key,
            prompt,
        }
    }
}

#[async_trait]
impl AiProvider for OpenAIClient {
    async fn send_request(&self, file_data: FileData) -> Result<String, NotedError> {
        let url = format!("{}/v1/chat/completions", self.url);
        let prompt = if let Some(custom_prompt) = &self.prompt {
            custom_prompt.clone()
        } else {
            "The user has provided an image of handwritten notes. Your task is to accurately transcribe these notes into a well-structured Markdown file. Preserve the original hierarchy, including headings and lists. Use LaTeX for any mathematical equations that appear in the notes. The output should only be the markdown content.".to_string()
        };
        let image_url = format!(
            "data:{};base64,{}",
            file_data.mime_type, file_data.encoded_data
        );

        let request_body = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![
                    Content {
                        content_type: "text".to_string(),
                        text: Some(prompt),
                        image_url: None,
                    },
                    Content {
                        content_type: "image_url".to_string(),
                        text: None,
                        image_url: Some(Image { url: image_url }),
                    },
                ],
            }],
        };

        let mut request = self.client.post(&url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.json(&request_body).send().await?;

        let status = response.status();
        let response_body = response.text().await?;

        if status != StatusCode::OK {
            let error_response: Result<OpenAIResponse, _> = serde_json::from_str(&response_body);
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

        let openai_response: OpenAIResponse = serde_json::from_str(&response_body)
            .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;

        if let Some(error) = openai_response.error {
            return Err(NotedError::ApiError(error.message));
        }

        let markdown_text = openai_response
            .choices
            .first()
            .map(|c| c.message.content.as_str())
            .unwrap_or("");

        let cleaned_markdown = markdown_text
            .trim_start_matches("```markdown\n")
            .trim_end_matches("```");

        Ok(cleaned_markdown.to_string())
    }
}
