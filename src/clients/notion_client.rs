use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;
use comrak::Arena;
use notion_client::objects::block::Block;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{config, error::NotedError, notion::converter};

// Request structs
#[derive(Serialize)]
pub struct NotionRequest {
    pub parent: Parent,
    pub properties: serde_json::Map<String, serde_json::Value>,
    pub children: Vec<Block>,
}

#[derive(Serialize)]
pub struct Parent {
    pub database_id: String,
}

// Response Struct
#[derive(Deserialize, Debug)]
pub struct NotionResponse {
    #[serde(rename = "id")]
    pub _id: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct NotionDatabase {
    pub properties: HashMap<String, DatabaseProperty>,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseProperty {
    #[serde(rename = "id")]
    pub _id: String,
    pub name: String,
    #[serde(flatten)]
    pub type_specific_config: PropertyType,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Title(EmptyStruct),
    RichText(EmptyStruct),
    Number(EmptyStruct),
    Select { select: SelectStruct },
    MultiSelect { multi_select: SelectStruct },
    Date(EmptyStruct),
    Checkbox(EmptyStruct),
    People(EmptyStruct),
    Files(EmptyStruct),
    Url(EmptyStruct),
    Email(EmptyStruct),
    CreatedTime(EmptyStruct),
    CreatedBy(EmptyStruct),
    LastEditedTime(EmptyStruct),
    LastEditedBy(EmptyStruct),
    Status { status: SelectStruct },
    Formula(EmptyStruct),
    Relation(EmptyStruct),
    Rollup(EmptyStruct),
    PhoneNumber(EmptyStruct),
    Button(EmptyStruct),
    UniqueId(EmptyStruct),
    Verification(EmptyStruct),
}

#[derive(Deserialize, Debug)]
pub struct SelectStruct {
    pub options: Vec<DatabaseSelectOption>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseSelectOption {
    #[serde(rename = "id")]
    pub _id: String,
    pub name: String,
    #[serde(rename = "color")]
    pub _color: String,
}

#[derive(Deserialize, Debug)]
pub struct NumberStruct {
    pub _number: NumberFormat,
}

#[derive(Deserialize, Debug)]
pub struct NumberFormat {
    pub _format: String,
}

#[derive(Deserialize, Debug)]
pub struct EmptyStruct {}

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

    pub async fn get_database_schema(&self) -> Result<NotionDatabase, NotedError> {
        let url = format!("https://api.notion.com/v1/databases/{}", self.database_id);
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;
        if status.is_success() {
            let notion_database: NotionDatabase = serde_json::from_str(&response_body)
                .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;
            Ok(notion_database)
        } else {
            let error_response: NotionError = serde_json::from_str(&response_body)
                .map_err(|e| NotedError::ResponseDecodeError(e.to_string()))?;
            Err(NotedError::ApiError(format!(
                "Notion API Error ({}): {}",
                status,
                error_response.message.red()
            )))
        }
    }

    pub async fn create_notion_page(
        &self,
        title: &str,
        title_property_name: &str,
        properties: &[config::NotionPropertyConfig],
        markdown_content: &str,
    ) -> Result<NotionResponse, NotedError> {
        let url = "https://api.notion.com/v1/pages";
        let arena = Arena::new();
        let blocks = converter::Converter::run(&markdown_content, &arena)
            .map_err(|e| NotedError::ApiError(e.to_string()))?;

        let mut props_map = serde_json::Map::new();
        props_map.insert(
            title_property_name.to_string(),
            serde_json::json!(
            {
                "title": [
                    {
                        "text":{
                            "content": title
                        }
                    }
                ]
            }),
        );

        for prop_config in properties {
            let prop_name = &prop_config.name;
            let prop_type = &prop_config.property_type;
            let prop_value = &prop_config.default_value;

            let notion_property_value = match prop_type.as_str() {
                "multi_select" => {
                    if let Some(arr) = prop_value.as_array() {
                        let options: Vec<_> = arr
                            .iter()
                            .map(|val| serde_json::json!({"name": val}))
                            .collect();
                        serde_json::json!({"multi_select": options})
                    } else {
                        continue;
                    }
                }
                "select" => serde_json::json!({
                    "select": {
                        "name": prop_value
                    }
                }),
                "rich_text" => serde_json::json!({
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": prop_value
                            }
                        }
                    ]
                }),
                "number" => serde_json::json!({
                    "number": prop_value
                }),
                "date" => serde_json::json!({
                    "date": {
                        "start": prop_value
                    }
                }),
                "checkbox" => serde_json::json!({
                    "checkbox": prop_value
                }),
                _ => continue,
            };

            props_map.insert(prop_name.clone(), notion_property_value);
        }
        let request_body = NotionRequest {
            parent: Parent {
                database_id: self.database_id.clone(),
            },
            properties: props_map,
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
