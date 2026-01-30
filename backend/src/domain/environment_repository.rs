use crate::domain::models::environment::StackEnvVar;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    async fn list_by_stack(&self, stack_id: &str) -> Result<Vec<StackEnvVar>>;
    async fn find_by_id(&self, id: &str) -> Result<StackEnvVar>;
    async fn find_existing(
        &self,
        stack_id: &str,
        container_name: &str,
        key: &str,
    ) -> Result<Option<StackEnvVar>>;
    async fn save(&self, var: StackEnvVar) -> Result<StackEnvVar>;
    async fn delete(&self, stack_id: &str, container_name: &str, key: &str) -> Result<()>;
}
