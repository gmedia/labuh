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
        let now = Utc::now().to_rfc3339();

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

    /// Verify domain DNS (basic check)
    pub async fn verify_domain(&self, domain: &str) -> Result<bool> {
        // For now, just mark as verified (real implementation would check DNS)
        sqlx::query("UPDATE domains SET verified = ? WHERE domain = ?")
            .bind(true)
            .bind(domain)
            .execute(&self.db)
            .await?;

        Ok(true)
    }
}


