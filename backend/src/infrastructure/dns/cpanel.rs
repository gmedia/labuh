use crate::domain::dns_provider::DnsProvider;
use crate::domain::models::dns::RemoteDnsRecord;
use crate::error::{AppError, Result};
use async_trait::async_trait;

pub struct CPanelProvider {
    // Basic implementation details for CPanel
}

impl CPanelProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl DnsProvider for CPanelProvider {
    async fn create_record(&self, _domain: &str, _target: &str) -> Result<String> {
        Err(AppError::Internal(
            "cPanel provider not yet implemented".to_string(),
        ))
    }

    async fn delete_record(&self, _record_id: &str) -> Result<()> {
        Err(AppError::Internal(
            "cPanel provider not yet implemented".to_string(),
        ))
    }

    async fn list_available_base_domains(&self) -> Result<Vec<String>> {
        Err(AppError::Internal(
            "cPanel provider not yet implemented".to_string(),
        ))
    }

    async fn list_records(&self) -> Result<Vec<RemoteDnsRecord>> {
        Ok(vec![])
    }
}
