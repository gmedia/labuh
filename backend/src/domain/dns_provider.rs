use crate::domain::models::dns::RemoteDnsRecord;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Create or update a DNS record
    /// domain: The full domain name (e.g., "sub.example.com")
    /// record_type: A, CNAME, TXT, etc.
    /// content: IP address, target domain, or text
    async fn create_record(&self, domain: &str, record_type: &str, content: &str) -> Result<String>;

    /// Delete a DNS record by its external ID and domain (to locate zone)
    async fn delete_record(&self, domain: &str, record_id: &str) -> Result<()>;

    /// Fetch available base domains (zones) from the provider
    async fn list_available_base_domains(&self) -> Result<Vec<String>>;

    /// Fetch all DNS records from the provider
    async fn list_records(&self) -> Result<Vec<RemoteDnsRecord>>;

    /// Update an existing DNS record
    async fn update_record(&self, domain: &str, record_id: &str, record_type: &str, content: &str) -> Result<()>;
}
