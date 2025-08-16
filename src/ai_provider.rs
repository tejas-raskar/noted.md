use crate::{error::NotedError, file_utils::FileData, examples::ExampleContext};
use async_trait::async_trait;

#[async_trait]
pub trait AiProvider {
    async fn send_request(&self, file_data: FileData) -> Result<String, NotedError>;
    
    async fn send_request_with_examples(
        &self, 
        file_data: FileData, 
        examples: &ExampleContext
    ) -> Result<String, NotedError>;
}
