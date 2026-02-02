use crate::domain::models::dns::DnsConfig;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DnsConfigRepository: Send + Sync {
    async fn find_by_team_id(&self, team_id: &str) -> Result<Vec<DnsConfig>>;
    async fn find_by_team_and_provider(
        &self,
        team_id: &str,
        provider: &str,
    ) -> Result<Option<DnsConfig>>;
    async fn save(&self, config: DnsConfig) -> Result<DnsConfig>;
    async fn delete(&self, team_id: &str, provider: &str) -> Result<()>;
}
