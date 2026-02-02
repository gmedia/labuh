use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::domain::models::{CloudflareConfig, DomainProvider};
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Create or update a DNS record for the given domain pointing to the target (IP or CNAME)
    async fn create_record(&self, domain: &str, target: &str) -> Result<String>;

    /// Delete a DNS record by its external ID
    async fn delete_record(&self, record_id: &str) -> Result<()>;
}

pub struct DnsProviderService {
    db: SqlitePool,
}

impl DnsProviderService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Get a DNS provider instance for a specific team and provider type
    pub async fn get_provider(
        &self,
        team_id: &str,
        provider_type: DomainProvider,
    ) -> Result<Box<dyn DnsProvider>> {
        match provider_type {
            DomainProvider::Custom => {
                Err(AppError::BadRequest("Custom provider does not support automated DNS".to_string()))
            }
            DomainProvider::Cloudflare => {
                let config_row = sqlx::query!(
                    "SELECT config FROM dns_configs WHERE team_id = ? AND provider = 'Cloudflare'",
                    team_id
                )
                .fetch_optional(&self.db)
                .await?
                .ok_or_else(|| AppError::NotFound("Cloudflare configuration not found for team".to_string()))?;

                let config: CloudflareConfig = serde_json::from_str(&config_row.config)
                    .map_err(|e| AppError::Internal(format!("Invalid Cloudflare config: {}", e)))?;

                Ok(Box::new(CloudflareProvider::new(config.api_token, config.zone_id)))
            }
            DomainProvider::CPanel => {
                Ok(Box::new(CPanelProvider {}))
            }
        }
    }

    /// Automatically provision a DNS record if a provider is configured
    pub async fn provision_record(
        &self,
        team_id: &str,
        provider_type: DomainProvider,
        domain: &str,
        target: &str,
    ) -> Result<Option<String>> {
        if matches!(provider_type, DomainProvider::Custom) {
            return Ok(None);
        }

        let provider = self.get_provider(team_id, provider_type).await?;
        let record_id = provider.create_record(domain, target).await?;
        Ok(Some(record_id))
    }

    /// Deprovision a DNS record
    pub async fn deprovision_record(
        &self,
        team_id: &str,
        provider_type: DomainProvider,
        record_id: &str,
    ) -> Result<()> {
        if matches!(provider_type, DomainProvider::Custom) {
            return Ok(());
        }

        let provider = self.get_provider(team_id, provider_type).await?;
        provider.delete_record(record_id).await
    }
}

pub struct CloudflareProvider {
    api_token: String,
    zone_id: String,
    client: reqwest::Client,
}

impl CloudflareProvider {
    pub fn new(api_token: String, zone_id: String) -> Self {
        Self {
            api_token,
            zone_id,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    async fn create_record(&self, domain: &str, target: &str) -> Result<String> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        );

        let record_type = if target.parse::<std::net::IpAddr>().is_ok() {
            "A"
        } else {
            "CNAME"
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&serde_json::json!({
                "type": record_type,
                "name": domain,
                "content": target,
                "ttl": 1, // Auto
                "proxied": false
            }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Cloudflare API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Cloudflare API error ({}): {}",
                response.status(),
                error_text
            )));
        }

        let body: serde_json::Value = response.json().await.map_err(|e| {
            AppError::Internal(format!("Failed to parse Cloudflare response: {}", e))
        })?;

        let id = body["result"]["id"]
            .as_str()
            .ok_or_else(|| AppError::Internal("Cloudflare response missing record ID".to_string()))?;

        Ok(id.to_string())
    }

    async fn delete_record(&self, record_id: &str) -> Result<()> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.zone_id, record_id
        );

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Cloudflare API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Cloudflare API error ({}): {}",
                response.status(),
                error_text
            )));
        }

        Ok(())
    }
}

pub struct CPanelProvider {
    // Basic implementation details for CPanel
}

#[async_trait]
impl DnsProvider for CPanelProvider {
    async fn create_record(&self, _domain: &str, _target: &str) -> Result<String> {
        Err(AppError::Internal("cPanel provider not yet implemented".to_string()))
    }

    async fn delete_record(&self, _record_id: &str) -> Result<()> {
        Err(AppError::Internal("cPanel provider not yet implemented".to_string()))
    }
}
