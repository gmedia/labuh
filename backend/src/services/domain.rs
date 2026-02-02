//! Domain service for managing custom domains and Caddy routing

use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::{Domain, DomainProvider, DomainType};
use crate::error::{AppError, Result};
use crate::services::dns_provider::DnsProviderService;
use crate::services::CaddyService;

pub struct DomainService {
    db: SqlitePool,
    caddy_service: Arc<CaddyService>,
    dns_service: Arc<DnsProviderService>,
}

impl DomainService {
    pub fn new(
        db: SqlitePool,
        caddy_service: Arc<CaddyService>,
        dns_service: Arc<DnsProviderService>,
    ) -> Self {
        Self {
            db,
            caddy_service,
            dns_service,
        }
    }

    /// List all domains for a stack
    pub async fn list_domains(&self, stack_id: &str) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE stack_id = ? ORDER BY created_at DESC",
        )
        .bind(stack_id)
        .fetch_all(&self.db)
        .await?;

        Ok(domains)
    }

    /// Add a domain to a stack with routing to a specific container
    pub async fn add_domain(
        &self,
        stack_id: &str,
        domain: &str,
        container_name: &str,
        container_port: i32,
        provider: DomainProvider,
        domain_type: DomainType,
        tunnel_id: Option<String>,
    ) -> Result<Domain> {
        // Check if domain already exists
        let existing = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE domain = ?")
            .bind(domain)
            .fetch_optional(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(format!(
                "Domain '{}' already exists",
                domain
            )));
        }

        // 1. Provision DNS record if needed
        let dns_record_id = if !matches!(provider, DomainProvider::Custom) {
            // We need a target for DNS. For Caddy, it's the public IP.
            // For Tunnel, it's the CNAME to the tunnel.
            let target = match domain_type {
                DomainType::Caddy => std::env::var("LABUH_PUBLIC_IP").unwrap_or_else(|_| "127.0.0.1".to_string()),
                DomainType::Tunnel => {
                    // Placeholder for tunnel target. Usually <tunnel-id>.cfargotunnel.com
                    format!("{}.cfargotunnel.com", tunnel_id.as_deref().unwrap_or("unknown"))
                }
            };

            // Get team_id from stack
            let stack = sqlx::query!("SELECT team_id FROM stacks WHERE id = ?", stack_id)
                .fetch_one(&self.db)
                .await?;

            self.dns_service
                .provision_record(&stack.team_id, provider.clone(), domain, &target)
                .await?
        } else {
            None
        };

        // Create domain record
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Build the upstream address (container_name:port)
        let container_upstream = format!("{}:{}", container_name, container_port);

        if let Err(e) = sqlx::query(
            "INSERT INTO domains (id, stack_id, container_name, container_port, domain, ssl_enabled, verified, provider, type, tunnel_id, dns_record_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(stack_id)
        .bind(container_name)
        .bind(container_port)
        .bind(domain)
        .bind(true)
        .bind(false)
        .bind(&provider)
        .bind(&domain_type)
        .bind(&tunnel_id)
        .bind(&dns_record_id)
        .bind(&now)
        .execute(&self.db)
        .await {
            // Cleanup DNS if DB insert fails
            if let (Some(rec_id), Some(team_id)) = (dns_record_id, self.get_team_id_by_stack(stack_id).await.ok()) {
                let _ = self.dns_service.deprovision_record(&team_id, provider, &rec_id).await;
            }
            return Err(e.into());
        }

        // Add route to Caddy if it's a Caddy type domain
        if matches!(domain_type, DomainType::Caddy) {
            if let Err(e) = self
                .caddy_service
                .add_route(domain, &container_upstream)
                .await
            {
                // Rollback DNS and DB
                if let (Some(rec_id), Some(team_id)) = (dns_record_id, self.get_team_id_by_stack(stack_id).await.ok()) {
                    let _ = self.dns_service.deprovision_record(&team_id, provider, &rec_id).await;
                }
                sqlx::query("DELETE FROM domains WHERE id = ?")
                    .bind(&id)
                    .execute(&self.db)
                    .await?;
                return Err(e);
            }
        }

        // Return created domain
        let domain_record = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.db)
            .await?;

        Ok(domain_record)
    }

    async fn get_team_id_by_stack(&self, stack_id: &str) -> Result<String> {
        let row = sqlx::query!("SELECT team_id FROM stacks WHERE id = ?", stack_id)
            .fetch_one(&self.db)
            .await?;
        Ok(row.team_id)
    }

    /// Remove a domain
    pub async fn remove_domain(&self, stack_id: &str, domain: &str) -> Result<()> {
        // Check domain belongs to stack
        let domain_record =
            sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE domain = ? AND stack_id = ?")
                .bind(domain)
                .bind(stack_id)
                .fetch_optional(&self.db)
                .await?
                .ok_or_else(|| AppError::NotFound("Domain not found".to_string()))?;

        // 1. Deprovision DNS if needed
        if let Some(record_id) = domain_record.dns_record_id {
            let team_id = self.get_team_id_by_stack(stack_id).await?;
            let _ = self.dns_service
                .deprovision_record(&team_id, domain_record.provider, &record_id)
                .await;
        }

        // Remove from Caddy (ignore errors if route doesn't exist)
        if matches!(domain_record.r#type, DomainType::Caddy) {
            let _ = self.caddy_service.remove_route(&domain_record.domain).await;
        }

        // Delete from database
        sqlx::query("DELETE FROM domains WHERE id = ?")
            .bind(&domain_record.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Verify domain DNS - checks if domain resolves to expected IP or has valid CNAME
    pub async fn verify_domain(
        &self,
        domain: &str,
        expected_ip: Option<&str>,
    ) -> Result<DnsVerificationResult> {
        use hickory_resolver::config::{ResolverConfig, ResolverOpts};
        use hickory_resolver::TokioAsyncResolver;

        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        // Try A record lookup
        let a_records = match resolver.lookup_ip(domain).await {
            Ok(lookup) => lookup.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
            Err(_) => vec![],
        };

        // Try CNAME lookup
        let cname_records = match resolver
            .lookup(domain, hickory_resolver::proto::rr::RecordType::CNAME)
            .await
        {
            Ok(lookup) => lookup
                .iter()
                .filter_map(|r| r.clone().into_cname().ok())
                .map(|cname| cname.to_string().trim_end_matches('.').to_string())
                .collect::<Vec<_>>(),
            Err(_) => vec![],
        };

        let verified = if let Some(expected) = expected_ip {
            a_records.iter().any(|ip| ip == expected)
        } else {
            !a_records.is_empty() || !cname_records.is_empty()
        };

        // Update database
        sqlx::query("UPDATE domains SET verified = ? WHERE domain = ?")
            .bind(verified)
            .bind(domain)
            .execute(&self.db)
            .await?;

        Ok(DnsVerificationResult {
            domain: domain.to_string(),
            verified,
            a_records,
            cname_records,
        })
    }
    /// Sync all domains from database to Caddy
    pub async fn sync_all_routes(&self) -> Result<()> {
        let domains = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE type = 'Caddy'")
            .fetch_all(&self.db)
            .await?;

        tracing::info!("Syncing {} Caddy domains to Caddy...", domains.len());

        for domain in domains {
            let container_upstream = format!("{}:{}", domain.container_name, domain.container_port);
            if let Err(e) = self
                .caddy_service
                .add_route(&domain.domain, &container_upstream)
                .await
            {
                tracing::error!("Failed to sync route for domain {}: {}", domain.domain, e);
            }
        }

        Ok(())
    }
}

/// Result of DNS verification
#[derive(Debug, Clone, serde::Serialize)]
pub struct DnsVerificationResult {
    pub domain: String,
    pub verified: bool,
    pub a_records: Vec<String>,
    pub cname_records: Vec<String>,
}
