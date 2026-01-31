use crate::domain::models::template::Template;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TemplateRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<Template>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Template>>;
    async fn save(&self, template: &Template) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn count(&self) -> Result<i64>;
}
