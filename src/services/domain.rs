//! Domain service for managing custom domains and Caddy routing

use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::Domain;
use crate::services::CaddyService;

pub struct DomainService {
    db: SqlitePool,
    caddy_service: Arc<CaddyService>,
}

impl DomainService {
    pub fn new(db: SqlitePool, caddy_service: Arc<CaddyService>) -> Self {
        Self { db, caddy_service }
    }

    /// List all domains for a stack
    pub async fn list_domains(&self, stack_id: &str) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE stack_id = ? ORDER BY created_at DESC"
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
    ) -> Result<Domain> {
        // Check if domain already exists
        let existing = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE domain = ?")
            .bind(domain)
            .fetch_optional(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(format!("Domain '{}' already exists", domain)));
        }

        // Create domain record
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Build the upstream address (container_name:port)
        let container_upstream = format!("{}:{}", container_name, container_port);

        sqlx::query(
            "INSERT INTO domains (id, stack_id, container_name, container_port, domain, ssl_enabled, verified, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(stack_id)
        .bind(container_name)
        .bind(container_port)
        .bind(domain)
        .bind(true)
        .bind(false)
        .bind(&now)
        .execute(&self.db)
        .await?;

        // Add route to Caddy
        if let Err(e) = self.caddy_service.add_route(domain, &container_upstream).await {
            // Rollback domain creation
            sqlx::query("DELETE FROM domains WHERE id = ?")
                .bind(&id)
                .execute(&self.db)
                .await?;
            return Err(e);
        }

        // Return created domain
        let domain_record = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
            .bind(&id)
            .fetch_one(&self.db)
            .await?;

        Ok(domain_record)
    }

    /// Remove a domain
    pub async fn remove_domain(&self, stack_id: &str, domain: &str) -> Result<()> {
        // Check domain belongs to stack
        let domain_record = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE domain = ? AND stack_id = ?"
        )
        .bind(domain)
        .bind(stack_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Domain not found".to_string()))?;

        // Remove from Caddy (ignore errors if route doesn't exist)
        let _ = self.caddy_service.remove_route(&domain_record.domain).await;

        // Delete from database
        sqlx::query("DELETE FROM domains WHERE id = ?")
            .bind(&domain_record.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Verify domain DNS - checks if domain resolves to expected IP or has valid CNAME
    pub async fn verify_domain(&self, domain: &str, expected_ip: Option<&str>) -> Result<DnsVerificationResult> {
        use hickory_resolver::TokioAsyncResolver;
        use hickory_resolver::config::{ResolverConfig, ResolverOpts};

        let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        // Try A record lookup
        let a_records = match resolver.lookup_ip(domain).await {
            Ok(lookup) => lookup.iter().map(|ip| ip.to_string()).collect::<Vec<_>>(),
            Err(_) => vec![],
        };

        // Try CNAME lookup
        let cname_records = match resolver.lookup(domain, hickory_resolver::proto::rr::RecordType::CNAME).await {
            Ok(lookup) => lookup.iter()
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
        let domains = sqlx::query_as::<_, Domain>("SELECT * FROM domains")
            .fetch_all(&self.db)
            .await?;

        tracing::info!("Syncing {} domains to Caddy...", domains.len());

        for domain in domains {
            let container_upstream = format!("{}:{}", domain.container_name, domain.container_port);
            if let Err(e) = self.caddy_service.add_route(&domain.domain, &container_upstream).await {
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
