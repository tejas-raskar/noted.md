use crate::ai_provider::AiProvider;
use crate::file_utils::FileData;
use async_trait::async_trait;
use reqwest::Client;
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
    pub candidates: Vec<Candidate>,
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
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl AiProvider for GeminiClient {
    async fn send_request(
        &self,
        file_data: FileData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemma-3-27b-it:generateContent?key={}",
            self.api_key
        );

        let prompt = "Take the handwritten notes from this image and convert them into a clean, well-structured Markdown file. Pay attention to headings, lists, and any other formatting. Resemble the hierarchy. Use latex for mathematical equations. For latex use the $$ syntax instead of ```latex. Do not skip anything from the original text. The output should be suitable for use in Obsidian. Just give me the markdown, do not include other text in the response apart from the markdown file. No explanation on how the changes where made is needed".to_string();

        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part {
                        text: Some(prompt),
                        inline_data: None,
                    },
                    Part {
                        text: None,
                        inline_data: Some(InlineData {
                            mime_type: file_data.mime_type,
                            data: file_data.encoded_data,
                        }),
                    },
                ],
            }],
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        let response_text = response
            .candidates
            .first()
            .and_then(|candidate| candidate.content.parts.first());
        let markdown_text = if let Some(part) = response_text {
            &part.text
        } else {
            println!("{}", "Could not find text in Gemini response.");
            std::process::exit(1);
        };

        let cleaned_markdown = markdown_text
            .trim_start_matches("```markdown\n")
            .trim_end_matches("```");

        Ok(cleaned_markdown.to_string())
    }
}
