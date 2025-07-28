use comrak::Arena;
use notion_client::objects::block::Block;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{error::NotedError, notion::converter};

// Request structs
#[derive(Serialize)]
pub struct NotionRequest {
    pub parent: Parent,
    pub properties: Properties,
    pub children: Vec<Block>,
}

#[derive(Serialize)]
pub struct Parent {
    pub database_id: String,
}

#[derive(Serialize)]
pub struct Properties {
    #[serde(rename = "Doc name")]
    pub name: Title,

    #[serde(rename = "Category")]
    pub category: Category,
}

#[derive(Serialize)]
pub struct Title {
    pub title: Vec<TextProperty>,
}

#[derive(Serialize)]
pub struct TextProperty {
    pub text: Content,
}

#[derive(Serialize)]
pub struct Content {
    pub content: String,
}

#[derive(Serialize)]
pub struct Category {
    pub multi_select: Vec<SelectOption>,
}

#[derive(Serialize)]
pub struct SelectOption {
    pub name: String,
}

// Response Struct
#[derive(Deserialize, Debug)]
pub struct NotionResponse {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct NotionError {
    pub message: String,
}

// Client
pub struct NotionClient {
    client: Client,
    api_key: String,
    database_id: String,
}

impl NotionClient {
    pub fn new(api_key: String, database_id: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            database_id,
        }
    }

    pub async fn create_notion_page(
        &self,
        title: &str,
        category: &str,
        markdown_content: &str,
    ) -> Result<NotionResponse, NotedError> {
        let url = "https://api.notion.com/v1/pages";
        let arena = Arena::new();
        let blocks = converter::Converter::run(&markdown_content, &arena)
            .map_err(|e| NotedError::ApiError(e.to_string()))?;

        let request_body = NotionRequest {
            parent: Parent {
                database_id: self.database_id.clone(),
            },
            properties: Properties {
                name: Title {
                    title: vec![TextProperty {
                        text: Content {
                            content: title.to_string(),
                        },
                    }],
                },
                category: Category {
                    multi_select: vec![SelectOption {
                        name: category.to_string(),
                    }],
                },
            },
            children: blocks,
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", "2022-06-28")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        if status.is_success() {
            let notion_reponse: NotionResponse = serde_json::from_str(&response_body)
                .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;
            Ok(notion_reponse)
        } else {
            let error_response: NotionError = serde_json::from_str(&response_body)
                .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;
            Err(NotedError::ApiError(format!(
                "Notion API Error ({}): {}",
                status, error_response.message
            )))
        }
    }
}
