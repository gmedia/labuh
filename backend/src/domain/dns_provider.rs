use crate::domain::models::dns::RemoteDnsRecord;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Create or update a DNS record for the given domain pointing to the target (IP or CNAME)
    async fn create_record(&self, domain: &str, target: &str) -> Result<String>;

    /// Delete a DNS record by its external ID
    async fn delete_record(&self, record_id: &str) -> Result<()>;

    /// Fetch available base domains (zones) from the provider
    async fn list_available_base_domains(&self) -> Result<Vec<String>>;

    /// Fetch all DNS records from the provider
    async fn list_records(&self) -> Result<Vec<RemoteDnsRecord>>;
}
