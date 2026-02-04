use crate::domain::models::dns::RemoteDnsRecord;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Create or update a DNS record
    async fn create_record(
        &self,
        domain: &str,
        record_type: &str,
        content: &str,
        proxied: bool,
    ) -> Result<String>;

    /// Delete a DNS record by its external ID and domain (to locate zone)
    async fn delete_record(&self, domain: &str, record_id: &str) -> Result<()>;

    /// Fetch available base domains (zones) from the provider
    async fn list_available_base_domains(&self) -> Result<Vec<String>>;

    /// Fetch all DNS records from the provider
    async fn list_records(&self) -> Result<Vec<RemoteDnsRecord>>;

    /// Update an existing DNS record
    async fn update_record(
        &self,
        domain: &str,
        record_id: &str,
        record_type: &str,
        content: &str,
        proxied: bool,
    ) -> Result<()>;

    /// Setup Cloudflare Tunnel ingress rules
    async fn setup_tunnel_ingress(
        &self,
        _tunnel_id: &str,
        _hostname: &str,
        _service_url: &str,
    ) -> Result<()> {
        Ok(())
    }

    /// Remove Cloudflare Tunnel ingress rules
    async fn remove_tunnel_ingress(&self, _tunnel_id: &str, _hostname: &str) -> Result<()> {
        Ok(())
    }
}
