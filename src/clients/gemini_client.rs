use crate::ai_provider::AiProvider;
use crate::error::NotedError;
use crate::file_utils::FileData;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

// Request structs

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    inline_data: Option<InlineData>,
}

#[derive(Serialize)]
struct InlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

//  Response structs

#[derive(Deserialize, Debug)]
pub struct GeminiResponse {
    pub candidates: Option<Vec<Candidate>>,
    #[serde(default)]
    pub error: Option<GeminiError>,
}

#[derive(Deserialize, Debug)]
pub struct GeminiError {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct Candidate {
    pub content: ContentResponse,
}

#[derive(Deserialize, Debug)]
pub struct ContentResponse {
    pub parts: Vec<PartResponse>,
}

#[derive(Deserialize, Debug)]
pub struct PartResponse {
    pub text: String,
}

// Client
pub struct GeminiClient {
    client: Client,
    api_key: String,
    prompt: Option<String>,
}

impl GeminiClient {
    pub fn new(api_key: String, prompt: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            prompt,
        }
    }
}

#[async_trait]
impl AiProvider for GeminiClient {
    async fn send_request(&self, files_data: Vec<FileData>) -> Result<String, NotedError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", // Use gemini-pro-vision for multi-modal
            self.api_key
        );

        let prompt = if let Some(custom_prompt) = &self.prompt {
            custom_prompt.clone()
        } else {
            "Take these texts from this image and convert them into a clean, well-structured Markdown file, with NO EXCLUDED TEXT LEFT BEHIND. Pay attention to headings, lists, formulas, math expressions, and any other formatting. Resemble the hierarchy. Use latex for mathematical equations, in which For latex use the $$ syntax instead of \"```latex\". Do not skip anything from the original text/book. Return the desired output as the response, do not include other texts/formulas in the response apart from the picture's containings. No explanation on how the changes were made is needed".to_string()
        };

        let mut parts: Vec<Part> = Vec::new();

        // Add the initial text prompt part
        parts.push(Part {
                        text: Some(prompt),
                        inline_data: None,
        });

        // Add all image data parts from the vector
        for file_data in files_data {
            parts.push(Part {
                        text: None,
                        inline_data: Some(InlineData {
                            mime_type: file_data.mime_type,
                            data: file_data.encoded_data,
                        }),
            });
        }

        let request_body = GeminiRequest {
            contents: vec![Content {
                parts, // Use the collected parts
            }],
        };
        let response = self.client.post(&url).json(&request_body).send().await?;

        let status = response.status();
        let response_body = response.text().await?;

        if status != StatusCode::OK {
            // ... existing error handling ...
            if status == StatusCode::UNAUTHORIZED {
                return Err(NotedError::InvalidApiKey);
            }
            let error_response: Result<GeminiResponse, _> = serde_json::from_str(&response_body);
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

        let gemini_response: GeminiResponse = serde_json::from_str(&response_body)
            .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;

        if let Some(error) = gemini_response.error {
            return Err(NotedError::ApiError(error.message));
        }

        let markdown_text = gemini_response
            .candidates
            .as_ref()
            .and_then(|candidates| candidates.first())
            .and_then(|candidate| candidate.content.parts.first())
            .map(|part| part.text.as_str())
            .unwrap_or("");

        let cleaned_markdown = markdown_text
            .trim_start_matches("```markdown\n")
            .trim_end_matches("```");

        Ok(cleaned_markdown.to_string())
    }
}
