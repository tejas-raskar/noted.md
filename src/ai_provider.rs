use crate::{error::NotedError, file_utils::FileData};
use async_trait::async_trait;

#[async_trait]
pub trait AiProvider {
    async fn send_request(&self, file_data: FileData) -> Result<String, NotedError>;
}
