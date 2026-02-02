use crate::domain::dns_provider::DnsProvider;
use crate::domain::dns_repository::DnsConfigRepository;
use crate::domain::models::dns::{CloudflareConfig, DnsConfig};
use crate::domain::models::domain::DomainProvider;
use crate::error::{AppError, Result};
use crate::infrastructure::dns::{CPanelProvider, CloudflareProvider};
use std::sync::Arc;

pub struct DnsUsecase {
    dns_repo: Arc<dyn DnsConfigRepository>,
}

impl DnsUsecase {
    pub fn new(dns_repo: Arc<dyn DnsConfigRepository>) -> Self {
        Self { dns_repo }
    }

    pub async fn get_provider(
        &self,
        team_id: &str,
        provider_type: DomainProvider,
    ) -> Result<Box<dyn DnsProvider>> {
        match provider_type {
            DomainProvider::Custom => Err(AppError::BadRequest(
                "Custom provider does not support automated DNS".to_string(),
            )),
            DomainProvider::Cloudflare => {
                let config_record = self
                    .dns_repo
                    .find_by_team_and_provider(team_id, "Cloudflare")
                    .await?
                    .ok_or_else(|| {
                        AppError::NotFound(
                            "Cloudflare configuration not found for team".to_string(),
                        )
                    })?;

                let config: CloudflareConfig = serde_json::from_str(&config_record.config)
                    .map_err(|e| AppError::Internal(format!("Invalid Cloudflare config: {}", e)))?;

                Ok(Box::new(CloudflareProvider::new(config.api_token)))
            }
            DomainProvider::CPanel => Ok(Box::new(CPanelProvider::new())),
        }
    }

    pub async fn list_configs(&self, team_id: &str) -> Result<Vec<DnsConfig>> {
        self.dns_repo.find_by_team_id(team_id).await
    }

    pub async fn save_config(
        &self,
        team_id: &str,
        provider: &str,
        config: serde_json::Value,
    ) -> Result<DnsConfig> {
        let id = uuid::Uuid::new_v4().to_string();
        let config_str = serde_json::to_string(&config)
            .map_err(|e| AppError::BadRequest(format!("Invalid config JSON: {}", e)))?;

        let now = chrono::Utc::now().to_rfc3339();
        let dns_config = DnsConfig {
            id,
            team_id: team_id.to_string(),
            provider: provider.to_string(),
            config: config_str,
            created_at: now.clone(),
            updated_at: now,
        };

        self.dns_repo.save(dns_config).await
    }

    pub async fn list_available_domains(
        &self,
        team_id: &str,
        provider: DomainProvider,
    ) -> Result<Vec<String>> {
        let provider_impl = self.get_provider(team_id, provider).await?;
        provider_impl.list_available_base_domains().await
    }

    pub async fn list_remote_records(
        &self,
        team_id: &str,
        provider: DomainProvider,
    ) -> Result<Vec<crate::domain::models::dns::RemoteDnsRecord>> {
        let provider_impl = self.get_provider(team_id, provider).await?;
        provider_impl.list_records().await
    }

    pub async fn remove_config(&self, team_id: &str, provider: &str) -> Result<()> {
        self.dns_repo.delete(team_id, provider).await
    }
}
