use crate::domain::models::domain::Domain;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DomainRepository: Send + Sync {
    async fn find_by_stack_id(&self, stack_id: &str) -> Result<Vec<Domain>>;
    async fn find_by_team_id(&self, team_id: &str) -> Result<Vec<Domain>>;
    async fn find_by_domain(&self, domain: &str) -> Result<Option<Domain>>;
    async fn list_all(&self) -> Result<Vec<Domain>>;
    async fn create(&self, domain: Domain) -> Result<Domain>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn update_verification(&self, domain: &str, verified: bool) -> Result<()>;
    async fn update_dns_record_id(&self, id: &str, dns_record_id: &str) -> Result<()>;
    async fn update_branding(&self, domain: &str, show_branding: bool) -> Result<()>;
}
