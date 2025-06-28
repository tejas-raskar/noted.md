use async_trait::async_trait;

use crate::file_utils::FileData;

#[async_trait]
pub trait AiProvider {
    async fn send_request(&self, file_data: FileData)
    -> Result<String, Box<dyn std::error::Error>>;
}
