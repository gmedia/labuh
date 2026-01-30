use crate::domain::models::system::SystemStats;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SystemProvider: Send + Sync {
    async fn get_stats(&self) -> Result<SystemStats>;
}
