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

    /// Add a domain to a stack
    pub async fn add_domain(&self, stack_id: &str, domain: &str, container_upstream: &str) -> Result<Domain> {
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
        let _now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO domains (id, stack_id, domain, ssl_enabled, verified, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(stack_id)
        .bind(domain)
        .bind(true)
        .bind(false)
        .execute(&self.db)
        .await?;

        // Add route to Caddy
        if let Err(e) = self.caddy_service.add_route(domain, container_upstream).await {
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

    /// Get stack upstream address (container_name:port)
    pub async fn get_stack_upstream(&self, stack_id: &str) -> Result<String> {
        // For stacks, we look at the first service in the compose file or use stack name
        #[derive(sqlx::FromRow)]
        struct StackInfo {
            name: String,
        }

        let stack = sqlx::query_as::<_, StackInfo>(
            "SELECT name FROM stacks WHERE id = ?"
        )
        .bind(stack_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Stack not found".to_string()))?;

        // Default to port 80, stack name as upstream
        Ok(format!("{}:80", stack.name))
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

    /// Generate a subdomain for a stack based on naming convention
    pub fn generate_subdomain(stack_name: &str, base_domain: &str) -> String {
        // Sanitize stack name: lowercase, replace spaces with hyphens, remove special chars
        let sanitized: String = stack_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();

        // Remove consecutive hyphens and trim
        let mut result = String::new();
        let mut prev_hyphen = false;
        for c in sanitized.chars() {
            if c == '-' {
                if !prev_hyphen && !result.is_empty() {
                    result.push(c);
                    prev_hyphen = true;
                }
            } else {
                result.push(c);
                prev_hyphen = false;
            }
        }
        result = result.trim_matches('-').to_string();

        format!("{}.{}", result, base_domain)
    }

    /// Create an auto-generated subdomain for a stack
    pub async fn create_stack_subdomain(&self, stack_id: &str, stack_name: &str, base_domain: &str) -> Result<Domain> {
        let subdomain = Self::generate_subdomain(stack_name, base_domain);
        let upstream = self.get_stack_upstream(stack_id).await?;

        // Check if subdomain already exists - if so, return existing
        if let Ok(existing) = self.list_domains(stack_id).await {
            if let Some(domain) = existing.iter().find(|d| d.domain == subdomain) {
                return Ok(domain.clone());
            }
        }

        self.add_domain(stack_id, &subdomain, &upstream).await
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
