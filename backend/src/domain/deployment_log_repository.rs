use crate::domain::models::deployment_log::DeploymentLog;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
#[allow(dead_code)]
pub trait DeploymentLogRepository: Send + Sync {
    async fn list_by_stack(&self, stack_id: &str, limit: i32) -> Result<Vec<DeploymentLog>>;
    async fn find_by_id(&self, id: &str) -> Result<DeploymentLog>;
    async fn save(&self, log: DeploymentLog) -> Result<DeploymentLog>;
    async fn append_log(&self, id: &str, log_line: &str) -> Result<()>;
}
