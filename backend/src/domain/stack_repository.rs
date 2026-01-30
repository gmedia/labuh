use crate::domain::models::Stack;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait StackRepository: Send + Sync {
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Stack>>;
    async fn find_by_id(&self, id: &str, user_id: &str) -> Result<Stack>;
    async fn find_by_id_internal(&self, id: &str) -> Result<Stack>;
    async fn create(&self, stack: Stack) -> Result<Stack>;
    async fn update_status(&self, id: &str, status: &str) -> Result<()>;
    async fn update_compose(&self, id: &str, content: &str) -> Result<()>;
    async fn update_webhook_token(&self, id: &str, token: &str) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn validate_webhook_token(&self, id: &str, token: &str) -> Result<Stack>;
}
